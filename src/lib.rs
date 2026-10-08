//! Async ROC Plus client with Etherbird recovery.
//!
//! The initial typed API covers configuration discovery, clock reads, parameter reads, legacy login and
//! device errors. The framing/transport seam is independent of supervision.
//! See docs/COVERAGE.md for scope and hardware validation limits.
mod client;
mod configuration;
mod error;
mod extensions;
pub mod protocol;
mod session;
#[cfg(feature = "simulator")]
pub mod simulator;
pub mod transport;

pub use client::{Client, ClientConfig};
pub use configuration::{LogicalCompatibility, SystemConfiguration, SystemMode};
pub use error::{AuthenticationFailure, DecodeError, Error, ErrorKind};
pub use etherbird::ResourceState;
pub use protocol::{Address, Clock, DataType, Login, Parameter, ParameterBlock, Tlp, Value};
pub use session::Session;

pub use extensions::{
    GenerationValue, LegacySetup, ManagedContext, Operation, SessionState, SetupStrategy, Workflow,
};

// Keep onboarding examples type-checked as part of the published library.
#[doc = include_str!("../README.md")]
pub mod guide {}

#[doc = include_str!("../docs/USAGE.md")]
pub mod usage {}
