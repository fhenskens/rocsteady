use rocsteady::{
    Address, Client, ClientConfig, Clock, DataType, Error, Login, Parameter, ParameterBlock,
    Session, Tlp, Value,
    protocol::CrcMode,
    simulator::{Fault, Model, Simulator},
    transport::{Connector, Stream, read_frame},
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
// POL-SESSION-MALFORMED: Managed ownership contract. See docs/REQUIREMENTS.md.
async fn standalone_session_retires_every_malformed_typed_reply() {
    for opcode in [6, 7, 17, 50, 100, 105, 108, 135, 167, 180] {
        let (stream, mut peer) = tokio::io::duplex(512);
        let responder = tokio::spawn(async move {
            assert_eq!(
                read_frame(&mut peer, CrcMode::Required)
                    .await
                    .unwrap()
                    .opcode,
                opcode
            );
            // Valid framing with independently specified invalid typed payloads.
            let data = if opcode == 7 { vec![0; 8] } else { vec![1] };
            let reply = rocsteady::protocol::Frame::new(
                Address::new(1, 0),
                Address::new(1, 2),
                opcode,
                data,
            )
            .encode()
            .unwrap();
            peer.write_all(&reply).await.unwrap();
        });
        let session = Session::new(
            Box::new(stream),
            Address::new(1, 0),
            Address::new(1, 2),
            Duration::from_secs(1),
            CrcMode::Required,
        )
        .unwrap();
        assert!(matches!(
            session.read_parameters(&[]).await,
            Err(Error::InvalidRequest(_))
        ));
        assert!(session.is_connected());
        let result = match opcode {
            6 => session.read_configuration().await.map(|_| ()),
            7 => session.read_clock().await.map(|_| ()),
            17 => session.login(&Login::new(*b"TST", 1, None).unwrap()).await,
            50 => session
                .read_io_points(rocsteady::IoPointRequest {
                    layout: rocsteady::IoLayout::Compatibility160,
                    selector: rocsteady::IoSelector::PointTypes,
                })
                .await
                .map(|_| ()),
            100 => session
                .read_point_types(rocsteady::PointTypeRequest::new(91, 1).unwrap())
                .await
                .map(|_| ()),
            105 => session
                .read_min_max(rocsteady::MinMaxRequest::new(0, 7).unwrap())
                .await
                .map(|_| ()),
            108 => session
                .read_history_tags(rocsteady::HistoryTagsRequest::new(0, vec![7]).unwrap())
                .await
                .map(|_| ()),
            135 => session
                .read_history(
                    rocsteady::HistoryRequest::new(0, 7, rocsteady::HistoryKind::Periodic, 0, 1)
                        .unwrap(),
                )
                .await
                .map(|_| ()),
            167 => session
                .read_parameter_block(
                    &ParameterBlock::new(103, 0, 0, vec![DataType::UInt8]).unwrap(),
                )
                .await
                .map(|_| ()),
            180 => session
                .read_parameters(&[Parameter::new(Tlp::new(103, 0, 0), DataType::UInt8)])
                .await
                .map(|_| ()),
            _ => unreachable!(),
        };
        assert!(
            matches!(result, Err(Error::InvalidResponse(_))),
            "opcode {opcode}: {result:?}"
        );
        assert!(!session.is_connected(), "opcode {opcode}");
        assert!(matches!(
            session.request(7, vec![]).await,
            Err(Error::Disconnected)
        ));
        responder.await.unwrap();
    }
}

#[tokio::test]
// POL-SESSION-REJECTION: Managed ownership contract. See docs/REQUIREMENTS.md.
async fn standalone_typed_device_rejection_keeps_session_usable() {
    let (stream, mut peer) = tokio::io::duplex(512);
    let responder = tokio::spawn(async move {
        for (opcode, data) in [(255, vec![2, 0]), (7, vec![1, 2, 3, 8, 10, 0xea, 7, 5])] {
            assert_eq!(
                read_frame(&mut peer, CrcMode::Required)
                    .await
                    .unwrap()
                    .opcode,
                7
            );
            let reply = rocsteady::protocol::Frame::new(
                Address::new(1, 0),
                Address::new(1, 2),
                opcode,
                data,
            )
            .encode()
            .unwrap();
            peer.write_all(&reply).await.unwrap();
        }
    });
    let session = Session::new(
        Box::new(stream),
        Address::new(1, 0),
        Address::new(1, 2),
        Duration::from_secs(1),
        CrcMode::Required,
    )
    .unwrap();
    assert!(matches!(session.read_clock().await, Err(Error::Device(_))));
    assert!(session.is_connected());
    assert_eq!(session.read_clock().await.unwrap().year, 2026);
    responder.await.unwrap();
}

fn config() -> ClientConfig {
    ClientConfig {
        login: Some(Login::new(*b"TST", 0x1234, None).unwrap()),
        response_timeout: Duration::from_millis(100),
        call_timeout: Duration::from_secs(3),
        heartbeat_interval: None,
        recovery: etherbird::Config {
            retry_delay: Duration::from_millis(5),
            max_retry_delay: Duration::from_millis(20),
            ..Default::default()
        },
        ..Default::default()
    }
}
fn model() -> Model {
    let mut model = Model {
        login: config().login,
        ..Default::default()
    };
    model
        .parameters
        .insert(Tlp::new(103, 16, 3), Value::Float(12.5));
    model
        .parameters
        .insert(Tlp::new(136, 0, 5), Value::UInt16(2026));
    model
        .parameters
        .insert(Tlp::new(136, 0, 6), Value::UInt8(5));
    model
}
async fn fixture() -> (Simulator, Client) {
    let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), model())
        .await
        .unwrap();
    let client = Client::tcp(server.endpoint().to_string(), config()).unwrap();
    if let Err(error) = client.connected().await {
        panic!(
            "fixture readiness: {error}; simulator {:?}; direct connect {:?}",
            server.stats(),
            tokio::net::TcpStream::connect(server.endpoint()).await
        );
    }
    (server, client)
}
async fn finish(server: Simulator, client: Client) {
    client.stop().await;
    assert_eq!(client.state(), rocsteady::ResourceState::Stopped);
    server.stop().await.unwrap();
}

#[tokio::test]
async fn configuration_variants_survive_recovery_and_device_rejections() {
    for length in [97, 98, 215] {
        let mut bytes = vec![0; length];
        bytes[0] = 1;
        bytes[1] = 2;
        bytes[5] = u8::from(length == 215);
        bytes[18] = 6;
        bytes[length - 1] = 7;
        let expected = rocsteady::SystemConfiguration::decode(&bytes).unwrap();
        let mut model = model();
        model.configuration = expected.clone();
        let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), model)
            .await
            .unwrap();
        let client = Client::tcp(server.endpoint().to_string(), config()).unwrap();
        client.connected().await.unwrap();
        assert_eq!(client.read_configuration().await.unwrap(), expected);
        server.inject(Fault::Error(vec![2, 4]));
        assert!(matches!(
            client.read_configuration().await,
            Err(etherbird::Error::Operation(Error::Device(_)))
        ));
        assert_eq!(server.stats().connections, 1);
        server.inject(Fault::Disconnect);
        assert!(client.read_configuration().await.is_err());
        assert_eq!(client.read_configuration().await.unwrap(), expected);
        assert!(server.stats().logins >= 2);
        finish(server, client).await;
    }
}

#[tokio::test]
async fn independent_tcp_peer_supplies_configuration_fixture() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = listener.local_addr().unwrap();
    let peer = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = [0; 8];
        stream.read_exact(&mut request).await.unwrap();
        assert_eq!(request, [1, 2, 1, 0, 7, 0, 0x7b, 0xdd]);
        stream
            .write_all(&[1, 0, 1, 2, 7, 8, 1, 2, 3, 8, 10, 0xea, 7, 5, 0xc3, 0x41])
            .await
            .unwrap();
        stream.read_exact(&mut request).await.unwrap();
        assert_eq!(request, [1, 2, 1, 0, 6, 0, 0x7a, 0x4d]);
        // Table 2-2 original response, independent of production codecs.
        let mut reply = [0; 106];
        reply[..13].copy_from_slice(&[1, 0, 1, 2, 6, 98, 1, 0x34, 0x12, 3, 2, 0, 1]);
        reply[24] = 6;
        reply[25] = 7;
        reply[103] = 9;
        reply[104..].copy_from_slice(&[0xbf, 0x56]);
        for chunk in reply.chunks(3) {
            stream.write_all(chunk).await.unwrap();
        }
    });
    let client = Client::tcp(
        endpoint.to_string(),
        ClientConfig {
            heartbeat_interval: None,
            ..Default::default()
        },
    )
    .unwrap();
    let configuration = client.read_configuration().await.unwrap();
    assert_eq!(configuration.communication_port(), Some(0x1234));
    assert_eq!(configuration.point_count(138), Some(9));
    client.stop().await;
    peer.await.unwrap();
}

#[tokio::test]
async fn malformed_configuration_retires_managed_session() {
    let (server, client) = fixture().await;
    let mut truncated = vec![0; 98];
    truncated[0] = 1;
    truncated[5] = 1; // Extended revision requires all counts through point 255.
    server.inject(Fault::ResponseData(truncated));
    assert!(matches!(
        client.read_configuration().await,
        Err(etherbird::Error::Operation(Error::InvalidResponse(_)))
    ));
    assert_eq!(client.read_configuration().await.unwrap().device_type(), 6);
    assert!(server.stats().logins >= 2);
    finish(server, client).await;
}

#[tokio::test]
async fn tcp_login_clock_and_both_parameter_reads_use_real_wire_channel() {
    let (server, client) = fixture().await;
    assert_eq!(server.stats().logins, 1);
    assert_eq!(client.read_clock().await.unwrap().year, 2026);
    assert_eq!(
        client
            .read_parameters(&[Parameter::new(Tlp::new(103, 16, 3), DataType::Float)])
            .await
            .unwrap(),
        [Value::Float(12.5)]
    );
    let block = ParameterBlock::new(136, 0, 5, vec![DataType::UInt16, DataType::UInt8]).unwrap();
    assert_eq!(
        client.read_parameter_block(&block).await.unwrap(),
        [Value::UInt16(2026), Value::UInt8(5)]
    );
    server.inject(Fault::Fragment(1));
    assert_eq!(client.read_clock().await.unwrap().year, 2026);
    finish(server, client).await;
}

#[tokio::test]
// POL-RECOVERY-SETUP: Managed setup contract. See docs/REQUIREMENTS.md.
async fn failures_return_once_then_new_session_logs_in_before_next_call() {
    for fault in [
        Fault::Disconnect,
        Fault::PartialThenClose(7),
        Fault::CorruptCrc,
        Fault::WrongOpcode,
        Fault::WrongAddress,
        Fault::Stall,
    ] {
        let (server, client) = fixture().await;
        let requests = server.stats().requests;
        server.inject(fault.clone());
        assert!(client.read_clock().await.is_err(), "{fault:?}");
        let after_failure = server.stats().requests;
        assert!(after_failure > requests);
        // There is no retry of the failed operation; readiness probes may run during recovery.
        assert_eq!(client.read_clock().await.unwrap().year, 2026);
        assert!(server.stats().connections >= 2);
        assert!(server.stats().logins >= 2);
        finish(server, client).await;
    }
}

#[tokio::test]
async fn valid_device_rejections_keep_the_connection_and_original_error_details() {
    let (server, client) = fixture().await;
    let err = client
        .read_parameters(&[Parameter::new(Tlp::new(103, 16, 99), DataType::Float)])
        .await
        .unwrap_err();
    assert!(
        matches!(err, etherbird::Error::Operation(Error::Device(ref errors)) if errors[0].code == 2)
    );
    assert_eq!(client.read_clock().await.unwrap().year, 2026);
    assert_eq!(server.stats().connections, 1);
    assert_eq!(server.stats().logins, 1);
    server.inject(Fault::Error(vec![2])); // malformed opcode 255 must retire the stream
    assert!(client.read_clock().await.is_err());
    assert!(client.read_clock().await.is_ok());
    assert!(server.stats().connections >= 2);
    finish(server, client).await;
}

#[tokio::test]
async fn stable_srbx_receiver_routes_interleaved_frames_across_replacement() {
    let (server, client) = fixture().await;
    let mut events = client.subscribe_events();
    server.inject(Fault::SrbxBeforeReply);
    assert!(client.read_clock().await.is_ok());
    assert_eq!(events.recv().await.unwrap().opcode, 224);
    server.inject(Fault::Disconnect);
    assert!(client.read_clock().await.is_err());
    client.connected().await.unwrap();
    server.inject(Fault::SrbxBeforeReply);
    assert!(client.read_clock().await.is_ok());
    assert_eq!(events.recv().await.unwrap().opcode, 224);
    finish(server, client).await;
}

#[tokio::test]
// POL-CANCELLATION: Managed ownership contract. See docs/REQUIREMENTS.md.
async fn timeout_or_cancellation_discards_late_response_and_does_not_contaminate_next_call() {
    let (server, client) = fixture().await;
    server.inject(Fault::Delay(Duration::from_millis(300)));
    let task_client = client.clone();
    let task = tokio::spawn(async move { task_client.read_clock().await });
    let target = server.stats().requests + 1;
    tokio::time::timeout(Duration::from_secs(2), async {
        while server.stats().requests < target {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert_eq!(client.read_clock().await.unwrap().year, 2026);
    assert!(server.stats().logins >= 2);
    finish(server, client).await;
}

#[tokio::test]
async fn shutdown_cancels_running_and_queued_calls_and_clones_remain_stopped() {
    let (server, client) = fixture().await;
    server.inject(Fault::Stall);
    let first = client.clone();
    let running = tokio::spawn(async move { first.read_clock().await });
    let second = client.clone();
    let queued = tokio::spawn(async move { second.read_clock().await });
    tokio::task::yield_now().await;
    client.stop().await;
    assert!(running.await.unwrap().is_err());
    assert!(queued.await.unwrap().is_err());
    assert!(matches!(
        client.read_clock().await,
        Err(etherbird::Error::Stopped)
    ));
    server.stop().await.unwrap();
}

#[tokio::test]
async fn heartbeats_detect_idle_failure_and_restore_authentication() {
    let mut cfg = config();
    cfg.heartbeat_interval = Some(Duration::from_millis(10));
    let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), model())
        .await
        .unwrap();
    let client = Client::tcp(server.endpoint().to_string(), cfg).unwrap();
    client.connected().await.unwrap();
    server.inject(Fault::Disconnect);
    tokio::time::timeout(Duration::from_secs(3), async {
        while server.stats().logins < 2 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    assert!(client.read_clock().await.is_ok());
    finish(server, client).await;
}

#[tokio::test]
async fn rejected_setup_is_terminal_never_ready_and_not_retried() {
    let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), model())
        .await
        .unwrap();
    let mut cfg = config();
    cfg.login = Some(Login::new(*b"BAD", 1, None).unwrap());
    cfg.call_timeout = Duration::from_millis(50);
    let client = Client::tcp(server.endpoint().to_string(), cfg).unwrap();
    assert!(matches!(
        client.connected().await,
        Err(etherbird::Error::Operation(Error::Authentication(
            rocsteady::AuthenticationFailure::Rejected
        )))
    ));
    assert_ne!(client.state(), rocsteady::ResourceState::Connected);
    assert_eq!(server.stats().logins, 0);
    let attempts = server.stats().connections;
    tokio::time::sleep(Duration::from_millis(40)).await;
    assert_eq!(server.stats().connections, attempts);
    assert!(matches!(
        client.clone().read_clock().await,
        Err(etherbird::Error::Operation(Error::Authentication(
            rocsteady::AuthenticationFailure::Rejected
        )))
    ));
    finish(server, client).await;
}

struct OnceConnector(Mutex<Option<Stream>>);
#[etherbird::async_trait]
impl Connector for OnceConnector {
    async fn connect(&self) -> Result<Stream, Error> {
        self.0.lock().unwrap().take().ok_or(Error::Disconnected)
    }
}
#[tokio::test]
async fn injectable_connector_runs_identical_protocol_over_memory_stream() {
    let (client_stream, mut server_stream) = tokio::io::duplex(64);
    let model = Model::default();
    let expected = model.clock;
    let server = tokio::spawn(async move {
        rocsteady::simulator::serve_stream(&mut server_stream, &model).await
    });
    let mut cfg = config();
    cfg.login = None;
    let client = Client::new(
        Arc::new(OnceConnector(Mutex::new(Some(Box::new(client_stream))))),
        cfg,
    )
    .unwrap();
    assert_eq!(client.read_clock().await.unwrap(), expected);
    client.stop().await;
    server.await.unwrap().unwrap();
}

#[tokio::test]
async fn independent_peer_checks_literal_request_and_supplies_clock_response() {
    // Prevent a client+simulator sharing a codec bug from being the only integration oracle.
    let (client_stream, mut peer) = tokio::io::duplex(64);
    let server = tokio::spawn(async move {
        let mut request = [0; 8];
        peer.read_exact(&mut request).await.unwrap();
        assert_eq!(request, [13, 5, 1, 0, 7, 0, 0xce, 0xd1]);
        // Independently fixed fixture bytes; peer uses no rocsteady codec.
        peer.write_all(&[1, 0, 13, 5, 7, 8, 1, 2, 3, 8, 10, 0xea, 7, 5, 0xc9, 0x25])
            .await
            .unwrap();
    });
    let session = Session::new(
        Box::new(client_stream),
        Address::new(1, 0),
        Address::new(13, 5),
        Duration::from_secs(1),
        CrcMode::Required,
    )
    .unwrap();
    assert_eq!(
        session.read_clock().await.unwrap(),
        Clock {
            second: 1,
            minute: 2,
            hour: 3,
            day: 8,
            month: 10,
            year: 2026,
            weekday: 5
        }
    );
    server.await.unwrap();
}

#[tokio::test]
async fn coalesced_frames_remain_separate_on_the_stream() {
    let (mut stream, mut peer) = tokio::io::duplex(64);
    let bytes = [1, 0, 1, 2, 224, 0, 232, 45];
    let task = tokio::spawn(async move {
        let mut both = bytes.to_vec();
        both.extend(bytes);
        peer.write_all(&both).await.unwrap();
    });
    assert_eq!(
        read_frame(&mut stream, CrcMode::Required)
            .await
            .unwrap()
            .opcode,
        224
    );
    assert_eq!(
        read_frame(&mut stream, CrcMode::Required)
            .await
            .unwrap()
            .opcode,
        224
    );
    task.await.unwrap();
}

#[tokio::test]
async fn bounded_queue_rejects_excess_work_and_invalid_requests_do_not_reconnect() {
    let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), model())
        .await
        .unwrap();
    let mut cfg = config();
    cfg.queue_capacity = 1;
    let client = Client::tcp(server.endpoint().to_string(), cfg).unwrap();
    client.connected().await.unwrap();
    let before = server.stats().requests;
    assert!(client.read_parameters(&[]).await.is_err());
    assert!(client.request(24, vec![]).await.is_err());
    assert_eq!(server.stats().requests, before);
    assert_eq!(server.stats().connections, 1);
    server.inject(Fault::Stall);
    let first = client.clone();
    let running = tokio::spawn(async move { first.read_clock().await });
    tokio::time::timeout(Duration::from_secs(1), async {
        while server.stats().requests == before {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let second = client.clone();
    let queued = tokio::spawn(async move { second.read_clock().await });
    tokio::task::yield_now().await;
    assert!(matches!(
        client.read_clock().await,
        Err(etherbird::Error::Queue(etherbird::QueueError::Full))
    ));
    client.stop().await;
    assert!(running.await.unwrap().is_err());
    assert!(queued.await.unwrap().is_err());
    server.stop().await.unwrap();
}

#[tokio::test]
// POL-NO-REPLAY: Managed operation contract. See docs/REQUIREMENTS.md.
async fn failed_request_is_not_replayed_and_concurrent_calls_are_serialized() {
    let (server, client) = fixture().await;
    // Raw opcode 8 is not a supported simulator operation. Its failed attempt
    // must not be replayed or force healthy connection replacement.
    let before = server.stats().requests;
    let result = client.request(8, vec![0; 8]).await;
    assert!(matches!(
        result,
        Err(etherbird::Error::Operation(Error::Device(_)))
    ));
    assert_eq!(server.stats().requests, before + 1);
    assert_eq!(server.stats().connections, 1);
    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..16 {
        let client = client.clone();
        tasks.spawn(async move { client.read_clock().await });
    }
    while let Some(result) = tasks.join_next().await {
        assert_eq!(result.unwrap().unwrap().year, 2026);
    }
    assert_eq!(server.stats().connections, 1);
    finish(server, client).await;
}
