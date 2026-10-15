//! Executable evidence for the example's explicit, caller-owned polling policy.
include!("../examples/support/discovery_fixture.rs");
#[path = "../examples/support/monitoring.rs"]
mod monitoring;
use rocsteady::{Client, ClientConfig, FirmwareVersion, IoLayout, Roc800Profile, Tlp, Value};
fn client(peer: &DiscoveryFixture) -> Client {
    Client::tcp(
        &peer.endpoint,
        ClientConfig {
            profile: Arc::new(Roc800Profile {
                layout: IoLayout::Compatibility160,
            }),
            heartbeat_interval: None,
            recovery: etherbird::Config {
                retry_delay: std::time::Duration::from_millis(1),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap()
}
#[tokio::test]
async fn polling_reports_one_failure_then_explicitly_rediscovers_changed_locations() {
    // POL-MONITOR-REDISCOVER / POL-MONITOR-NO-REPLAY: each caller invocation is one new sample.
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer);
    let target = [monitoring::MonitorTarget {
        point_type: 108,
        logical_number: 0,
        parameters: vec![19, 35, 50],
    }];
    let first = monitoring::poll_once(&client, FirmwareVersion::new(3, 60).unwrap(), &target)
        .await
        .unwrap();
    assert_eq!(
        first.tlps,
        [
            Tlp::new(108, 128, 19),
            Tlp::new(108, 128, 35),
            Tlp::new(108, 128, 50)
        ]
    );
    assert_eq!(first.values, vec![Value::Float(12.5); 3]);
    let reads = peer.peer.parameter_reads.load(Ordering::SeqCst);
    peer.peer
        .parameter_fault_at
        .store(reads + 1, Ordering::SeqCst);
    peer.peer.parameter_fault.store(4, Ordering::SeqCst);
    let failure = monitoring::poll_once(&client, FirmwareVersion::new(3, 60).unwrap(), &target)
        .await
        .unwrap_err();
    assert_eq!(failure.progress.completed_batches, 0);
    assert!(failure.progress.values.is_empty());
    // A real controller replacement can move physical positions. The next explicit
    // call must re-discover and resolve logical zero to this new location.
    peer.peer.io_positions.lock().unwrap().insert(108, 129);
    let next = monitoring::poll_once(&client, FirmwareVersion::new(3, 60).unwrap(), &target)
        .await
        .unwrap();
    assert_ne!(first.generation, next.generation);
    assert_eq!(
        next.tlps,
        [
            Tlp::new(108, 129, 19),
            Tlp::new(108, 129, 35),
            Tlp::new(108, 129, 50)
        ]
    );
    assert_eq!(peer.peer.parameter_reads.load(Ordering::SeqCst), reads + 2);
    assert_eq!(peer.peer.config_queries.load(Ordering::SeqCst), 6); // Every call explicitly discovers twice.
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
    client.stop().await;
}
#[tokio::test]
async fn polling_retains_partial_samples_and_validates_empty_targets_before_io() {
    // POL-MONITOR-PROGRESS: failures expose confirmed prefix; no internal second attempt.
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer);
    assert!(
        monitoring::poll_once(&client, FirmwareVersion::new(3, 60).unwrap(), &[])
            .await
            .is_err()
    );
    assert_eq!(peer.peer.config_queries.load(Ordering::SeqCst), 0);
    let targets = [monitoring::MonitorTarget {
        point_type: 108,
        logical_number: 0,
        parameters: (0..67).collect(),
    }];
    peer.peer.parameter_fault_at.store(2, Ordering::SeqCst);
    peer.peer.parameter_fault.store(1, Ordering::SeqCst);
    let failure = monitoring::poll_once(&client, FirmwareVersion::new(3, 60).unwrap(), &targets)
        .await
        .unwrap_err();
    assert_eq!(failure.progress.completed_batches, 1);
    assert!(!failure.progress.values.is_empty());
    client.read_clock().await.unwrap();
    assert_eq!(peer.peer.parameter_reads.load(Ordering::SeqCst), 2);
    client.stop().await;
}
