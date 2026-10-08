include!("../examples/support/managed_fixture.rs");
#[tokio::test]
async fn external_setup_operation_workflow_rejection_decode_and_replacement() {
    let (peer, client) = fixture(Duration::from_secs(2));
    client.connected().await.unwrap();
    assert_eq!(client.operation(FixtureClock).await.unwrap(), 2026);
    let generation = client.workflow(Pair).await.unwrap();
    peer.fault.store(1, Ordering::SeqCst);
    assert!(matches!(
        client.operation(FixtureClock).await,
        Err(etherbird::Error::Operation(Error::Device(_)))
    ));
    assert_eq!(peer.connections.load(Ordering::SeqCst), 1);
    peer.fault.store(2, Ordering::SeqCst);
    assert!(matches!(
        client.operation(FixtureClock).await,
        Err(etherbird::Error::Operation(Error::InvalidResponse(_)))
    ));
    let replacement = client.workflow(Pair).await.unwrap();
    assert_ne!(generation, replacement);
    client.stop().await;
}
struct SideEffect;
#[etherbird::async_trait]
impl Workflow for SideEffect {
    type Output = ();
    async fn run(self, context: &mut ManagedContext<'_>) -> Result<(), Error> {
        context.operation(FixtureClock).await?;
        context.operation(FixtureClock).await?;
        Ok(())
    }
}
#[tokio::test]
async fn disconnect_after_observed_request_does_not_replay_workflow() {
    let (peer, client) = fixture(Duration::from_secs(2));
    client.connected().await.unwrap();
    peer.fault.store(3, Ordering::SeqCst);
    assert!(client.workflow(SideEffect).await.is_err());
    assert_eq!(client.operation(FixtureClock).await.unwrap(), 2026);
    assert!(peer.connections.load(Ordering::SeqCst) >= 2);
    client.stop().await;
}
#[tokio::test]
async fn active_deadline_queued_deadline_and_stop() {
    let (peer, client) = fixture(Duration::from_millis(40));
    client.connected().await.unwrap();
    peer.fault.store(4, Ordering::SeqCst);
    let before = peer.requests.load(Ordering::SeqCst);
    let first = client.clone();
    let active = tokio::spawn(async move { first.workflow(SideEffect).await });
    while peer.requests.load(Ordering::SeqCst) == before {
        tokio::task::yield_now().await;
    }
    let queued = client.operation(FixtureClock).await;
    assert!(active.await.unwrap().is_err());
    assert!(queued.is_err());
    assert_eq!(client.operation(FixtureClock).await.unwrap(), 2026);
    peer.fault.store(4, Ordering::SeqCst);
    let clone = client.clone();
    let active = tokio::spawn(async move { clone.workflow(SideEffect).await });
    tokio::time::sleep(Duration::from_millis(2)).await;
    client.stop().await;
    assert!(active.await.unwrap().is_err());
    assert!(matches!(
        client.operation(FixtureClock).await,
        Err(etherbird::Error::Stopped)
    ));
}

struct HoldBetween {
    entered: tokio::sync::oneshot::Sender<()>,
    release: tokio::sync::oneshot::Receiver<()>,
}
#[etherbird::async_trait]
impl Workflow for HoldBetween {
    type Output = ();
    async fn run(self, context: &mut ManagedContext<'_>) -> Result<(), Error> {
        context.operation(FixtureClock).await?;
        let _ = self.entered.send(());
        let _ = self.release.await;
        context.operation(FixtureClock).await?;
        Ok(())
    }
}
#[tokio::test]
async fn competing_clone_cannot_enter_between_steps_and_cancellation_retires() {
    let (peer, client) = fixture(Duration::from_secs(2));
    client.connected().await.unwrap();
    let (entered, wait) = tokio::sync::oneshot::channel();
    let (release, receiver) = tokio::sync::oneshot::channel();
    let clone = client.clone();
    let running = tokio::spawn(async move {
        clone
            .workflow(HoldBetween {
                entered,
                release: receiver,
            })
            .await
    });
    wait.await.unwrap();
    let before = peer.requests.load(Ordering::SeqCst);
    let clone = client.clone();
    let queued = tokio::spawn(async move { clone.operation(FixtureClock).await });
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert_eq!(peer.requests.load(Ordering::SeqCst), before);
    release.send(()).unwrap();
    running.await.unwrap().unwrap();
    queued.await.unwrap().unwrap();
    let (entered, wait) = tokio::sync::oneshot::channel();
    let (_release, receiver) = tokio::sync::oneshot::channel();
    let clone = client.clone();
    let running = tokio::spawn(async move {
        clone
            .workflow(HoldBetween {
                entered,
                release: receiver,
            })
            .await
    });
    wait.await.unwrap();
    running.abort();
    assert!(running.await.unwrap_err().is_cancelled());
    assert_eq!(client.operation(FixtureClock).await.unwrap(), 2026);
    assert!(peer.connections.load(Ordering::SeqCst) >= 2);
    client.stop().await;
}

#[tokio::test]
async fn heartbeat_cannot_read_inside_workflow_scope() {
    let (peer, client) =
        fixture_with_heartbeat(Duration::from_secs(2), Some(Duration::from_millis(2)));
    client.connected().await.unwrap();
    let (entered, wait) = tokio::sync::oneshot::channel();
    let (release, receiver) = tokio::sync::oneshot::channel();
    let clone = client.clone();
    let running = tokio::spawn(async move {
        clone
            .workflow(HoldBetween {
                entered,
                release: receiver,
            })
            .await
    });
    wait.await.unwrap();
    let before = peer.requests.load(Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(15)).await;
    assert_eq!(peer.requests.load(Ordering::SeqCst), before);
    release.send(()).unwrap();
    running.await.unwrap().unwrap();
    client.stop().await;
}

struct BindHandle;
#[etherbird::async_trait]
impl Workflow for BindHandle {
    type Output = rocsteady::GenerationValue<u16>;
    async fn run(self, context: &mut ManagedContext<'_>) -> Result<Self::Output, Error> {
        let handle = context.bind(123);
        assert_eq!(*handle.get(context)?, 123);
        Ok(handle)
    }
}
struct UseHandle(rocsteady::GenerationValue<u16>);
#[etherbird::async_trait]
impl Workflow for UseHandle {
    type Output = ();
    async fn run(self, context: &mut ManagedContext<'_>) -> Result<(), Error> {
        let _ = self.0.get(context)?;
        context.operation(FixtureClock).await?;
        Ok(())
    }
}
#[tokio::test]
async fn stale_handle_and_missing_state_fail_locally_without_reconnecting() {
    let (peer, client) = fixture(Duration::from_secs(2));
    let handle = client.workflow(BindHandle).await.unwrap();
    peer.fault.store(3, Ordering::SeqCst);
    assert!(client.operation(FixtureClock).await.is_err());
    client.connected().await.unwrap();
    let connections = peer.connections.load(Ordering::SeqCst);
    let requests = peer.requests.load(Ordering::SeqCst);
    assert!(matches!(
        client.workflow(UseHandle(handle)).await,
        Err(etherbird::Error::Operation(Error::StaleGeneration { .. }))
    ));
    assert!(matches!(
        client.workflow(MissingState).await,
        Err(etherbird::Error::Operation(Error::CapabilityUnavailable(_)))
    ));
    assert_eq!(peer.requests.load(Ordering::SeqCst), requests);
    assert_eq!(client.operation(FixtureClock).await.unwrap(), 2026);
    assert_eq!(peer.connections.load(Ordering::SeqCst), connections);
    client.stop().await;
}
struct MissingState;
#[etherbird::async_trait]
impl Workflow for MissingState {
    type Output = ();
    async fn run(self, context: &mut ManagedContext<'_>) -> Result<(), Error> {
        context.require_state::<String>()?;
        Ok(())
    }
}

struct UnsupportedSetup(Arc<AtomicUsize>);
#[etherbird::async_trait]
impl SetupStrategy for UnsupportedSetup {
    async fn setup(&self, _: &mut ManagedContext<'_>) -> Result<rocsteady::SessionState, Error> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err(Error::Authentication(
            rocsteady::AuthenticationFailure::Unsupported,
        ))
    }
}
#[tokio::test]
async fn terminal_authentication_wakes_queued_clones_and_stop_takes_precedence() {
    let (peer, old) = fixture(Duration::from_secs(2));
    old.stop().await;
    let attempts = Arc::new(AtomicUsize::new(0));
    let client = Client::with_setup(
        Arc::new(FixtureConnector(peer.clone())),
        ClientConfig {
            heartbeat_interval: None,
            recovery: etherbird::Config {
                retry_delay: Duration::from_millis(1),
                ..Default::default()
            },
            ..Default::default()
        },
        Arc::new(UnsupportedSetup(attempts.clone())),
    )
    .unwrap();
    let clone = client.clone();
    let (ready, operation) = tokio::join!(client.connected(), clone.operation(FixtureClock));
    for result in [ready, operation.map(|_| ())] {
        assert!(matches!(
            result,
            Err(etherbird::Error::Operation(Error::Authentication(
                rocsteady::AuthenticationFailure::Unsupported
            )))
        ));
    }
    tokio::time::sleep(Duration::from_millis(15)).await;
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    assert_eq!(peer.connections.load(Ordering::SeqCst), 1);
    assert_eq!(
        client.authentication_failure(),
        Some(rocsteady::AuthenticationFailure::Unsupported)
    );
    client.stop().await;
    assert!(matches!(
        client.read_clock().await,
        Err(etherbird::Error::Stopped)
    ));
}

struct InvalidOperation;
impl Operation for InvalidOperation {
    type Output = ();
    fn encode(&self) -> Result<(u8, Vec<u8>), Error> {
        Ok((7, vec![0; 256]))
    }
    fn decode(&self, _: &[u8]) -> Result<(), rocsteady::DecodeError> {
        unreachable!()
    }
}
#[tokio::test]
async fn invalid_operation_fails_before_connection_or_admission() {
    let (peer, client) = fixture(Duration::from_secs(2));
    assert!(matches!(
        client.operation(InvalidOperation).await,
        Err(etherbird::Error::Operation(Error::InvalidRequest(_)))
    ));
    assert_eq!(peer.connections.load(Ordering::SeqCst), 0);
    client.stop().await;
}

struct SwallowFailure(bool);
#[etherbird::async_trait]
impl Workflow for SwallowFailure {
    type Output = ();
    async fn run(self, context: &mut ManagedContext<'_>) -> Result<(), Error> {
        let _ = context.operation(FixtureClock).await;
        if self.0 {
            Err(Error::InvalidRequest(
                "fixture attempts to hide retired stream",
            ))
        } else {
            Ok(())
        }
    }
}
#[tokio::test]
async fn swallowed_decode_failure_cannot_report_success() {
    let (peer, client) = fixture(Duration::from_secs(2));
    client.connected().await.unwrap();
    for local_error in [false, true] {
        peer.fault.store(2, Ordering::SeqCst);
        assert!(matches!(
            client.workflow(SwallowFailure(local_error)).await,
            Err(etherbird::Error::Operation(Error::Disconnected))
        ));
        assert_eq!(client.operation(FixtureClock).await.unwrap(), 2026);
    }
    assert!(peer.connections.load(Ordering::SeqCst) >= 2);
    client.stop().await;
}

struct TrackedState(Arc<AtomicUsize>);
impl Drop for TrackedState {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
struct TrackedSetup(Arc<AtomicUsize>);
#[etherbird::async_trait]
impl SetupStrategy for TrackedSetup {
    async fn setup(
        &self,
        context: &mut ManagedContext<'_>,
    ) -> Result<rocsteady::SessionState, Error> {
        context.operation(FixtureClock).await?;
        Ok(Box::new(TrackedState(self.0.clone())))
    }
}
#[tokio::test]
async fn generation_state_is_dropped_on_replacement_and_awaited_stop() {
    let (peer, unused) = fixture(Duration::from_secs(2));
    unused.stop().await;
    let dropped = Arc::new(AtomicUsize::new(0));
    let client = Client::with_setup(
        Arc::new(FixtureConnector(peer.clone())),
        ClientConfig {
            heartbeat_interval: None,
            recovery: etherbird::Config {
                retry_delay: Duration::from_millis(1),
                ..Default::default()
            },
            ..Default::default()
        },
        Arc::new(TrackedSetup(dropped.clone())),
    )
    .unwrap();
    client.connected().await.unwrap();
    assert_eq!(dropped.load(Ordering::SeqCst), 0);
    peer.fault.store(3, Ordering::SeqCst);
    assert!(client.operation(FixtureClock).await.is_err());
    client.connected().await.unwrap();
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
    client.stop().await;
    assert_eq!(dropped.load(Ordering::SeqCst), 2);
}

#[test]
fn invalid_lifecycle_deadlines_and_retry_bounds_fail_at_construction() {
    for recovery in [
        etherbird::Config {
            setup_timeout: Duration::ZERO,
            ..Default::default()
        },
        etherbird::Config {
            create_timeout: Some(Duration::ZERO),
            ..Default::default()
        },
        etherbird::Config {
            retry_delay: Duration::ZERO,
            ..Default::default()
        },
        etherbird::Config {
            retry_delay: Duration::from_secs(2),
            max_retry_delay: Duration::from_secs(1),
            ..Default::default()
        },
    ] {
        assert!(matches!(
            Client::tcp(
                "unused",
                ClientConfig {
                    recovery,
                    ..Default::default()
                }
            ),
            Err(Error::InvalidConfig(_))
        ));
    }
}
