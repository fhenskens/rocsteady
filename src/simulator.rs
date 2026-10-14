//! Read-only ROC Plus fixture model and fault-injectable TCP server.
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
    sync::watch,
    task::{JoinHandle, JoinSet},
};

#[derive(Debug, Clone)]
pub struct Model {
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
            point_types: [91, 101, 103, 109, 136]
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
    /// Model-level request dispatch also works over in-memory or serial fixture channels.
    pub fn respond(&self, request: &Frame, authenticated: &mut bool) -> Option<Frame> {
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
}
#[derive(Default)]
struct Control {
    faults: Mutex<VecDeque<Fault>>,
    connections: AtomicUsize,
    logins: AtomicUsize,
    requests: AtomicUsize,
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
        let listener = TcpListener::bind(endpoint).await?;
        let endpoint = listener.local_addr()?;
        let model = Arc::new(model);
        let control = Arc::new(Control::default());
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
    serve(stream, model, &Control::default()).await
}
async fn serve<S: ByteStream + ?Sized>(
    stream: &mut S,
    model: &Model,
    control: &Control,
) -> Result<(), Error> {
    let mut authenticated = false;
    loop {
        let request = match read_frame(stream, CrcMode::Required).await {
            Err(Error::Io(error)) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Ok(());
            }
            result => result?,
        };
        control.requests.fetch_add(1, Ordering::SeqCst);
        let Some(mut response) = model.respond(&request, &mut authenticated) else {
            continue;
        };
        if request.opcode == 17 && response.opcode == 17 {
            control.logins.fetch_add(1, Ordering::SeqCst);
        }
        let fault = control.faults.lock().unwrap().pop_front();
        match &fault {
            Some(Fault::ResponseData(data)) => response.data.clone_from(data),
            Some(Fault::Disconnect) => return Ok(()),
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
                    stream,
                    &Frame::new(request.source, model.address, 224, vec![]),
                )
                .await?
            }
            _ => {}
        }
        let mut bytes = response.encode()?;
        match fault {
            Some(Fault::CorruptCrc) => {
                let end = bytes.len() - 1;
                bytes[end] ^= 0x80;
                stream.write_all(&bytes).await?;
            }
            Some(Fault::PartialThenClose(n)) => {
                stream.write_all(&bytes[..n.min(bytes.len())]).await?;
                return Ok(());
            }
            Some(Fault::Fragment(chunk)) => {
                for chunk in bytes.chunks(chunk.max(1)) {
                    stream.write_all(chunk).await?;
                    tokio::task::yield_now().await;
                }
            }
            _ => stream.write_all(&bytes).await?,
        }
    }
}
