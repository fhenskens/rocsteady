//! Replaceable byte-stream seam shared by TCP, serial, tests and simulator.
use crate::{
    Error,
    protocol::{CrcMode, Frame, HEADER_LEN},
};
use std::sync::Arc;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};

pub trait ByteStream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> ByteStream for T {}
pub type Stream = Box<dyn ByteStream>;

/// Produce a fresh connected stream on every call, without protocol setup.
#[etherbird::async_trait]
pub trait Connector: Send + Sync + 'static {
    async fn connect(&self) -> Result<Stream, Error>;
}
pub type SharedConnector = Arc<dyn Connector>;

#[derive(Debug, Clone)]
pub struct TcpConnector {
    endpoint: String,
}
impl TcpConnector {
    /// Endpoint includes the configured port, e.g. "192.0.2.1:4000".
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}
#[etherbird::async_trait]
impl Connector for TcpConnector {
    async fn connect(&self) -> Result<Stream, Error> {
        let stream = TcpStream::connect(&self.endpoint).await?;
        stream.set_nodelay(true)?;
        Ok(Box::new(stream))
    }
}

#[cfg(feature = "serial")]
#[derive(Debug, Clone)]
pub struct SerialConnector {
    pub port: String,
    pub baud_rate: u32,
    pub parity: tokio_serial::Parity,
    pub stop_bits: tokio_serial::StopBits,
    pub data_bits: tokio_serial::DataBits,
    pub flow_control: tokio_serial::FlowControl,
}
#[cfg(feature = "serial")]
impl SerialConnector {
    pub fn new(port: impl Into<String>, baud_rate: u32) -> Self {
        Self {
            port: port.into(),
            baud_rate,
            parity: tokio_serial::Parity::None,
            stop_bits: tokio_serial::StopBits::One,
            data_bits: tokio_serial::DataBits::Eight,
            flow_control: tokio_serial::FlowControl::None,
        }
    }
}
#[cfg(feature = "serial")]
#[etherbird::async_trait]
impl Connector for SerialConnector {
    async fn connect(&self) -> Result<Stream, Error> {
        use tokio_serial::SerialPortBuilderExt;
        let stream = tokio_serial::new(&self.port, self.baud_rate)
            .parity(self.parity)
            .stop_bits(self.stop_bits)
            .data_bits(self.data_bits)
            .flow_control(self.flow_control)
            .open_native_async()
            .map_err(std::io::Error::other)?;
        Ok(Box::new(stream))
    }
}

/// Exact reads work for fragmented TCP/serial packets and leave coalesced frames queued.
/// Caller owns deadlines and must retire a partially read stream on cancellation.
pub async fn read_frame<S: AsyncRead + Unpin + ?Sized>(
    stream: &mut S,
    crc_mode: CrcMode,
) -> Result<Frame, Error> {
    let mut header = [0; HEADER_LEN];
    stream.read_exact(&mut header).await?;
    let mut bytes = Vec::with_capacity(usize::from(header[5]) + 8);
    bytes.extend(header);
    bytes.resize(usize::from(header[5]) + 8, 0);
    stream.read_exact(&mut bytes[HEADER_LEN..]).await?;
    Frame::decode(&bytes, crc_mode)
}
pub async fn write_frame<S: AsyncWrite + Unpin + ?Sized>(
    stream: &mut S,
    frame: &Frame,
) -> Result<(), Error> {
    stream.write_all(&frame.encode()?).await?;
    // Byte-stream protocols need the reply, not serial tcdrain/flush.
    Ok(())
}
