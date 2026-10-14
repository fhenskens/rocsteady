#![cfg(feature = "simulator")]
include!("../examples/support/log_literals.rs");
use rocsteady::{
    Address, AlarmRecord, Client, ClientConfig, EventRecord, LogRecord, LogRequest,
    protocol::Frame,
    simulator::{Model, Simulator},
};

#[tokio::test]
async fn simulator_returns_empty_markers_index_only_and_every_record_kind() {
    let mut model = Model::default();
    model
        .alarm_log
        .extend((0..6).map(|i| (i, alarm_literal(i))));
    model
        .event_log
        .extend((0..9).map(|i| (i, event_literal(i))));
    model.alarm_index = 5;
    model.event_index = 8;
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
    let alarms = client.read_alarm_range(0, 6).await.unwrap();
    for (i, r) in alarms.records().enumerate() {
        assert_eq!(r.raw(), &alarm_literal(i as u16));
    }
    let events = client.read_event_range(0, 9).await.unwrap();
    for (i, r) in events.records().enumerate() {
        assert_eq!(r.raw(), &event_literal(i as u16));
    }
    assert_eq!(events.pages[0].current_index(), 8);
    let empty = client
        .read_alarms(LogRequest::new(400, 1).unwrap())
        .await
        .unwrap();
    assert!(empty.records()[0].is_empty());
    assert!(empty.checkpoint(1).is_err());
    let index = client
        .read_events(LogRequest::new(449, 0).unwrap())
        .await
        .unwrap();
    assert!(index.records().is_empty());
    assert_eq!(index.current_index(), 8);
    let checkpoint = alarms.checkpoint(2).unwrap();
    assert!(
        client
            .resume_alarms(checkpoint, 3)
            .await
            .unwrap()
            .anchor_verified
    );
    for (opcode, payload) in [
        (118, vec![11, 0, 0]),
        (119, vec![2, 0xc1, 1]),
        (118, vec![1, 0xc2, 1]),
        (119, vec![0]),
    ] {
        assert!(matches!(
            client.request(opcode, payload).await,
            Err(etherbird::Error::Operation(rocsteady::Error::Device(_)))
        ));
    }
    assert_eq!(server.stats().connections, 1);
    client.stop().await;
    server.stop().await.unwrap();
}
#[test]
fn simulator_log_append_is_bounded_rolls_over_and_overwrites_old_slots() {
    let mut model = Model::default();
    let first = alarm_literal(1);
    assert_eq!(model.append_alarm(first).unwrap(), 0);
    for i in 1..=450u32 {
        let mut bytes = alarm_literal(1);
        bytes[1..5].copy_from_slice(&i.to_le_bytes());
        model.append_alarm(bytes).unwrap();
        let mut event = event_literal(1);
        event[1..5].copy_from_slice(&i.to_le_bytes());
        model.append_event(event).unwrap();
    }
    assert_eq!(model.alarm_log.len(), 450);
    assert_eq!(model.event_log.len(), 450);
    assert_eq!(model.alarm_index, 0);
    assert_eq!(model.event_index, 449);
    assert_ne!(model.alarm_log[&0], first);
    assert_eq!(
        AlarmRecord::decode(&model.alarm_log[&0])
            .unwrap()
            .timestamp(),
        450
    );
    assert_eq!(
        EventRecord::decode(&model.event_log[&449])
            .unwrap()
            .timestamp(),
        450
    );
    let response = model
        .respond(
            &Frame::new(Address::new(1, 2), Address::new(1, 0), 118, vec![1, 0, 0]),
            &mut true,
        )
        .unwrap();
    assert_eq!(&response.data[..5], &[1, 0, 0, 0, 0]);
    model.alarm_index = 450;
    assert!(model.append_alarm(first).is_err());
}
