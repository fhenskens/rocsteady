//! Bounded logs, caller-confirmed consumption, idle polling and explicit acknowledgement.
include!("support/discovery_fixture.rs");
include!("support/log_literals.rs");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rocsteady::{Client, ClientConfig};
    let peer = DiscoveryFixture::start(0).await;
    peer.peer
        .alarm_records
        .lock()
        .unwrap()
        .extend((0..6).map(|i| (i, alarm_literal(i))));
    peer.peer
        .event_records
        .lock()
        .unwrap()
        .extend((0..9).map(|i| (i, event_literal(i))));
    let client = Client::tcp(
        &peer.endpoint,
        ClientConfig {
            heartbeat_interval: None,
            ..Default::default()
        },
    )?;
    peer.peer.current_log_index.store(5, Ordering::SeqCst);
    peer.peer.srbx_pending.store(true, Ordering::SeqCst);
    let mut poller = client.srbx_poller(std::time::Duration::from_millis(100))?;
    client.connected().await?;
    // A real unsolicited signal is emitted while the client is otherwise idle.
    peer.peer.idle_signals.store(1, Ordering::SeqCst);
    peer.peer.signal_wake.notify_one();
    let result = async {
        let index = poller.poll().await?;
        println!(
            "reported alarm index: {} (not a durable sequence)",
            index.current_alarm_index
        );
        // The application chooses a known slot window; no last/newest-index convention is inferred.
        let alarms = client.read_alarm_range(1, 2).await?;
        for record in alarms.records() {
            println!("alarm at time {}: {:?}", record.timestamp(), record.data());
        }
        // Persist/advance only after processing these records successfully.
        let consumed = alarms.checkpoint(2)?;
        let next = client.resume_alarms(consumed, 3).await?;
        println!(
            "resumed alarms: {}, anchor verified: {}",
            next.records().count(),
            next.anchor_verified
        );
        let events = client.read_event_range(0, 9).await?;
        for record in events.records() {
            println!("event at time {}: {:?}", record.timestamp(), record.data());
        }
        for record in next.records() {
            println!("consume remaining alarm: {:?}", record.data());
        }
        // This fixture consumer processed all records through slot 5. A real
        // application commits durable ingestion before creating this checkpoint.
        let consumed = next.checkpoint(3)?;
        let acknowledgement = client.acknowledge_alarms(&consumed).await?;
        println!(
            "acknowledgement accepted at index {} (receipt does not prove SRBX cleared)",
            acknowledgement.index
        );
        assert!(!peer.peer.srbx_pending.load(Ordering::SeqCst));
        // A changed anchor requires a new explicit window; failures never replay acknowledgement.
        Ok::<_, Box<dyn std::error::Error>>(())
    }
    .await;
    client.stop().await;
    result
}
