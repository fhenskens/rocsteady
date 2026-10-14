//! Fixed records from D301180X012, sections 2.13-2.14; raw bytes survive interpretation.
use crate::{DataType, DecodeError, Tlp, Value};

/// Fixed-width log text. No padding/encoding convention is silently assumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogText<const N: usize>(pub [u8; N]);
impl<const N: usize> LogText<N> {
    /// Exact printable ASCII, including spaces; otherwise retain/access the raw bytes.
    pub fn printable_ascii(&self) -> Option<&str> {
        self.0
            .iter()
            .all(|b| (0x20..=0x7e).contains(b))
            .then(|| std::str::from_utf8(&self.0).ok())
            .flatten()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlarmReason(pub u8);
impl AlarmReason {
    /// A source reason label, not a context-free interpretation of its TLP.
    pub fn key(self) -> Option<&'static str> {
        const KEYS: &[&str] = &[
            "low",
            "low_low",
            "high",
            "high_high",
            "rate",
            "status_change",
            "point_fail",
            "scanning_disabled",
            "scanning_manual",
            "redundant_counts",
            "redundant_flow",
            "no_flow",
            "input_freeze",
            "sensor_communication",
            "rs485_communication",
            "off_scan",
            "manual_flow_inputs",
            "meter_temperature",
            "compressibility",
            "sequence",
            "phase",
            "pulse_synchronization",
            "frequency",
            "pulse_input_1",
            "pulse_input_2",
            "pulse_buffer_overrun",
            "pulse_buffer_warning",
            "relay_fault",
            "relay_failure",
            "pressure_low_limit",
            "temperature_low_limit",
            "analog_readback",
            "level_a_pulse",
            "market_pulse",
        ];
        KEYS.get(usize::from(self.0)).copied()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemEventReason(pub u8);
impl SystemEventReason {
    pub const fn key(self) -> Option<&'static str> {
        Some(match self.0 {
            144 => "initialization",
            145 => "power_removed",
            146 => "defaults",
            147 => "rom_crc",
            148 => "database_initialization",
            150 => "program_flash",
            151..=153 => "reserved",
            154 => "module_inserted",
            155 => "module_removed",
            200 => "clock_set",
            248 => "text",
            249 => "configuration_download",
            250 => "configuration_upload",
            251 => "calibration_timeout",
            252 => "calibration_cancel",
            253 => "calibration_success",
            254 => "mvs_defaults",
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum AlarmData {
    None,
    Parameter {
        reason: AlarmReason,
        tlp: Tlp,
        description: LogText<10>,
        value: f32,
    },
    Fst {
        number: u8,
        description: LogText<13>,
        value: f32,
    },
    UserText(LogText<18>),
    UserValue {
        description: LogText<14>,
        value: f32,
    },
    Unknown {
        kind: u8,
        bytes: [u8; 18],
    },
}
#[derive(Debug, Clone, PartialEq)]
pub struct AlarmRecord {
    raw: [u8; 23],
    data: AlarmData,
}
impl AlarmRecord {
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let raw: [u8; 23] = bytes
            .try_into()
            .map_err(|_| DecodeError("alarm record must be 23 bytes"))?;
        let data = match raw[0] & 0x3f {
            0 => AlarmData::None,
            1 => AlarmData::Parameter {
                reason: AlarmReason(raw[5]),
                tlp: tlp(&raw[6..9]),
                description: LogText(array(&raw[9..19])),
                value: float(&raw[19..23]),
            },
            2 => AlarmData::Fst {
                number: raw[5],
                description: LogText(array(&raw[6..19])),
                value: float(&raw[19..23]),
            },
            3 => AlarmData::UserText(LogText(array(&raw[5..23]))),
            4 => AlarmData::UserValue {
                description: LogText(array(&raw[5..19])),
                value: float(&raw[19..23]),
            },
            kind => AlarmData::Unknown {
                kind,
                bytes: array(&raw[5..23]),
            },
        };
        Ok(Self { raw, data })
    }
    pub fn raw(&self) -> &[u8; 23] {
        &self.raw
    }
    pub fn timestamp(&self) -> u32 {
        uint32(&self.raw[1..5])
    }
    pub fn srbx(&self) -> bool {
        self.raw[0] & 0x80 != 0
    }
    pub fn is_set(&self) -> bool {
        self.raw[0] & 0x40 != 0
    }
    pub fn data(&self) -> &AlarmData {
        &self.data
    }
}

/// Event-specific tag numbering from Table 2-17; not transaction/parameter tag numbering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventDataType {
    Known(DataType),
    Unknown(u8),
}
impl EventDataType {
    pub const fn from_byte(tag: u8) -> Self {
        Self::Known(match tag {
            0 => DataType::Binary,
            1 => DataType::Int8,
            2 => DataType::Int16,
            3 => DataType::Int32,
            4 => DataType::UInt8,
            5 => DataType::UInt16,
            6 => DataType::UInt32,
            7 => DataType::Float,
            8 => DataType::Tlp,
            9 => DataType::Ascii(3),
            10 => DataType::Ascii(7),
            11 => DataType::Ascii(10),
            12 => DataType::Ascii(12),
            13 => DataType::Ascii(20),
            14 => DataType::Ascii(30),
            15 => DataType::Ascii(40),
            16 => DataType::Double,
            17 => DataType::Time,
            _ => return Self::Unknown(tag),
        })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum LoggedValue {
    Complete(Value),
    /// The type needs more than the record's ten new-value bytes. No value is fabricated.
    Truncated {
        data_type: DataType,
        bytes: [u8; 10],
    },
    Unknown {
        tag: u8,
        bytes: [u8; 10],
    },
}
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum EventData {
    None,
    ParameterChange {
        operator: [u8; 3],
        tlp: Tlp,
        data_type: EventDataType,
        new_value: LoggedValue,
        old_value: Option<Value>,
    },
    System {
        reason: SystemEventReason,
        description: LogText<16>,
    },
    Fst {
        number: u8,
        value: f32,
        description: LogText<10>,
    },
    User {
        operator: [u8; 3],
        reason: SystemEventReason,
        description: LogText<13>,
    },
    PowerLost {
        time: u32,
    },
    ClockSet {
        time: u32,
    },
    CalibrationVerification {
        operator: [u8; 3],
        tlp: Tlp,
        raw_value: f32,
        calibrated_value: f32,
    },
    Unknown {
        kind: u8,
        bytes: [u8; 17],
    },
}
#[derive(Debug, Clone, PartialEq)]
pub struct EventRecord {
    raw: [u8; 22],
    data: EventData,
}
impl EventRecord {
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let raw: [u8; 22] = bytes
            .try_into()
            .map_err(|_| DecodeError("event record must be 22 bytes"))?;
        let data = match raw[0] {
            0 => EventData::None,
            1 => {
                let data_type = EventDataType::from_byte(raw[11]);
                let (new_value, old_value) = match data_type {
                    EventDataType::Known(kind) if kind.width() <= 10 => {
                        let new = decode_value(kind, &raw[12..12 + kind.width()])?;
                        // >4-byte new values occupy old-value storage. Do not interpret overlap.
                        let old = if kind.width() <= 4 {
                            Some(decode_value(kind, &raw[16..16 + kind.width()])?)
                        } else {
                            None
                        };
                        (LoggedValue::Complete(new), old)
                    }
                    EventDataType::Known(kind) => (
                        LoggedValue::Truncated {
                            data_type: kind,
                            bytes: array(&raw[12..22]),
                        },
                        None,
                    ),
                    EventDataType::Unknown(tag) => (
                        LoggedValue::Unknown {
                            tag,
                            bytes: array(&raw[12..22]),
                        },
                        None,
                    ),
                };
                EventData::ParameterChange {
                    operator: array(&raw[5..8]),
                    tlp: tlp(&raw[8..11]),
                    data_type,
                    new_value,
                    old_value,
                }
            }
            2 => EventData::System {
                reason: SystemEventReason(raw[5]),
                description: LogText(array(&raw[6..22])),
            },
            3 => EventData::Fst {
                number: raw[5],
                value: float(&raw[6..10]),
                description: LogText(array(&raw[10..20])),
            },
            4 => EventData::User {
                operator: array(&raw[5..8]),
                reason: SystemEventReason(raw[8]),
                description: LogText(array(&raw[9..22])),
            },
            5 => EventData::PowerLost {
                time: uint32(&raw[5..9]),
            },
            6 => EventData::ClockSet {
                time: uint32(&raw[5..9]),
            },
            7 => EventData::CalibrationVerification {
                operator: array(&raw[5..8]),
                tlp: tlp(&raw[8..11]),
                raw_value: float(&raw[11..15]),
                calibrated_value: float(&raw[15..19]),
            },
            kind => EventData::Unknown {
                kind,
                bytes: array(&raw[5..22]),
            },
        };
        Ok(Self { raw, data })
    }
    pub fn raw(&self) -> &[u8; 22] {
        &self.raw
    }
    pub fn timestamp(&self) -> u32 {
        uint32(&self.raw[1..5])
    }
    pub fn data(&self) -> &EventData {
        &self.data
    }
}
fn decode_value(kind: DataType, bytes: &[u8]) -> Result<Value, DecodeError> {
    Value::decode(kind, bytes).map_err(|_| DecodeError("malformed tagged event value"))
}
// All callers first establish a complete fixed record; these slices have fixed widths.
fn array<const N: usize>(bytes: &[u8]) -> [u8; N] {
    bytes.try_into().expect("fixed record field width")
}
fn uint32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(array(bytes))
}
fn float(bytes: &[u8]) -> f32 {
    f32::from_le_bytes(array(bytes))
}
fn tlp(bytes: &[u8]) -> Tlp {
    Tlp::new(bytes[0], bytes[1], bytes[2])
}
