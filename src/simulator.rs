//! ROC Plus read/write/SRBX fixture model and fault-injectable TCP server.
//! Uses the same byte-stream seam as client transports, not a mocked Client API.
//! This is a bounded test fixture, not a full ROC800 emulator.
use crate::{
    Address, Clock, Error, Login, SystemConfiguration, Tlp, Value,
    protocol::{CrcMode, Frame, MAX_PARAMETER_RESPONSE},
    transport::{ByteStream, read_frame, write_frame},
};
use std::{
    collections::{BTreeMap, VecDeque},
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::AsyncWriteExt,
    net::TcpListener,
    sync::{broadcast, watch},
    task::{JoinHandle, JoinSet},
};

#[derive(Debug, Clone)]
pub struct Model {
    /// Explicit raw log slots. Missing slots return the NoAlarm/NoEvent marker.
    pub alarm_log: BTreeMap<u16, [u8; 23]>,
    pub event_log: BTreeMap<u16, [u8; 22]>,
    /// Fixture convention: last appended slot. This is not an automatic client cursor policy.
    pub alarm_index: u16,
    pub event_index: u16,
    pub initial_srbx_pending: bool,
    pub srbx_host: Address,
    pub io_layout: crate::IoLayout,
    pub io_point_types: Vec<u8>,
    pub io_logical_numbers: Vec<u8>,
    pub point_types: BTreeMap<u8, crate::PointTypeClassification>,
    pub configuration: SystemConfiguration,
    pub address: Address,
    pub clock: Clock,
    /// OP-8 effects survive reconnects; fixture clock does not advance with wall time.
    pub written_clock: Arc<Mutex<Option<Clock>>>,
    pub login: Option<Login>,
    pub parameters: BTreeMap<Tlp, Value>,
    /// Mutable PT-99 fixtures; configuration edits are test controls, not a write API.
    pub configurable_tables: Arc<Mutex<BTreeMap<u8, crate::ConfigurableTableDefinition>>>,
    /// Explicit OP-166 effects, retained across fixture reconnects.
    pub written_parameters: Arc<Mutex<BTreeMap<Tlp, Value>>>,
}
impl Default for Model {
    fn default() -> Self {
        Self {
            initial_srbx_pending: false,
            srbx_host: Address::new(1, 0),
            alarm_log: BTreeMap::new(),
            event_log: BTreeMap::new(),
            alarm_index: 449,
            event_index: 449,
            io_layout: crate::IoLayout::Compatibility160,
            io_point_types: {
                let mut points = vec![0; 160];
                points[..5].fill(109);
                points
            },
            io_logical_numbers: {
                let mut points = vec![0; 160];
                points[..5].copy_from_slice(&[0, 1, 2, 3, 4]);
                points
            },
            point_types: [
                91, 95, 101, 102, 103, 104, 105, 106, 107, 108, 109, 136, 139,
            ]
            .into_iter()
            .map(|point_type| (point_type, crate::PointTypeClassification::Roc))
            .collect(),
            configuration: {
                let mut bytes = vec![0; 215];
                bytes[0] = 1;
                bytes[1] = 1;
                bytes[5] = 1;
                bytes[18] = 6;
                bytes[50] = 1; // One system-variables point.
                bytes[68] = 5; // Five CPU diagnostic points.
                bytes[95] = 1; // One ROC clock point; values are supplied via parameters.
                SystemConfiguration::decode(&bytes).expect("valid fixture configuration")
            },
            address: Address::new(1, 2),
            written_clock: Arc::new(Mutex::new(None)),
            clock: Clock {
                second: 1,
                minute: 2,
                hour: 3,
                day: 8,
                month: 10,
                year: 2026,
                weekday: 5,
            },
            login: None,
            parameters: BTreeMap::new(),
            configurable_tables: Arc::default(),
            written_parameters: Arc::default(),
        }
    }
}
/// Explicit OP-138 words; selector 255 represents raw timestamp words.
#[derive(Debug, Clone, Default)]
pub struct DayHistoryData {
    pub periodic: Vec<[u8; 4]>,
    pub daily: Vec<[u8; 4]>,
}
/// Table 2-29 fixture. Records are (transaction number, raw creation TIME, raw bytes).
/// Page sizes are fixture controls; fields may split anywhere. No tag encoder is used.
#[derive(Debug, Clone)]
pub struct TransactionSegmentModel {
    pub description: [u8; 10],
    pub payload_size: u16,
    pub records: Vec<(u16, u32, Vec<u8>)>,
    pub list_page_size: usize,
    pub data_page_size: usize,
}
impl Default for TransactionSegmentModel {
    fn default() -> Self {
        Self {
            description: *b"TX DEFAULT",
            payload_size: 0,
            records: Vec::new(),
            list_page_size: 40,
            data_page_size: 252,
        }
    }
}
/// Optional fixture data for OP-105/108/135/136/137/138/139/206, supplied with Simulator::bind_with_history.
/// Separate optional history/transaction fixture configuration. In 0.2.0, use
/// `..Default::default()` when constructing fixtures to accept additional fields.
#[derive(Debug, Clone)]
pub struct HistoryMetadataModel {
    /// Explicit Table 2-14 reply bodies (48 bytes after segment/point echoes).
    pub history_min_max: BTreeMap<(u8, u8), [u8; 48]>,
    pub history_tags: BTreeMap<(u8, u8), [u8; 10]>,
    pub history_periodic_indices: BTreeMap<u8, u16>,
    /// Fixture applicability only; these limits are not firmware discovery.
    pub history_max_segment: u8,
    pub history_max_point: u8,
    /// Explicit OP-135 words indexed by (segment, point, selector), not codec-generated values.
    pub history_samples: BTreeMap<(u8, u8, u8), Vec<[u8; 4]>>,
    /// OP-136 raw timestamp words by (segment, selector 0–2).
    pub multi_history_timestamps: BTreeMap<(u8, u8), Vec<[u8; 4]>>,
    /// OP-137 bytes after the segment echo, keyed by (segment, day, month).
    pub day_indices: BTreeMap<(u8, u8, u8), [u8; 8]>,
    /// OP-138 explicit words keyed by (segment, point-or-255, day, month).
    pub day_history: BTreeMap<(u8, u8, u8, u8), DayHistoryData>,
    /// Explicit command-0 inventory by segment; not inferred from fixture sample keys.
    pub configured_history_points: BTreeMap<u8, Vec<u8>>,
    pub history_sample_indices: BTreeMap<(u8, u8), u16>,
    /// Explicit transaction storage order and raw records; no rollover/retention simulation.
    pub transactions: BTreeMap<u8, TransactionSegmentModel>,
}
impl Default for HistoryMetadataModel {
    fn default() -> Self {
        Self {
            history_min_max: BTreeMap::new(),
            history_tags: BTreeMap::new(),
            history_periodic_indices: BTreeMap::new(),
            history_max_segment: 10,
            history_max_point: 199,
            history_samples: BTreeMap::new(),
            history_sample_indices: BTreeMap::new(),
            transactions: BTreeMap::new(),
            configured_history_points: BTreeMap::new(),
            day_indices: BTreeMap::new(),
            day_history: BTreeMap::new(),
            multi_history_timestamps: BTreeMap::new(),
        }
    }
}
impl Model {
    fn parameter_value(&self, tlp: Tlp) -> Option<Value> {
        if tlp.point_type == 99 {
            let tables = self.configurable_tables.lock().unwrap();
            let table = tables.get(&tlp.logical)?;
            return match tlp.parameter {
                0 => Some(Value::Float(table.revision.as_float())),
                1..=44 => Some(Value::Tlp(table.entries[usize::from(tlp.parameter) - 1])),
                _ => None,
            };
        }
        self.written_parameters
            .lock()
            .unwrap()
            .get(&tlp)
            .cloned()
            .or_else(|| self.parameters.get(&tlp).cloned())
    }
    /// Reset fixture I/O to five CPU diagnostics and absent field modules in an explicit layout.
    pub fn set_io_layout(&mut self, layout: crate::IoLayout) {
        self.io_layout = layout;
        self.io_point_types = vec![0; layout.len()];
        self.io_point_types[..5].fill(109);
        self.io_logical_numbers = vec![0; layout.len()];
        self.io_logical_numbers[..5].copy_from_slice(&[0, 1, 2, 3, 4]);
        let mut bytes = self.configuration.as_bytes().to_vec();
        bytes[4] = layout.status();
        self.configuration =
            SystemConfiguration::decode(&bytes).expect("existing fixture configuration");
    }
    /// Append using the fixture's last-written cursor convention, replacing one ring slot.
    pub fn append_alarm(&mut self, bytes: [u8; 23]) -> Result<u16, Error> {
        append_log(&mut self.alarm_log, &mut self.alarm_index, bytes)
    }
    pub fn append_event(&mut self, bytes: [u8; 22]) -> Result<u16, Error> {
        append_log(&mut self.event_log, &mut self.event_index, bytes)
    }
    // Independent fixture implementation of Tables 2-23/26 and the audited write subset.
    // Valid tuples apply in order, even if a later tuple fails: a deliberate partial-effect
    // fixture policy, never a claim about controller atomicity.
    fn parameter_write(&self, request: &Frame) -> Frame {
        let reply = |opcode, data| Frame::new(request.source, self.address, opcode, data);
        let arbitrary = request.opcode == 181;
        let bytes = &request.data;
        if bytes.len() < 5 || bytes.len() > if arbitrary { 240 } else { 234 } {
            return reply(255, vec![6, 5]);
        }
        let count = usize::from(bytes[if arbitrary { 0 } else { 2 }]);
        if count == 0 || (!arbitrary && usize::from(bytes[3]) + count > 256) {
            return reply(255, vec![6, 5]);
        }
        let mut offset = if arbitrary { 1 } else { 4 };
        let mut errors = Vec::new();
        for index in 0..count {
            let tlp = if arbitrary {
                let Some(tuple) = bytes.get(offset..offset + 3) else {
                    errors.extend([6, (index + 1) as u8]);
                    break;
                };
                offset += 3;
                Tlp::new(tuple[0], tuple[1], tuple[2])
            } else {
                Tlp::new(bytes[0], bytes[1], bytes[3] + index as u8)
            };
            let error_offset = if arbitrary {
                (index + 1) as u8
            } else {
                tlp.parameter
            };
            let kind = match (tlp.point_type, tlp.parameter) {
                (91, 2) if tlp.logical == 0 => Some(crate::DataType::Ascii(20)),
                (101, 0) | (102..=105, 0..=1)
                    if self.io_point_types.get(usize::from(tlp.logical))
                        == Some(&tlp.point_type) =>
                {
                    Some(crate::DataType::Ascii(10))
                }
                (101, 1 | 4 | 5 | 9..=11)
                    if self.io_point_types.get(usize::from(tlp.logical)) == Some(&101) =>
                {
                    Some(crate::DataType::UInt8)
                }
                (101, 2 | 13)
                    if self.io_point_types.get(usize::from(tlp.logical)) == Some(&101) =>
                {
                    Some(crate::DataType::Float)
                }
                _ => None,
            };
            let Some(kind) = kind else {
                // Unsupported width: stop rather than guessing the next tuple boundary.
                errors.extend([2, error_offset]);
                break;
            };
            let width = kind.width();
            let Some(raw) = bytes.get(offset..offset + width) else {
                errors.extend([6, error_offset]);
                break;
            };
            offset += width;
            let value = Value::decode(kind, raw);
            match value {
                Ok(Value::UInt8(v)) if v > 1 => errors.extend([4, error_offset]),
                // Table 3-13: filter /2 admits zero; scan period /13 starts at
                // 0.004 seconds. Compare in f64 as the public schema validator does.
                Ok(Value::Float(v))
                    if !v.is_finite()
                        || !(if tlp.parameter == 13 { 0.004 } else { 0.0 }..=43200.0)
                            .contains(&f64::from(v)) =>
                {
                    errors.extend([4, error_offset])
                }
                Ok(value) => {
                    self.written_parameters.lock().unwrap().insert(tlp, value);
                }
                Err(_) => errors.extend([4, error_offset]),
            }
        }
        if offset != bytes.len() && errors.is_empty() {
            errors.extend([6, 5]);
        }
        if errors.is_empty() {
            reply(request.opcode, vec![])
        } else {
            reply(255, errors)
        }
    }
    /// Model-level request dispatch also works over in-memory or serial fixture channels.
    pub fn respond(&self, request: &Frame, authenticated: &mut bool) -> Option<Frame> {
        self.respond_with_history(request, authenticated, &HistoryMetadataModel::default())
    }
    /// Serve explicit metadata fixtures through the same authentication/address checks.
    pub fn respond_with_history(
        &self,
        request: &Frame,
        authenticated: &mut bool,
        history: &HistoryMetadataModel,
    ) -> Option<Frame> {
        if request.destination != self.address || request.source.unit == 0 {
            return None;
        }
        let reply = |opcode, data| Frame::new(request.source, self.address, opcode, data);
        let error = |code, offset| reply(255, vec![code, offset]);
        if request.opcode == 17 {
            if !matches!(request.data.len(), 5 | 6) {
                return Some(error(6, 5));
            }
            if self
                .login
                .as_ref()
                .is_some_and(|expected| request.data != expected.payload())
            {
                return Some(error(21, 9));
            }
            *authenticated = true;
            return Some(reply(17, vec![]));
        }
        if self.login.is_some() && !*authenticated {
            return Some(error(20, 4));
        }
        Some(match request.opcode {
            225 => {
                let [low, high] = request.data.as_slice() else {
                    return Some(error(6, 5));
                };
                if u16::from_le_bytes([*low, *high]) >= crate::LOG_CAPACITY {
                    error(25, 6)
                } else {
                    reply(225, vec![])
                }
            }
            10 | 11 => {
                let b = &request.data;
                if b.len() < 3 {
                    return Some(error(6, 5));
                }
                let (table, start, count) = (b[0], usize::from(b[1]), usize::from(b[2]));
                if table > 15 || start > 43 || count == 0 || count > 44 || start + count > 44 {
                    return Some(error(25, 6));
                }
                let definition = self
                    .configurable_tables
                    .lock()
                    .unwrap()
                    .get(&table)
                    .cloned();
                let Some(definition) = definition else {
                    return Some(error(25, 6));
                };
                let entries = &definition.entries[start..start + count];
                if request.opcode == 10 {
                    if b.len() != 3 {
                        return Some(error(5, 5));
                    }
                    let mut data = b[..3].to_vec();
                    data.extend(definition.revision.bits().to_le_bytes());
                    for tlp in entries {
                        let Some(value) = self.parameter_value(*tlp) else {
                            return Some(error(2, 6));
                        };
                        let Ok(bytes) = value.encode() else {
                            return Some(error(4, 6));
                        };
                        data.extend(bytes);
                    }
                    if data.len() > 255 {
                        error(5, 5)
                    } else {
                        reply(10, data)
                    }
                } else {
                    let mut offset = 3;
                    let mut errors = Vec::new();
                    for (i, tlp) in entries.iter().enumerate() {
                        let kind =
                            self.parameter_value(*tlp)
                                .map(|v| v.data_type())
                                .or_else(|| {
                                    crate::Roc800Catalog::entry(tlp.point_type, tlp.parameter)
                                        .map(|e| e.data_type)
                                });
                        let Some(kind) = kind else {
                            errors.extend([2, (start + i) as u8]);
                            break;
                        };
                        let Some(raw) = b.get(offset..offset + kind.width()) else {
                            errors.extend([6, (start + i) as u8]);
                            break;
                        };
                        offset += kind.width();
                        let mut data = vec![1, tlp.point_type, tlp.logical, tlp.parameter];
                        data.extend(raw);
                        let response = self.parameter_write(&Frame::new(
                            request.destination,
                            request.source,
                            181,
                            data,
                        ));
                        if response.opcode == 255 {
                            errors.extend([response.data[0], (start + i) as u8]);
                        }
                    }
                    if offset != b.len() && errors.is_empty() {
                        errors.extend([6, 5]);
                    }
                    if errors.is_empty() {
                        reply(11, vec![])
                    } else {
                        reply(255, errors)
                    }
                }
            }
            206 => {
                let b = &request.data;
                if b.len() < 2 {
                    return Some(error(6, 5));
                }
                let Some(segment) = history.transactions.get(&b[1]) else {
                    return Some(error(14, 7));
                };
                match b[0] {
                    1 if b.len() == 4 => {
                        let start = usize::from(u16::from_le_bytes([b[2], b[3]]));
                        if start > segment.records.len() {
                            return Some(error(14, 8));
                        }
                        let count = segment
                            .list_page_size
                            .clamp(1, 40)
                            .min(segment.records.len() - start);
                        let mut data = vec![
                            1,
                            count as u8,
                            u8::from(start + count < segment.records.len()),
                        ];
                        data.extend(segment.description);
                        data.extend(segment.payload_size.to_le_bytes());
                        for (number, time, _) in &segment.records[start..start + count] {
                            data.extend(number.to_le_bytes());
                            data.extend(time.to_le_bytes());
                        }
                        reply(206, data)
                    }
                    2 if b.len() == 6 => {
                        let number = u16::from_le_bytes([b[2], b[3]]);
                        let offset = usize::from(u16::from_le_bytes([b[4], b[5]]));
                        let Some((_, _, bytes)) =
                            segment.records.iter().find(|(n, _, _)| *n == number)
                        else {
                            return Some(error(14, 8));
                        };
                        if offset > bytes.len() {
                            return Some(error(14, 10));
                        }
                        let end = (offset + segment.data_page_size.clamp(1, 252)).min(bytes.len());
                        let mut data =
                            vec![2, (end - offset + 1) as u8, u8::from(end < bytes.len())];
                        data.extend(&bytes[offset..end]);
                        reply(206, data)
                    }
                    1 | 2 => error(6, 5),
                    _ => error(4, 6),
                }
            }
            105 => {
                let [segment, point] = request.data.as_slice() else {
                    return Some(error(6, 5));
                };
                if *segment > history.history_max_segment {
                    return Some(error(14, 6));
                }
                if let Some(body) = history.history_min_max.get(&(*segment, *point)) {
                    let mut bytes = vec![*segment, *point];
                    bytes.extend(body);
                    reply(105, bytes)
                } else {
                    error(14, 7)
                }
            }
            108 => {
                let Some((&segment, rest)) = request.data.split_first() else {
                    return Some(error(6, 5));
                };
                let Some((&count, points)) = rest.split_first() else {
                    return Some(error(6, 5));
                };
                if segment > history.history_max_segment
                    || !(1..=20).contains(&count)
                    || points.len() != usize::from(count)
                {
                    return Some(error(14, 6));
                }
                let Some(index) = history.history_periodic_indices.get(&segment) else {
                    return Some(error(14, 6));
                };
                let mut bytes = vec![segment, count];
                bytes.extend(index.to_le_bytes());
                for point in points {
                    if *point > history.history_max_point {
                        return Some(error(14, 7));
                    }
                    let Some(tag) = history.history_tags.get(&(segment, *point)) else {
                        return Some(error(14, 7));
                    };
                    bytes.push(*point);
                    bytes.extend(tag);
                }
                reply(108, bytes)
            }
            139 => match request.data.as_slice() {
                [0, segment] => {
                    if *segment > history.history_max_segment {
                        return Some(error(14, 7));
                    }
                    let Some(points) = history.configured_history_points.get(segment) else {
                        return Some(error(14, 7));
                    };
                    if points.len() > 252 {
                        return Some(error(5, 5));
                    }
                    let mut bytes = vec![0, *segment, points.len() as u8];
                    bytes.extend(points);
                    reply(139, bytes)
                }
                [
                    1,
                    segment,
                    lo,
                    hi,
                    kind,
                    periods,
                    timestamps,
                    count,
                    points @ ..,
                ] => {
                    let width = points.len() + usize::from(*timestamps);
                    if *segment > history.history_max_segment
                        || *kind > 2
                        || *timestamps > 1
                        || *count == 0
                        || usize::from(*count) != points.len()
                        || points.len() > 247
                        || width * usize::from(*periods) > 60
                    {
                        return Some(error(14, 6));
                    }
                    let Some(configured) = history.configured_history_points.get(segment) else {
                        return Some(error(14, 7));
                    };
                    if points
                        .iter()
                        .any(|p| !configured.contains(p) || *p > history.history_max_point)
                    {
                        return Some(error(14, 14));
                    }
                    let start = usize::from(u16::from_le_bytes([*lo, *hi]));
                    let Some(current) = history.history_sample_indices.get(&(*segment, *kind))
                    else {
                        return Some(error(14, 8));
                    };
                    let mut available = 65536 - start;
                    for point in points {
                        let Some(words) = history.history_samples.get(&(*segment, *point, *kind))
                        else {
                            return Some(error(14, 14));
                        };
                        if words.len() > 65536
                            || start >= words.len()
                            || usize::from(*current) >= words.len()
                        {
                            return Some(error(14, 8));
                        }
                        available = available.min(words.len() - start);
                    }
                    let times = history.multi_history_timestamps.get(&(*segment, *kind));
                    if *timestamps == 1 {
                        let Some(times) = times else {
                            return Some(error(14, 12));
                        };
                        if times.len() > 65536
                            || start >= times.len()
                            || usize::from(*current) >= times.len()
                        {
                            return Some(error(14, 8));
                        }
                        available = available.min(times.len() - start);
                    }
                    let periods = usize::from(*periods).min(available);
                    let mut bytes = vec![1, *segment];
                    bytes.extend(current.to_le_bytes());
                    bytes.extend([periods as u8, *timestamps, *count]);
                    for index in start..start + periods {
                        if *timestamps == 1 {
                            bytes.extend(times.unwrap()[index]);
                        }
                        for point in points {
                            bytes
                                .extend(history.history_samples[&(*segment, *point, *kind)][index]);
                        }
                    }
                    reply(139, bytes)
                }
                _ => error(14, 6),
            },
            137 => {
                let [segment, day, month] = request.data.as_slice() else {
                    return Some(error(6, 5));
                };
                if *segment > history.history_max_segment
                    || crate::HistoryDay::new(*day, *month).is_err()
                {
                    return Some(error(14, 6));
                }
                let Some(body) = history.day_indices.get(&(*segment, *day, *month)) else {
                    return Some(error(14, 7));
                };
                let mut bytes = vec![*segment];
                bytes.extend(body);
                reply(137, bytes)
            }
            138 => {
                let [segment, point, day, month] = request.data.as_slice() else {
                    return Some(error(6, 5));
                };
                if *segment > history.history_max_segment
                    || (*point != 255 && *point > history.history_max_point)
                    || crate::HistoryDay::new(*day, *month).is_err()
                {
                    return Some(error(14, 6));
                }
                let Some(data) = history.day_history.get(&(*segment, *point, *day, *month)) else {
                    return Some(error(14, 8));
                };
                // A whole day is not paginated by OP-138. Never silently truncate it.
                if data.periodic.len().saturating_add(data.daily.len()) > 61 {
                    return Some(error(5, 5));
                }
                let mut bytes = vec![*segment, *point, *day, *month];
                bytes.extend((data.periodic.len() as u16).to_le_bytes());
                bytes.extend((data.daily.len() as u16).to_le_bytes());
                for word in data.periodic.iter().chain(&data.daily) {
                    bytes.extend(word);
                }
                reply(138, bytes)
            }
            136 => {
                let [segment, low, high, kind, first, points, periods] = request.data.as_slice()
                else {
                    return Some(error(6, 5));
                };
                let start = u16::from_le_bytes([*low, *high]);
                let current = history
                    .history_sample_indices
                    .get(&(*segment, *kind))
                    .copied()
                    .unwrap_or(0);
                let mut bytes = vec![*segment, *low, *high];
                bytes.extend(current.to_le_bytes());
                bytes.push(0);
                let end = u16::from(*first) + u16::from(*points);
                let timestamps = history.multi_history_timestamps.get(&(*segment, *kind));
                let valid = *segment <= history.history_max_segment
                    && *kind <= 2
                    && end <= 256
                    && end <= u16::from(history.history_max_point) + 1
                    && (usize::from(*points) + 1) * usize::from(*periods) <= 60;
                if !valid {
                    return Some(reply(136, bytes));
                }
                let Some(timestamps) = timestamps else {
                    return Some(reply(136, bytes));
                };
                if timestamps.len() > 65536
                    || usize::from(start) >= timestamps.len()
                    || usize::from(current) >= timestamps.len()
                {
                    return Some(reply(136, bytes));
                }
                let mut available = timestamps.len() - usize::from(start);
                for point in u16::from(*first)..end {
                    let Some(values) = history.history_samples.get(&(*segment, point as u8, *kind))
                    else {
                        return Some(reply(136, bytes));
                    };
                    if values.len() > 65536 || usize::from(start) >= values.len() {
                        return Some(reply(136, bytes));
                    }
                    available = available.min(values.len() - usize::from(start));
                }
                let count = usize::from(*periods).min(available);
                bytes[5] = ((usize::from(*points) + 1) * count) as u8;
                for (index, timestamp) in timestamps
                    .iter()
                    .enumerate()
                    .skip(usize::from(start))
                    .take(count)
                {
                    bytes.extend(timestamp);
                    for point in u16::from(*first)..end {
                        bytes.extend(
                            history.history_samples[&(*segment, point as u8, *kind)][index],
                        );
                    }
                }
                reply(136, bytes)
            }
            135 => {
                let [segment, point, kind, low, high, count] = request.data.as_slice() else {
                    return Some(error(6, 5));
                };
                if *segment > history.history_max_segment
                    || *point > history.history_max_point
                    || *kind > 4
                    || *count > 60
                {
                    return Some(error(14, 6));
                }
                let Some(words) = history.history_samples.get(&(*segment, *point, *kind)) else {
                    return Some(error(14, 7));
                };
                let Some(current) = history.history_sample_indices.get(&(*segment, *kind)) else {
                    return Some(error(14, 8));
                };
                let start = usize::from(u16::from_le_bytes([*low, *high]));
                if start >= words.len()
                    || words.len() > 65536
                    || usize::from(*current) >= words.len()
                {
                    return Some(error(14, 9));
                }
                let n = usize::from(*count).min(words.len() - start);
                let mut bytes = vec![*segment, *point];
                bytes.extend(current.to_le_bytes());
                bytes.push(n as u8);
                for word in &words[start..start + n] {
                    bytes.extend(word);
                }
                reply(135, bytes)
            }
            118 => match log_response(&request.data, self.alarm_index, &self.alarm_log) {
                Ok(data) => reply(118, data),
                Err(_) => error(25, 6),
            },
            119 => match log_response(&request.data, self.event_index, &self.event_log) {
                Ok(data) => reply(119, data),
                Err(_) => error(25, 6),
            },
            50 => {
                let selector = match request.data.as_slice() {
                    [0] => &self.io_point_types,
                    [1] => &self.io_logical_numbers,
                    [_] => return Some(error(25, 6)),
                    _ => return Some(error(if request.data.is_empty() { 6 } else { 5 }, 5)),
                };
                if selector.len() != self.io_layout.len() {
                    error(50, 4)
                } else {
                    reply(50, selector.clone())
                }
            }
            100 => {
                let [command, start, count] = request.data.as_slice() else {
                    return Some(error(if request.data.len() < 3 { 6 } else { 5 }, 5));
                };
                if *command != 11 {
                    return Some(error(50, 6));
                }
                if *count > 245 || u16::from(*start) + u16::from(*count) > 256 {
                    return Some(error(25, 8));
                }
                let mut bytes = request.data.clone();
                for point_type in u16::from(*start)..u16::from(*start) + u16::from(*count) {
                    bytes.push(
                        self.point_types
                            .get(&(point_type as u8))
                            .copied()
                            .unwrap_or(crate::PointTypeClassification::Absent)
                            .as_byte(),
                    );
                }
                reply(100, bytes)
            }
            6 => {
                if !request.data.is_empty() {
                    return Some(error(5, 5));
                }
                reply(6, self.configuration.as_bytes().to_vec())
            }
            7 => {
                if !request.data.is_empty() {
                    return Some(error(5, 5));
                }
                match self
                    .written_clock
                    .lock()
                    .unwrap()
                    .unwrap_or(self.clock)
                    .encode()
                {
                    Ok(bytes) => reply(7, bytes),
                    Err(_) => error(50, 4),
                }
            }
            8 => {
                let b = &request.data;
                if b.len() != 7 {
                    return Some(error(6, 5));
                }
                match crate::ClockSet::new(
                    u16::from_le_bytes([b[5], b[6]]),
                    b[4],
                    b[3],
                    b[2],
                    b[1],
                    b[0],
                ) {
                    Ok(value) => {
                        *self.written_clock.lock().unwrap() = Some(value.simulated_clock());
                        reply(8, vec![])
                    }
                    Err(_) => error(4, 6),
                }
            }
            180 => {
                let Some(&count) = request.data.first() else {
                    return Some(error(6, 5));
                };
                if count == 0 || request.data.len() != 1 + usize::from(count) * 3 {
                    return Some(error(6, 5));
                }
                let mut bytes = vec![count];
                for (i, tlp) in request.data[1..].as_chunks::<3>().0.iter().enumerate() {
                    let key = Tlp::new(tlp[0], tlp[1], tlp[2]);
                    let value = self.parameter_value(key);
                    let Some(value) = value else {
                        return Some(error(2, (i + 1) as u8));
                    };
                    let Ok(value) = value.encode() else {
                        return Some(error(50, 4));
                    };
                    bytes.extend_from_slice(tlp);
                    bytes.extend(value);
                }
                if bytes.len() > MAX_PARAMETER_RESPONSE {
                    error(5, 5)
                } else {
                    reply(180, bytes)
                }
            }
            166 | 181 => self.parameter_write(request),
            167 => {
                if request.data.len() != 4 {
                    return Some(error(6, 5));
                }
                let [point_type, logical, count, start] = request.data[..].try_into().unwrap();
                if count == 0 || u16::from(start) + u16::from(count) > 256 {
                    return Some(error(25, 9));
                }
                let mut bytes = request.data.clone();
                for p in u16::from(start)..u16::from(start) + u16::from(count) {
                    let key = Tlp::new(point_type, logical, p as u8);
                    let value = self.parameter_value(key);
                    let Some(value) = value else {
                        return Some(error(2, p as u8));
                    };
                    let Ok(value) = value.encode() else {
                        return Some(error(50, 4));
                    };
                    bytes.extend(value);
                }
                if bytes.len() > 234 {
                    error(5, 5)
                } else {
                    reply(167, bytes)
                }
            }
            _ => error(1, 4),
        })
    }
}

/// Inject once, on the next request accepted by any simulator connection.
#[derive(Debug, Clone)]
pub enum Fault {
    /// Replace a typed payload while retaining valid framing and CRC.
    ResponseData(Vec<u8>),
    Disconnect,
    Stall,
    Delay(Duration),
    Fragment(usize),
    CorruptCrc,
    PartialThenClose(usize),
    WrongOpcode,
    WrongAddress,
    Error(Vec<u8>),
    SrbxBeforeReply,
    /// Keep queued until opcode 225, then drop its reply after the pending-state effect.
    LoseAlarmAcknowledgement,
}
/// Fixture pending state shared across TCP connection replacement.
#[derive(Debug, Clone, Copy)]
pub struct SrbxStatus {
    pub current_alarm_index: u16,
    pub pending: bool,
    pub last_acknowledged: Option<u16>,
    pub acknowledgements: usize,
    pub signals_sent: usize,
}
struct Control {
    history: HistoryMetadataModel,
    srbx: Mutex<SrbxStatus>,
    signals: broadcast::Sender<()>,
    faults: Mutex<VecDeque<(Option<u8>, Fault)>>,
    connections: AtomicUsize,
    logins: AtomicUsize,
    requests: AtomicUsize,
}
impl Control {
    fn new(model: &Model) -> Self {
        Self {
            history: HistoryMetadataModel::default(),
            srbx: Mutex::new(SrbxStatus {
                current_alarm_index: model.alarm_index,
                pending: model.initial_srbx_pending,
                last_acknowledged: None,
                acknowledgements: 0,
                signals_sent: 0,
            }),
            signals: broadcast::channel(crate::SRBX_BUFFER_CAPACITY).0,
            faults: Mutex::new(VecDeque::new()),
            connections: AtomicUsize::new(0),
            logins: AtomicUsize::new(0),
            requests: AtomicUsize::new(0),
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Stats {
    pub connections: usize,
    pub logins: usize,
    pub requests: usize,
}

pub struct Simulator {
    endpoint: SocketAddr,
    model: Arc<Model>,
    control: Arc<Control>,
    stopping: watch::Sender<bool>,
    task: Option<JoinHandle<Result<(), Error>>>,
}
impl Simulator {
    pub async fn bind(endpoint: SocketAddr, model: Model) -> Result<Self, Error> {
        Self::bind_with_history(endpoint, model, HistoryMetadataModel::default()).await
    }
    pub async fn bind_with_history(
        endpoint: SocketAddr,
        model: Model,
        history: HistoryMetadataModel,
    ) -> Result<Self, Error> {
        let listener = TcpListener::bind(endpoint).await?;
        let endpoint = listener.local_addr()?;
        let model = Arc::new(model);
        let control = Arc::new(Control {
            history,
            ..Control::new(&model)
        });
        let (stopping, receiver) = watch::channel(false);
        let shared_model = model.clone();
        let shared_control = control.clone();
        let task = tokio::spawn(run(listener, shared_model, shared_control, receiver));
        Ok(Self {
            endpoint,
            model,
            control,
            stopping,
            task: Some(task),
        })
    }
    pub fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }
    pub fn model(&self) -> &Model {
        &self.model
    }
    /// Hold this FIFO fault until the selected request opcode, then apply it
    /// after model dispatch. Useful when metadata checks precede a state change.
    pub fn inject_for(&self, opcode: u8, fault: Fault) {
        self.control
            .faults
            .lock()
            .unwrap()
            .push_back((Some(opcode), fault));
    }
    pub fn inject(&self, fault: Fault) {
        self.control.faults.lock().unwrap().push_back((None, fault));
    }
    /// Raise/repeat a fixture SRBX and update its current alarm index, including while idle.
    /// Populate the corresponding raw log slot before binding; this does not create a record.
    pub fn raise_srbx(&self, index: u16) -> Result<(), Error> {
        if index >= crate::LOG_CAPACITY {
            return Err(Error::InvalidRequest("SRBX index outside ring"));
        }
        {
            let mut state = self.control.srbx.lock().unwrap();
            state.current_alarm_index = index;
            state.pending = true;
        }
        let _ = self.control.signals.send(());
        Ok(())
    }
    pub fn srbx_status(&self) -> SrbxStatus {
        *self.control.srbx.lock().unwrap()
    }
    pub fn stats(&self) -> Stats {
        Stats {
            connections: self.control.connections.load(Ordering::SeqCst),
            logins: self.control.logins.load(Ordering::SeqCst),
            requests: self.control.requests.load(Ordering::SeqCst),
        }
    }
    pub async fn stop(mut self) -> Result<(), Error> {
        self.stopping.send_replace(true);
        if let Some(task) = self.task.take() {
            task.await.map_err(std::io::Error::other)??;
        }
        Ok(())
    }
}
impl Drop for Simulator {
    fn drop(&mut self) {
        self.stopping.send_replace(true);
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}
async fn run(
    listener: TcpListener,
    model: Arc<Model>,
    control: Arc<Control>,
    mut stopping: watch::Receiver<bool>,
) -> Result<(), Error> {
    let mut connections = JoinSet::new();
    loop {
        tokio::select! {
            biased;
            _ = stopping.changed() => break,
            result = connections.join_next(), if !connections.is_empty() => {
                if let Some(Err(error)) = result { return Err(std::io::Error::other(error).into()); }
            }
            connection = listener.accept() => {
                let (mut stream, _) = connection?;
                control.connections.fetch_add(1, Ordering::SeqCst);
                let model = model.clone();
                let control = control.clone();
                let mut stop = stopping.clone();
                connections.spawn(async move {
                    tokio::select! {
                        _ = stop.changed() => Ok(()),
                        result = serve(&mut stream, &model, &control) => result,
                    }
                });
            }
        }
    }
    connections.abort_all();
    while connections.join_next().await.is_some() {}
    Ok(())
}

/// Serve a model over a caller-owned channel. Caller bounds lifetime/cancellation.
/// TCP Simulator adds fault injection, statistics and connection teardown.
pub async fn serve_stream<S: ByteStream + ?Sized>(
    stream: &mut S,
    model: &Model,
) -> Result<(), Error> {
    serve(stream, model, &Control::new(model)).await
}
async fn serve<S: ByteStream + ?Sized>(
    stream: &mut S,
    model: &Model,
    control: &Control,
) -> Result<(), Error> {
    let mut authenticated = false;
    let (mut reader, mut writer) = tokio::io::split(stream);
    let mut signals = control.signals.subscribe();
    let signal = Frame::new(model.srbx_host, model.address, 224, vec![]);
    if control.srbx.lock().unwrap().pending {
        write_frame(&mut writer, &signal).await?;
        control.srbx.lock().unwrap().signals_sent += 1;
    }
    loop {
        // Keep a partial request read alive when an idle notification is emitted.
        let request = {
            let read = read_frame(&mut reader, CrcMode::Required);
            tokio::pin!(read);
            loop {
                tokio::select! {
                    biased;
                    result = &mut read => break match result {
                        Err(Error::Io(error)) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(()),
                        result => result?,
                    },
                    notice = signals.recv() => {
                        if notice.is_err_and(|e| matches!(e, broadcast::error::RecvError::Closed)) { return Ok(()); }
                        let pending = control.srbx.lock().unwrap().pending;
                        if pending {
                            write_frame(&mut writer, &signal).await?;
                            control.srbx.lock().unwrap().signals_sent += 1;
                        }
                    }
                }
            }
        };
        control.requests.fetch_add(1, Ordering::SeqCst);
        let Some(mut response) =
            model.respond_with_history(&request, &mut authenticated, &control.history)
        else {
            continue;
        };
        if request.opcode == 17 && response.opcode == 17 {
            control.logins.fetch_add(1, Ordering::SeqCst);
        }
        let fault = {
            let mut faults = control.faults.lock().unwrap();
            let waiting = faults.front().is_some_and(|(opcode, fault)| {
                opcode.is_some_and(|opcode| opcode != request.opcode)
                    || (matches!(fault, Fault::LoseAlarmAcknowledgement) && request.opcode != 225)
            });
            if waiting {
                None
            } else {
                faults.pop_front().map(|(_, fault)| fault)
            }
        };
        if response.opcode == 118 {
            response.data[3..5].copy_from_slice(
                &control
                    .srbx
                    .lock()
                    .unwrap()
                    .current_alarm_index
                    .to_le_bytes(),
            );
        }
        if response.opcode == 225 && !matches!(fault, Some(Fault::Error(_))) {
            let index = u16::from_le_bytes([request.data[0], request.data[1]]);
            let mut state = control.srbx.lock().unwrap();
            state.acknowledgements += 1;
            state.last_acknowledged = Some(index);
            if index == state.current_alarm_index {
                state.pending = false;
            }
        }
        match &fault {
            Some(Fault::ResponseData(data)) => response.data.clone_from(data),
            Some(Fault::Disconnect | Fault::LoseAlarmAcknowledgement) => return Ok(()),
            Some(Fault::Stall) => return std::future::pending().await,
            Some(Fault::Delay(delay)) => tokio::time::sleep(*delay).await,
            Some(Fault::WrongOpcode) => response.opcode = 254,
            Some(Fault::WrongAddress) => response.source.unit = 2,
            Some(Fault::Error(errors)) => {
                response.opcode = 255;
                response.data.clone_from(errors);
            }
            Some(Fault::SrbxBeforeReply) => {
                write_frame(
                    &mut writer,
                    &Frame::new(request.source, model.address, 224, vec![]),
                )
                .await?
            }
            _ => {}
        }
        let bytes = response.encode()?;
        match fault {
            Some(Fault::CorruptCrc) => {
                let mut bytes = bytes;
                let end = bytes.len() - 1;
                bytes[end] ^= 0x80;
                writer.write_all(&bytes).await?;
            }
            Some(Fault::PartialThenClose(n)) => {
                writer.write_all(&bytes[..n.min(bytes.len())]).await?;
                return Ok(());
            }
            Some(Fault::Fragment(chunk)) => {
                for bytes in bytes.chunks(chunk.max(1)) {
                    writer.write_all(bytes).await?;
                    tokio::task::yield_now().await;
                }
            }
            _ => writer.write_all(&bytes).await?,
        }
    }
}

fn append_log<const N: usize>(
    slots: &mut BTreeMap<u16, [u8; N]>,
    current: &mut u16,
    bytes: [u8; N],
) -> Result<u16, Error> {
    if *current >= crate::LOG_CAPACITY {
        return Err(Error::InvalidConfig("simulator log index outside ring"));
    }
    *current = (*current + 1) % crate::LOG_CAPACITY;
    slots.insert(*current, bytes);
    Ok(*current)
}
fn log_response<const N: usize>(
    payload: &[u8],
    current: u16,
    slots: &BTreeMap<u16, [u8; N]>,
) -> Result<Vec<u8>, Error> {
    let [count, low, high] = payload else {
        return Err(Error::InvalidRequest("log header length"));
    };
    let request = crate::LogRequest::new(u16::from_le_bytes([*low, *high]), *count)?;
    if current >= crate::LOG_CAPACITY {
        return Err(Error::InvalidConfig("simulator log index outside ring"));
    }
    let mut bytes = request.payload();
    bytes.extend(current.to_le_bytes());
    for index in request.start()..request.start() + u16::from(request.count()) {
        bytes.extend(slots.get(&index).copied().unwrap_or([0; N]));
    }
    Ok(bytes)
}
