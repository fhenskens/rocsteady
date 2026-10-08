use crate::protocol::DeviceError;

/// Protocol, transport or configuration failure.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("transport error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid ROC Plus frame: {0}")]
    InvalidFrame(&'static str),
    #[error("CRC mismatch: expected {expected:04x}, received {received:04x}")]
    Crc { expected: u16, received: u16 },
    #[error("invalid request: {0}")]
    InvalidRequest(&'static str),
    #[error("invalid response: {0}")]
    InvalidResponse(&'static str),
    #[error("device rejected request: {0:?}")]
    Device(Vec<DeviceError>),
    #[error("request deadline elapsed")]
    Timeout,
    #[error("invalid configuration: {0}")]
    InvalidConfig(&'static str),
    #[error("transport is disconnected")]
    Disconnected,
}
