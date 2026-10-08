use crate::protocol::DeviceError;

/// Protocol, transport or configuration failure.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
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
    #[error("authentication {0}; stop this client and create one with corrected setup")]
    Authentication(AuthenticationFailure),
    #[error("required capability unavailable: {0}")]
    CapabilityUnavailable(&'static str),
    #[error("remote handle belongs to generation {expected}, current generation is {actual}")]
    StaleGeneration { expected: u64, actual: u64 },
}

/// Credential-free, terminal setup failure. Transport failures remain retryable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AuthenticationFailure {
    #[error("rejected")]
    Rejected,
    #[error("unsupported")]
    Unsupported,
}

/// Stable categories for application error handling; no remote completion inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    LocalValidation,
    DeviceRejection,
    MalformedReply,
    Transport,
    Authentication,
}
impl Error {
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::InvalidRequest(_)
            | Self::InvalidConfig(_)
            | Self::CapabilityUnavailable(_)
            | Self::StaleGeneration { .. } => ErrorKind::LocalValidation,
            Self::Device(_) => ErrorKind::DeviceRejection,
            Self::InvalidFrame(_) | Self::Crc { .. } | Self::InvalidResponse(_) => {
                ErrorKind::MalformedReply
            }
            Self::Io(_) | Self::Timeout | Self::Disconnected => ErrorKind::Transport,
            Self::Authentication(_) => ErrorKind::Authentication,
        }
    }
    pub(crate) fn preserves_connection(&self) -> bool {
        matches!(
            self.kind(),
            ErrorKind::LocalValidation | ErrorKind::DeviceRejection
        )
    }
}

/// Typed decoders can report malformed data, never fabricate a healthy rejection.
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("{0}")]
pub struct DecodeError(pub &'static str);
impl From<DecodeError> for Error {
    fn from(error: DecodeError) -> Self {
        Self::InvalidResponse(error.0)
    }
}
