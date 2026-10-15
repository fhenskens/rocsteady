//! ROC Plus read/SRBX-acknowledgement fixture model and fault-injectable TCP server.
//! Uses the same byte-stream seam as client transports, not a mocked Client API.
//! This is a bounded test fixture, not a full ROC800 emulator.
use crate::{
    Address, Clock, Error, Login, SystemConfiguration, Tlp, Value,
    protocol::{CrcMode, Frame, MAX_PARAMETER_RESPONSE},
    transport::{ByteStream, read_frame, write_frame},
};
use std::{
    collections::{BTreeMap, VecDeque},
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::AsyncWriteExt,
    net::TcpListener,
    sync::{broadcast, watch},
    task::{JoinHandle, JoinSet},
};

#[derive(Debug, Clone)]
pub struct Model {
    /// Explicit raw log slots. Missing slots return the NoAlarm/NoEvent marker.
    pub alarm_log: BTreeMap<u16, [u8; 23]>,
    pub event_log: BTreeMap<u16, [u8; 22]>,
    /// Fixture convention: last appended slot. This is not an automatic client cursor policy.
    pub alarm_index: u16,
    pub event_index: u16,
    pub initial_srbx_pending: bool,
    pub srbx_host: Address,
    pub io_layout: crate::IoLayout,
    pub io_point_types: Vec<u8>,
    pub io_logical_numbers: Vec<u8>,
    pub point_types: BTreeMap<u8, crate::PointTypeClassification>,
    pub configuration: SystemConfiguration,
    pub address: Address,
    pub clock: Clock,
    pub login: Option<Login>,
    pub parameters: BTreeMap<Tlp, Value>,
}
impl Default for Model {
    fn default() -> Self {
        Self {
            initial_srbx_pending: false,
            srbx_host: Address::new(1, 0),
            alarm_log: BTreeMap::new(),
            event_log: BTreeMap::new(),
            alarm_index: 449,
            event_index: 449,
            io_layout: crate::IoLayout::Compatibility160,
            io_point_types: {
                let mut points = vec![0; 160];
                points[..5].fill(109);
                points
            },
            io_logical_numbers: {
                let mut points = vec![0; 160];
                points[..5].copy_from_slice(&[0, 1, 2, 3, 4]);
                points
            },
            point_types: [91, 95, 101, 102, 103, 104, 105, 109, 136, 139]
                .into_iter()
                .map(|point_type| (point_type, crate::PointTypeClassification::Roc))
                .collect(),
            configuration: {
                let mut bytes = vec![0; 215];
                bytes[0] = 1;
                bytes[1] = 1;
                bytes[5] = 1;
                bytes[18] = 6;
                bytes[50] = 1; // One system-variables point.
                bytes[68] = 5; // Five CPU diagnostic points.
                bytes[95] = 1; // One ROC clock point; values are supplied via parameters.
                SystemConfiguration::decode(&bytes).expect("valid fixture configuration")
            },
            address: Address::new(1, 2),
            clock: Clock {
                second: 1,
                minute: 2,
                hour: 3,
                day: 8,
                month: 10,
                year: 2026,
                weekday: 5,
            },
            login: None,
            parameters: BTreeMap::new(),
        }
    }
}
/// Optional fixture data for OP-105/108, supplied with Simulator::bind_with_history.
/// Kept separate so existing public Model struct literals remain compatible.
#[derive(Debug, Clone)]
pub struct HistoryMetadataModel {
    /// Explicit Table 2-14 reply bodies (48 bytes after segment/point echoes).
    pub history_min_max: BTreeMap<(u8, u8), [u8; 48]>,
    pub history_tags: BTreeMap<(u8, u8), [u8; 10]>,
    pub history_periodic_indices: BTreeMap<u8, u16>,
    /// Fixture applicability only; these limits are not firmware discovery.
    pub history_max_segment: u8,
    pub history_max_point: u8,
    /// Explicit OP-135 words indexed by (segment, point, selector), not codec-generated values.
    pub history_samples: BTreeMap<(u8, u8, u8), Vec<[u8; 4]>>,
    pub history_sample_indices: BTreeMap<(u8, u8), u16>,
}
impl Default for HistoryMetadataModel {
    fn default() -> Self {
        Self {
            history_min_max: BTreeMap::new(),
            history_tags: BTreeMap::new(),
            history_periodic_indices: BTreeMap::new(),
            history_max_segment: 10,
            history_max_point: 199,
            history_samples: BTreeMap::new(),
            history_sample_indices: BTreeMap::new(),
        }
    }
}
impl Model {
    /// Reset fixture I/O to five CPU diagnostics and absent field modules in an explicit layout.
    pub fn set_io_layout(&mut self, layout: crate::IoLayout) {
        self.io_layout = layout;
        self.io_point_types = vec![0; layout.len()];
        self.io_point_types[..5].fill(109);
        self.io_logical_numbers = vec![0; layout.len()];
        self.io_logical_numbers[..5].copy_from_slice(&[0, 1, 2, 3, 4]);
        let mut bytes = self.configuration.as_bytes().to_vec();
        bytes[4] = layout.status();
        self.configuration =
            SystemConfiguration::decode(&bytes).expect("existing fixture configuration");
    }
    /// Append using the fixture's last-written cursor convention, replacing one ring slot.
    pub fn append_alarm(&mut self, bytes: [u8; 23]) -> Result<u16, Error> {
        append_log(&mut self.alarm_log, &mut self.alarm_index, bytes)
    }
    pub fn append_event(&mut self, bytes: [u8; 22]) -> Result<u16, Error> {
        append_log(&mut self.event_log, &mut self.event_index, bytes)
    }
    /// Model-level request dispatch also works over in-memory or serial fixture channels.
    pub fn respond(&self, request: &Frame, authenticated: &mut bool) -> Option<Frame> {
        self.respond_with_history(request, authenticated, &HistoryMetadataModel::default())
    }
    /// Serve explicit metadata fixtures through the same authentication/address checks.
    pub fn respond_with_history(
        &self,
        request: &Frame,
        authenticated: &mut bool,
        history: &HistoryMetadataModel,
    ) -> Option<Frame> {
        if request.destination != self.address || request.source.unit == 0 {
            return None;
        }
        let reply = |opcode, data| Frame::new(request.source, self.address, opcode, data);
        let error = |code, offset| reply(255, vec![code, offset]);
        if request.opcode == 17 {
            if !matches!(request.data.len(), 5 | 6) {
                return Some(error(6, 5));
            }
            if self
                .login
                .as_ref()
                .is_some_and(|expected| request.data != expected.payload())
            {
                return Some(error(21, 9));
            }
            *authenticated = true;
            return Some(reply(17, vec![]));
        }
        if self.login.is_some() && !*authenticated {
            return Some(error(20, 4));
        }
        Some(match request.opcode {
            225 => {
                let [low, high] = request.data.as_slice() else {
                    return Some(error(6, 5));
                };
                if u16::from_le_bytes([*low, *high]) >= crate::LOG_CAPACITY {
                    error(25, 6)
                } else {
                    reply(225, vec![])
                }
            }
            105 => {
                let [segment, point] = request.data.as_slice() else {
                    return Some(error(6, 5));
                };
                if *segment > history.history_max_segment {
                    return Some(error(14, 6));
                }
                if let Some(body) = history.history_min_max.get(&(*segment, *point)) {
                    let mut bytes = vec![*segment, *point];
                    bytes.extend(body);
                    reply(105, bytes)
                } else {
                    error(14, 7)
                }
            }
            108 => {
                let Some((&segment, rest)) = request.data.split_first() else {
                    return Some(error(6, 5));
                };
                let Some((&count, points)) = rest.split_first() else {
                    return Some(error(6, 5));
                };
                if segment > history.history_max_segment
                    || !(1..=20).contains(&count)
                    || points.len() != usize::from(count)
                {
                    return Some(error(14, 6));
                }
                let Some(index) = history.history_periodic_indices.get(&segment) else {
                    return Some(error(14, 6));
                };
                let mut bytes = vec![segment, count];
                bytes.extend(index.to_le_bytes());
                for point in points {
                    if *point > history.history_max_point {
                        return Some(error(14, 7));
                    }
                    let Some(tag) = history.history_tags.get(&(segment, *point)) else {
                        return Some(error(14, 7));
                    };
                    bytes.push(*point);
                    bytes.extend(tag);
                }
                reply(108, bytes)
            }
            135 => {
                let [segment, point, kind, low, high, count] = request.data.as_slice() else {
                    return Some(error(6, 5));
                };
                if *segment > history.history_max_segment
                    || *point > history.history_max_point
                    || *kind > 4
                    || *count > 60
                {
                    return Some(error(14, 6));
                }
                let Some(words) = history.history_samples.get(&(*segment, *point, *kind)) else {
                    return Some(error(14, 7));
                };
                let Some(current) = history.history_sample_indices.get(&(*segment, *kind)) else {
                    return Some(error(14, 8));
                };
                let start = usize::from(u16::from_le_bytes([*low, *high]));
                if start >= words.len()
                    || words.len() > 65536
                    || usize::from(*current) >= words.len()
                {
                    return Some(error(14, 9));
                }
                let n = usize::from(*count).min(words.len() - start);
                let mut bytes = vec![*segment, *point];
                bytes.extend(current.to_le_bytes());
                bytes.push(n as u8);
                for word in &words[start..start + n] {
                    bytes.extend(word);
                }
                reply(135, bytes)
            }
            118 => match log_response(&request.data, self.alarm_index, &self.alarm_log) {
                Ok(data) => reply(118, data),
                Err(_) => error(25, 6),
            },
            119 => match log_response(&request.data, self.event_index, &self.event_log) {
                Ok(data) => reply(119, data),
                Err(_) => error(25, 6),
            },
            50 => {
                let selector = match request.data.as_slice() {
                    [0] => &self.io_point_types,
                    [1] => &self.io_logical_numbers,
                    [_] => return Some(error(25, 6)),
                    _ => return Some(error(if request.data.is_empty() { 6 } else { 5 }, 5)),
                };
                if selector.len() != self.io_layout.len() {
                    error(50, 4)
                } else {
                    reply(50, selector.clone())
                }
            }
            100 => {
                let [command, start, count] = request.data.as_slice() else {
                    return Some(error(if request.data.len() < 3 { 6 } else { 5 }, 5));
                };
                if *command != 11 {
                    return Some(error(50, 6));
                }
                if *count > 245 || u16::from(*start) + u16::from(*count) > 256 {
                    return Some(error(25, 8));
                }
                let mut bytes = request.data.clone();
                for point_type in u16::from(*start)..u16::from(*start) + u16::from(*count) {
                    bytes.push(
                        self.point_types
                            .get(&(point_type as u8))
                            .copied()
                            .unwrap_or(crate::PointTypeClassification::Absent)
                            .as_byte(),
                    );
                }
                reply(100, bytes)
            }
            6 => {
                if !request.data.is_empty() {
                    return Some(error(5, 5));
                }
                reply(6, self.configuration.as_bytes().to_vec())
            }
            7 => {
                if !request.data.is_empty() {
                    return Some(error(5, 5));
                }
                match self.clock.encode() {
                    Ok(bytes) => reply(7, bytes),
                    Err(_) => error(50, 4),
                }
            }
            180 => {
                let Some(&count) = request.data.first() else {
                    return Some(error(6, 5));
                };
                if count == 0 || request.data.len() != 1 + usize::from(count) * 3 {
                    return Some(error(6, 5));
                }
                let mut bytes = vec![count];
                for (i, tlp) in request.data[1..].as_chunks::<3>().0.iter().enumerate() {
                    let key = Tlp::new(tlp[0], tlp[1], tlp[2]);
                    let Some(value) = self.parameters.get(&key) else {
                        return Some(error(2, (i + 1) as u8));
                    };
                    let Ok(value) = value.encode() else {
                        return Some(error(50, 4));
                    };
                    bytes.extend_from_slice(tlp);
                    bytes.extend(value);
                }
                if bytes.len() > MAX_PARAMETER_RESPONSE {
                    error(5, 5)
                } else {
                    reply(180, bytes)
                }
            }
            167 => {
                if request.data.len() != 4 {
                    return Some(error(6, 5));
                }
                let [point_type, logical, count, start] = request.data[..].try_into().unwrap();
                if count == 0 || u16::from(start) + u16::from(count) > 256 {
                    return Some(error(25, 9));
                }
                let mut bytes = request.data.clone();
                for p in u16::from(start)..u16::from(start) + u16::from(count) {
                    let Some(value) = self.parameters.get(&Tlp::new(point_type, logical, p as u8))
                    else {
                        return Some(error(2, p as u8));
                    };
                    let Ok(value) = value.encode() else {
                        return Some(error(50, 4));
                    };
                    bytes.extend(value);
                }
                if bytes.len() > 234 {
                    error(5, 5)
                } else {
                    reply(167, bytes)
                }
            }
            _ => error(1, 4),
        })
    }
}

/// Inject once, on the next request accepted by any simulator connection.
#[derive(Debug, Clone)]
pub enum Fault {
    /// Replace a typed payload while retaining valid framing and CRC.
    ResponseData(Vec<u8>),
    Disconnect,
    Stall,
    Delay(Duration),
    Fragment(usize),
    CorruptCrc,
    PartialThenClose(usize),
    WrongOpcode,
    WrongAddress,
    Error(Vec<u8>),
    SrbxBeforeReply,
    /// Keep queued until opcode 225, then drop its reply after the pending-state effect.
    LoseAlarmAcknowledgement,
}
/// Fixture pending state shared across TCP connection replacement.
#[derive(Debug, Clone, Copy)]
pub struct SrbxStatus {
    pub current_alarm_index: u16,
    pub pending: bool,
    pub last_acknowledged: Option<u16>,
    pub acknowledgements: usize,
    pub signals_sent: usize,
}
struct Control {
    history: HistoryMetadataModel,
    srbx: Mutex<SrbxStatus>,
    signals: broadcast::Sender<()>,
    faults: Mutex<VecDeque<Fault>>,
    connections: AtomicUsize,
    logins: AtomicUsize,
    requests: AtomicUsize,
}
impl Control {
    fn new(model: &Model) -> Self {
        Self {
            history: HistoryMetadataModel::default(),
            srbx: Mutex::new(SrbxStatus {
                current_alarm_index: model.alarm_index,
                pending: model.initial_srbx_pending,
                last_acknowledged: None,
                acknowledgements: 0,
                signals_sent: 0,
            }),
            signals: broadcast::channel(crate::SRBX_BUFFER_CAPACITY).0,
            faults: Mutex::new(VecDeque::new()),
            connections: AtomicUsize::new(0),
            logins: AtomicUsize::new(0),
            requests: AtomicUsize::new(0),
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Stats {
    pub connections: usize,
    pub logins: usize,
    pub requests: usize,
}

pub struct Simulator {
    endpoint: SocketAddr,
    model: Arc<Model>,
    control: Arc<Control>,
    stopping: watch::Sender<bool>,
    task: Option<JoinHandle<Result<(), Error>>>,
}
impl Simulator {
    pub async fn bind(endpoint: SocketAddr, model: Model) -> Result<Self, Error> {
        Self::bind_with_history(endpoint, model, HistoryMetadataModel::default()).await
    }
    pub async fn bind_with_history(
        endpoint: SocketAddr,
        model: Model,
        history: HistoryMetadataModel,
    ) -> Result<Self, Error> {
        let listener = TcpListener::bind(endpoint).await?;
        let endpoint = listener.local_addr()?;
        let model = Arc::new(model);
        let control = Arc::new(Control {
            history,
            ..Control::new(&model)
        });
        let (stopping, receiver) = watch::channel(false);
        let shared_model = model.clone();
        let shared_control = control.clone();
        let task = tokio::spawn(run(listener, shared_model, shared_control, receiver));
        Ok(Self {
            endpoint,
            model,
            control,
            stopping,
            task: Some(task),
        })
    }
    pub fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }
    pub fn model(&self) -> &Model {
        &self.model
    }
    pub fn inject(&self, fault: Fault) {
        self.control.faults.lock().unwrap().push_back(fault);
    }
    /// Raise/repeat a fixture SRBX and update its current alarm index, including while idle.
    /// Populate the corresponding raw log slot before binding; this does not create a record.
    pub fn raise_srbx(&self, index: u16) -> Result<(), Error> {
        if index >= crate::LOG_CAPACITY {
            return Err(Error::InvalidRequest("SRBX index outside ring"));
        }
        {
            let mut state = self.control.srbx.lock().unwrap();
            state.current_alarm_index = index;
            state.pending = true;
        }
        let _ = self.control.signals.send(());
        Ok(())
    }
    pub fn srbx_status(&self) -> SrbxStatus {
        *self.control.srbx.lock().unwrap()
    }
    pub fn stats(&self) -> Stats {
        Stats {
            connections: self.control.connections.load(Ordering::SeqCst),
            logins: self.control.logins.load(Ordering::SeqCst),
            requests: self.control.requests.load(Ordering::SeqCst),
        }
    }
    pub async fn stop(mut self) -> Result<(), Error> {
        self.stopping.send_replace(true);
        if let Some(task) = self.task.take() {
            task.await.map_err(std::io::Error::other)??;
        }
        Ok(())
    }
}
impl Drop for Simulator {
    fn drop(&mut self) {
        self.stopping.send_replace(true);
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}
async fn run(
    listener: TcpListener,
    model: Arc<Model>,
    control: Arc<Control>,
    mut stopping: watch::Receiver<bool>,
) -> Result<(), Error> {
    let mut connections = JoinSet::new();
    loop {
        tokio::select! {
            biased;
            _ = stopping.changed() => break,
            result = connections.join_next(), if !connections.is_empty() => {
                if let Some(Err(error)) = result { return Err(std::io::Error::other(error).into()); }
            }
            connection = listener.accept() => {
                let (mut stream, _) = connection?;
                control.connections.fetch_add(1, Ordering::SeqCst);
                let model = model.clone();
                let control = control.clone();
                let mut stop = stopping.clone();
                connections.spawn(async move {
                    tokio::select! {
                        _ = stop.changed() => Ok(()),
                        result = serve(&mut stream, &model, &control) => result,
                    }
                });
            }
        }
    }
    connections.abort_all();
    while connections.join_next().await.is_some() {}
    Ok(())
}

/// Serve a model over a caller-owned channel. Caller bounds lifetime/cancellation.
/// TCP Simulator adds fault injection, statistics and connection teardown.
pub async fn serve_stream<S: ByteStream + ?Sized>(
    stream: &mut S,
    model: &Model,
) -> Result<(), Error> {
    serve(stream, model, &Control::new(model)).await
}
async fn serve<S: ByteStream + ?Sized>(
    stream: &mut S,
    model: &Model,
    control: &Control,
) -> Result<(), Error> {
    let mut authenticated = false;
    let (mut reader, mut writer) = tokio::io::split(stream);
    let mut signals = control.signals.subscribe();
    let signal = Frame::new(model.srbx_host, model.address, 224, vec![]);
    if control.srbx.lock().unwrap().pending {
        write_frame(&mut writer, &signal).await?;
        control.srbx.lock().unwrap().signals_sent += 1;
    }
    loop {
        // Keep a partial request read alive when an idle notification is emitted.
        let request = {
            let read = read_frame(&mut reader, CrcMode::Required);
            tokio::pin!(read);
            loop {
                tokio::select! {
                    biased;
                    result = &mut read => break match result {
                        Err(Error::Io(error)) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(()),
                        result => result?,
                    },
                    notice = signals.recv() => {
                        if notice.is_err_and(|e| matches!(e, broadcast::error::RecvError::Closed)) { return Ok(()); }
                        let pending = control.srbx.lock().unwrap().pending;
                        if pending {
                            write_frame(&mut writer, &signal).await?;
                            control.srbx.lock().unwrap().signals_sent += 1;
                        }
                    }
                }
            }
        };
        control.requests.fetch_add(1, Ordering::SeqCst);
        let Some(mut response) =
            model.respond_with_history(&request, &mut authenticated, &control.history)
        else {
            continue;
        };
        if request.opcode == 17 && response.opcode == 17 {
            control.logins.fetch_add(1, Ordering::SeqCst);
        }
        let fault = {
            let mut faults = control.faults.lock().unwrap();
            if matches!(faults.front(), Some(Fault::LoseAlarmAcknowledgement))
                && request.opcode != 225
            {
                None
            } else {
                faults.pop_front()
            }
        };
        if response.opcode == 118 {
            response.data[3..5].copy_from_slice(
                &control
                    .srbx
                    .lock()
                    .unwrap()
                    .current_alarm_index
                    .to_le_bytes(),
            );
        }
        if response.opcode == 225 && !matches!(fault, Some(Fault::Error(_))) {
            let index = u16::from_le_bytes([request.data[0], request.data[1]]);
            let mut state = control.srbx.lock().unwrap();
            state.acknowledgements += 1;
            state.last_acknowledged = Some(index);
            if index == state.current_alarm_index {
                state.pending = false;
            }
        }
        match &fault {
            Some(Fault::ResponseData(data)) => response.data.clone_from(data),
            Some(Fault::Disconnect | Fault::LoseAlarmAcknowledgement) => return Ok(()),
            Some(Fault::Stall) => return std::future::pending().await,
            Some(Fault::Delay(delay)) => tokio::time::sleep(*delay).await,
            Some(Fault::WrongOpcode) => response.opcode = 254,
            Some(Fault::WrongAddress) => response.source.unit = 2,
            Some(Fault::Error(errors)) => {
                response.opcode = 255;
                response.data.clone_from(errors);
            }
            Some(Fault::SrbxBeforeReply) => {
                write_frame(
                    &mut writer,
                    &Frame::new(request.source, model.address, 224, vec![]),
                )
                .await?
            }
            _ => {}
        }
        let bytes = response.encode()?;
        match fault {
            Some(Fault::CorruptCrc) => {
                let mut bytes = bytes;
                let end = bytes.len() - 1;
                bytes[end] ^= 0x80;
                writer.write_all(&bytes).await?;
            }
            Some(Fault::PartialThenClose(n)) => {
                writer.write_all(&bytes[..n.min(bytes.len())]).await?;
                return Ok(());
            }
            Some(Fault::Fragment(chunk)) => {
                for bytes in bytes.chunks(chunk.max(1)) {
                    writer.write_all(bytes).await?;
                    tokio::task::yield_now().await;
                }
            }
            _ => writer.write_all(&bytes).await?,
        }
    }
}

fn append_log<const N: usize>(
    slots: &mut BTreeMap<u16, [u8; N]>,
    current: &mut u16,
    bytes: [u8; N],
) -> Result<u16, Error> {
    if *current >= crate::LOG_CAPACITY {
        return Err(Error::InvalidConfig("simulator log index outside ring"));
    }
    *current = (*current + 1) % crate::LOG_CAPACITY;
    slots.insert(*current, bytes);
    Ok(*current)
}
fn log_response<const N: usize>(
    payload: &[u8],
    current: u16,
    slots: &BTreeMap<u16, [u8; N]>,
) -> Result<Vec<u8>, Error> {
    let [count, low, high] = payload else {
        return Err(Error::InvalidRequest("log header length"));
    };
    let request = crate::LogRequest::new(u16::from_le_bytes([*low, *high]), *count)?;
    if current >= crate::LOG_CAPACITY {
        return Err(Error::InvalidConfig("simulator log index outside ring"));
    }
    let mut bytes = request.payload();
    bytes.extend(current.to_le_bytes());
    for index in request.start()..request.start() + u16::from(request.count()) {
        bytes.extend(slots.get(&index).copied().unwrap_or([0; N]));
    }
    Ok(bytes)
}
