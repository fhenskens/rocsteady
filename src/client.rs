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
use std::{collections::VecDeque, sync::Arc, time::Duration};
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
    events: broadcast::Sender<Frame>,
}
#[etherbird::async_trait]
impl Lifecycle for Hooks {
    type Resource = Session;
    type Error = Error;
    async fn create(&self) -> Result<Session, Error> {
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
        if let Some(login) = &self.config.login {
            session.login(login).await?;
        }
        // Transport connection is insufficient: prove protocol readiness before publishing resource.
        session.read_clock().await?;
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
}
impl Client {
    pub fn tcp(endpoint: impl Into<String>, config: ClientConfig) -> Result<Self, Error> {
        Self::new(Arc::new(TcpConnector::new(endpoint)), config)
    }
    pub fn new(connector: Arc<dyn Connector>, mut config: ClientConfig) -> Result<Self, Error> {
        config.host.validate_unicast()?;
        config.device.validate_unicast()?;
        if config.host == config.device
            || config.queue_capacity == 0
            || config.response_timeout.is_zero()
            || config.call_timeout.is_zero()
            || config.heartbeat_interval.is_some_and(|d| d.is_zero())
        {
            return Err(Error::InvalidConfig(
                "invalid addresses, capacity or deadlines",
            ));
        }
        if config.recovery.create_timeout.is_none() {
            config.recovery.create_timeout = Some(config.recovery.connect_timeout);
        }
        let (events, _) = broadcast::channel(32);
        let hooks = Hooks {
            connector,
            config: config.clone(),
            events: events.clone(),
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
        timeout(self.call_timeout, self.pool.connected())
            .await
            .map_err(|_| etherbird::Error::Timeout)?
    }
    pub async fn stop(&self) {
        self.pool.stop().await;
    }

    /// Low-level escape hatch for single unicast exchanges. Payload interpretation is caller-owned.
    pub async fn request(
        &self,
        opcode: u8,
        data: Vec<u8>,
    ) -> Result<Vec<u8>, etherbird::Error<Error>> {
        validate_request(opcode, &data).map_err(etherbird::Error::Operation)?;
        let result = timeout(
            self.call_timeout,
            self.pool.execute(move |session| async move {
                preserve_device_error(session.request(opcode, data).await)
            }),
        )
        .await
        .map_err(|_| etherbird::Error::Timeout)??;
        result.map_err(etherbird::Error::Operation)
    }
    pub async fn read_configuration(&self) -> Result<SystemConfiguration, etherbird::Error<Error>> {
        let result = timeout(
            self.call_timeout,
            self.pool.execute(|session| async move {
                preserve_device_error(session.read_configuration().await)
            }),
        )
        .await
        .map_err(|_| etherbird::Error::Timeout)??;
        result.map_err(etherbird::Error::Operation)
    }
    pub async fn read_clock(&self) -> Result<Clock, etherbird::Error<Error>> {
        let result = timeout(
            self.call_timeout,
            self.pool.execute(|session| async move {
                preserve_device_error(session.read_clock().await)
            }),
        )
        .await
        .map_err(|_| etherbird::Error::Timeout)??;
        result.map_err(etherbird::Error::Operation)
    }
    pub async fn read_parameters(
        &self,
        parameters: &[Parameter],
    ) -> Result<Vec<Value>, etherbird::Error<Error>> {
        parameter_request(parameters).map_err(etherbird::Error::Operation)?;
        let parameters = parameters.to_vec();
        let result = timeout(
            self.call_timeout,
            self.pool.execute(move |session| async move {
                preserve_device_error(session.read_parameters(&parameters).await)
            }),
        )
        .await
        .map_err(|_| etherbird::Error::Timeout)??;
        result.map_err(etherbird::Error::Operation)
    }
    pub async fn read_parameter_block(
        &self,
        block: &ParameterBlock,
    ) -> Result<Vec<Value>, etherbird::Error<Error>> {
        let block = block.clone();
        let result = timeout(
            self.call_timeout,
            self.pool.execute(move |session| async move {
                preserve_device_error(session.read_parameter_block(&block).await)
            }),
        )
        .await
        .map_err(|_| etherbird::Error::Timeout)??;
        result.map_err(etherbird::Error::Operation)
    }
}
// Device rejection is a successful exchange at the transport layer. It must not
// force reconnect/login on a healthy session; retain the original protocol error.
fn preserve_device_error<T>(result: Result<T, Error>) -> Result<Result<T, Error>, Error> {
    match result {
        Err(error @ Error::Device(_)) => Ok(Err(error)),
        Err(error) => Err(error),
        Ok(value) => Ok(Ok(value)),
    }
}
