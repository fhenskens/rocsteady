//! Async ROC Plus client with Etherbird recovery.
//!
//! The initial typed API covers configuration discovery, clock reads, parameter reads, legacy login and
//! device errors. The framing/transport seam is independent of supervision.
//! See docs/COVERAGE.md for scope and hardware validation limits.
mod client;
mod configuration;
mod error;
pub mod protocol;
mod session;
#[cfg(feature = "simulator")]
pub mod simulator;
pub mod transport;

pub use client::{Client, ClientConfig};
pub use configuration::{LogicalCompatibility, SystemConfiguration, SystemMode};
pub use error::Error;
pub use etherbird::ResourceState;
pub use protocol::{Address, Clock, DataType, Login, Parameter, ParameterBlock, Tlp, Value};
pub use session::Session;
