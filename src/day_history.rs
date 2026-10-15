//! Day-oriented history: D301180X012 §2.17–2.18/Tables 2-20–2-21, pp.2-34–35.
use crate::{
    DecodeError, Error, HistoryContinuity, HistoryIndexPolicy, HistoryKind, HistoryPage,
    HistoryRange, HistorySamples, ReplyCodec,
};
use std::sync::{Arc, Mutex};

/// Day/month only: no inferred year, timezone or unique archival date.
/// February 29 is accepted because the wire request carries no year.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoryDay {
    day: u8,
    month: u8,
}
impl HistoryDay {
    pub fn new(day: u8, month: u8) -> Result<Self, Error> {
        let maximum = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => 29,
            _ => 0,
        };
        if day == 0 || day > maximum {
            return Err(Error::InvalidRequest("invalid history day/month"));
        }
        Ok(Self { day, month })
    }
    pub fn day(self) -> u8 {
        self.day
    }
    pub fn month(self) -> u8 {
        self.month
    }
}
/// Table 2-20 has no point field and the reply echoes only the segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayIndicesRequest {
    segment: u8,
    day: HistoryDay,
}
impl DayIndicesRequest {
    pub fn new(segment: u8, day: HistoryDay) -> Result<Self, Error> {
        Self::with_segment_limit(segment, day, 10)
    }
    /// Explicit caller assertion; does not infer expanded firmware applicability.
    pub fn with_segment_limit(
        segment: u8,
        day: HistoryDay,
        max_segment: u8,
    ) -> Result<Self, Error> {
        if segment > max_segment {
            return Err(Error::InvalidRequest(
                "history segment outside selected applicability",
            ));
        }
        Ok(Self { segment, day })
    }
    pub fn segment(self) -> u8 {
        self.segment
    }
    pub fn day(self) -> HistoryDay {
        self.day
    }
    pub fn payload(self) -> Vec<u8> {
        vec![self.segment, self.day.day, self.day.month]
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayIndices {
    pub request: DayIndicesRequest,
    pub periodic_start: u16,
    pub periodic_count: u16,
    /// None when daily_count is zero; raw_daily_start still preserves the wire field.
    pub daily_start: Option<u16>,
    pub daily_count: u16,
    pub raw_daily_start: u16,
    pub raw: Vec<u8>,
}
impl DayIndices {
    pub(crate) fn validate_for(&self, q: &DayIndicesRequest) -> Result<(), DecodeError> {
        if self.request != *q
            || self.daily_start != (self.daily_count != 0).then_some(self.raw_daily_start)
        {
            return Err(DecodeError(
                "day indices request or daily validity mismatch",
            ));
        }
        Ok(())
    }
    fn same_mapping(&self, other: &Self) -> bool {
        self.request == other.request
            && self.periodic_start == other.periodic_start
            && self.periodic_count == other.periodic_count
            && self.daily_start == other.daily_start
            && self.daily_count == other.daily_count
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub struct DayIndicesCodec;
impl ReplyCodec<DayIndicesRequest> for DayIndicesCodec {
    type Output = DayIndices;
    fn decode(&self, q: &DayIndicesRequest, b: &[u8]) -> Result<DayIndices, DecodeError> {
        if b.len() != 9 || b[0] != q.segment {
            return Err(DecodeError("day indices length or segment echo mismatch"));
        }
        let word = |i| u16::from_le_bytes([b[i], b[i + 1]]);
        let daily_count = word(7);
        let raw_daily_start = word(5);
        Ok(DayIndices {
            request: *q,
            periodic_start: word(1),
            periodic_count: word(3),
            daily_start: (daily_count != 0).then_some(raw_daily_start),
            daily_count,
            raw_daily_start,
            raw: b.to_vec(),
        })
    }
}
/// OP-138 reserves point 255 for timestamps. It cannot select value point 255.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayHistoryTarget {
    Values(u8),
    Timestamps,
}
impl DayHistoryTarget {
    pub fn as_byte(self) -> u8 {
        match self {
            Self::Values(point) => point,
            Self::Timestamps => 255,
        }
    }
    fn validate(self) -> Result<(), Error> {
        if self == Self::Values(255) {
            return Err(Error::InvalidRequest(
                "OP-138 point 255 is reserved for timestamps",
            ));
        }
        Ok(())
    }
    fn is_timestamp(self) -> bool {
        self == Self::Timestamps
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayHistoryRequest {
    indices: DayIndicesRequest,
    target: DayHistoryTarget,
}
impl DayHistoryRequest {
    pub fn new(segment: u8, day: HistoryDay, target: DayHistoryTarget) -> Result<Self, Error> {
        Self::with_segment_limit(segment, day, target, 10)
    }
    pub fn with_segment_limit(
        segment: u8,
        day: HistoryDay,
        target: DayHistoryTarget,
        max_segment: u8,
    ) -> Result<Self, Error> {
        target.validate()?;
        Ok(Self {
            indices: DayIndicesRequest::with_segment_limit(segment, day, max_segment)?,
            target,
        })
    }
    pub fn indices_request(self) -> DayIndicesRequest {
        self.indices
    }
    pub fn segment(self) -> u8 {
        self.indices.segment
    }
    pub fn day(self) -> HistoryDay {
        self.indices.day
    }
    pub fn target(self) -> DayHistoryTarget {
        self.target
    }
    pub fn payload(self) -> Vec<u8> {
        vec![
            self.segment(),
            self.target.as_byte(),
            self.day().day,
            self.day().month,
        ]
    }
}
#[derive(Debug, Clone)]
pub struct DayHistoryReply {
    pub request: DayHistoryRequest,
    pub periodic: HistorySamples,
    pub daily: HistorySamples,
    pub raw: Vec<u8>,
}
impl DayHistoryReply {
    pub(crate) fn validate_for(&self, q: &DayHistoryRequest) -> Result<(), DecodeError> {
        if self.request != *q
            || matches!(self.periodic, HistorySamples::Timestamps(_)) != q.target.is_timestamp()
            || matches!(self.daily, HistorySamples::Timestamps(_)) != q.target.is_timestamp()
            || self.periodic.len() + self.daily.len() > 61
        {
            return Err(DecodeError(
                "day history request, sample kinds or count exceed wire budget",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub struct DayHistoryCodec;
impl ReplyCodec<DayHistoryRequest> for DayHistoryCodec {
    type Output = DayHistoryReply;
    fn decode(&self, q: &DayHistoryRequest, b: &[u8]) -> Result<DayHistoryReply, DecodeError> {
        if b.len() < 8 || b[..4] != q.payload() {
            return Err(DecodeError("day history header or echoes mismatch"));
        }
        let periodic = usize::from(u16::from_le_bytes([b[4], b[5]]));
        let daily = usize::from(u16::from_le_bytes([b[6], b[7]]));
        if periodic + daily > 61 || b.len() != 8 + 4 * (periodic + daily) {
            return Err(DecodeError(
                "day history counts, widths or trailing bytes mismatch",
            ));
        }
        let samples = |bytes: &[u8]| {
            if q.target.is_timestamp() {
                HistorySamples::Timestamps(
                    bytes
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .map(|w| u32::from_le_bytes(*w))
                        .collect(),
                )
            } else {
                HistorySamples::Values(
                    bytes
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .map(|w| f32::from_le_bytes(*w))
                        .collect(),
                )
            }
        };
        Ok(DayHistoryReply {
            request: *q,
            periodic: samples(&b[8..8 + 4 * periodic]),
            daily: samples(&b[8 + 4 * periodic..]),
            raw: b.to_vec(),
        })
    }
}

/// Explicit indexed alternative: OP-137 plus bounded OP-135 pages, never an
/// automatic retry after OP-138 rejection. Geometry is an independent caller choice.
#[derive(Clone)]
pub struct IndexedDayHistoryRequest {
    request: DayHistoryRequest,
    policy: Arc<dyn HistoryIndexPolicy>,
    timestamp_point: Option<u8>,
    continuity: HistoryContinuity,
}
impl IndexedDayHistoryRequest {
    pub fn new(request: DayHistoryRequest, policy: Arc<dyn HistoryIndexPolicy>) -> Self {
        Self {
            request,
            policy,
            timestamp_point: None,
            continuity: HistoryContinuity::default(),
        }
    }
    /// Required for timestamp requests: an explicitly known OP-135 point, not OP-138's sentinel.
    pub fn with_timestamp_point(mut self, point: u8) -> Self {
        self.timestamp_point = Some(point);
        self
    }
    pub fn with_continuity(mut self, continuity: HistoryContinuity) -> Self {
        self.continuity = continuity;
        self
    }
    pub fn request(&self) -> DayHistoryRequest {
        self.request
    }
    fn point(&self) -> Result<u8, Error> {
        match self.request.target {
            DayHistoryTarget::Values(point) => Ok(point),
            DayHistoryTarget::Timestamps => self.timestamp_point.ok_or(Error::InvalidRequest(
                "indexed timestamp history requires an explicit OP-135 point",
            )),
        }
    }
}
#[derive(Debug)]
pub struct DayHistoryProgress {
    pub request: DayHistoryRequest,
    pub indices: Option<DayIndices>,
    pub indices_after: Option<DayIndices>,
    pub periodic_pages: Vec<HistoryPage>,
    pub daily_pages: Vec<HistoryPage>,
    /// Unknown until the first OP-137 reply has decoded successfully.
    pub periodic_remaining: Option<u32>,
    pub daily_remaining: Option<u32>,
    pub periodic_next_index: Option<u16>,
    pub daily_next_index: Option<u16>,
    pub generation: Option<u64>,
    pub index_changed: bool,
    pub mapping_changed: bool,
    pub short_page: bool,
    /// All requested samples read and post-read mapping unchanged; not snapshot proof.
    pub complete: bool,
}
impl DayHistoryProgress {
    fn initial(request: DayHistoryRequest) -> Self {
        Self {
            request,
            indices: None,
            indices_after: None,
            periodic_pages: Vec::new(),
            daily_pages: Vec::new(),
            periodic_remaining: None,
            daily_remaining: None,
            periodic_next_index: None,
            daily_next_index: None,
            generation: None,
            index_changed: false,
            mapping_changed: false,
            short_page: false,
            complete: false,
        }
    }
}
#[derive(Debug, thiserror::Error)]
#[error("indexed day history failed: {error}")]
pub struct DayHistoryReadError {
    pub progress: Box<DayHistoryProgress>,
    #[source]
    pub error: etherbird::Error<Error>,
}
struct IndexedDayRead {
    request: IndexedDayHistoryRequest,
    progress: Arc<Mutex<DayHistoryProgress>>,
}
#[etherbird::async_trait]
impl crate::Workflow for IndexedDayRead {
    type Output = ();
    async fn run(self, c: &mut crate::ManagedContext<'_>) -> Result<(), Error> {
        self.progress.lock().unwrap().generation = Some(c.generation());
        let q = self.request.request;
        let indices = c.read_day_indices(q.indices).await?;
        {
            let mut p = self.progress.lock().unwrap();
            p.periodic_remaining = Some(u32::from(indices.periodic_count));
            p.daily_remaining = Some(u32::from(indices.daily_count));
            p.periodic_next_index = Some(indices.periodic_start);
            p.daily_next_index = indices.daily_start;
            p.indices = Some(indices.clone());
        }
        let point = self.request.point()?;
        // Plan both columns before reading either, so invalid geometry cannot yield a half-planned day.
        let mut columns = Vec::new();
        for (start, total, kind) in [
            (
                Some(indices.periodic_start),
                indices.periodic_count,
                HistoryKind::Periodic,
            ),
            (indices.daily_start, indices.daily_count, HistoryKind::Daily),
        ] {
            let mut requests = Vec::new();
            if total != 0 {
                let kind = if q.target.is_timestamp() {
                    match kind {
                        HistoryKind::Periodic => HistoryKind::PeriodicTimestamps,
                        _ => HistoryKind::DailyTimestamps,
                    }
                } else {
                    kind
                };
                let range = HistoryRange::with_policy(
                    q.segment(),
                    point,
                    kind,
                    start.expect("nonzero daily count has valid index"),
                    u32::from(total),
                    self.request.policy.as_ref(),
                )?;
                requests = crate::history::plan_history_range(range, c.profile())?;
            }
            columns.push(requests);
        }
        for (column, requests) in columns.into_iter().enumerate() {
            let mut first_index = None;
            for request in requests {
                let page = c.read_history(request).await?;
                let count = page.samples.len() as u32;
                let changed = first_index.is_some_and(|index| index != page.current_index);
                first_index.get_or_insert(page.current_index);
                let short = count < u32::from(request.count());
                let next = u32::from(request.start()) + count;
                let next_index = if let Some(capacity) = request.bounds().capacity() {
                    Some((next % capacity) as u16)
                } else {
                    u16::try_from(next).ok()
                };
                {
                    let mut p = self.progress.lock().unwrap();
                    p.index_changed |= changed;
                    p.short_page = short;
                    if column == 0 {
                        *p.periodic_remaining.as_mut().unwrap() -= count;
                        p.periodic_next_index = next_index;
                        p.periodic_pages.push(page);
                    } else {
                        *p.daily_remaining.as_mut().unwrap() -= count;
                        p.daily_next_index = next_index;
                        p.daily_pages.push(page);
                    }
                }
                if changed && self.request.continuity == HistoryContinuity::RequireUnchangedIndex {
                    return Err(Error::HistoryChanged(
                        "current history index changed during indexed day read",
                    ));
                }
                if short {
                    return Ok(());
                }
            }
        }
        let after = c.read_day_indices(q.indices).await?;
        let changed = !indices.same_mapping(&after);
        let mut p = self.progress.lock().unwrap();
        p.mapping_changed = changed;
        p.indices_after = Some(after);
        if changed {
            return Err(Error::HistoryChanged(
                "day index/count mapping changed during indexed read",
            ));
        }
        p.complete = true;
        Ok(())
    }
}
impl crate::Client {
    /// Explicit OP-137/135 alternative for large days. One exclusive generation,
    /// bounded pages and a final mapping check; never retries OP-138 or resumes
    /// across replacement. Errors retain confirmed progress. See docs/DAY_HISTORY.md.
    pub async fn read_history_day_indexed(
        &self,
        request: IndexedDayHistoryRequest,
    ) -> Result<DayHistoryProgress, DayHistoryReadError> {
        let q = request.request;
        let preflight = || {
            request.point()?;
            crate::profile::validate_profile_request_size(self.profile(), 137, 3)?;
            crate::profile::validate_expected_response(self.profile(), 137, 9)?;
            // The indexed path needs no OP-138 capability. OP-135 budgets are checked after counts are known.
            Ok::<_, Error>(())
        };
        preflight().map_err(|error| DayHistoryReadError {
            progress: Box::new(DayHistoryProgress::initial(q)),
            error: etherbird::Error::Operation(error),
        })?;
        let progress = Arc::new(Mutex::new(DayHistoryProgress::initial(q)));
        let result = self
            .workflow(IndexedDayRead {
                request,
                progress: progress.clone(),
            })
            .await;
        let snapshot = std::mem::replace(
            &mut *progress.lock().unwrap(),
            DayHistoryProgress::initial(q),
        );
        match result {
            Ok(()) => Ok(snapshot),
            Err(error) => Err(DayHistoryReadError {
                progress: Box::new(snapshot),
                error,
            }),
        }
    }
}
