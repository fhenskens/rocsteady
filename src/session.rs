use crate::{
    Error, SystemConfiguration,
    protocol::{
        Address, Clock, CrcMode, DeviceError, Frame, Login, Parameter, ParameterBlock, Value,
    },
    transport::{Stream, read_frame, write_frame},
};
use std::{
    sync::Arc,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use tokio::{
    sync::{Mutex, broadcast, watch},
    time::timeout,
};

/// One serialized connection generation. Use Client for managed replacement.
pub struct Session {
    stream: Mutex<Option<Stream>>,
    ownership: Mutex<()>,
    pub(crate) generation: u64,
    state: Mutex<Option<crate::SessionState>>,
    host: Address,
    device: Address,
    response_timeout: Duration,
    crc_mode: CrcMode,
    pub(crate) connected: watch::Sender<bool>,
    events: broadcast::Sender<Frame>,
    pub(crate) profile: Arc<dyn crate::ProtocolProfile>,
}
impl Session {
    pub fn new(
        stream: Stream,
        host: Address,
        device: Address,
        response_timeout: Duration,
        crc_mode: CrcMode,
    ) -> Result<Self, Error> {
        let (events, _) = broadcast::channel(crate::SRBX_BUFFER_CAPACITY);
        Self::with_events(
            stream,
            host,
            device,
            response_timeout,
            crc_mode,
            events,
            Arc::new(crate::DefaultProfile),
        )
    }
    pub(crate) fn with_events(
        stream: Stream,
        host: Address,
        device: Address,
        response_timeout: Duration,
        crc_mode: CrcMode,
        events: broadcast::Sender<Frame>,
        profile: Arc<dyn crate::ProtocolProfile>,
    ) -> Result<Self, Error> {
        host.validate_unicast()?;
        device.validate_unicast()?;
        if host == device {
            return Err(Error::InvalidConfig(
                "host and device addresses must differ",
            ));
        }
        if response_timeout.is_zero() {
            return Err(Error::InvalidConfig("response timeout must be positive"));
        }
        Ok(Self {
            stream: Mutex::new(Some(stream)),
            ownership: Mutex::new(()),
            generation: NEXT_GENERATION.fetch_add(1, Ordering::Relaxed),
            state: Mutex::new(None),
            host,
            device,
            response_timeout,
            crc_mode,
            connected: watch::channel(true).0,
            events,
            profile,
        })
    }
    /// Subscribe to SRBX frames encountered by requests/heartbeats. Receiver may lag.
    pub fn subscribe_events(&self) -> broadcast::Receiver<Frame> {
        self.events.subscribe()
    }
    pub fn is_connected(&self) -> bool {
        *self.connected.borrow()
    }
    pub async fn disconnect(&self) {
        self.stream.lock().await.take();
        self.connected.send_replace(false);
        self.state.lock().await.take();
    }

    /// Run one unicast exchange. No retry or side-effect inference is performed.
    /// Routed/store-and-forward requests and broadcasts are not supported.
    pub async fn request(&self, opcode: u8, data: Vec<u8>) -> Result<Vec<u8>, Error> {
        self.request_operation(crate::operations::raw(opcode, data))
            .await
    }

    pub(crate) async fn managed(&self) -> crate::ManagedContext<'_> {
        let ownership = self.ownership.lock().await;
        crate::ManagedContext {
            session: self,
            complete: false,
            state: self.state.lock().await,
            _ownership: ownership,
        }
    }
    pub(crate) async fn exchange_operation<T, D>(
        &self,
        operation: crate::operations::PreparedOperation<D>,
    ) -> Result<T, Error>
    where
        D: FnOnce(&[u8]) -> Result<T, Error>,
    {
        operation.validate(self.profile.as_ref())?;
        let crate::operations::PreparedOperation {
            opcode,
            payload: data,
            decode,
            ..
        } = operation;
        let frame = Frame::new(self.device, self.host, opcode, data);
        let mut slot = self.stream.lock().await;
        if slot.is_none() || !self.is_connected() {
            return Err(Error::Disconnected);
        }
        let mut exchange = etherbird::RetireOnDrop::new(&mut slot, &self.connected);
        let response = timeout(self.response_timeout, async {
            let stream = exchange.resource().ok_or(Error::Disconnected)?;
            write_frame(stream.as_mut(), &frame).await?;
            loop {
                let response = read_frame(stream.as_mut(), self.crc_mode).await?;
                if response.destination != self.host
                    || response.source.validate_unicast().is_err()
                    || !self.profile.accepts_source(self.device, response.source)
                {
                    return Err(Error::InvalidResponse(
                        "response addresses do not match request",
                    ));
                }
                if response.opcode == 224 {
                    if !response.data.is_empty() {
                        return Err(Error::InvalidResponse("SRBX signal has unexpected data"));
                    }
                    let _ = self.events.send(response);
                    continue;
                }
                if response.opcode == 255 {
                    // A valid protocol rejection completes this exchange without retiring the connection.
                    let errors = DeviceError::decode(&response.data)?;
                    return Ok(Err(Error::Device(errors)));
                }
                if response.opcode != opcode {
                    return Err(Error::InvalidResponse(
                        "response opcode does not match request",
                    ));
                }
                if response.data.len() > self.profile.limits(opcode).response.min(255) {
                    return Err(Error::InvalidResponse(
                        "reply exceeds selected profile limit",
                    ));
                }
                return decode(&response.data).map(Ok);
            }
        })
        .await
        .map_err(|_| Error::Timeout)??;
        exchange.complete();
        response
    }
    pub(crate) async fn request_operation<T, D>(
        &self,
        operation: crate::operations::PreparedOperation<D>,
    ) -> Result<T, Error>
    where
        D: FnOnce(&[u8]) -> Result<T, Error>,
    {
        operation.validate(self.profile.as_ref())?;
        let _ownership = self.ownership.lock().await;
        self.exchange_operation(operation).await
    }
    pub async fn read_configuration(&self) -> Result<SystemConfiguration, Error> {
        self.request_operation(crate::operations::configuration(self.profile.clone()))
            .await
    }
    pub async fn read_alarms(&self, request: crate::LogRequest) -> Result<crate::AlarmPage, Error> {
        self.request_operation(crate::operations::logs::<crate::AlarmRecord>(
            request,
            self.profile.clone(),
        ))
        .await
    }
    pub async fn read_events(&self, request: crate::LogRequest) -> Result<crate::EventPage, Error> {
        self.request_operation(crate::operations::logs::<crate::EventRecord>(
            request,
            self.profile.clone(),
        ))
        .await
    }
    /// Table 2-18: one bounded OP-135 page or zero-count current-index probe.
    pub async fn read_history(
        &self,
        request: crate::HistoryRequest,
    ) -> Result<crate::HistoryPage, Error> {
        self.request_operation(crate::operations::history(request, self.profile.clone()))
            .await
    }
    /// Select a prebuilt or consumer contextual codec without changing transport ownership.
    pub async fn read_history_with_codec<C: crate::ReplyCodec<crate::HistoryRequest>>(
        &self,
        request: crate::HistoryRequest,
        codec: C,
    ) -> Result<C::Output, Error> {
        self.request_operation(crate::operations::history_with_codec(request, codec))
            .await
    }
    /// Read bounded min/max metadata; occurrence bytes stay raw unless a profile selects a codec.
    pub async fn read_min_max(
        &self,
        request: crate::MinMaxRequest,
    ) -> Result<crate::MinMaxReply, Error> {
        self.request_operation(crate::operations::min_max(request, self.profile.clone()))
            .await
    }
    /// Explicit per-call codec selection through the same interface as caller implementations.
    /// Preparation uses the baseline 50-byte reply budget; other payload sizes require a custom operation.
    pub async fn read_min_max_with_codec<C: crate::ReplyCodec<crate::MinMaxRequest>>(
        &self,
        request: crate::MinMaxRequest,
        codec: C,
    ) -> Result<C::Output, Error> {
        self.request_operation(crate::operations::min_max_with_codec(request, codec))
            .await
    }
    /// Read 1–20 ordered history tags and the reported periodic index, without inferred continuity.
    pub async fn read_history_tags(
        &self,
        request: crate::HistoryTagsRequest,
    ) -> Result<crate::HistoryTagsReply, Error> {
        self.request_operation(crate::operations::history_tags(
            request,
            self.profile.clone(),
        ))
        .await
    }
    pub async fn read_clock(&self) -> Result<Clock, Error> {
        self.request_operation(crate::operations::clock(self.profile.clone()))
            .await
    }
    pub async fn read_point_types(
        &self,
        request: crate::PointTypeRequest,
    ) -> Result<crate::PointTypePage, Error> {
        self.request_operation(crate::operations::point_types(
            request,
            self.profile.clone(),
        ))
        .await
    }
    pub async fn read_io_points(
        &self,
        request: crate::IoPointRequest,
    ) -> Result<crate::IoPointReply, Error> {
        self.request_operation(crate::operations::io(request, self.profile.clone()))
            .await
    }
    pub async fn login(&self, login: &Login) -> Result<(), Error> {
        self.request_operation(crate::operations::login(login))
            .await
    }
    pub async fn read_parameters(&self, parameters: &[Parameter]) -> Result<Vec<Value>, Error> {
        self.request_operation(crate::operations::parameters(parameters)?)
            .await
    }
    pub async fn read_parameter_block(&self, block: &ParameterBlock) -> Result<Vec<Value>, Error> {
        self.request_operation(crate::operations::block(block))
            .await
    }
}

static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);
