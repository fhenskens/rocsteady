#![cfg(feature = "simulator")]
include!("../examples/support/log_literals.rs");
use rocsteady::{
    AlarmAcknowledgementOutcome as Outcome, Client, ClientConfig, LogRequest,
    simulator::{Fault, Model, Simulator},
};
use std::time::Duration;
fn model() -> Model {
    let mut model = Model {
        alarm_index: 7,
        ..Default::default()
    };
    model.alarm_log.insert(7, alarm_literal(7));
    model.event_log.insert(1, event_literal(1)); // Independent event fixture remains usable.
    model
}
#[tokio::test]
async fn simulator_emits_while_idle_and_models_consumed_index_comparison() {
    let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), model())
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
    let mut poller = client.srbx_poller(Duration::from_millis(1)).unwrap();
    client.connected().await.unwrap();
    server.raise_srbx(7).unwrap();
    tokio::time::timeout(Duration::from_secs(3), async {
        while server.srbx_status().signals_sent == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(server.stats().requests, 1); // Signal was sent without a polling or business request.
    let poll = poller.poll().await.unwrap();
    assert_eq!(poll.current_alarm_index, 7);
    assert_eq!(poll.notifications.signals.len(), 1);
    let consumed = client
        .read_alarms(LogRequest::new(7, 1).unwrap())
        .await
        .unwrap()
        .checkpoint(1)
        .unwrap();
    server.raise_srbx(8).unwrap();
    client.acknowledge_alarms(&consumed).await.unwrap();
    assert!(server.srbx_status().pending);
    assert_eq!(server.srbx_status().last_acknowledged, Some(7));
    server.raise_srbx(7).unwrap();
    client.acknowledge_alarms(&consumed).await.unwrap();
    assert!(!server.srbx_status().pending);
    assert_eq!(server.srbx_status().acknowledgements, 2);
    client.stop().await;
    server.stop().await.unwrap();
}
#[tokio::test]
async fn lost_acknowledgement_applies_effect_and_recovers_without_replay() {
    let mut model = model();
    model.initial_srbx_pending = true;
    let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), model)
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
    let consumed = client
        .read_alarms(LogRequest::new(7, 1).unwrap())
        .await
        .unwrap()
        .checkpoint(1)
        .unwrap();
    server.inject(Fault::LoseAlarmAcknowledgement);
    let error = client.acknowledge_alarms(&consumed).await.unwrap_err();
    assert_eq!(error.outcome, Outcome::Unconfirmed);
    assert!(!server.srbx_status().pending);
    assert_eq!(server.srbx_status().acknowledgements, 1);
    client.read_clock().await.unwrap();
    assert_eq!(server.stats().connections, 2);
    assert_eq!(server.srbx_status().acknowledgements, 1);
    client.stop().await;
    server.stop().await.unwrap();
}
#[tokio::test]
async fn rollover_acknowledges_only_consumed_slot_and_new_slot_must_be_consumed_explicitly() {
    let mut model = model();
    model.alarm_index = 449;
    model.alarm_log.insert(449, alarm_literal(1));
    model.alarm_log.insert(0, alarm_literal(4));
    let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), model)
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
    let last = client
        .read_alarms(LogRequest::new(449, 1).unwrap())
        .await
        .unwrap()
        .checkpoint(1)
        .unwrap();
    server.raise_srbx(0).unwrap();
    client.acknowledge_alarms(&last).await.unwrap();
    assert!(server.srbx_status().pending);
    let first = client
        .read_alarms(LogRequest::new(0, 1).unwrap())
        .await
        .unwrap()
        .checkpoint(1)
        .unwrap();
    client.acknowledge_alarms(&first).await.unwrap();
    assert!(!server.srbx_status().pending);
    assert_eq!(server.srbx_status().last_acknowledged, Some(0));
    client.stop().await;
    server.stop().await.unwrap();
}
#[tokio::test]
async fn idle_notification_does_not_cancel_a_partially_read_request_in_the_simulator() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), model())
        .await
        .unwrap();
    let mut stream = tokio::net::TcpStream::connect(server.endpoint())
        .await
        .unwrap();
    let bytes = rocsteady::protocol::Frame::new(
        rocsteady::Address::new(1, 2),
        rocsteady::Address::new(1, 0),
        7,
        vec![],
    )
    .encode()
    .unwrap();
    tokio::time::timeout(Duration::from_secs(3), async {
        while server.stats().connections == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    stream.write_all(&bytes[..3]).await.unwrap();
    tokio::task::yield_now().await;
    server.raise_srbx(7).unwrap();
    let signal =
        rocsteady::transport::read_frame(&mut stream, rocsteady::protocol::CrcMode::Required)
            .await
            .unwrap();
    assert_eq!(signal.opcode, 224);
    stream.write_all(&bytes[3..]).await.unwrap();
    let response =
        rocsteady::transport::read_frame(&mut stream, rocsteady::protocol::CrcMode::Required)
            .await
            .unwrap();
    assert_eq!(response.opcode, 7);
    assert_eq!(server.stats().requests, 1);
    stream.shutdown().await.unwrap();
    let mut tail = Vec::new();
    stream.read_to_end(&mut tail).await.unwrap();
    server.stop().await.unwrap();
}
