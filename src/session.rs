use crate::{
    Error, SystemConfiguration,
    protocol::{
        Address, Clock, CrcMode, DeviceError, Frame, Login, Parameter, ParameterBlock, Value,
        decode_parameters,
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
struct Exchange<'a> {
    stream: &'a mut Option<Stream>,
    connected: &'a watch::Sender<bool>,
    complete: bool,
}
impl Drop for Exchange<'_> {
    fn drop(&mut self) {
        if !self.complete {
            self.stream.take();
            self.connected.send_replace(false);
        }
    }
}
impl Session {
    pub fn new(
        stream: Stream,
        host: Address,
        device: Address,
        response_timeout: Duration,
        crc_mode: CrcMode,
    ) -> Result<Self, Error> {
        let (events, _) = broadcast::channel(32);
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
        self.request_decoded(opcode, data, |bytes| Ok(bytes.to_vec()))
            .await
    }

    // Decode while the exchange still owns the stream. A malformed typed reply
    // must poison this generation before another caller can acquire it.
    async fn request_decoded<T>(
        &self,
        opcode: u8,
        data: Vec<u8>,
        decode: impl FnOnce(&[u8]) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let _ownership = self.ownership.lock().await;
        self.exchange_decoded(opcode, data, decode).await
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
    pub(crate) async fn exchange_decoded<T>(
        &self,
        opcode: u8,
        data: Vec<u8>,
        decode: impl FnOnce(&[u8]) -> Result<T, Error>,
    ) -> Result<T, Error> {
        crate::profile::validate_profile_request(self.profile.as_ref(), opcode, &data)?;
        let frame = Frame::new(self.device, self.host, opcode, data);
        let mut slot = self.stream.lock().await;
        if slot.is_none() || !self.is_connected() {
            return Err(Error::Disconnected);
        }
        let mut exchange = Exchange {
            stream: &mut slot,
            connected: &self.connected,
            complete: false,
        };
        let response = timeout(self.response_timeout, async {
            let stream = exchange.stream.as_mut().ok_or(Error::Disconnected)?;
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
        exchange.complete = true;
        response
    }
    pub async fn read_configuration(&self) -> Result<SystemConfiguration, Error> {
        self.request_decoded(6, vec![], |bytes| {
            self.profile
                .decode_configuration(bytes)
                .map_err(Error::from)
        })
        .await
    }
    pub async fn read_clock(&self) -> Result<Clock, Error> {
        self.request_decoded(7, vec![], |bytes| {
            self.profile.decode_clock(bytes).map_err(Error::from)
        })
        .await
    }
    pub async fn read_point_types(
        &self,
        request: crate::PointTypeRequest,
    ) -> Result<crate::PointTypePage, Error> {
        crate::profile::validate_expected_response(
            self.profile.as_ref(),
            100,
            request.response_size(),
        )?;
        self.request_decoded(100, request.payload(), |bytes| {
            self.profile
                .decode_point_types(&request, bytes)
                .map_err(Error::from)
        })
        .await
    }
    pub async fn read_io_points(
        &self,
        request: crate::IoPointRequest,
    ) -> Result<crate::IoPointReply, Error> {
        crate::profile::validate_expected_response(
            self.profile.as_ref(),
            50,
            request.layout.len(),
        )?;
        self.request_decoded(50, request.payload(), |bytes| {
            self.profile
                .decode_io_points(&request, bytes)
                .map_err(Error::from)
        })
        .await
    }
    pub async fn login(&self, login: &Login) -> Result<(), Error> {
        self.request_decoded(17, login.payload(), |bytes| {
            if !bytes.is_empty() {
                return Err(Error::InvalidResponse(
                    "login acknowledgement must be empty",
                ));
            }
            Ok(())
        })
        .await
    }
    pub async fn read_parameters(&self, parameters: &[Parameter]) -> Result<Vec<Value>, Error> {
        self.request_decoded(
            180,
            crate::profile::validate_parameter_read(self.profile.as_ref(), parameters)?,
            |bytes| decode_parameters(parameters, bytes),
        )
        .await
    }
    pub async fn read_parameter_block(&self, block: &ParameterBlock) -> Result<Vec<Value>, Error> {
        crate::profile::validate_expected_response(
            self.profile.as_ref(),
            167,
            block.response_size(),
        )?;
        self.request_decoded(167, block.payload(), |bytes| block.decode(bytes))
            .await
    }
}
pub(crate) fn validate_request(opcode: u8, data: &[u8]) -> Result<(), Error> {
    if matches!(opcode, 24 | 224 | 255) {
        return Err(Error::InvalidRequest(
            "opcode is routed or device-originated; unsupported by unicast request API",
        ));
    }
    if data.len() > u8::MAX as usize {
        return Err(Error::InvalidRequest("payload exceeds 255 bytes"));
    }
    Ok(())
}

static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);
