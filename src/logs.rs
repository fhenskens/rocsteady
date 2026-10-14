//! Bounded alarm/event retrieval. Ring positions are not durable sequence numbers.
mod records;
use crate::{DecodeError, Error, ProtocolProfile, ReplyCodec};
pub use records::*;
use std::sync::{Arc, Mutex};

pub const LOG_CAPACITY: u16 = 450;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogRequest {
    start: u16,
    count: u8,
}
impl LogRequest {
    /// A single page never crosses the ring end; range reads split there explicitly.
    pub fn new(start: u16, count: u8) -> Result<Self, Error> {
        if start >= LOG_CAPACITY || count > 10 || start + u16::from(count) > LOG_CAPACITY {
            return Err(Error::InvalidRequest(
                "log page requires index 0-449, count 0-10 and no ring crossing",
            ));
        }
        Ok(Self { start, count })
    }
    pub fn start(self) -> u16 {
        self.start
    }
    pub fn count(self) -> u8 {
        self.count
    }
    pub fn payload(self) -> Vec<u8> {
        let mut b = vec![self.count];
        b.extend(self.start.to_le_bytes());
        b
    }
}
mod sealed {
    pub trait Sealed {}
}
/// The two fixed log record kinds. This trait is sealed to keep their wire budgets exact.
pub trait LogRecord: sealed::Sealed + Clone + Send + Sync + std::fmt::Debug + 'static {
    const OPCODE: u8;
    const WIDTH: usize;
    fn raw_bytes(&self) -> &[u8];
    fn is_empty(&self) -> bool;
    #[doc(hidden)]
    fn decode_record(bytes: &[u8]) -> Result<Self, DecodeError>;
    #[doc(hidden)]
    fn decode_page(
        profile: &dyn ProtocolProfile,
        request: &LogRequest,
        bytes: &[u8],
    ) -> Result<LogPage<Self>, DecodeError>;
}
impl sealed::Sealed for AlarmRecord {}
impl LogRecord for AlarmRecord {
    const OPCODE: u8 = 118;
    const WIDTH: usize = 23;
    fn raw_bytes(&self) -> &[u8] {
        self.raw()
    }
    fn is_empty(&self) -> bool {
        matches!(self.data(), AlarmData::None)
    }
    fn decode_record(bytes: &[u8]) -> Result<Self, DecodeError> {
        Self::decode(bytes)
    }
    fn decode_page(
        profile: &dyn ProtocolProfile,
        request: &LogRequest,
        bytes: &[u8],
    ) -> Result<LogPage<Self>, DecodeError> {
        profile.decode_alarms(request, bytes)
    }
}
impl sealed::Sealed for EventRecord {}
impl LogRecord for EventRecord {
    const OPCODE: u8 = 119;
    const WIDTH: usize = 22;
    fn raw_bytes(&self) -> &[u8] {
        self.raw()
    }
    fn is_empty(&self) -> bool {
        matches!(self.data(), EventData::None)
    }
    fn decode_record(bytes: &[u8]) -> Result<Self, DecodeError> {
        Self::decode(bytes)
    }
    fn decode_page(
        profile: &dyn ProtocolProfile,
        request: &LogRequest,
        bytes: &[u8],
    ) -> Result<LogPage<Self>, DecodeError> {
        profile.decode_events(request, bytes)
    }
}
#[derive(Debug, Clone)]
pub struct LogPage<R: LogRecord> {
    request: LogRequest,
    current_index: u16,
    records: Vec<R>,
}
pub type AlarmPage = LogPage<AlarmRecord>;
pub type EventPage = LogPage<EventRecord>;
impl<R: LogRecord> LogPage<R> {
    pub fn request(&self) -> LogRequest {
        self.request
    }
    pub fn current_index(&self) -> u16 {
        self.current_index
    }
    pub fn records(&self) -> &[R] {
        &self.records
    }
    /// Create only after the application consumes this many returned records.
    /// No-empty markers cannot establish a useful anchor and are rejected.
    pub fn checkpoint(&self, consumed: usize) -> Result<LogCheckpoint<R>, Error> {
        let offset = consumed
            .checked_sub(1)
            .filter(|i| *i < self.records.len())
            .ok_or(Error::InvalidRequest(
                "checkpoint requires consumed returned records",
            ))?;
        let record = &self.records[offset];
        if record.is_empty() {
            return Err(Error::InvalidRequest(
                "empty log marker cannot anchor a checkpoint",
            ));
        }
        LogCheckpoint::new(
            (self.request.start + offset as u16) % LOG_CAPACITY,
            record.clone(),
        )
    }
}
#[derive(Debug, Clone)]
pub struct LogCheckpoint<R: LogRecord> {
    index: u16,
    record: R,
}
impl<R: LogRecord> LogCheckpoint<R> {
    /// Restore an application-persisted consumed anchor for the same intended controller.
    /// This validates its shape, not whether the application actually consumed it.
    pub fn new(index: u16, record: R) -> Result<Self, Error> {
        if index >= LOG_CAPACITY || record.is_empty() {
            return Err(Error::InvalidRequest(
                "checkpoint requires a nonempty consumed record at index 0-449",
            ));
        }
        Ok(Self { index, record })
    }

    pub fn index(&self) -> u16 {
        self.index
    }
    pub fn record(&self) -> &R {
        &self.record
    }
    pub fn next_index(&self) -> u16 {
        (self.index + 1) % LOG_CAPACITY
    }
}
pub struct AlarmCodec;
impl ReplyCodec<LogRequest> for AlarmCodec {
    type Output = AlarmPage;
    fn decode(&self, request: &LogRequest, bytes: &[u8]) -> Result<AlarmPage, DecodeError> {
        decode_page(request, bytes)
    }
}
pub struct EventCodec;
impl ReplyCodec<LogRequest> for EventCodec {
    type Output = EventPage;
    fn decode(&self, request: &LogRequest, bytes: &[u8]) -> Result<EventPage, DecodeError> {
        decode_page(request, bytes)
    }
}
fn decode_page<R: LogRecord>(
    request: &LogRequest,
    bytes: &[u8],
) -> Result<LogPage<R>, DecodeError> {
    if bytes.len() < 5 {
        return Err(DecodeError("truncated log reply header"));
    }
    let count = usize::from(bytes[0]);
    let start = u16::from_le_bytes([bytes[1], bytes[2]]);
    let current_index = u16::from_le_bytes([bytes[3], bytes[4]]);
    if count > usize::from(request.count)
        || start != request.start
        || current_index >= LOG_CAPACITY
        || bytes.len() != 5 + count * R::WIDTH
    {
        return Err(DecodeError(
            "log count/index echo or record boundary mismatch",
        ));
    }
    let records = bytes[5..]
        .chunks_exact(R::WIDTH)
        .map(R::decode_record)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(LogPage {
        request: *request,
        current_index,
        records,
    })
}

/// Successfully decoded pages only. Positions/indices alone cannot prove continuity.
#[derive(Debug)]
pub struct LogReadProgress<R: LogRecord> {
    pub pages: Vec<LogPage<R>>,
    /// True only after a resumed read matches its consumed anchor before AND after paging.
    /// Even this does not rule out full rollover, identical rewrites or device replacement.
    pub anchor_verified: bool,
}
impl<R: LogRecord> Default for LogReadProgress<R> {
    fn default() -> Self {
        Self {
            pages: Vec::new(),
            anchor_verified: false,
        }
    }
}
impl<R: LogRecord> LogReadProgress<R> {
    pub fn records(&self) -> impl Iterator<Item = &R> {
        self.pages.iter().flat_map(|page| page.records())
    }
    /// Application-confirmed consumption across returned pages; no automatic advancement.
    pub fn checkpoint(&self, mut consumed: usize) -> Result<LogCheckpoint<R>, Error> {
        if consumed == 0 {
            return Err(Error::InvalidRequest(
                "checkpoint requires consumed returned records",
            ));
        }
        for page in &self.pages {
            if consumed <= page.records.len() {
                return page.checkpoint(consumed);
            }
            consumed -= page.records.len();
        }
        Err(Error::InvalidRequest("checkpoint exceeds returned records"))
    }
}
#[derive(Debug, thiserror::Error)]
#[error("log read failed: {error}")]
pub struct LogReadError<R: LogRecord> {
    pub progress: LogReadProgress<R>,
    #[source]
    pub error: etherbird::Error<Error>,
}
pub(crate) struct LogRead<R: LogRecord> {
    pub requests: Vec<LogRequest>,
    pub checkpoint: Option<LogCheckpoint<R>>,
    pub progress: Arc<Mutex<LogReadProgress<R>>>,
}
impl<R: LogRecord> LogRead<R> {
    pub fn prepare(
        start: u16,
        count: u16,
        checkpoint: Option<LogCheckpoint<R>>,
        profile: &dyn ProtocolProfile,
    ) -> Result<Self, Error> {
        if start >= LOG_CAPACITY
            || count > LOG_CAPACITY
            || (checkpoint.is_some() && count >= LOG_CAPACITY)
        {
            return Err(Error::InvalidRequest(
                "log range exceeds one ring; resume must exclude the consumed anchor",
            ));
        }
        crate::profile::validate_profile_request_size(profile, R::OPCODE, 3)?;
        let limit = profile.limits(R::OPCODE).response.min(255);
        if limit < 5 {
            return Err(Error::InvalidRequest("log header cannot fit profile"));
        }
        let capacity = ((limit - 5) / R::WIDTH).min(10) as u16;
        if (count > 0 || checkpoint.is_some()) && capacity == 0 {
            return Err(Error::InvalidRequest("log record cannot fit profile"));
        }
        let mut requests = Vec::new();
        let (mut cursor, mut remaining) = (start, count);
        if remaining == 0 {
            requests.push(LogRequest::new(start, 0)?);
        }
        while remaining > 0 {
            let n = remaining.min(capacity).min(LOG_CAPACITY - cursor);
            requests.push(LogRequest::new(cursor, n as u8)?);
            remaining -= n;
            cursor = (cursor + n) % LOG_CAPACITY;
        }
        Ok(Self {
            requests,
            checkpoint,
            progress: Arc::new(Mutex::new(LogReadProgress::default())),
        })
    }
    async fn check_anchor(&self, context: &mut crate::ManagedContext<'_>) -> Result<(), Error> {
        if let Some(anchor) = &self.checkpoint {
            let request = LogRequest::new(anchor.index, 1)?;
            let page = context
                .exchange_operation(crate::operations::logs::<R>(
                    request,
                    context.session.profile.clone(),
                ))
                .await?;
            if page.records.first().map(LogRecord::raw_bytes) != Some(anchor.record.raw_bytes()) {
                return Err(Error::LogChanged(
                    "consumed anchor changed or is unavailable",
                ));
            }
        }
        Ok(())
    }
}
#[etherbird::async_trait]
impl<R: LogRecord> crate::Workflow for LogRead<R> {
    type Output = ();
    async fn run(self, context: &mut crate::ManagedContext<'_>) -> Result<(), Error> {
        self.check_anchor(context).await?;
        for request in &self.requests {
            let page = context
                .exchange_operation(crate::operations::logs::<R>(
                    *request,
                    context.session.profile.clone(),
                ))
                .await?;
            let short = page.records.len() < usize::from(request.count);
            self.progress.lock().unwrap().pages.push(page);
            if short {
                break;
            }
        }
        self.check_anchor(context).await?;
        self.progress.lock().unwrap().anchor_verified = self.checkpoint.is_some();
        Ok(())
    }
}
impl crate::Client {
    async fn read_log_range<R: LogRecord>(
        &self,
        start: u16,
        count: u16,
        checkpoint: Option<LogCheckpoint<R>>,
    ) -> Result<LogReadProgress<R>, LogReadError<R>> {
        let scan = LogRead::prepare(start, count, checkpoint, self.profile()).map_err(|error| {
            LogReadError {
                progress: Default::default(),
                error: etherbird::Error::Operation(error),
            }
        })?;
        let progress = scan.progress.clone();
        let result = self.workflow(scan).await;
        let progress = std::mem::take(&mut *progress.lock().unwrap());
        match result {
            Ok(()) => Ok(progress),
            Err(error) => Err(LogReadError { progress, error }),
        }
    }
    /// Fixed caller-selected slot window, at most one ring. No implicit resume or snapshot claim.
    pub async fn read_alarm_range(
        &self,
        start: u16,
        count: u16,
    ) -> Result<LogReadProgress<AlarmRecord>, LogReadError<AlarmRecord>> {
        self.read_log_range(start, count, None).await
    }
    pub async fn read_event_range(
        &self,
        start: u16,
        count: u16,
    ) -> Result<LogReadProgress<EventRecord>, LogReadError<EventRecord>> {
        self.read_log_range(start, count, None).await
    }
    /// Explicit resume on the same intended controller, validating the consumed anchor twice.
    /// Checkpoint identity is caller-owned; matching bytes cannot prove absence of a gap.
    pub async fn resume_alarms(
        &self,
        checkpoint: LogCheckpoint<AlarmRecord>,
        count: u16,
    ) -> Result<LogReadProgress<AlarmRecord>, LogReadError<AlarmRecord>> {
        self.read_log_range(checkpoint.next_index(), count, Some(checkpoint))
            .await
    }
    pub async fn resume_events(
        &self,
        checkpoint: LogCheckpoint<EventRecord>,
        count: u16,
    ) -> Result<LogReadProgress<EventRecord>, LogReadError<EventRecord>> {
        self.read_log_range(checkpoint.next_index(), count, Some(checkpoint))
            .await
    }
}
