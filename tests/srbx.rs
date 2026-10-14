include!("../examples/support/discovery_fixture.rs");
include!("../examples/support/log_literals.rs");
use rocsteady::{
    AlarmAcknowledgementOutcome as Outcome, AlarmRecord, Client, ClientConfig, Error,
    LogCheckpoint, LogRequest, ProtocolProfile,
};
use std::time::Duration;

async fn fixture() -> DiscoveryFixture {
    let peer = DiscoveryFixture::start(0).await;
    peer.peer
        .alarm_records
        .lock()
        .unwrap()
        .extend((0..450).map(|i| (i, alarm_literal(i))));
    peer.peer.current_log_index.store(7, Ordering::SeqCst);
    peer.peer.srbx_pending.store(true, Ordering::SeqCst);
    peer
}
fn make_client(peer: &DiscoveryFixture) -> Client {
    Client::tcp(
        &peer.endpoint,
        ClientConfig {
            heartbeat_interval: None,
            response_timeout: Duration::from_secs(2),
            call_timeout: Duration::from_secs(4),
            recovery: etherbird::Config {
                retry_delay: Duration::from_millis(1),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap()
}
async fn checkpoint(client: &Client, index: u16) -> LogCheckpoint<AlarmRecord> {
    client
        .read_alarms(LogRequest::new(index, 1).unwrap())
        .await
        .unwrap()
        .checkpoint(1)
        .unwrap()
}
async fn idle_signals(peer: &DiscoveryFixture, count: usize) {
    let target = peer.peer.signals_sent.load(Ordering::SeqCst) + count;
    peer.peer.idle_signals.fetch_add(count, Ordering::SeqCst);
    peer.peer.signal_wake.notify_one();
    tokio::time::timeout(Duration::from_secs(3), async {
        while peer.peer.signals_sent.load(Ordering::SeqCst) < target {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
}
#[tokio::test]
async fn chapter_6_consumed_index_seven_is_acknowledged_with_published_request_bytes() {
    let peer = fixture().await;
    let client = make_client(&peer);
    let consumed = checkpoint(&client, 7).await;
    let result = client.acknowledge_alarms(&consumed).await.unwrap();
    assert_eq!(result.index, 7);
    assert_eq!(peer.peer.acknowledgements.load(Ordering::SeqCst), 1);
    assert!(!peer.peer.srbx_pending.load(Ordering::SeqCst));
    let ack = peer
        .peer
        .requests
        .lock()
        .unwrap()
        .iter()
        .find(|(op, _)| *op == 225)
        .cloned()
        .unwrap();
    assert_eq!(ack, (225, vec![7, 0]));
    assert_eq!(
        rocsteady::protocol::Frame::new(
            rocsteady::Address::new(1, 2),
            rocsteady::Address::new(1, 0),
            225,
            ack.1
        )
        .encode()
        .unwrap(),
        [1, 2, 1, 0, 225, 2, 7, 0, 118, 17]
    );
    assert_eq!(peer.peer.log_reads.load(Ordering::SeqCst), 2);
    client.stop().await;
}
#[tokio::test]
async fn new_alarm_before_ack_leaves_pending_and_does_not_jump_to_unconsumed_index() {
    let peer = fixture().await;
    let client = make_client(&peer);
    let consumed = checkpoint(&client, 7).await;
    peer.peer.grow_before_ack.store(1, Ordering::SeqCst);
    client.acknowledge_alarms(&consumed).await.unwrap();
    assert_eq!(peer.peer.last_ack_index.load(Ordering::SeqCst), 7);
    assert_eq!(peer.peer.current_log_index.load(Ordering::SeqCst), 8);
    assert!(peer.peer.srbx_pending.load(Ordering::SeqCst));
    client.read_clock().await.unwrap();
    assert_eq!(peer.peer.acknowledgements.load(Ordering::SeqCst), 1);
    client.stop().await;
}
#[tokio::test]
async fn changed_or_missing_consumed_anchor_never_sends_ack_and_keeps_connection() {
    for missing in [false, true] {
        let peer = fixture().await;
        let client = make_client(&peer);
        let consumed = checkpoint(&client, 7).await;
        if missing {
            peer.peer.alarm_records.lock().unwrap().remove(&7);
        } else {
            peer.peer.alarm_records.lock().unwrap().get_mut(&7).unwrap()[1] ^= 1;
        }
        let error = client.acknowledge_alarms(&consumed).await.unwrap_err();
        assert_eq!(error.outcome, Outcome::NotAttempted);
        assert!(matches!(
            error.error,
            etherbird::Error::Operation(Error::LogChanged(_))
        ));
        assert_eq!(peer.peer.acknowledgements.load(Ordering::SeqCst), 0);
        client.read_clock().await.unwrap();
        assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
        client.stop().await;
    }
}
#[tokio::test]
async fn rejection_is_explicit_but_malformed_or_lost_replies_leave_outcome_unconfirmed() {
    for (fault, outcome, committed) in [
        (2, Outcome::Rejected, 0),
        (1, Outcome::Unconfirmed, 1),
        (3, Outcome::Unconfirmed, 1),
    ] {
        let peer = fixture().await;
        let client = make_client(&peer);
        let consumed = checkpoint(&client, 7).await;
        peer.peer.ack_fault.store(fault, Ordering::SeqCst);
        let error = client.acknowledge_alarms(&consumed).await.unwrap_err();
        assert_eq!(error.outcome, outcome);
        assert_eq!(peer.peer.acknowledgements.load(Ordering::SeqCst), committed);
        client.read_clock().await.unwrap();
        assert_eq!(
            peer.peer.connections.load(Ordering::SeqCst),
            if fault == 2 { 1 } else { 2 }
        );
        assert_eq!(peer.peer.acknowledgements.load(Ordering::SeqCst), committed); // Reconnection never resends.
        client.stop().await;
    }
}
#[tokio::test]
async fn deadline_before_send_is_not_attempted_and_after_send_may_have_applied() {
    for after_send in [false, true] {
        let peer = fixture().await;
        let client = Client::tcp(
            &peer.endpoint,
            ClientConfig {
                heartbeat_interval: None,
                call_timeout: Duration::from_secs(1),
                response_timeout: Duration::from_secs(3),
                recovery: etherbird::Config {
                    retry_delay: Duration::from_millis(1),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();
        let consumed = checkpoint(&client, 7).await;
        if after_send {
            peer.peer.ack_fault.store(4, Ordering::SeqCst);
        } else {
            peer.peer.log_fault_at.store(2, Ordering::SeqCst);
            peer.peer.log_fault.store(5, Ordering::SeqCst);
        }
        let error = client.acknowledge_alarms(&consumed).await.unwrap_err();
        assert!(matches!(error.error, etherbird::Error::Timeout));
        assert_eq!(
            error.outcome,
            if after_send {
                Outcome::Unconfirmed
            } else {
                Outcome::NotAttempted
            }
        );
        tokio::time::timeout(Duration::from_secs(3), async {
            while peer.peer.connections.load(Ordering::SeqCst) < 2 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        client.connected().await.unwrap();
        client.read_clock().await.unwrap();
        assert_eq!(
            peer.peer.acknowledgements.load(Ordering::SeqCst),
            usize::from(after_send)
        );
        client.stop().await;
    }
}
#[tokio::test]
async fn caller_cancellation_before_send_cannot_ack_later() {
    let peer = fixture().await;
    let client = make_client(&peer);
    let consumed = checkpoint(&client, 7).await;
    peer.peer.log_fault_at.store(2, Ordering::SeqCst);
    peer.peer.log_fault.store(5, Ordering::SeqCst);
    let task = tokio::spawn({
        let client = client.clone();
        async move { client.acknowledge_alarms(&consumed).await }
    });
    tokio::time::timeout(Duration::from_secs(3), async {
        while peer.peer.log_reads.load(Ordering::SeqCst) < 2 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    client.read_clock().await.unwrap();
    assert_eq!(peer.peer.acknowledgements.load(Ordering::SeqCst), 0);
    client.stop().await;
}
#[tokio::test]
async fn stop_during_send_reports_uncertainty_and_never_replays_effect() {
    let peer = fixture().await;
    let client = make_client(&peer);
    let consumed = checkpoint(&client, 7).await;
    peer.peer.ack_fault.store(4, Ordering::SeqCst);
    let task = tokio::spawn({
        let client = client.clone();
        async move { client.acknowledge_alarms(&consumed).await }
    });
    tokio::time::timeout(Duration::from_secs(3), async {
        while peer.peer.acknowledgements.load(Ordering::SeqCst) < 1 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    client.stop().await;
    let error = task.await.unwrap().unwrap_err();
    assert_eq!(error.outcome, Outcome::Unconfirmed);
    assert!(matches!(error.error, etherbird::Error::Stopped));
    assert_eq!(peer.peer.acknowledgements.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn idle_polling_receives_repeated_signals_and_missed_signal_still_reports_index_growth() {
    let peer = fixture().await;
    let client = make_client(&peer);
    let mut poller = client.srbx_poller(Duration::from_millis(10)).unwrap();
    client.connected().await.unwrap();
    idle_signals(&peer, 2).await;
    assert_eq!(peer.peer.log_reads.load(Ordering::SeqCst), 0); // No business requests to wake reception.
    let poll = poller.poll().await.unwrap();
    assert_eq!(poll.current_alarm_index, 7);
    assert_eq!(poll.notifications.signals.len(), 2);
    assert_eq!(poll.notifications.lagged, 0);
    peer.peer.current_log_index.store(8, Ordering::SeqCst); // Deliberately omit the signal.
    let missed = poller.poll().await.unwrap();
    assert_eq!(missed.current_alarm_index, 8);
    assert!(missed.notifications.signals.is_empty());
    assert_eq!(peer.peer.acknowledgements.load(Ordering::SeqCst), 0);
    client.stop().await;
}
#[tokio::test]
async fn notification_lag_is_reported_with_bounded_batches_and_no_implicit_acknowledgement() {
    let peer = fixture().await;
    let client = make_client(&peer);
    let mut poller = client.srbx_poller(Duration::from_millis(1)).unwrap();
    client.connected().await.unwrap();
    idle_signals(&peer, 100).await;
    let poll = poller.poll().await.unwrap();
    assert_eq!(poll.notifications.signals.len(), 32);
    assert_eq!(poll.notifications.lagged, 68);
    assert_eq!(peer.peer.acknowledgements.load(Ordering::SeqCst), 0);
    client.stop().await;
}
#[tokio::test]
async fn polling_survives_replacement_and_stop_wakes_interval_wait_without_an_extra_request() {
    let peer = fixture().await;
    let client = make_client(&peer);
    let mut poller = client.srbx_poller(Duration::from_secs(3600)).unwrap();
    poller.poll().await.unwrap();
    let task = tokio::spawn(async move { poller.poll().await });
    tokio::task::yield_now().await;
    client.stop().await;
    let error = tokio::time::timeout(Duration::from_secs(2), task)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert!(matches!(error.error, etherbird::Error::Stopped));
    assert_eq!(peer.peer.log_reads.load(Ordering::SeqCst), 1);
    let client = make_client(&peer);
    let mut poller = client.srbx_poller(Duration::from_millis(1)).unwrap();
    peer.peer.log_fault_at.store(2, Ordering::SeqCst);
    peer.peer.log_fault.store(4, Ordering::SeqCst);
    assert!(poller.poll().await.is_err());
    client.read_clock().await.unwrap();
    idle_signals(&peer, 1).await;
    assert_eq!(poller.poll().await.unwrap().notifications.signals.len(), 1);
    client.stop().await;
}
#[tokio::test]
async fn malformed_srbx_is_not_published_and_retires_connection_but_prior_signals_are_reported() {
    let peer = fixture().await;
    let client = make_client(&peer);
    let mut poller = client.srbx_poller(Duration::from_millis(1)).unwrap();
    client.connected().await.unwrap();
    idle_signals(&peer, 1).await;
    let mut invalid = vec![1, 0, 1, 2, 224, 1, 1];
    let crc = discovery_crc(&invalid);
    invalid.extend(crc.to_le_bytes());
    *peer.peer.signal_frame.lock().unwrap() = Some(invalid);
    idle_signals(&peer, 1).await;
    let error = poller.poll().await.unwrap_err();
    assert_eq!(error.notifications.signals.len(), 1);
    assert!(matches!(
        error.error,
        etherbird::Error::Operation(Error::InvalidResponse(_))
    ));
    client.read_clock().await.unwrap();
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
    client.stop().await;
}
struct NoAck;
impl ProtocolProfile for NoAck {
    fn metadata(&self) -> rocsteady::ProfileMetadata {
        rocsteady::DefaultProfile.metadata()
    }
    fn capability(&self, op: u8) -> rocsteady::Capability {
        if op == 225 {
            rocsteady::Capability::Unavailable
        } else {
            rocsteady::DefaultProfile.capability(op)
        }
    }
}
#[tokio::test]
async fn unsupported_ack_and_invalid_poll_intervals_fail_locally() {
    let peer = fixture().await;
    let client = Client::tcp(
        &peer.endpoint,
        ClientConfig {
            profile: Arc::new(NoAck),
            heartbeat_interval: None,
            ..Default::default()
        },
    )
    .unwrap();
    let consumed = LogCheckpoint::new(7, AlarmRecord::decode(&alarm_literal(7)).unwrap()).unwrap();
    let error = client.acknowledge_alarms(&consumed).await.unwrap_err();
    assert_eq!(error.outcome, Outcome::NotAttempted);
    assert_eq!(peer.peer.log_reads.load(Ordering::SeqCst), 0);
    assert_eq!(peer.peer.acknowledgements.load(Ordering::SeqCst), 0);
    assert!(client.srbx_poller(Duration::ZERO).is_err());
    assert!(client.srbx_poller(Duration::MAX).is_err());
    client.stop().await;
}
struct ConsumeAndAck;
#[etherbird::async_trait]
impl rocsteady::Workflow for ConsumeAndAck {
    type Output = rocsteady::AlarmAcknowledgement;
    async fn run(self, context: &mut rocsteady::ManagedContext<'_>) -> Result<Self::Output, Error> {
        let page = context.read_alarms(LogRequest::new(7, 1)?).await?;
        // Fixture consumer has handled this one record; real applications commit ingestion here.
        context
            .acknowledge_alarms(&page.checkpoint(1)?)
            .await
            .map_err(|e| e.error)
    }
}
#[tokio::test]
async fn managed_and_standalone_acknowledgement_interfaces_preserve_exclusive_ownership() {
    use rocsteady::transport::Connector;
    let peer = fixture().await;
    let client = make_client(&peer);
    assert_eq!(client.workflow(ConsumeAndAck).await.unwrap().index, 7);
    client.stop().await;
    let stream = rocsteady::transport::TcpConnector::new(&peer.endpoint)
        .connect()
        .await
        .unwrap();
    let session = rocsteady::Session::new(
        stream,
        rocsteady::Address::new(1, 0),
        rocsteady::Address::new(1, 2),
        Duration::from_secs(2),
        rocsteady::protocol::CrcMode::Required,
    )
    .unwrap();
    let page = session
        .read_alarms(LogRequest::new(7, 1).unwrap())
        .await
        .unwrap();
    assert_eq!(
        session
            .acknowledge_alarms(&page.checkpoint(1).unwrap())
            .await
            .unwrap()
            .index,
        7
    );
    session.disconnect().await;
}

#[tokio::test]
async fn canceling_only_the_poll_interval_keeps_connection_and_sends_no_request() {
    let peer = fixture().await;
    let client = make_client(&peer);
    let mut poller = client.srbx_poller(Duration::from_secs(3600)).unwrap();
    poller.poll().await.unwrap();
    assert!(
        tokio::time::timeout(Duration::from_millis(20), poller.poll())
            .await
            .is_err()
    );
    client.read_clock().await.unwrap();
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
    assert_eq!(peer.peer.log_reads.load(Ordering::SeqCst), 1);
    client.stop().await;
}
#[tokio::test]
async fn sustained_srbx_does_not_extend_the_wire_deadline_or_discard_received_notifications() {
    let peer = fixture().await;
    let client = Client::tcp(
        &peer.endpoint,
        ClientConfig {
            heartbeat_interval: None,
            response_timeout: Duration::from_secs(1),
            call_timeout: Duration::from_secs(3),
            recovery: etherbird::Config {
                retry_delay: Duration::from_millis(1),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap();
    let mut poller = client.srbx_poller(Duration::from_millis(1)).unwrap();
    client.connected().await.unwrap();
    peer.peer.storm_on_stall.store(true, Ordering::SeqCst);
    peer.peer.log_fault_at.store(1, Ordering::SeqCst);
    peer.peer.log_fault.store(5, Ordering::SeqCst);
    let error = tokio::time::timeout(Duration::from_secs(3), poller.poll())
        .await
        .unwrap()
        .unwrap_err();
    assert!(matches!(
        error.error,
        etherbird::Error::Operation(Error::Timeout)
    ));
    assert!(!error.notifications.signals.is_empty());
    assert!(error.notifications.signals.len() <= 32);
    assert_eq!(peer.peer.acknowledgements.load(Ordering::SeqCst), 0);
    client.read_clock().await.unwrap();
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
    client.stop().await;
}
