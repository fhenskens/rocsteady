//! OP-139 §2.19/Table 2-22, D301180X012 October 2022, printed p.2-36.
use crate::{
    DecodeError, Error, HistoryBounds, HistoryContinuity, HistoryIndexPolicy, HistoryKind,
    ProtocolProfile, ReplyCodec, TableHistoryPolicy,
};
use std::sync::{Arc, Mutex};

/// Command 0 is one whole inventory; its request defines no paging cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfiguredHistoryRequest {
    segment: u8,
}
impl ConfiguredHistoryRequest {
    pub fn new(segment: u8) -> Result<Self, Error> {
        Self::with_segment_limit(segment, 10)
    }
    /// Explicit caller assertion, without automatic firmware applicability inference.
    pub fn with_segment_limit(segment: u8, max_segment: u8) -> Result<Self, Error> {
        if segment > max_segment {
            return Err(Error::InvalidRequest(
                "history segment outside selected applicability",
            ));
        }
        Ok(Self { segment })
    }
    pub fn segment(self) -> u8 {
        self.segment
    }
    pub fn payload(self) -> Vec<u8> {
        vec![0, self.segment]
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfiguredHistoryPoints {
    pub request: ConfiguredHistoryRequest,
    pub points: Vec<u8>,
    pub raw: Vec<u8>,
}
impl ConfiguredHistoryPoints {
    pub(crate) fn validate_for(&self, q: &ConfiguredHistoryRequest) -> Result<(), DecodeError> {
        if self.request != *q || self.points.len() > 252 {
            return Err(DecodeError(
                "configured history request/count outside wire budget",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub struct ConfiguredHistoryCodec;
impl ReplyCodec<ConfiguredHistoryRequest> for ConfiguredHistoryCodec {
    type Output = ConfiguredHistoryPoints;
    fn decode(&self, q: &ConfiguredHistoryRequest, b: &[u8]) -> Result<Self::Output, DecodeError> {
        if b.len() < 3
            || b.len() > 255
            || b[..2] != [0, q.segment]
            || b.len() != 3 + usize::from(b[2])
        {
            return Err(DecodeError(
                "configured history command/segment/count/length mismatch",
            ));
        }
        Ok(ConfiguredHistoryPoints {
            request: *q,
            points: b[3..].to_vec(),
            raw: b.to_vec(),
        })
    }
}
/// Ordered nonempty explicit point list. Duplicates are retained; this is a library
/// selection contract, not a claim that the source defines zero-point behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedHistoryRequest {
    segment: u8,
    start: u16,
    kind: HistoryKind,
    points: Vec<u8>,
    periods: u8,
    timestamps: bool,
    bounds: HistoryBounds,
}
impl SelectedHistoryRequest {
    pub fn new(
        segment: u8,
        start: u16,
        kind: HistoryKind,
        points: Vec<u8>,
        periods: u8,
        timestamps: bool,
    ) -> Result<Self, Error> {
        Self::with_policy(
            segment,
            start,
            kind,
            points,
            periods,
            timestamps,
            &TableHistoryPolicy,
        )
    }
    pub fn with_policy(
        segment: u8,
        start: u16,
        kind: HistoryKind,
        points: Vec<u8>,
        periods: u8,
        timestamps: bool,
        policy: &dyn HistoryIndexPolicy,
    ) -> Result<Self, Error> {
        if kind.is_timestamp() || points.is_empty() || points.len() > 247 {
            return Err(Error::InvalidRequest(
                "selected history needs selector 0–2 and 1–247 points",
            ));
        }
        let bounds = policy.bounds(segment, points[0], kind)?;
        for point in &points {
            let selected = policy.bounds(segment, *point, kind)?;
            selected.validate(segment, *point, start)?;
            if selected != bounds {
                return Err(Error::InvalidRequest(
                    "selected history needs common point geometry",
                ));
            }
        }
        Self::from_bounds(segment, start, kind, points, periods, timestamps, bounds)
    }
    fn from_bounds(
        segment: u8,
        start: u16,
        kind: HistoryKind,
        points: Vec<u8>,
        periods: u8,
        timestamps: bool,
        bounds: HistoryBounds,
    ) -> Result<Self, Error> {
        if (points.len() + usize::from(timestamps)) * usize::from(periods) > 60
            || u32::from(start) + u32::from(periods) > bounds.capacity().unwrap_or(65536)
        {
            return Err(Error::InvalidRequest(
                "selected history exceeds element or index boundary",
            ));
        }
        Ok(Self {
            segment,
            start,
            kind,
            points,
            periods,
            timestamps,
            bounds,
        })
    }
    pub fn segment(&self) -> u8 {
        self.segment
    }
    pub fn start(&self) -> u16 {
        self.start
    }
    pub fn kind(&self) -> HistoryKind {
        self.kind
    }
    pub fn points(&self) -> &[u8] {
        &self.points
    }
    pub fn periods(&self) -> u8 {
        self.periods
    }
    pub fn timestamps(&self) -> bool {
        self.timestamps
    }
    pub fn bounds(&self) -> HistoryBounds {
        self.bounds
    }
    fn width(&self) -> usize {
        self.points.len() + usize::from(self.timestamps)
    }
    pub fn payload(&self) -> Vec<u8> {
        let [lo, hi] = self.start.to_le_bytes();
        let mut b = vec![
            1,
            self.segment,
            lo,
            hi,
            self.kind.as_byte(),
            self.periods,
            u8::from(self.timestamps),
            self.points.len() as u8,
        ];
        b.extend(&self.points);
        b
    }
    pub(crate) fn response_size(&self) -> usize {
        7 + 4 * self.width() * usize::from(self.periods)
    }
    fn at(&self, start: u16, periods: u8) -> Result<Self, Error> {
        Self::from_bounds(
            self.segment,
            start,
            self.kind,
            self.points.clone(),
            periods,
            self.timestamps,
            self.bounds,
        )
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct SelectedHistoryPeriod {
    pub timestamp: Option<u32>,
    pub values: Vec<f32>,
}
#[derive(Debug, Clone)]
pub struct SelectedHistoryPage {
    pub request: SelectedHistoryRequest,
    pub current_index: u16,
    pub periods: Vec<SelectedHistoryPeriod>,
    pub raw: Vec<u8>,
}
impl SelectedHistoryPage {
    pub(crate) fn validate_for(&self, q: &SelectedHistoryRequest) -> Result<(), DecodeError> {
        if self.request != *q
            || self.periods.len() > usize::from(q.periods)
            || self.periods.iter().any(|row| {
                row.values.len() != q.points.len() || row.timestamp.is_some() != q.timestamps
            })
            || q.bounds
                .capacity()
                .is_some_and(|n| u32::from(self.current_index) >= n)
        {
            return Err(DecodeError(
                "selected history shape or current index mismatch",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub struct SelectedHistoryCodec;
impl ReplyCodec<SelectedHistoryRequest> for SelectedHistoryCodec {
    type Output = SelectedHistoryPage;
    fn decode(&self, q: &SelectedHistoryRequest, b: &[u8]) -> Result<Self::Output, DecodeError> {
        if b.len() < 7
            || b[..2] != [1, q.segment]
            || b[4] > q.periods
            || b[5] != u8::from(q.timestamps)
            || usize::from(b[6]) != q.points.len()
            || b.len() != 7 + 4 * q.width() * usize::from(b[4])
        {
            return Err(DecodeError(
                "selected history command/segment/count/flag/length mismatch",
            ));
        }
        let periods = b[7..]
            .chunks_exact(4 * q.width())
            .map(|row| {
                let offset = if q.timestamps { 4 } else { 0 };
                SelectedHistoryPeriod {
                    timestamp: q
                        .timestamps
                        .then(|| u32::from_le_bytes(row[..4].try_into().unwrap())),
                    values: row[offset..]
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .map(|w| f32::from_le_bytes(*w))
                        .collect(),
                }
            })
            .collect();
        let page = SelectedHistoryPage {
            request: q.clone(),
            current_index: u16::from_le_bytes([b[2], b[3]]),
            periods,
            raw: b.to_vec(),
        };
        page.validate_for(q)?;
        Ok(page)
    }
}
/// At most one known ring or an unwrapped unknown-capacity window. Total counts periods.
#[derive(Debug, Clone)]
pub struct SelectedHistoryRange {
    request: SelectedHistoryRequest,
    total: u32,
    continuity: HistoryContinuity,
    inventory_check: bool,
}
impl SelectedHistoryRange {
    pub fn new(
        segment: u8,
        start: u16,
        kind: HistoryKind,
        points: Vec<u8>,
        total: u32,
        timestamps: bool,
    ) -> Result<Self, Error> {
        Self::with_policy(
            segment,
            start,
            kind,
            points,
            total,
            timestamps,
            &TableHistoryPolicy,
        )
    }
    pub fn with_policy(
        segment: u8,
        start: u16,
        kind: HistoryKind,
        points: Vec<u8>,
        total: u32,
        timestamps: bool,
        policy: &dyn HistoryIndexPolicy,
    ) -> Result<Self, Error> {
        let request = SelectedHistoryRequest::with_policy(
            segment, start, kind, points, 0, timestamps, policy,
        )?;
        if total
            > request
                .bounds
                .capacity()
                .unwrap_or(65536 - u32::from(start))
        {
            return Err(Error::InvalidRequest(
                "selected history window exceeds one ring or unwrapped wire indices",
            ));
        }
        Ok(Self {
            request,
            total,
            continuity: HistoryContinuity::default(),
            inventory_check: false,
        })
    }
    pub fn with_continuity(mut self, continuity: HistoryContinuity) -> Self {
        self.continuity = continuity;
        self
    }
    /// Optional membership plus before/after inventory observation; not an atomic snapshot.
    pub fn with_inventory_check(mut self, enabled: bool) -> Self {
        self.inventory_check = enabled;
        self
    }
    pub fn request(&self) -> &SelectedHistoryRequest {
        &self.request
    }
    pub fn total(&self) -> u32 {
        self.total
    }
}
#[derive(Debug)]
pub struct SelectedHistoryProgress {
    pub pages: Vec<SelectedHistoryPage>,
    pub next_index: Option<u16>,
    pub remaining: u32,
    pub generation: Option<u64>,
    pub index_changed: bool,
    pub short_page: bool,
    pub complete: bool,
    pub inventory_before: Option<ConfiguredHistoryPoints>,
    pub inventory_after: Option<ConfiguredHistoryPoints>,
    pub inventory_changed: bool,
}
impl SelectedHistoryProgress {
    fn initial(range: &SelectedHistoryRange) -> Self {
        Self {
            pages: Vec::new(),
            next_index: Some(range.request.start),
            remaining: range.total,
            generation: None,
            index_changed: false,
            short_page: false,
            complete: false,
            inventory_before: None,
            inventory_after: None,
            inventory_changed: false,
        }
    }
}
#[derive(Debug, thiserror::Error)]
#[error("selected history read failed: {error}")]
pub struct SelectedHistoryReadError {
    pub progress: Box<SelectedHistoryProgress>,
    #[source]
    pub error: etherbird::Error<Error>,
}
struct SelectedHistoryRead {
    range: SelectedHistoryRange,
    requests: Vec<SelectedHistoryRequest>,
    progress: Arc<Mutex<SelectedHistoryProgress>>,
}
impl SelectedHistoryRead {
    fn prepare(range: SelectedHistoryRange, profile: &dyn ProtocolProfile) -> Result<Self, Error> {
        crate::profile::validate_profile_request_size(profile, 139, range.request.payload().len())?;
        crate::profile::validate_expected_response(profile, 139, 7)?;
        let available =
            profile
                .limits(139)
                .response
                .min(255)
                .checked_sub(7)
                .ok_or(Error::InvalidRequest(
                    "selected history header cannot fit profile",
                ))?;
        let capacity = ((available / 4).min(60) / range.request.width()) as u32;
        if range.total > 0 && capacity == 0 {
            return Err(Error::InvalidRequest(
                "selected history period cannot fit profile",
            ));
        }
        let boundary = range.request.bounds.capacity().unwrap_or(65536);
        let (mut cursor, mut remaining) = (u32::from(range.request.start), range.total);
        let mut requests = Vec::new();
        if remaining == 0 {
            requests.push(range.request.clone());
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
        let progress = Arc::new(Mutex::new(SelectedHistoryProgress::initial(&range)));
        Ok(Self {
            range,
            requests,
            progress,
        })
    }
}
#[etherbird::async_trait]
impl crate::Workflow for SelectedHistoryRead {
    type Output = ();
    async fn run(self, c: &mut crate::ManagedContext<'_>) -> Result<(), Error> {
        self.progress.lock().unwrap().generation = Some(c.generation());
        let inventory_request = ConfiguredHistoryRequest {
            segment: self.range.request.segment,
        };
        if self.range.inventory_check {
            let inventory = c.read_configured_history(inventory_request).await?;
            let missing = self
                .range
                .request
                .points
                .iter()
                .any(|p| !inventory.points.contains(p));
            self.progress.lock().unwrap().inventory_before = Some(inventory);
            if missing {
                return Err(Error::HistoryChanged(
                    "selected point is absent from observed configured inventory",
                ));
            }
        }
        let mut first_index = None;
        for request in self.requests {
            let page = c.read_selected_history(request.clone()).await?;
            let count = page.periods.len() as u32;
            let changed = first_index.is_some_and(|index| index != page.current_index);
            first_index.get_or_insert(page.current_index);
            let short = count < u32::from(request.periods);
            {
                let mut p = self.progress.lock().unwrap();
                let next = u32::from(request.start) + count;
                p.next_index = if let Some(capacity) = request.bounds.capacity() {
                    Some((next % capacity) as u16)
                } else {
                    u16::try_from(next).ok()
                };
                p.remaining -= count;
                p.index_changed |= changed;
                p.short_page = short;
                p.pages.push(page);
            }
            if changed && self.range.continuity == HistoryContinuity::RequireUnchangedIndex {
                return Err(Error::HistoryChanged(
                    "selected history current index changed between pages",
                ));
            }
            if short {
                return Ok(());
            }
        }
        if self.range.inventory_check {
            let after = c.read_configured_history(inventory_request).await?;
            let mut p = self.progress.lock().unwrap();
            let changed = p.inventory_before.as_ref().unwrap().points != after.points;
            p.inventory_changed = changed;
            p.inventory_after = Some(after);
            if changed {
                return Err(Error::HistoryChanged(
                    "configured history inventory changed during selected read",
                ));
            }
        }
        self.progress.lock().unwrap().complete = true;
        Ok(())
    }
}
impl crate::Client {
    /// Bounded one-generation periods; optional inventory recheck, no implicit replay/resume.
    pub async fn read_selected_history_range(
        &self,
        range: SelectedHistoryRange,
    ) -> Result<SelectedHistoryProgress, SelectedHistoryReadError> {
        let initial = || SelectedHistoryProgress::initial(&range);
        let scan =
            SelectedHistoryRead::prepare(range.clone(), self.profile()).map_err(|error| {
                SelectedHistoryReadError {
                    progress: Box::new(initial()),
                    error: etherbird::Error::Operation(error),
                }
            })?;
        let progress = scan.progress.clone();
        let result = self.workflow(scan).await;
        let snapshot = std::mem::replace(&mut *progress.lock().unwrap(), initial());
        match result {
            Ok(()) => Ok(snapshot),
            Err(error) => Err(SelectedHistoryReadError {
                progress: Box::new(snapshot),
                error,
            }),
        }
    }
}
