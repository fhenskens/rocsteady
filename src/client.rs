use crate::{
    Error, Session, SystemConfiguration,
    protocol::{Address, Clock, CrcMode, Frame, Login, Parameter, ParameterBlock, Value},
    transport::{Connector, SharedConnector, TcpConnector},
};
use etherbird::{
    BoundedFifoQueue, Config, Lifecycle, LifecycleFuture, Pool, PoolConfig, Supervisor,
};
use std::{future::Future, sync::Arc, time::Duration};
use tokio::{
    sync::{broadcast, watch},
    time::sleep,
};

/// Response deadline starts after stream acquisition. Call deadline includes queueing/setup/recovery.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub profile: Arc<dyn crate::ProtocolProfile>,
    pub host: Address,
    pub device: Address,
    pub login: Option<Login>,
    pub response_timeout: Duration,
    pub call_timeout: Duration,
    pub heartbeat_interval: Option<Duration>,
    pub crc_mode: CrcMode,
    pub queue_capacity: usize,
    pub recovery: Config,
}

#[cfg(all(test, feature = "simulator"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn awaited_stop_releases_caches_but_external_handle_retains_closed_session() {
        let server = crate::simulator::Simulator::bind(
            "127.0.0.1:0".parse().unwrap(),
            crate::simulator::Model::default(),
        )
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
        client.connected().await.unwrap();
        let handle = client.pool.resources().pop().unwrap();
        let weak = Arc::downgrade(handle.resource());
        client.stop().await;
        assert!(client.pool.resources().is_empty());
        assert!(client.pool.supervisors().is_empty());
        assert!(!handle.resource().is_connected());
        assert!(weak.upgrade().is_some());
        drop(handle);
        assert!(weak.upgrade().is_none());
        server.stop().await.unwrap();
    }
}
impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            profile: Arc::new(crate::DefaultProfile),
            host: Address::new(1, 0),
            device: Address::new(1, 2),
            login: None,
            response_timeout: Duration::from_secs(3),
            call_timeout: Duration::from_secs(15),
            heartbeat_interval: Some(Duration::from_secs(30)),
            crc_mode: CrcMode::Required,
            queue_capacity: 64,
            recovery: Config {
                resource_name: "rocsteady".into(),
                ..Config::default()
            },
        }
    }
}
#[derive(Clone)]
struct Hooks {
    connector: SharedConnector,
    config: ClientConfig,
    setup: Arc<dyn crate::SetupStrategy>,
    events: broadcast::Sender<Frame>,
}
#[etherbird::async_trait]
impl Lifecycle for Hooks {
    type Resource = Session;
    type Error = Error;
    async fn create(&self) -> Result<Session, Error> {
        // Fresh stream on every generation, bounded by etherbird create timeout.
        let stream = self.connector.connect().await?;
        Session::with_events(
            stream,
            self.config.host,
            self.config.device,
            self.config.response_timeout,
            self.config.crc_mode,
            self.events.clone(),
            self.config.profile.clone(),
        )
    }
    async fn connect(&self, _session: &Session) -> Result<(), Error> {
        Ok(())
    }
    async fn setup(&self, session: &Session) -> Result<(), Error> {
        let mut context = session.managed().await;
        let state = self.setup.setup(&mut context).await?;
        if !session.is_connected() {
            return Err(Error::Disconnected);
        }
        *context.state = Some(state);
        context.scope.complete();
        Ok(())
    }
    async fn disconnect(&self, session: &Session) -> Result<(), Error> {
        session.disconnect().await;
        Ok(())
    }
    fn is_connected(&self, session: &Session) -> bool {
        session.is_connected()
    }
    fn watch_disconnect(&self, session: Arc<Session>) -> Option<LifecycleFuture<Error>> {
        let interval = self.config.heartbeat_interval;
        // Always observe cancellation poisoning, even when protocol heartbeat is disabled.
        Some(Box::pin(async move {
            let mut connected = session.connected.subscribe();
            loop {
                if !*connected.borrow_and_update() {
                    return Err(Error::Disconnected);
                }
                if let Some(interval) = interval {
                    tokio::select! {
                        result = connected.changed() => { if result.is_err() { return Err(Error::Disconnected); } }
                        _ = sleep(interval) => { session.read_clock().await?; }
                    }
                } else {
                    connected.changed().await.map_err(|_| Error::Disconnected)?;
                }
            }
        }))
    }
    fn lifecycle_failure(&self, error: &Error) -> etherbird::LifecycleFailurePolicy {
        if matches!(
            error,
            Error::Authentication(_) | Error::CapabilityUnavailable(_)
        ) {
            etherbird::LifecycleFailurePolicy::Fail
        } else {
            etherbird::LifecycleFailurePolicy::Retry
        }
    }
    fn operation_failure(&self, error: &Error) -> etherbird::OperationFailurePolicy {
        if error.preserves_connection() {
            etherbird::OperationFailurePolicy::Retain
        } else {
            etherbird::OperationFailurePolicy::Recover
        }
    }
    fn is_expected(&self, error: &Error) -> bool {
        matches!(
            error,
            Error::Io(_)
                | Error::Timeout
                | Error::Disconnected
                | Error::Device(_)
                | Error::DiscoveryChanged(_)
                | Error::LogChanged(_)
        )
    }
}
/// Stable client clones share one supervised, exclusively accessed channel.
/// Operations run once. A failure may still have taken effect remotely.
#[derive(Clone)]
pub struct Client {
    profile: Arc<dyn crate::ProtocolProfile>,
    pool: Pool<Hooks>,
    call_timeout: Duration,
    events: broadcast::Sender<Frame>,
}
impl Client {
    pub fn tcp(endpoint: impl Into<String>, config: ClientConfig) -> Result<Self, Error> {
        Self::new(Arc::new(TcpConnector::new(endpoint)), config)
    }
    pub fn new(connector: Arc<dyn Connector>, config: ClientConfig) -> Result<Self, Error> {
        crate::profile::validate_profile_request(config.profile.as_ref(), 7, &[])?;
        if let Some(login) = &config.login {
            crate::profile::validate_profile_request(
                config.profile.as_ref(),
                17,
                &login.payload(),
            )?;
        }
        let setup = Arc::new(crate::LegacySetup {
            login: config.login.clone(),
        });
        Self::with_setup(connector, config, setup)
    }
    /// Immutable custom setup replaces legacy login and readiness.
    pub fn with_setup(
        connector: Arc<dyn Connector>,
        mut config: ClientConfig,
        setup: Arc<dyn crate::SetupStrategy>,
    ) -> Result<Self, Error> {
        config.host.validate_unicast()?;
        config.device.validate_unicast()?;
        if config.host == config.device
            || config.queue_capacity == 0
            || config.response_timeout.is_zero()
            || config.call_timeout.is_zero()
            || config.heartbeat_interval.is_some_and(|d| d.is_zero())
            || config.recovery.connect_timeout.is_zero()
            || config.recovery.setup_timeout.is_zero()
            || config.recovery.cleanup_timeout.is_zero()
            || config.recovery.disconnect_timeout.is_zero()
            || config.recovery.create_timeout.is_some_and(|d| d.is_zero())
            || config.recovery.created_timeout.is_some_and(|d| d.is_zero())
            || config.recovery.destroy_timeout.is_some_and(|d| d.is_zero())
        {
            return Err(Error::InvalidConfig(
                "invalid addresses, capacity or deadlines",
            ));
        }
        // Preserve the public error shape and our stricter positive hook deadlines.
        config
            .recovery
            .validate()
            .map_err(|_| Error::InvalidConfig("invalid addresses, capacity or deadlines"))?;
        if config.recovery.create_timeout.is_none() {
            config.recovery.create_timeout = Some(config.recovery.connect_timeout);
        }
        let (events, _) = broadcast::channel(crate::SRBX_BUFFER_CAPACITY);
        let hooks = Hooks {
            connector,
            config: config.clone(),
            setup,
            events: events.clone(),
        };
        let recovery = config.recovery.clone();
        let capacity = config.queue_capacity;
        let pool = Pool::try_new_with_queue_factory(
            move || Supervisor::new(hooks.clone(), recovery.clone()),
            PoolConfig {
                min_size: 1,
                max_size: 1,
                ..PoolConfig::default()
            },
            move || BoundedFifoQueue::new(capacity),
        )
        .map_err(|_| Error::InvalidConfig("invalid pool configuration"))?;
        Ok(Self {
            profile: config.profile,
            pool,
            call_timeout: config.call_timeout,
            events,
        })
    }
    pub fn state(&self) -> etherbird::ResourceState {
        self.pool.state()
    }
    pub fn profile(&self) -> &dyn crate::ProtocolProfile {
        self.profile.as_ref()
    }
    pub fn subscribe_state(&self) -> watch::Receiver<u64> {
        self.pool.subscribe()
    }
    /// This receiver survives transport replacements. Lag is reported by Tokio.
    pub fn subscribe_events(&self) -> broadcast::Receiver<Frame> {
        self.events.subscribe()
    }
    /// Receive queued SRBX while otherwise idle through bounded index-only polls.
    /// Drive poll() repeatedly; dropping the poller stops polling, and Client::stop wakes its timer.
    pub fn srbx_poller(&self, interval: Duration) -> Result<crate::SrbxPoller, Error> {
        crate::SrbxPoller::new(self.clone(), self.pool.stopped(), interval)
    }
    pub async fn connected(&self) -> Result<(), etherbird::Error<Error>> {
        self.map_lifecycle_error(self.pool.connected_with_timeout(self.call_timeout))
            .await
    }
    /// Terminal setup failure, shared by clones. It cannot be cleared in place.
    /// Awaited stop releases the pool entries and may clear this diagnostic.
    pub fn authentication_failure(&self) -> Option<crate::AuthenticationFailure> {
        self.pool.failure().and_then(|cause| match cause.as_ref() {
            Error::Authentication(failure) => Some(*failure),
            _ => None,
        })
    }
    async fn map_lifecycle_error<T>(
        &self,
        work: impl Future<Output = Result<T, etherbird::Error<Error>>>,
    ) -> Result<T, etherbird::Error<Error>> {
        match work.await {
            Err(etherbird::Error::Lifecycle(cause)) => match cause.as_ref() {
                Error::Authentication(failure) => {
                    Err(etherbird::Error::Operation(Error::Authentication(*failure)))
                }
                _ => Err(etherbird::Error::Lifecycle(cause)),
            },
            result => result,
        }
    }
    pub async fn stop(&self) {
        self.pool.stop().await;
    }

    /// Encoding and mandatory request validation precede bounded admission.
    pub async fn operation<O: crate::Operation>(
        &self,
        operation: O,
    ) -> Result<O::Output, etherbird::Error<Error>> {
        let prepared = crate::operations::custom(operation).map_err(etherbird::Error::Operation)?;
        self.execute_prepared(prepared).await
    }
    async fn execute_prepared<T, D>(
        &self,
        operation: crate::operations::PreparedOperation<D>,
    ) -> Result<T, etherbird::Error<Error>>
    where
        T: Send + 'static,
        D: FnOnce(&[u8]) -> Result<T, Error> + Send + 'static,
    {
        operation
            .validate(self.profile())
            .map_err(etherbird::Error::Operation)?;
        self.map_lifecycle_error(
            self.pool
                .execute_with_timeout(self.call_timeout, move |session| async move {
                    session.request_operation(operation).await
                }),
        )
        .await
    }
    /// call_timeout covers admission and the entire workflow; response_timeout bounds each exchange.
    pub async fn workflow<W: crate::Workflow>(
        &self,
        workflow: W,
    ) -> Result<W::Output, etherbird::Error<Error>> {
        self.map_lifecycle_error(self.pool.execute_with_timeout(
            self.call_timeout,
            move |session| async move {
                let mut context = session.managed().await;
                let result = workflow.run(&mut context).await;
                if result
                    .as_ref()
                    .is_err_and(|error| !error.preserves_connection())
                {
                    session.connected.send_replace(false);
                }
                context.scope.complete();
                let healthy_result = match &result {
                    Ok(_) => true,
                    Err(error) => error.preserves_connection(),
                };
                if healthy_result && !session.is_connected() {
                    return Err(Error::Disconnected);
                }
                result
            },
        ))
        .await
    }
    /// Low-level escape hatch for single unicast exchanges. Payload interpretation is caller-owned.
    pub async fn request(
        &self,
        opcode: u8,
        data: Vec<u8>,
    ) -> Result<Vec<u8>, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::raw(opcode, data))
            .await
    }
    pub async fn read_configuration(&self) -> Result<SystemConfiguration, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::configuration(self.profile.clone()))
            .await
    }
    pub async fn read_point_types(
        &self,
        request: crate::PointTypeRequest,
    ) -> Result<crate::PointTypePage, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::point_types(
            request,
            self.profile.clone(),
        ))
        .await
    }
    /// Returns complete pages from one exclusive generation, or fails without resuming/replay.
    pub async fn scan_point_types(
        &self,
        start: u8,
        count: u16,
    ) -> Result<Vec<crate::PointTypePage>, etherbird::Error<Error>> {
        let scan = crate::discovery::PointTypeScan::new(start, count, self.profile())
            .map_err(etherbird::Error::Operation)?;
        self.workflow(scan).await
    }
    pub async fn read_io_points(
        &self,
        request: crate::IoPointRequest,
    ) -> Result<crate::IoPointReply, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::io(request, self.profile.clone()))
            .await
    }
    pub async fn discover_io(
        &self,
        firmware: crate::FirmwareVersion,
    ) -> Result<crate::IoSnapshot, etherbird::Error<Error>> {
        let layout = self
            .profile()
            .io_layout()
            .ok_or(etherbird::Error::Operation(Error::CapabilityUnavailable(
                "explicit I/O layout profile",
            )))?;
        self.discover_io_with_layout(firmware, layout).await
    }
    pub async fn discover_io_with_layout(
        &self,
        firmware: crate::FirmwareVersion,
        layout: crate::IoLayout,
    ) -> Result<crate::IoSnapshot, etherbird::Error<Error>> {
        crate::profile::validate_profile_request(self.profile(), 6, &[])
            .map_err(etherbird::Error::Operation)?;
        crate::profile::validate_profile_request(self.profile(), 50, &[0])
            .map_err(etherbird::Error::Operation)?;
        crate::profile::validate_expected_response(self.profile(), 50, layout.len())
            .map_err(etherbird::Error::Operation)?;
        self.workflow(crate::IoDiscovery { firmware, layout }).await
    }
    pub async fn read_alarms(
        &self,
        request: crate::LogRequest,
    ) -> Result<crate::AlarmPage, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::logs::<crate::AlarmRecord>(
            request,
            self.profile.clone(),
        ))
        .await
    }
    pub async fn read_events(
        &self,
        request: crate::LogRequest,
    ) -> Result<crate::EventPage, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::logs::<crate::EventRecord>(
            request,
            self.profile.clone(),
        ))
        .await
    }
    /// Table 2-18: one bounded OP-135 page or zero-count current-index probe.
    pub async fn read_history(
        &self,
        request: crate::HistoryRequest,
    ) -> Result<crate::HistoryPage, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::history(request, self.profile.clone()))
            .await
    }
    /// Select a prebuilt or consumer contextual codec without changing transport ownership.
    pub async fn read_history_with_codec<C: crate::ReplyCodec<crate::HistoryRequest>>(
        &self,
        request: crate::HistoryRequest,
        codec: C,
    ) -> Result<C::Output, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::history_with_codec(request, codec))
            .await
    }
    /// Table 2-19: one bounded OP-136 page or zero-period current-index probe.
    pub async fn read_multi_history(
        &self,
        request: crate::MultiHistoryRequest,
    ) -> Result<crate::MultiHistoryPage, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::multi_history(
            request,
            self.profile.clone(),
        ))
        .await
    }
    /// Select a prebuilt or consumer contextual codec without changing transport ownership.
    pub async fn read_multi_history_with_codec<C: crate::ReplyCodec<crate::MultiHistoryRequest>>(
        &self,
        request: crate::MultiHistoryRequest,
        codec: C,
    ) -> Result<C::Output, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::multi_history_with_codec(request, codec))
            .await
    }
    /// Day/month history with no inferred year or snapshot; see docs/DAY_HISTORY.md.
    pub async fn read_day_indices(
        &self,
        request: crate::DayIndicesRequest,
    ) -> Result<crate::DayIndices, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::day_indices(
            request,
            self.profile.clone(),
        ))
        .await
    }
    /// Explicit contextual codec selection under the exchange retirement guard.
    pub async fn read_day_indices_with_codec<C: crate::ReplyCodec<crate::DayIndicesRequest>>(
        &self,
        request: crate::DayIndicesRequest,
        codec: C,
    ) -> Result<C::Output, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::day_indices_with_codec(request, codec))
            .await
    }
    /// Day/month history with no inferred year or snapshot; see docs/DAY_HISTORY.md.
    pub async fn read_history_day(
        &self,
        request: crate::DayHistoryRequest,
    ) -> Result<crate::DayHistoryReply, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::history_day(
            request,
            self.profile.clone(),
        ))
        .await
    }
    /// Explicit contextual codec selection under the exchange retirement guard.
    pub async fn read_history_day_with_codec<C: crate::ReplyCodec<crate::DayHistoryRequest>>(
        &self,
        request: crate::DayHistoryRequest,
        codec: C,
    ) -> Result<C::Output, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::history_day_with_codec(request, codec))
            .await
    }
    /// Table 2-22 inventory or ordered selection; see docs/HISTORY_INFORMATION.md.
    pub async fn read_configured_history(
        &self,
        request: crate::ConfiguredHistoryRequest,
    ) -> Result<crate::ConfiguredHistoryPoints, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::configured_history(
            request,
            self.profile.clone(),
        ))
        .await
    }
    /// Explicit contextual codec under the exchange retirement guard.
    pub async fn read_configured_history_with_codec<
        C: crate::ReplyCodec<crate::ConfiguredHistoryRequest>,
    >(
        &self,
        request: crate::ConfiguredHistoryRequest,
        codec: C,
    ) -> Result<C::Output, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::configured_history_with_codec(
            request, codec,
        ))
        .await
    }
    /// Table 2-22 inventory or ordered selection; see docs/HISTORY_INFORMATION.md.
    pub async fn read_selected_history(
        &self,
        request: crate::SelectedHistoryRequest,
    ) -> Result<crate::SelectedHistoryPage, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::selected_history(
            request,
            self.profile.clone(),
        ))
        .await
    }
    /// Explicit contextual codec under the exchange retirement guard.
    pub async fn read_selected_history_with_codec<
        C: crate::ReplyCodec<crate::SelectedHistoryRequest>,
    >(
        &self,
        request: crate::SelectedHistoryRequest,
        codec: C,
    ) -> Result<C::Output, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::selected_history_with_codec(
            request, codec,
        ))
        .await
    }
    /// Read bounded min/max metadata; occurrence bytes stay raw unless a profile selects a codec.
    pub async fn read_min_max(
        &self,
        request: crate::MinMaxRequest,
    ) -> Result<crate::MinMaxReply, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::min_max(request, self.profile.clone()))
            .await
    }
    /// Explicit per-call codec selection through the same interface as caller implementations.
    /// Preparation uses the baseline 50-byte reply budget; other payload sizes require a custom operation.
    pub async fn read_min_max_with_codec<C: crate::ReplyCodec<crate::MinMaxRequest>>(
        &self,
        request: crate::MinMaxRequest,
        codec: C,
    ) -> Result<C::Output, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::min_max_with_codec(request, codec))
            .await
    }
    /// Read 1–20 ordered history tags and the reported periodic index, without inferred continuity.
    pub async fn read_history_tags(
        &self,
        request: crate::HistoryTagsRequest,
    ) -> Result<crate::HistoryTagsReply, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::history_tags(
            request,
            self.profile.clone(),
        ))
        .await
    }
    pub async fn read_clock(&self) -> Result<Clock, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::clock(self.profile.clone()))
            .await
    }
    pub async fn read_parameters(
        &self,
        parameters: &[Parameter],
    ) -> Result<Vec<Value>, etherbird::Error<Error>> {
        let prepared =
            crate::operations::parameters(parameters).map_err(etherbird::Error::Operation)?;
        self.execute_prepared(prepared).await
    }
    pub async fn read_parameter_block(
        &self,
        block: &ParameterBlock,
    ) -> Result<Vec<Value>, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::block(block)).await
    }
    /// Lookup and plan all parameters before admission. Each attempt holds one generation.
    pub async fn read_schema(
        &self,
        provider: &dyn crate::SchemaProvider,
        context: &crate::SchemaContext,
        tlps: &[crate::Tlp],
    ) -> Result<Vec<Value>, crate::SchemaReadError> {
        let plan =
            crate::ReadPlan::build(provider, context, tlps, self.profile()).map_err(|error| {
                crate::SchemaReadError {
                    progress: Default::default(),
                    error: etherbird::Error::Operation(error),
                }
            })?;
        self.read_plan(plan).await
    }
    pub async fn read_plan(
        &self,
        plan: crate::ReadPlan,
    ) -> Result<Vec<Value>, crate::SchemaReadError> {
        // A plan may have been constructed for another profile; revalidate all budgets.
        for batch in plan.batches() {
            let check = crate::profile::validate_parameter_read(self.profile(), batch);
            check.map_err(|error| crate::SchemaReadError {
                progress: Default::default(),
                error: etherbird::Error::Operation(error),
            })?;
        }
        let progress = Arc::new(std::sync::Mutex::new(crate::ReadProgress::default()));
        let result = self
            .workflow(crate::schema::PlannedRead {
                plan,
                progress: progress.clone(),
            })
            .await;
        let progress = std::mem::take(&mut *progress.lock().unwrap());
        match result {
            Ok(()) => Ok(progress.values),
            Err(error) => Err(crate::SchemaReadError { progress, error }),
        }
    }
}
