//! Async ROC Plus client with etherbird recovery.
//!
//! Experimental APIs cover discovery, schema-backed parameter reads and writes, alarm/event logs,
//! single/multi-point, day-oriented and selected history, clock setting, configurable
//! tables, raw transaction retrieval, legacy login and device errors.
//! Opt-in SRBX polling and explicit consumed-alarm
//! acknowledgement support monitoring; acknowledgement changes controller status.
//! No operation is automatically replayed. Physical interoperability is unverified.
//! The framing/transport seam is independent of supervision.
//! See the packaged docs/COVERAGE.md and docs/LOGS.md for scope and contracts.
mod catalog;
mod client;
mod clock_set;
mod configurable_tables;
mod configuration;
mod day_history;
mod discovery;
mod error;
mod extensions;
mod history;
mod history_information;
mod history_metadata;
mod logs;
mod multi_history;
mod operations;
mod profile;
pub mod protocol;
mod schema;
mod session;
#[cfg(feature = "simulator")]
pub mod simulator;
mod srbx;
mod transactions;
pub mod transport;
mod writes;
pub use transactions::{
    TransactionDataPage, TransactionDataRequest, TransactionEntry, TransactionField,
    TransactionFieldLayout, TransactionInventoryError, TransactionInventoryProgress,
    TransactionListPage, TransactionListRequest, TransactionReadError, TransactionReadProgress,
    TransactionSegment, TransactionValue, decode_transaction_fields,
};

pub use catalog::{BitMeaning, CatalogEntry, CatalogMetadata, EnumMeaning, Roc800Catalog};
pub use client::{Client, ClientConfig};
pub use clock_set::{ClockSet, ClockSetError};
pub use configuration::{LogicalCompatibility, SystemConfiguration, SystemMode};
pub use discovery::{
    FirmwareVersion, IoDiscovery, IoLayout, IoPoint, IoPointCodec, IoPointReply, IoPointRequest,
    IoSelector, IoSnapshot, PointTypeClassification, PointTypeCodec, PointTypePage,
    PointTypeRequest,
};
pub use error::{AuthenticationFailure, DecodeError, Error, ErrorKind};
pub use etherbird::ResourceState;
pub use history::{
    ConfiguredHistoryPolicy, HistoryBounds, HistoryCodec, HistoryContinuity, HistoryIndexPolicy,
    HistoryKind, HistoryPage, HistoryRange, HistoryReadError, HistoryReadProgress, HistoryRequest,
    HistorySamples, ProseMinuteHistoryPolicy, TableHistoryPolicy,
};
pub use multi_history::{
    MultiHistoryCodec, MultiHistoryPage, MultiHistoryPeriod, MultiHistoryRange,
    MultiHistoryReadError, MultiHistoryReadProgress, MultiHistoryRequest,
};

pub use day_history::{
    DayHistoryCodec, DayHistoryProgress, DayHistoryReadError, DayHistoryReply, DayHistoryRequest,
    DayHistoryTarget, DayIndices, DayIndicesCodec, DayIndicesRequest, HistoryDay,
    IndexedDayHistoryRequest,
};

pub use history_information::{
    ConfiguredHistoryCodec, ConfiguredHistoryPoints, ConfiguredHistoryRequest,
    SelectedHistoryCodec, SelectedHistoryPage, SelectedHistoryPeriod, SelectedHistoryProgress,
    SelectedHistoryRange, SelectedHistoryReadError, SelectedHistoryRequest,
};

pub use history_metadata::{
    ArchivalMethod, CalendarOccurrenceCodec, HistoryExtrema, HistoryTag, HistoryTagsCodec,
    HistoryTagsReply, HistoryTagsRequest, MinMaxCodec, MinMaxReply, MinMaxRequest, OccurrenceTime,
    PrefixedEpochOccurrenceCodec, RawOccurrenceCodec,
};
pub use logs::{
    AlarmCodec, AlarmData, AlarmPage, AlarmReason, AlarmRecord, EventCodec, EventData,
    EventDataType, EventPage, EventRecord, LOG_CAPACITY, LogCheckpoint, LogPage, LogReadError,
    LogReadProgress, LogRecord, LogRequest, LogText, LoggedValue, SystemEventReason,
};
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
pub use srbx::{
    AlarmAcknowledgement, AlarmAcknowledgementError, AlarmAcknowledgementOutcome,
    SRBX_BUFFER_CAPACITY, SrbxNotifications, SrbxPoll, SrbxPollError, SrbxPoller,
};

pub use extensions::{
    GenerationValue, LegacySetup, ManagedContext, Operation, SessionState, SetupStrategy, Workflow,
};

// Keep onboarding examples type-checked as part of the published library.
#[doc = include_str!("../README.md")]
pub mod guide {}

#[doc = include_str!("../docs/USAGE.md")]
pub mod usage {}

pub use writes::{
    MAX_ARBITRARY_WRITE_REQUEST, SchemaWriteError, WriteBatch, WriteOutcome, WritePlan,
    WriteProgress,
};

#[doc = include_str!("../docs/WRITES.md")]
pub mod write_guide {}

#[doc = include_str!("../docs/TEMPERATURE.md")]
pub mod temperature_guide {}

#[doc = include_str!("../docs/MVS.md")]
pub mod mvs_guide {}

#[doc = include_str!("../docs/CLOCK_SET.md")]
pub mod clock_set_guide {}
#[doc = include_str!("../docs/TRANSACTIONS.md")]
pub mod transaction_guide {}

pub use configurable_tables::{
    ConfigurableTable, ConfigurableTableDefinition, TableDefinitionRequest, TableReadBatch,
    TableReadError, TableReadPlan, TableReadProgress, TableReadReply, TableRevision,
};

#[doc = include_str!("../docs/CONFIGURABLE_TABLES.md")]
pub mod configurable_table_guide {}
