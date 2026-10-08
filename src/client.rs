use crate::{
    Error, Session, SystemConfiguration,
    protocol::{
        Address, Clock, CrcMode, Frame, Login, Parameter, ParameterBlock, Value, parameter_request,
    },
    session::validate_request,
    transport::{Connector, SharedConnector, TcpConnector},
};
use etherbird::{
    Config, Lifecycle, LifecycleFuture, OperationQueue, Pool, PoolConfig, QueueError, Supervisor,
};
use std::{collections::VecDeque, future::Future, sync::Arc, time::Duration};
use tokio::{
    sync::{broadcast, watch},
    time::{sleep, timeout},
};

/// Response deadline starts after stream acquisition. Call deadline includes queueing/setup/recovery.
#[derive(Debug, Clone)]
pub struct ClientConfig {
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
impl Default for ClientConfig {
    fn default() -> Self {
        Self {
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
    authentication: watch::Sender<Option<crate::AuthenticationFailure>>,
}
#[etherbird::async_trait]
impl Lifecycle for Hooks {
    type Resource = Session;
    type Error = Error;
    async fn create(&self) -> Result<Session, Error> {
        if let Some(failure) = *self.authentication.borrow() {
            return Err(Error::Authentication(failure));
        }
        // Fresh stream on every generation, bounded by Etherbird create timeout.
        let stream = self.connector.connect().await?;
        Session::with_events(
            stream,
            self.config.host,
            self.config.device,
            self.config.response_timeout,
            self.config.crc_mode,
            self.events.clone(),
        )
    }
    async fn connect(&self, _session: &Session) -> Result<(), Error> {
        Ok(())
    }
    async fn setup(&self, session: &Session) -> Result<(), Error> {
        let mut context = session.managed().await;
        let state = match self.setup.setup(&mut context).await {
            Err(Error::Authentication(failure)) => {
                self.authentication.send_replace(Some(failure));
                return Err(Error::Authentication(failure));
            }
            result => result?,
        };
        if !session.is_connected() {
            return Err(Error::Disconnected);
        }
        *context.state = Some(state);
        context.complete = true;
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
    fn is_expected(&self, error: &Error) -> bool {
        matches!(
            error,
            Error::Io(_) | Error::Timeout | Error::Disconnected | Error::Device(_)
        )
    }
}
struct BoundedFifo<T> {
    items: VecDeque<T>,
    capacity: usize,
}
impl<T: Send> OperationQueue<T> for BoundedFifo<T> {
    fn push(&mut self, item: T) -> Result<(), QueueError> {
        if self.items.len() >= self.capacity {
            return Err(QueueError::Full);
        }
        self.items.push_back(item);
        Ok(())
    }
    fn pop(&mut self) -> Option<T> {
        self.items.pop_front()
    }
    fn len(&self) -> usize {
        self.items.len()
    }
    fn retain(&mut self, predicate: &mut dyn FnMut(&T) -> bool) {
        self.items.retain(predicate);
    }
}

/// Stable client clones share one supervised, exclusively accessed channel.
/// Operations run once. A failure may still have taken effect remotely.
#[derive(Clone)]
pub struct Client {
    pool: Pool<Hooks>,
    call_timeout: Duration,
    events: broadcast::Sender<Frame>,
    authentication: watch::Sender<Option<crate::AuthenticationFailure>>,
}
impl Client {
    pub fn tcp(endpoint: impl Into<String>, config: ClientConfig) -> Result<Self, Error> {
        Self::new(Arc::new(TcpConnector::new(endpoint)), config)
    }
    pub fn new(connector: Arc<dyn Connector>, config: ClientConfig) -> Result<Self, Error> {
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
            || config.recovery.retry_delay.is_zero()
            || config.recovery.max_retry_delay < config.recovery.retry_delay
        {
            return Err(Error::InvalidConfig(
                "invalid addresses, capacity or deadlines",
            ));
        }
        if config.recovery.create_timeout.is_none() {
            config.recovery.create_timeout = Some(config.recovery.connect_timeout);
        }
        let (events, _) = broadcast::channel(32);
        let authentication = watch::channel(None).0;
        let hooks = Hooks {
            connector,
            config: config.clone(),
            setup,
            events: events.clone(),
            authentication: authentication.clone(),
        };
        let recovery = config.recovery.clone();
        let capacity = config.queue_capacity;
        let pool = Pool::new_with_queue_factory(
            move || Supervisor::new(hooks.clone(), recovery.clone()),
            PoolConfig {
                min_size: 1,
                max_size: 1,
                ..PoolConfig::default()
            },
            move || BoundedFifo {
                items: VecDeque::new(),
                capacity,
            },
        );
        Ok(Self {
            pool,
            call_timeout: config.call_timeout,
            events,
            authentication,
        })
    }
    pub fn state(&self) -> etherbird::ResourceState {
        self.pool.state()
    }
    pub fn subscribe_state(&self) -> watch::Receiver<u64> {
        self.pool.subscribe()
    }
    /// This receiver survives transport replacements. Lag is reported by Tokio.
    pub fn subscribe_events(&self) -> broadcast::Receiver<Frame> {
        self.events.subscribe()
    }
    pub async fn connected(&self) -> Result<(), etherbird::Error<Error>> {
        self.pool.begin();
        self.wait_managed(self.pool.connected()).await
    }
    /// Terminal setup failure, shared by clones. It cannot be cleared in place.
    pub fn authentication_failure(&self) -> Option<crate::AuthenticationFailure> {
        *self.authentication.borrow()
    }
    async fn wait_managed<T>(
        &self,
        work: impl Future<Output = Result<T, etherbird::Error<Error>>>,
    ) -> Result<T, etherbird::Error<Error>> {
        let mut authentication = self.authentication.subscribe();
        let mut stopping = self.pool.stopped();
        let failure = async {
            loop {
                if let Some(failure) = *authentication.borrow_and_update() {
                    return etherbird::Error::Operation(Error::Authentication(failure));
                }
                if authentication.changed().await.is_err() {
                    return etherbird::Error::Operation(Error::Disconnected);
                }
            }
        };
        let stopped = async {
            loop {
                if *stopping.borrow_and_update() {
                    return;
                }
                if stopping.changed().await.is_err() {
                    return;
                }
            }
        };
        tokio::select! {
            biased;
            _ = stopped => Err(etherbird::Error::Stopped),
            error = failure => Err(error),
            result = timeout(self.call_timeout, work) => result.map_err(|_| etherbird::Error::Timeout)?,
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
        let (opcode, data) = operation.encode().map_err(etherbird::Error::Operation)?;
        validate_request(opcode, &data).map_err(etherbird::Error::Operation)?;
        let result = self
            .wait_managed(self.pool.execute(move |session| async move {
                let mut context = session.managed().await;
                let result = context
                    .session
                    .exchange_decoded(opcode, data, |bytes| {
                        operation.decode(bytes).map_err(Error::from)
                    })
                    .await;
                context.complete = true;
                preserve_device_error(result)
            }))
            .await?;
        result.map_err(etherbird::Error::Operation)
    }
    /// call_timeout covers admission and the entire workflow; response_timeout bounds each exchange.
    pub async fn workflow<W: crate::Workflow>(
        &self,
        workflow: W,
    ) -> Result<W::Output, etherbird::Error<Error>> {
        let result = self
            .wait_managed(self.pool.execute(move |session| async move {
                let mut context = session.managed().await;
                let result = workflow.run(&mut context).await;
                if result
                    .as_ref()
                    .is_err_and(|error| !error.preserves_connection())
                {
                    session.connected.send_replace(false);
                }
                context.complete = true;
                let healthy_result = match &result {
                    Ok(_) => true,
                    Err(error) => error.preserves_connection(),
                };
                if healthy_result && !session.is_connected() {
                    return Err(Error::Disconnected);
                }
                preserve_device_error(result)
            }))
            .await?;
        result.map_err(etherbird::Error::Operation)
    }
    /// Low-level escape hatch for single unicast exchanges. Payload interpretation is caller-owned.
    pub async fn request(
        &self,
        opcode: u8,
        data: Vec<u8>,
    ) -> Result<Vec<u8>, etherbird::Error<Error>> {
        validate_request(opcode, &data).map_err(etherbird::Error::Operation)?;
        let result = self
            .wait_managed(self.pool.execute(move |session| async move {
                preserve_device_error(session.request(opcode, data).await)
            }))
            .await?;
        result.map_err(etherbird::Error::Operation)
    }
    pub async fn read_configuration(&self) -> Result<SystemConfiguration, etherbird::Error<Error>> {
        let result = self
            .wait_managed(self.pool.execute(|session| async move {
                preserve_device_error(session.read_configuration().await)
            }))
            .await?;
        result.map_err(etherbird::Error::Operation)
    }
    pub async fn read_clock(&self) -> Result<Clock, etherbird::Error<Error>> {
        let result = self
            .wait_managed(self.pool.execute(|session| async move {
                preserve_device_error(session.read_clock().await)
            }))
            .await?;
        result.map_err(etherbird::Error::Operation)
    }
    pub async fn read_parameters(
        &self,
        parameters: &[Parameter],
    ) -> Result<Vec<Value>, etherbird::Error<Error>> {
        parameter_request(parameters).map_err(etherbird::Error::Operation)?;
        let parameters = parameters.to_vec();
        let result = self
            .wait_managed(self.pool.execute(move |session| async move {
                preserve_device_error(session.read_parameters(&parameters).await)
            }))
            .await?;
        result.map_err(etherbird::Error::Operation)
    }
    pub async fn read_parameter_block(
        &self,
        block: &ParameterBlock,
    ) -> Result<Vec<Value>, etherbird::Error<Error>> {
        let block = block.clone();
        let result = self
            .wait_managed(self.pool.execute(move |session| async move {
                preserve_device_error(session.read_parameter_block(&block).await)
            }))
            .await?;
        result.map_err(etherbird::Error::Operation)
    }
}
// Device rejection is a successful exchange at the transport layer. It must not
// force reconnect/login on a healthy session; retain the original protocol error.
fn preserve_device_error<T>(result: Result<T, Error>) -> Result<Result<T, Error>, Error> {
    match result {
        Err(error) if error.preserves_connection() => Ok(Err(error)),
        Err(error) => Err(error),
        Ok(value) => Ok(Ok(value)),
    }
}
