//! Bounded history metadata reads. No ring continuity, year or timezone is inferred.
use crate::{DecodeError, Error, LogText, ReplyCodec, Tlp};

/// Baseline Table 2-14 request. The point is a wire byte; no point capacity is inferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinMaxRequest {
    segment: u8,
    point: u8,
}
impl MinMaxRequest {
    pub fn new(segment: u8, point: u8) -> Result<Self, Error> {
        Self::with_segment_limit(segment, point, 10)
    }
    /// Explicit caller assertion for independently established expanded-segment applicability.
    /// This does not establish support by the controller or change its point capacity.
    pub fn with_segment_limit(segment: u8, point: u8, max_segment: u8) -> Result<Self, Error> {
        if segment > max_segment {
            return Err(Error::InvalidRequest(
                "history segment outside selected applicability",
            ));
        }
        Ok(Self { segment, point })
    }
    pub fn segment(self) -> u8 {
        self.segment
    }
    pub fn point(self) -> u8 {
        self.point
    }
    pub fn payload(self) -> Vec<u8> {
        vec![self.segment, self.point]
    }
}

/// Table 2-15: 1–20 explicitly ordered points in one segment; duplicates retain order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryTagsRequest {
    segment: u8,
    points: Vec<u8>,
}
impl HistoryTagsRequest {
    pub fn new(segment: u8, points: Vec<u8>) -> Result<Self, Error> {
        Self::with_limits(segment, points, 10, 199)
    }
    /// Explicit caller-supplied segment/point limits; the 20-point wire budget stays mandatory.
    pub fn with_limits(
        segment: u8,
        points: Vec<u8>,
        max_segment: u8,
        max_point: u8,
    ) -> Result<Self, Error> {
        if segment > max_segment
            || points.is_empty()
            || points.len() > 20
            || points.iter().any(|p| *p > max_point)
        {
            return Err(Error::InvalidRequest(
                "history tags require selected segment, 1-20 points and applicable point bounds",
            ));
        }
        Ok(Self { segment, points })
    }
    pub fn segment(&self) -> u8 {
        self.segment
    }
    pub fn points(&self) -> &[u8] {
        &self.points
    }
    pub fn payload(&self) -> Vec<u8> {
        let mut bytes = vec![self.segment, self.points.len() as u8];
        bytes.extend(&self.points);
        bytes
    }
    pub(crate) fn response_size(&self) -> usize {
        4 + 11 * self.points.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchivalMethod(pub u8);
impl ArchivalMethod {
    pub fn key(self) -> Option<&'static str> {
        Some(match self.0 {
            0 => "undefined",
            128 => "hourly_average",
            129 => "hourly_accumulated",
            130 => "hourly_current",
            134 => "hourly_totalize",
            67 => "fst_timestamp",
            65 => "fst_value",
            _ => return None,
        })
    }
}
/// An occurrence interpretation always retains the exact five bytes.
/// Calendar values have no year/timezone. The prefixed epoch byte has no assigned meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OccurrenceTime {
    Raw([u8; 5]),
    Calendar {
        raw: [u8; 5],
        second: u8,
        minute: u8,
        hour: u8,
        day: u8,
        month: u8,
    },
    PrefixedEpoch {
        raw: [u8; 5],
        prefix: u8,
        seconds: u32,
    },
}
impl OccurrenceTime {
    pub fn raw(&self) -> &[u8; 5] {
        match self {
            Self::Raw(raw) | Self::Calendar { raw, .. } | Self::PrefixedEpoch { raw, .. } => raw,
        }
    }
}
/// Sensible lossless default for contradictory Table 2-14 occurrence semantics.
#[derive(Debug, Clone, Copy, Default)]
pub struct RawOccurrenceCodec;
impl ReplyCodec<()> for RawOccurrenceCodec {
    type Output = OccurrenceTime;
    fn decode(&self, _: &(), bytes: &[u8]) -> Result<Self::Output, DecodeError> {
        Ok(OccurrenceTime::Raw(bytes.try_into().map_err(|_| {
            DecodeError("occurrence must be five bytes")
        })?))
    }
}
/// Explicitly provisional: five calendar fields in Table 2-14 prose; DEC-4.01.
/// Related-family corroboration does not establish ROC800 firmware applicability.
#[derive(Debug, Clone, Copy, Default)]
pub struct CalendarOccurrenceCodec;
impl ReplyCodec<()> for CalendarOccurrenceCodec {
    type Output = OccurrenceTime;
    fn decode(&self, _: &(), bytes: &[u8]) -> Result<Self::Output, DecodeError> {
        let raw: [u8; 5] = bytes
            .try_into()
            .map_err(|_| DecodeError("occurrence must be five bytes"))?;
        let [second, minute, hour, day, month] = raw;
        let max_day = match month {
            2 => 29,
            4 | 6 | 9 | 11 => 30,
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            _ => 0,
        };
        if second > 59 || minute > 59 || hour > 23 || day == 0 || day > max_day {
            return Err(DecodeError("invalid provisional calendar occurrence"));
        }
        Ok(OccurrenceTime::Calendar {
            raw,
            second,
            minute,
            hour,
            day,
            month,
        })
    }
}
/// Explicitly provisional: opaque prefix plus little-endian UINT32 epoch seconds,
/// following the pinned CISA parser candidate. No status/padding meaning is assigned.
#[derive(Debug, Clone, Copy, Default)]
pub struct PrefixedEpochOccurrenceCodec;
impl ReplyCodec<()> for PrefixedEpochOccurrenceCodec {
    type Output = OccurrenceTime;
    fn decode(&self, _: &(), bytes: &[u8]) -> Result<Self::Output, DecodeError> {
        let raw: [u8; 5] = bytes
            .try_into()
            .map_err(|_| DecodeError("occurrence must be five bytes"))?;
        Ok(OccurrenceTime::PrefixedEpoch {
            raw,
            prefix: raw[0],
            seconds: u32::from_le_bytes(raw[1..].try_into().unwrap()),
        })
    }
}
#[derive(Debug, Clone)]
pub struct HistoryExtrema<T = OccurrenceTime> {
    pub minimum: f32,
    pub maximum: f32,
    pub minimum_time: T,
    pub maximum_time: T,
}
#[derive(Debug, Clone)]
pub struct MinMaxReply<T = OccurrenceTime> {
    pub request: MinMaxRequest,
    pub archival_method: ArchivalMethod,
    pub tlp: Tlp,
    pub current: f32,
    pub today: HistoryExtrema<T>,
    pub yesterday: HistoryExtrema<T>,
    pub last_completed_period: f32,
    pub raw: [u8; 50],
}
/// All built-in occurrence choices use this public generic codec. Consumers may
/// supply a ReplyCodec<()> with any output type, or replace the entire reply codec.
#[derive(Debug, Clone, Copy, Default)]
pub struct MinMaxCodec<C = RawOccurrenceCodec>(pub C);
impl<C: ReplyCodec<()>> ReplyCodec<MinMaxRequest> for MinMaxCodec<C> {
    type Output = MinMaxReply<C::Output>;
    fn decode(&self, request: &MinMaxRequest, bytes: &[u8]) -> Result<Self::Output, DecodeError> {
        let raw: [u8; 50] = bytes
            .try_into()
            .map_err(|_| DecodeError("min/max reply must be 50 bytes"))?;
        if bytes[..2] != [request.segment, request.point] {
            return Err(DecodeError("min/max segment/point echo mismatch"));
        }
        let float = |i| f32::from_le_bytes(raw[i..i + 4].try_into().unwrap());
        Ok(MinMaxReply {
            request: *request,
            archival_method: ArchivalMethod(raw[2]),
            tlp: Tlp::new(raw[3], raw[4], raw[5]),
            current: float(6),
            today: HistoryExtrema {
                minimum: float(10),
                maximum: float(14),
                minimum_time: self.0.decode(&(), &raw[18..23])?,
                maximum_time: self.0.decode(&(), &raw[23..28])?,
            },
            yesterday: HistoryExtrema {
                minimum: float(28),
                maximum: float(32),
                minimum_time: self.0.decode(&(), &raw[36..41])?,
                maximum_time: self.0.decode(&(), &raw[41..46])?,
            },
            last_completed_period: float(46),
            raw,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryTag {
    pub point: u8,
    pub tag: LogText<10>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryTagsReply {
    pub request: HistoryTagsRequest,
    /// Raw common periodic ring index; capacity, year and continuity are not inferred.
    pub periodic_index: u16,
    pub tags: Vec<HistoryTag>,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct HistoryTagsCodec;
impl ReplyCodec<HistoryTagsRequest> for HistoryTagsCodec {
    type Output = HistoryTagsReply;
    fn decode(
        &self,
        request: &HistoryTagsRequest,
        bytes: &[u8],
    ) -> Result<Self::Output, DecodeError> {
        if bytes.len() != request.response_size()
            || bytes[..2] != [request.segment, request.points.len() as u8]
        {
            return Err(DecodeError("history tag length/segment/count mismatch"));
        }
        let mut tags = Vec::with_capacity(request.points.len());
        for (point, entry) in request.points.iter().zip(bytes[4..].as_chunks::<11>().0) {
            if entry[0] != *point {
                return Err(DecodeError("history tag point echo/order mismatch"));
            }
            tags.push(HistoryTag {
                point: *point,
                tag: LogText(entry[1..].try_into().unwrap()),
            });
        }
        Ok(HistoryTagsReply {
            request: request.clone(),
            periodic_index: u16::from_le_bytes([bytes[2], bytes[3]]),
            tags,
        })
    }
}
