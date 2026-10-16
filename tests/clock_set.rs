//! D301180X012 October 2022, §2.4/Table 2-4, printed p.2-11.
use rocsteady::*;
#[cfg(feature = "simulator")]
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[test]
fn clock_set_calendar_and_seven_byte_wire_contract() {
    // REQ-008-WIRE: seconds/minutes/hour/day/month/UINT16 year, no weekday.
    let q = ClockSet::new(2024, 2, 29, 23, 58, 59).unwrap();
    assert_eq!(q.payload(), [59, 58, 23, 29, 2, 0xe8, 7]);
    assert_eq!(q.encode().unwrap(), (8, vec![59, 58, 23, 29, 2, 0xe8, 7]));
    assert!(q.decode(&[]).is_ok());
    assert!(q.decode(&[0]).is_err());
    // POL-008-CALENDAR: Gregorian bounds follow existing clock-read policy;
    // year width is not proof of hardware acceptance of every year.
    for (year, month, day, valid) in [
        (0, 1, 1, false),
        (1, 1, 1, true),
        (65535, 12, 31, true),
        (1900, 2, 29, false),
        (2000, 2, 29, true),
        (2100, 2, 29, false),
        (2024, 2, 29, true),
        (2025, 2, 29, false),
        (2024, 0, 1, false),
        (2024, 13, 1, false),
        (2024, 4, 31, false),
        (2024, 1, 0, false),
        (2024, 1, 32, false),
    ] {
        assert_eq!(ClockSet::new(year, month, day, 0, 0, 0).is_ok(), valid);
    }
    for (hour, minute, second) in [(24, 0, 0), (0, 60, 0), (0, 0, 60)] {
        assert!(matches!(
            ClockSet::new(2024, 1, 1, hour, minute, second),
            Err(Error::InvalidRequest(_))
        ));
    }
}
fn crc(b: &[u8]) -> u16 {
    let mut c = 0u16;
    for b in b {
        c ^= u16::from(*b);
        for _ in 0..8 {
            c = if c & 1 == 0 {
                c >> 1
            } else {
                (c >> 1) ^ 0xa001
            };
        }
    }
    c
}
#[tokio::test]
async fn clock_set_independent_tcp_ack_errors_and_standalone_retirement() {
    // REQ-008-WIRE / REQ-008-ACK: independent TCP peer checks literal bytes;
    // POL-008-ERROR: raw device rejection keeps channel, malformed ack retires it.
    for managed in [false, true] {
        for fault in [0, 1, 2] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let endpoint = listener.local_addr().unwrap();
            let task = tokio::spawn(async move {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut requests = 0;
                loop {
                    let mut h = [0; 6];
                    if socket.read_exact(&mut h).await.is_err() {
                        break;
                    }
                    let mut b = vec![0; usize::from(h[5]) + 2];
                    socket.read_exact(&mut b).await.unwrap();
                    let mut wire = h.to_vec();
                    wire.extend(&b[..b.len() - 2]);
                    assert_eq!(crc(&wire).to_le_bytes(), b[b.len() - 2..]);
                    let (opcode, payload) = if h[4] == 8 {
                        assert_eq!(&h[..], &[1, 2, 1, 0, 8, 7]);
                        assert_eq!(&b[..7], &[59, 58, 23, 29, 2, 0xe8, 7]);
                        requests += 1;
                        match fault {
                            1 => (8, vec![0]),
                            2 => (255, vec![4, 6, 21, 7]),
                            _ => (8, vec![]),
                        }
                    } else {
                        assert_eq!(h[4], 7);
                        (7, vec![1, 2, 3, 8, 10, 0xea, 7, 5])
                    };
                    let mut response = vec![1, 0, 1, 2, opcode, payload.len() as u8];
                    response.extend(payload);
                    response.extend(crc(&response).to_le_bytes());
                    socket.write_all(&response).await.unwrap();
                    if fault == 1 && requests > 0 {
                        break;
                    }
                }
                requests
            });
            let q = ClockSet::new(2024, 2, 29, 23, 58, 59).unwrap();
            if managed {
                let client = Client::tcp(
                    endpoint.to_string(),
                    ClientConfig {
                        heartbeat_interval: None,
                        ..Default::default()
                    },
                )
                .unwrap();
                let result = client.set_clock(q).await;
                if fault == 0 {
                    assert_eq!(result.unwrap(), WriteOutcome::Acknowledged);
                } else {
                    let error = result.unwrap_err();
                    assert_eq!(
                        error.outcome,
                        if fault == 1 {
                            WriteOutcome::Uncertain
                        } else {
                            WriteOutcome::DeviceError
                        }
                    );
                    if fault == 2 {
                        let etherbird::Error::Operation(Error::Device(errors)) = error.error else {
                            panic!("device errors required")
                        };
                        assert_eq!(
                            errors
                                .iter()
                                .map(|e| (e.code, e.offset))
                                .collect::<Vec<_>>(),
                            [(4, 6), (21, 7)]
                        );
                    }
                }
                if fault != 1 {
                    client.read_clock().await.unwrap();
                }
                client.stop().await;
            } else {
                let stream = tokio::net::TcpStream::connect(endpoint).await.unwrap();
                let session = Session::new(
                    Box::new(stream),
                    Address::new(1, 0),
                    Address::new(1, 2),
                    Duration::from_secs(1),
                    protocol::CrcMode::Required,
                )
                .unwrap();
                assert_eq!(session.set_clock(q).await.is_ok(), fault == 0);
                assert_eq!(session.is_connected(), fault != 1);
                session.disconnect().await;
            }
            assert_eq!(task.await.unwrap(), 1);
        }
    }
}

#[cfg(feature = "simulator")]
#[tokio::test]
async fn clock_set_effect_readback_loss_deadline_and_active_cancellation() {
    // REQ-008-SIMULATOR / POL-008-NO-REPLAY: effects precede injected response
    // failures. Read-back is a new explicit call, not a replay or rollback claim.
    use simulator::{Fault, Model, Simulator};
    for fault in [0, 1, 2, 3] {
        let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), Model::default())
            .await
            .unwrap();
        let client = Client::tcp(
            server.endpoint().to_string(),
            ClientConfig {
                heartbeat_interval: None,
                call_timeout: Duration::from_millis(100),
                ..Default::default()
            },
        )
        .unwrap();
        client.connected().await.unwrap();
        if fault == 1 {
            server.inject(Fault::Disconnect);
        }
        if fault >= 2 {
            server.inject(Fault::Stall);
        }
        let before = server.stats().requests;
        let q = ClockSet::new(2000, 2, 29, 23, 58, 59).unwrap();
        if fault == 3 {
            // POL-008-CANCEL: abort only after dispatch; remote effect remains and
            // subsequent work uses a replacement connection, never another OP-8.
            let c = client.clone();
            let task = tokio::spawn(async move { c.set_clock(q).await });
            tokio::time::timeout(Duration::from_secs(2), async {
                while server.stats().requests == before {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
            task.abort();
            assert!(task.await.unwrap_err().is_cancelled());
        } else if fault == 0 {
            assert_eq!(
                client.set_clock(q).await.unwrap(),
                WriteOutcome::Acknowledged
            );
        } else {
            assert_eq!(
                client.set_clock(q).await.unwrap_err().outcome,
                WriteOutcome::Uncertain
            );
        }
        if fault != 0 {
            let mut states = client.subscribe_state();
            tokio::time::timeout(Duration::from_secs(5), async {
                loop {
                    states.borrow_and_update();
                    if server.stats().connections >= 2 && client.state() == ResourceState::Connected
                    {
                        break;
                    }
                    states.changed().await.unwrap();
                }
            })
            .await
            .unwrap();
        }
        let clock = client.read_clock().await.unwrap();
        assert_eq!(
            (
                clock.year,
                clock.month,
                clock.day,
                clock.hour,
                clock.minute,
                clock.second,
                clock.weekday
            ),
            (2000, 2, 29, 23, 58, 59, 3)
        );
        assert_eq!(
            server.stats().requests - before,
            if fault == 0 { 2 } else { 3 }
        );
        client.stop().await;
        server.stop().await.unwrap();
    }
}

#[cfg(feature = "simulator")]
#[tokio::test]
async fn clock_set_queued_cancellation_and_profile_rejection_never_send() {
    // POL-008-BEFORE-SEND / POL-008-CANCEL: reuse the synchronized write send gate;
    // cancellation while queued must not permit a later clock change.
    use simulator::{Model, Simulator};
    use std::future::Future;
    struct Hold(Arc<tokio::sync::Notify>, Arc<tokio::sync::Notify>);
    #[etherbird::async_trait]
    impl Workflow for Hold {
        type Output = ();
        async fn run(self, _: &mut ManagedContext<'_>) -> Result<(), Error> {
            self.0.notify_one();
            self.1.notified().await;
            Ok(())
        }
    }
    let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), Model::default())
        .await
        .unwrap();
    let client = Client::tcp(
        server.endpoint().to_string(),
        ClientConfig {
            heartbeat_interval: None,
            ..Default::default()
        },
    )
    .unwrap();
    let entered = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let c = client.clone();
    let hold = Hold(entered.clone(), release.clone());
    let holder = tokio::spawn(async move { c.workflow(hold).await });
    entered.notified().await;
    let mut future = Box::pin(client.set_clock(ClockSet::new(2024, 1, 1, 0, 0, 0).unwrap()));
    std::future::poll_fn(|cx| {
        assert!(future.as_mut().poll(cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    drop(future);
    release.notify_one();
    holder.await.unwrap().unwrap();
    client.read_clock().await.unwrap();
    assert!(server.model().written_clock.lock().unwrap().is_none());
    client.stop().await;
    struct Blocked;
    impl ProtocolProfile for Blocked {
        fn metadata(&self) -> ProfileMetadata {
            DefaultProfile.metadata()
        }
        fn capability(&self, op: u8) -> Capability {
            if op == 8 {
                Capability::Unavailable
            } else {
                DefaultProfile.capability(op)
            }
        }
    }
    let client = Client::tcp(
        server.endpoint().to_string(),
        ClientConfig {
            profile: Arc::new(Blocked),
            heartbeat_interval: None,
            ..Default::default()
        },
    )
    .unwrap();
    let before = server.stats().requests;
    assert_eq!(
        client
            .set_clock(ClockSet::new(2024, 1, 1, 0, 0, 0).unwrap())
            .await
            .unwrap_err()
            .outcome,
        WriteOutcome::NotAttempted
    );
    assert_eq!(server.stats().requests, before);
    client.stop().await;
    server.stop().await.unwrap();
}

#[cfg(feature = "simulator")]
#[test]
fn clock_set_simulator_rejects_invalid_payload_without_effect() {
    // REQ-008-SIMULATOR: exactly seven request bytes and a valid calendar.
    use simulator::Model;
    let model = Model::default();
    let mut auth = false;
    for payload in [
        vec![],
        vec![0; 6],
        vec![0; 8],
        vec![0, 0, 0, 29, 2, 0x6c, 7],
    ] {
        let reply = model
            .respond(
                &protocol::Frame::new(Address::new(1, 2), Address::new(1, 0), 8, payload),
                &mut auth,
            )
            .unwrap();
        assert_eq!(reply.opcode, 255);
        assert!(model.written_clock.lock().unwrap().is_none());
    }
}

#[cfg(feature = "simulator")]
#[tokio::test]
async fn clock_set_exact_request_budget_and_active_stop_have_explicit_outcomes() {
    // POL-008-BUDGET / POL-008-BEFORE-SEND: all seven request bytes must fit;
    // a rejected plan never connects or changes the fixture clock.
    // POL-008-CANCEL / POL-008-NO-REPLAY: awaited stop after dispatch leaves
    // the once-only remote effect and reports uncertainty, without replacement.
    use simulator::{Fault, Model, Simulator};
    struct Budget(usize);
    impl ProtocolProfile for Budget {
        fn metadata(&self) -> ProfileMetadata {
            DefaultProfile.metadata()
        }
        fn capability(&self, op: u8) -> Capability {
            DefaultProfile.capability(op)
        }
        fn limits(&self, op: u8) -> PayloadLimits {
            if op == 8 {
                PayloadLimits {
                    request: self.0,
                    response: 0,
                }
            } else {
                PayloadLimits::default()
            }
        }
    }
    for limit in [6, 7] {
        let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), Model::default())
            .await
            .unwrap();
        let client = Client::tcp(
            server.endpoint().to_string(),
            ClientConfig {
                profile: Arc::new(Budget(limit)),
                heartbeat_interval: None,
                ..Default::default()
            },
        )
        .unwrap();
        let q = ClockSet::new(2024, 1, 1, 0, 0, 0).unwrap();
        if limit == 6 {
            assert_eq!(
                client.set_clock(q).await.unwrap_err().outcome,
                WriteOutcome::NotAttempted
            );
            assert_eq!(server.stats().connections, 0);
            assert_eq!(server.stats().requests, 0);
            assert!(server.model().written_clock.lock().unwrap().is_none());
        } else {
            assert_eq!(
                client.set_clock(q).await.unwrap(),
                WriteOutcome::Acknowledged
            );
            server.inject_for(8, Fault::Stall);
            let before = server.stats().requests;
            let c = client.clone();
            let replacement = ClockSet::new(2025, 12, 31, 23, 59, 59).unwrap();
            let task = tokio::spawn(async move { c.set_clock(replacement).await });
            tokio::time::timeout(Duration::from_secs(2), async {
                while server.stats().requests == before {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
            client.stop().await;
            assert_eq!(
                task.await.unwrap().unwrap_err().outcome,
                WriteOutcome::Uncertain
            );
            assert_eq!(server.stats().connections, 1);
            assert_eq!(server.stats().requests, before + 1);
            let stored = server.model().written_clock.lock().unwrap().unwrap();
            assert_eq!((stored.year, stored.month, stored.day), (2025, 12, 31));
        }
        client.stop().await;
        server.stop().await.unwrap();
    }
}

#[cfg(feature = "simulator")]
#[test]
fn clock_set_simulator_weekday_literals_cover_month_and_century_boundaries() {
    // REQ-008-SIMULATOR: controller-computed weekday has no request byte.
    // These independent Gregorian dates exercise January/February correction,
    // all months, leap day and non-leap/leap centuries in the explicit fixture.
    use simulator::Model;
    let model = Model::default();
    let mut auth = false;
    for (year, month, day, weekday) in [
        (1900, 1, 1, 2),
        (2000, 1, 1, 7),
        (2000, 2, 29, 3),
        (2024, 1, 1, 2),
        (2024, 2, 1, 5),
        (2024, 3, 1, 6),
        (2024, 4, 1, 2),
        (2024, 5, 1, 4),
        (2024, 6, 1, 7),
        (2024, 7, 1, 2),
        (2024, 8, 1, 5),
        (2024, 9, 1, 1),
        (2024, 10, 1, 3),
        (2024, 11, 1, 6),
        (2024, 12, 1, 1),
        (2100, 3, 1, 2),
    ] {
        let q = ClockSet::new(year, month, day, 0, 0, 0).unwrap();
        let reply = model
            .respond(
                &protocol::Frame::new(
                    Address::new(1, 2),
                    Address::new(1, 0),
                    8,
                    q.payload().to_vec(),
                ),
                &mut auth,
            )
            .unwrap();
        assert_eq!(reply.opcode, 8);
        assert!(reply.data.is_empty());
        assert_eq!(
            model.written_clock.lock().unwrap().unwrap().weekday,
            weekday,
            "{year}-{month}-{day}"
        );
    }
}
