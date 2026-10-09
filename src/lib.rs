//! Async ROC Plus client with Etherbird recovery.
//!
//! The initial typed API covers configuration discovery, clock reads, parameter reads, legacy login and
//! device errors. The framing/transport seam is independent of supervision.
//! See docs/COVERAGE.md for scope and hardware validation limits.
mod catalog;
mod client;
mod configuration;
mod discovery;
mod error;
mod extensions;
mod profile;
pub mod protocol;
mod schema;
mod session;
#[cfg(feature = "simulator")]
pub mod simulator;
pub mod transport;

pub use catalog::{CatalogEntry, Roc800Catalog};
pub use client::{Client, ClientConfig};
pub use configuration::{LogicalCompatibility, SystemConfiguration, SystemMode};
pub use discovery::{
    FirmwareVersion, IoDiscovery, IoLayout, IoPoint, IoPointCodec, IoPointReply, IoPointRequest,
    IoSelector, IoSnapshot, PointTypeClassification, PointTypeCodec, PointTypePage,
    PointTypeRequest,
};
pub use error::{AuthenticationFailure, DecodeError, Error, ErrorKind};
pub use etherbird::ResourceState;
pub use profile::{
    Capability, ClockCodec, ConfigurationCodec, DecodedOperation, DefaultProfile, PayloadLimits,
    ProfileMetadata, ProtocolProfile, ReplyCodec, Roc800Profile,
};
pub use protocol::{Address, Clock, DataType, Login, Parameter, ParameterBlock, Tlp, Value};
pub use schema::{
    Access, ParameterSchema, ReadPlan, ReadProgress, SchemaChain, SchemaContext, SchemaProvider,
    SchemaReadError,
};
pub use session::Session;

pub use extensions::{
    GenerationValue, LegacySetup, ManagedContext, Operation, SessionState, SetupStrategy, Workflow,
};

// Keep onboarding examples type-checked as part of the published library.
#[doc = include_str!("../README.md")]
pub mod guide {}

#[doc = include_str!("../docs/USAGE.md")]
pub mod usage {}
