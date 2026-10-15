//! OP-136 time-major history; D301180X012 §2.16/Table 2-19, pp.2-32–33.
use crate::{
    DecodeError, Error, HistoryBounds, HistoryContinuity, HistoryIndexPolicy, HistoryKind,
    ProtocolProfile, ReplyCodec, TableHistoryPolicy,
};
use std::sync::{Arc, Mutex};

/// A contiguous point block and bounded period block. Zero periods probes the index;
/// zero points requests timestamps only. Point counts are never split implicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultiHistoryRequest {
    segment: u8,
    start: u16,
    kind: HistoryKind,
    first_point: u8,
    points: u8,
    periods: u8,
    bounds: HistoryBounds,
}
impl MultiHistoryRequest {
    pub fn new(
        segment: u8,
        start: u16,
        kind: HistoryKind,
        first_point: u8,
        points: u8,
        periods: u8,
    ) -> Result<Self, Error> {
        Self::with_policy(
            segment,
            start,
            kind,
            first_point,
            points,
            periods,
            &TableHistoryPolicy,
        )
    }
    pub fn with_policy(
        segment: u8,
        start: u16,
        kind: HistoryKind,
        first_point: u8,
        points: u8,
        periods: u8,
        policy: &dyn HistoryIndexPolicy,
    ) -> Result<Self, Error> {
        if kind.is_timestamp() || u16::from(first_point) + u16::from(points) > 256 {
            return Err(Error::InvalidRequest(
                "multi-point history requires selector 0–2 and an unwrapped point block",
            ));
        }
        let bounds = policy.bounds(segment, first_point, kind)?;
        // One shared cursor requires coherent geometry for every selected point.
        for point in u16::from(first_point)..u16::from(first_point) + u16::from(points).max(1) {
            let selected = policy.bounds(segment, point as u8, kind)?;
            selected.validate(segment, point as u8, start)?;
            if selected != bounds {
                return Err(Error::InvalidRequest(
                    "multi-point history requires common selected geometry",
                ));
            }
        }
        Self::from_bounds(segment, start, kind, first_point, points, periods, bounds)
    }
    fn from_bounds(
        segment: u8,
        start: u16,
        kind: HistoryKind,
        first_point: u8,
        points: u8,
        periods: u8,
        bounds: HistoryBounds,
    ) -> Result<Self, Error> {
        bounds.validate(segment, first_point, start)?;
        if (usize::from(points) + 1) * usize::from(periods) > 60
            || u32::from(start) + u32::from(periods) > bounds.capacity().unwrap_or(65536)
        {
            return Err(Error::InvalidRequest(
                "multi-point history exceeds 60 elements or selected index boundary",
            ));
        }
        Ok(Self {
            segment,
            start,
            kind,
            first_point,
            points,
            periods,
            bounds,
        })
    }
    pub fn segment(self) -> u8 {
        self.segment
    }
    pub fn start(self) -> u16 {
        self.start
    }
    pub fn kind(self) -> HistoryKind {
        self.kind
    }
    pub fn first_point(self) -> u8 {
        self.first_point
    }
    pub fn points(self) -> u8 {
        self.points
    }
    pub fn periods(self) -> u8 {
        self.periods
    }
    pub fn bounds(self) -> HistoryBounds {
        self.bounds
    }
    pub fn payload(self) -> Vec<u8> {
        let [lo, hi] = self.start.to_le_bytes();
        vec![
            self.segment,
            lo,
            hi,
            self.kind.as_byte(),
            self.first_point,
            self.points,
            self.periods,
        ]
    }
    pub(crate) fn response_size(self) -> usize {
        6 + 4 * (usize::from(self.points) + 1) * usize::from(self.periods)
    }
    fn at(self, start: u16, periods: u8) -> Result<Self, Error> {
        Self::from_bounds(
            self.segment,
            start,
            self.kind,
            self.first_point,
            self.points,
            periods,
            self.bounds,
        )
    }
}
/// One period, in wire order. Values correspond to ascending contiguous point numbers.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiHistoryPeriod {
    /// Raw TIME/UINT32 seconds; no inferred timezone conversion.
    pub timestamp: u32,
    pub values: Vec<f32>,
}
#[derive(Debug, Clone)]
pub struct MultiHistoryPage {
    pub request: MultiHistoryRequest,
    pub current_index: u16,
    pub periods: Vec<MultiHistoryPeriod>,
    pub raw: Vec<u8>,
}
impl MultiHistoryPage {
    pub(crate) fn validate_for(&self, request: &MultiHistoryRequest) -> Result<(), DecodeError> {
        if self.request != *request
            || self.periods.len() > usize::from(request.periods)
            || self
                .periods
                .iter()
                .any(|p| p.values.len() != usize::from(request.points))
            || request
                .bounds
                .capacity()
                .is_some_and(|n| u32::from(self.current_index) >= n)
        {
            return Err(DecodeError(
                "multi-point history shape or current index outside selected bounds",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub struct MultiHistoryCodec;
impl ReplyCodec<MultiHistoryRequest> for MultiHistoryCodec {
    type Output = MultiHistoryPage;
    fn decode(
        &self,
        request: &MultiHistoryRequest,
        b: &[u8],
    ) -> Result<MultiHistoryPage, DecodeError> {
        if b.len() < 6 {
            return Err(DecodeError("truncated multi-point history header"));
        }
        let elements = usize::from(b[5]);
        let width = usize::from(request.points) + 1;
        if b[0] != request.segment
            || u16::from_le_bytes([b[1], b[2]]) != request.start
            || elements > width * usize::from(request.periods)
            || !elements.is_multiple_of(width)
            || b.len() != 6 + 4 * elements
        {
            return Err(DecodeError(
                "multi-point history echo, element count or width mismatch",
            ));
        }
        let periods = b[6..]
            .chunks_exact(4 * width)
            .map(|row| MultiHistoryPeriod {
                timestamp: u32::from_le_bytes(row[..4].try_into().unwrap()),
                values: row[4..]
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|w| f32::from_le_bytes(*w))
                    .collect(),
            })
            .collect();
        let page = MultiHistoryPage {
            request: *request,
            current_index: u16::from_le_bytes([b[3], b[4]]),
            periods,
            raw: b.to_vec(),
        };
        page.validate_for(request)?;
        Ok(page)
    }
}

/// At most one selected ring, with explicit continuity policy. Unknown capacity
/// permits only an unwrapped wire-index window. Periods, not elements, are counted.
#[derive(Debug, Clone, Copy)]
pub struct MultiHistoryRange {
    request: MultiHistoryRequest,
    total: u32,
    continuity: HistoryContinuity,
}
impl MultiHistoryRange {
    pub fn new(
        segment: u8,
        start: u16,
        kind: HistoryKind,
        first_point: u8,
        points: u8,
        total: u32,
    ) -> Result<Self, Error> {
        Self::with_policy(
            segment,
            start,
            kind,
            first_point,
            points,
            total,
            &TableHistoryPolicy,
        )
    }
    pub fn with_policy(
        segment: u8,
        start: u16,
        kind: HistoryKind,
        first_point: u8,
        points: u8,
        total: u32,
        policy: &dyn HistoryIndexPolicy,
    ) -> Result<Self, Error> {
        let request =
            MultiHistoryRequest::with_policy(segment, start, kind, first_point, points, 0, policy)?;
        if total
            > request
                .bounds
                .capacity()
                .unwrap_or(65536 - u32::from(start))
        {
            return Err(Error::InvalidRequest(
                "multi-point history window exceeds one ring or unwrapped index space",
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
    pub fn request(self) -> MultiHistoryRequest {
        self.request
    }
    pub fn total(self) -> u32 {
        self.total
    }
}
/// Confirmed whole periods only. Equal indices do not prove continuity or identity.
#[derive(Debug)]
pub struct MultiHistoryReadProgress {
    pub pages: Vec<MultiHistoryPage>,
    pub next_index: Option<u16>,
    pub remaining: u32,
    pub generation: Option<u64>,
    pub index_changed: bool,
    pub complete: bool,
    /// Includes an empty reply, which may indicate an invalid request per Table 2-19.
    pub short_page: bool,
}
impl MultiHistoryReadProgress {
    fn initial(range: MultiHistoryRange) -> Self {
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
#[error("multi-point history read failed: {error}")]
pub struct MultiHistoryReadError {
    pub progress: MultiHistoryReadProgress,
    #[source]
    pub error: etherbird::Error<Error>,
}
struct MultiHistoryRead {
    range: MultiHistoryRange,
    requests: Vec<MultiHistoryRequest>,
    progress: Arc<Mutex<MultiHistoryReadProgress>>,
}
impl MultiHistoryRead {
    fn prepare(range: MultiHistoryRange, profile: &dyn ProtocolProfile) -> Result<Self, Error> {
        crate::profile::validate_profile_request_size(profile, 136, 7)?;
        crate::profile::validate_expected_response(profile, 136, 6)?;
        let elements = ((profile.limits(136).response.min(255) - 6) / 4).min(60);
        let capacity = (elements / (usize::from(range.request.points) + 1)) as u32;
        if range.total > 0 && capacity == 0 {
            return Err(Error::InvalidRequest(
                "multi-point history period cannot fit reply budget",
            ));
        }
        let boundary = range.request.bounds.capacity().unwrap_or(65536);
        let (mut cursor, mut remaining) = (u32::from(range.request.start), range.total);
        let mut requests = Vec::new();
        if remaining == 0 {
            requests.push(range.request);
        }
        while remaining > 0 {
            let n = remaining.min(capacity).min(boundary - cursor);
            requests.push(range.request.at(cursor as u16, n as u8)?);
            remaining -= n;
            cursor += n;
            if cursor == boundary && remaining > 0 {
                cursor = 0;
            }
        }
        Ok(Self {
            range,
            requests,
            progress: Arc::new(Mutex::new(MultiHistoryReadProgress::initial(range))),
        })
    }
}
#[etherbird::async_trait]
impl crate::Workflow for MultiHistoryRead {
    type Output = ();
    async fn run(self, context: &mut crate::ManagedContext<'_>) -> Result<(), Error> {
        self.progress.lock().unwrap().generation = Some(context.generation());
        let mut first_index = None;
        for request in self.requests {
            let page = context.read_multi_history(request).await?;
            let n = page.periods.len() as u32;
            let changed = first_index.is_some_and(|index| index != page.current_index);
            first_index.get_or_insert(page.current_index);
            let short = n < u32::from(request.periods);
            {
                let mut progress = self.progress.lock().unwrap();
                let next = u32::from(request.start) + n;
                progress.next_index = if let Some(capacity) = request.bounds.capacity() {
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
                    "current multi-point history index changed between pages",
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
    /// Bounded exclusive generation; failures retain confirmed progress without replay/resume.
    pub async fn read_multi_history_range(
        &self,
        range: MultiHistoryRange,
    ) -> Result<MultiHistoryReadProgress, MultiHistoryReadError> {
        let scan = MultiHistoryRead::prepare(range, self.profile()).map_err(|error| {
            MultiHistoryReadError {
                progress: MultiHistoryReadProgress::initial(range),
                error: etherbird::Error::Operation(error),
            }
        })?;
        let progress = scan.progress.clone();
        let result = self.workflow(scan).await;
        let snapshot = std::mem::replace(
            &mut *progress.lock().unwrap(),
            MultiHistoryReadProgress::initial(range),
        );
        match result {
            Ok(()) => Ok(snapshot),
            Err(error) => Err(MultiHistoryReadError {
                progress: snapshot,
                error,
            }),
        }
    }
}
