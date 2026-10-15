//! OP-135 bounded single-point history; D301180X012 §2.15/Table 2-18.
use crate::{DecodeError, Error, ProtocolProfile, ReplyCodec};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[non_exhaustive]
pub enum HistoryKind {
    Minute = 0,
    Periodic = 1,
    Daily = 2,
    PeriodicTimestamps = 3,
    DailyTimestamps = 4,
}
impl HistoryKind {
    pub fn as_byte(self) -> u8 {
        self as u8
    }
    pub fn is_timestamp(self) -> bool {
        matches!(self, Self::PeriodicTimestamps | Self::DailyTimestamps)
    }
}

/// Independently selected geometry, not inferred from a returned cursor or firmware label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoryBounds {
    max_segment: u8,
    point_count: Option<u16>,
    capacity: Option<u32>,
}
impl HistoryBounds {
    pub fn new(
        max_segment: u8,
        point_count: Option<u16>,
        capacity: Option<u32>,
    ) -> Result<Self, Error> {
        if point_count.is_some_and(|n| n == 0 || n > 256)
            || capacity.is_some_and(|n| n == 0 || n > 65536)
        {
            return Err(Error::InvalidRequest(
                "history geometry exceeds wire address space or is empty",
            ));
        }
        Ok(Self {
            max_segment,
            point_count,
            capacity,
        })
    }
    pub fn max_segment(self) -> u8 {
        self.max_segment
    }
    pub fn point_count(self) -> Option<u16> {
        self.point_count
    }
    pub fn capacity(self) -> Option<u32> {
        self.capacity
    }
    pub(crate) fn validate(self, segment: u8, point: u8, start: u16) -> Result<(), Error> {
        if segment > self.max_segment
            || self.point_count.is_some_and(|n| u16::from(point) >= n)
            || self.capacity.is_some_and(|n| u32::from(start) >= n)
        {
            return Err(Error::InvalidRequest(
                "history segment, point or index outside selected applicability",
            ));
        }
        Ok(())
    }
}
/// Public policy seam for known segment/point capacities and disputed minute bounds.
/// Returned bounds are immutable in each request; selection never follows parsing success.
pub trait HistoryIndexPolicy: Send + Sync {
    fn bounds(&self, segment: u8, point: u8, kind: HistoryKind) -> Result<HistoryBounds, Error>;
}
/// Default: Tables 2-18/2-19 minute indices 0–59, segments 0–10. Other capacities unknown.
#[derive(Debug, Clone, Copy, Default)]
pub struct TableHistoryPolicy;
impl HistoryIndexPolicy for TableHistoryPolicy {
    fn bounds(&self, _: u8, _: u8, kind: HistoryKind) -> Result<HistoryBounds, Error> {
        HistoryBounds::new(
            10,
            None,
            if kind == HistoryKind::Minute {
                Some(60)
            } else {
                None
            },
        )
    }
}
/// Explicitly provisional DEC-4.03 alternative: inclusive 0–60 minute prose,
/// treated as 61 distinct slots. No alias/sentinel or verified firmware is implied.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProseMinuteHistoryPolicy;
impl HistoryIndexPolicy for ProseMinuteHistoryPolicy {
    fn bounds(&self, _: u8, _: u8, kind: HistoryKind) -> Result<HistoryBounds, Error> {
        HistoryBounds::new(
            10,
            None,
            if kind == HistoryKind::Minute {
                Some(61)
            } else {
                None
            },
        )
    }
}
/// Caller-asserted device configuration, with no automatic firmware applicability claim.
#[derive(Debug, Clone, Copy)]
pub struct ConfiguredHistoryPolicy {
    max_segment: u8,
    point_count: u16,
    minute: u32,
    periodic: u32,
    daily: u32,
}
impl ConfiguredHistoryPolicy {
    pub fn new(
        max_segment: u8,
        point_count: u16,
        minute: u32,
        periodic: u32,
        daily: u32,
    ) -> Result<Self, Error> {
        for capacity in [minute, periodic, daily] {
            HistoryBounds::new(max_segment, Some(point_count), Some(capacity))?;
        }
        Ok(Self {
            max_segment,
            point_count,
            minute,
            periodic,
            daily,
        })
    }
}
impl HistoryIndexPolicy for ConfiguredHistoryPolicy {
    fn bounds(&self, _: u8, _: u8, kind: HistoryKind) -> Result<HistoryBounds, Error> {
        let capacity = match kind {
            HistoryKind::Minute => self.minute,
            HistoryKind::Periodic | HistoryKind::PeriodicTimestamps => self.periodic,
            HistoryKind::Daily | HistoryKind::DailyTimestamps => self.daily,
        };
        HistoryBounds::new(self.max_segment, Some(self.point_count), Some(capacity))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoryRequest {
    segment: u8,
    point: u8,
    kind: HistoryKind,
    start: u16,
    count: u8,
    bounds: HistoryBounds,
}
impl HistoryRequest {
    /// A single page never crosses a selected ring boundary. Zero count probes the index.
    pub fn new(
        segment: u8,
        point: u8,
        kind: HistoryKind,
        start: u16,
        count: u8,
    ) -> Result<Self, Error> {
        Self::with_policy(segment, point, kind, start, count, &TableHistoryPolicy)
    }
    pub fn with_policy(
        segment: u8,
        point: u8,
        kind: HistoryKind,
        start: u16,
        count: u8,
        policy: &dyn HistoryIndexPolicy,
    ) -> Result<Self, Error> {
        Self::from_bounds(
            segment,
            point,
            kind,
            start,
            count,
            policy.bounds(segment, point, kind)?,
        )
    }
    fn from_bounds(
        segment: u8,
        point: u8,
        kind: HistoryKind,
        start: u16,
        count: u8,
        bounds: HistoryBounds,
    ) -> Result<Self, Error> {
        bounds.validate(segment, point, start)?;
        if count > 60 || u32::from(start) + u32::from(count) > bounds.capacity.unwrap_or(65536) {
            return Err(Error::InvalidRequest(
                "history page exceeds 60 items or selected index boundary",
            ));
        }
        Ok(Self {
            segment,
            point,
            kind,
            start,
            count,
            bounds,
        })
    }
    pub fn segment(self) -> u8 {
        self.segment
    }
    pub fn point(self) -> u8 {
        self.point
    }
    pub fn kind(self) -> HistoryKind {
        self.kind
    }
    pub fn start(self) -> u16 {
        self.start
    }
    pub fn count(self) -> u8 {
        self.count
    }
    pub fn bounds(self) -> HistoryBounds {
        self.bounds
    }
    pub fn payload(self) -> Vec<u8> {
        let mut b = vec![self.segment, self.point, self.kind.as_byte()];
        b.extend(self.start.to_le_bytes());
        b.push(self.count);
        b
    }
    pub(crate) fn response_size(self) -> usize {
        5 + 4 * usize::from(self.count)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum HistorySamples {
    Values(Vec<f32>),
    Timestamps(Vec<u32>),
}
impl HistorySamples {
    pub fn len(&self) -> usize {
        match self {
            Self::Values(v) => v.len(),
            Self::Timestamps(v) => v.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
#[derive(Debug, Clone)]
pub struct HistoryPage {
    pub request: HistoryRequest,
    pub current_index: u16,
    pub samples: HistorySamples,
    pub raw: Vec<u8>,
}
impl HistoryPage {
    /// Structural invariants retained even when a profile replaces the wire decoder.
    pub(crate) fn validate_for(&self, request: &HistoryRequest) -> Result<(), DecodeError> {
        if self.request != *request
            || self.samples.len() > usize::from(request.count)
            || matches!(self.samples, HistorySamples::Timestamps(_)) != request.kind.is_timestamp()
            || request
                .bounds
                .capacity
                .is_some_and(|n| u32::from(self.current_index) >= n)
        {
            return Err(DecodeError(
                "history page shape or current index outside selected bounds",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub struct HistoryCodec;
impl ReplyCodec<HistoryRequest> for HistoryCodec {
    type Output = HistoryPage;
    fn decode(&self, request: &HistoryRequest, b: &[u8]) -> Result<HistoryPage, DecodeError> {
        if b.len() < 5 {
            return Err(DecodeError("truncated history header"));
        }
        let count = usize::from(b[4]);
        if b[..2] != [request.segment, request.point]
            || count > usize::from(request.count)
            || b.len() != 5 + 4 * count
        {
            return Err(DecodeError(
                "history segment/point/count or value width mismatch",
            ));
        }
        let words = b[5..].as_chunks::<4>().0;
        let samples = if request.kind.is_timestamp() {
            HistorySamples::Timestamps(words.iter().map(|w| u32::from_le_bytes(*w)).collect())
        } else {
            HistorySamples::Values(words.iter().map(|w| f32::from_le_bytes(*w)).collect())
        };
        let page = HistoryPage {
            request: *request,
            current_index: u16::from_le_bytes([b[2], b[3]]),
            samples,
            raw: b.to_vec(),
        };
        page.validate_for(request)?;
        Ok(page)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HistoryContinuity {
    /// Stop after an observed change; equal cursors cannot prove gap-free continuity.
    #[default]
    RequireUnchangedIndex,
    /// Return all bounded pages and flag observed changes, without snapshot claims.
    ObserveIndexChanges,
}
#[derive(Debug, Clone, Copy)]
pub struct HistoryRange {
    request: HistoryRequest,
    total: u32,
    continuity: HistoryContinuity,
}
impl HistoryRange {
    pub fn new(
        segment: u8,
        point: u8,
        kind: HistoryKind,
        start: u16,
        total: u32,
    ) -> Result<Self, Error> {
        Self::with_policy(segment, point, kind, start, total, &TableHistoryPolicy)
    }
    pub fn with_policy(
        segment: u8,
        point: u8,
        kind: HistoryKind,
        start: u16,
        total: u32,
        policy: &dyn HistoryIndexPolicy,
    ) -> Result<Self, Error> {
        let request = HistoryRequest::with_policy(segment, point, kind, start, 0, policy)?;
        let limit = request.bounds.capacity.unwrap_or(65536 - u32::from(start));
        if total > limit {
            return Err(Error::InvalidRequest(
                "history window exceeds one selected ring or unwrapped wire index space",
            ));
        }
        Ok(Self {
            request,
            total,
            continuity: HistoryContinuity::default(),
        })
    }
    pub fn with_continuity(mut self, continuity: HistoryContinuity) -> Self {
        self.continuity = continuity;
        self
    }
    pub fn request(self) -> HistoryRequest {
        self.request
    }
    pub fn total(self) -> u32 {
        self.total
    }
}
/// Fully decoded pages, including a page which first exposed an index change.
/// Indices are positions, not durable sequence numbers or an automatic resume token.
#[derive(Debug)]
pub struct HistoryReadProgress {
    pub pages: Vec<HistoryPage>,
    pub next_index: Option<u16>,
    pub remaining: u32,
    pub generation: Option<u64>,
    pub index_changed: bool,
    pub complete: bool,
    pub short_page: bool,
}
impl HistoryReadProgress {
    fn initial(range: HistoryRange) -> Self {
        Self {
            pages: Vec::new(),
            next_index: Some(range.request.start),
            remaining: range.total,
            generation: None,
            index_changed: false,
            complete: false,
            short_page: false,
        }
    }
}
#[derive(Debug, thiserror::Error)]
#[error("history read failed: {error}")]
pub struct HistoryReadError {
    pub progress: HistoryReadProgress,
    #[source]
    pub error: etherbird::Error<Error>,
}
struct HistoryRead {
    range: HistoryRange,
    requests: Vec<HistoryRequest>,
    progress: Arc<Mutex<HistoryReadProgress>>,
}
pub(crate) fn plan_history_range(
    range: HistoryRange,
    profile: &dyn ProtocolProfile,
) -> Result<Vec<HistoryRequest>, Error> {
    crate::profile::validate_profile_request_size(profile, 135, 6)?;
    crate::profile::validate_expected_response(profile, 135, 5)?;
    let available = profile
        .limits(135)
        .response
        .min(255)
        .checked_sub(5)
        .ok_or(Error::InvalidRequest("history header cannot fit profile"))?;
    let capacity = (available / 4).min(60) as u32;
    if range.total > 0 && capacity == 0 {
        return Err(Error::InvalidRequest(
            "history item cannot fit profile reply budget",
        ));
    }
    let q = range.request;
    let boundary = q.bounds.capacity.unwrap_or(65536);
    let (mut cursor, mut remaining) = (u32::from(q.start), range.total);
    let mut requests = Vec::new();
    if remaining == 0 {
        requests.push(q);
    }
    while remaining > 0 {
        let n = remaining.min(capacity).min(boundary - cursor);
        requests.push(HistoryRequest::from_bounds(
            q.segment,
            q.point,
            q.kind,
            cursor as u16,
            n as u8,
            q.bounds,
        )?);
        remaining -= n;
        cursor += n;
        if cursor == boundary && remaining > 0 {
            cursor = 0;
        }
    }
    Ok(requests)
}
impl HistoryRead {
    fn prepare(range: HistoryRange, profile: &dyn ProtocolProfile) -> Result<Self, Error> {
        let requests = plan_history_range(range, profile)?;
        Ok(Self {
            range,
            requests,
            progress: Arc::new(Mutex::new(HistoryReadProgress::initial(range))),
        })
    }
}
#[etherbird::async_trait]
impl crate::Workflow for HistoryRead {
    type Output = ();
    async fn run(self, context: &mut crate::ManagedContext<'_>) -> Result<(), Error> {
        self.progress.lock().unwrap().generation = Some(context.generation());
        let mut first_index = None;
        for request in self.requests {
            let page = context.read_history(request).await?;
            let n = page.samples.len() as u32;
            let changed = first_index.is_some_and(|index| index != page.current_index);
            first_index.get_or_insert(page.current_index);
            let short = n < u32::from(request.count);
            {
                let mut progress = self.progress.lock().unwrap();
                let next = u32::from(request.start) + n;
                progress.next_index = if let Some(capacity) = request.bounds.capacity {
                    Some((next % capacity) as u16)
                } else {
                    u16::try_from(next).ok()
                };
                progress.remaining -= n;
                progress.index_changed |= changed;
                progress.short_page = short;
                progress.pages.push(page);
            }
            if changed && self.range.continuity == HistoryContinuity::RequireUnchangedIndex {
                return Err(Error::HistoryChanged(
                    "current history index changed between pages",
                ));
            }
            if short {
                return Ok(());
            }
        }
        self.progress.lock().unwrap().complete = true;
        Ok(())
    }
}
impl crate::Client {
    /// One exclusive generation, at most one selected ring. No implicit replay/resume.
    pub async fn read_history_range(
        &self,
        range: HistoryRange,
    ) -> Result<HistoryReadProgress, HistoryReadError> {
        let scan =
            HistoryRead::prepare(range, self.profile()).map_err(|error| HistoryReadError {
                progress: HistoryReadProgress::initial(range),
                error: etherbird::Error::Operation(error),
            })?;
        let progress = scan.progress.clone();
        let result = self.workflow(scan).await;
        let snapshot = std::mem::replace(
            &mut *progress.lock().unwrap(),
            HistoryReadProgress::initial(range),
        );
        match result {
            Ok(()) => Ok(snapshot),
            Err(error) => Err(HistoryReadError {
                progress: snapshot,
                error,
            }),
        }
    }
}
