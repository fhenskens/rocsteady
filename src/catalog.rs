//! Small source-audited catalog slice; no complete point-type table is claimed.
use crate::{
    Access, DataType, Error, FirmwareVersion, IoSnapshot, ParameterSchema, SchemaContext,
    SchemaProvider, Tlp,
};

#[derive(Debug, Clone, Copy)]
pub struct CatalogEntry {
    pub point_type: u8,
    pub parameter: u8,
    pub key: &'static str,
    pub data_type: DataType,
    pub access: Access,
    pub numeric_range: Option<(f64, f64)>,
    pub introduced: (u8, u8),
    pub source: &'static str,
}
macro_rules! entry {
    ($pt:literal, $p:literal, $key:literal, $ty:expr, $access:ident, $range:expr, $version:expr, $source:literal) => {
        CatalogEntry {
            point_type: $pt,
            parameter: $p,
            key: $key,
            data_type: $ty,
            access: Access::$access,
            numeric_range: $range,
            introduced: $version,
            source: $source,
        }
    };
}
// Technical facts only; vendor descriptions/defaults are not reproduced.
const ENTRIES: &[CatalogEntry] = &[
    entry!(
        91,
        0,
        "unit_address",
        DataType::UInt8,
        ReadWrite,
        Some((1.0, 255.0)),
        (1, 0),
        "3.4.4/Table 3-5, p.3-34"
    ),
    entry!(
        91,
        1,
        "group_address",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 255.0)),
        (1, 0),
        "3.4.4/Table 3-5, p.3-34"
    ),
    entry!(
        91,
        2,
        "station_name",
        DataType::Ascii(20),
        ReadWrite,
        None,
        (1, 0),
        "3.4.4/Table 3-5, p.3-34"
    ),
    entry!(
        91,
        3,
        "firmware_identification",
        DataType::Ascii(20),
        ReadOnly,
        None,
        (1, 0),
        "3.4.4/Table 3-5, p.3-34"
    ),
    entry!(
        91,
        4,
        "firmware_build_time",
        DataType::Ascii(20),
        ReadOnly,
        None,
        (1, 0),
        "3.4.4/Table 3-5, p.3-34"
    ),
    entry!(
        91,
        5,
        "manufacturer",
        DataType::Ascii(20),
        ReadOnly,
        None,
        (1, 0),
        "3.4.4/Table 3-5, p.3-34"
    ),
    entry!(
        91,
        6,
        "product",
        DataType::Ascii(20),
        ReadOnly,
        None,
        (1, 0),
        "3.4.4/Table 3-5, p.3-34"
    ),
    entry!(
        91,
        8,
        "event_capacity",
        DataType::UInt16,
        ReadOnly,
        Some((450.0, 450.0)),
        (1, 0),
        "3.4.4/Table 3-5, p.3-34"
    ),
    entry!(
        91,
        9,
        "alarm_capacity",
        DataType::UInt16,
        ReadOnly,
        Some((450.0, 450.0)),
        (1, 0),
        "3.4.4/Table 3-5, p.3-34"
    ),
    entry!(
        91,
        41,
        "flash_used",
        DataType::UInt32,
        ReadOnly,
        Some((0.0, 4294967295.0)),
        (1, 20),
        "3.4.4/Table 3-5, p.3-37"
    ),
    entry!(
        91,
        42,
        "flash_free",
        DataType::UInt32,
        ReadOnly,
        Some((0.0, 4294967295.0)),
        (1, 20),
        "3.4.4/Table 3-5, p.3-37"
    ),
    entry!(
        91,
        50,
        "points_per_slot_mode",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (2, 0),
        "3.4.4/Table 3-5, p.3-38; Series 2 introduced 3.00"
    ),
    entry!(
        101,
        0,
        "di_tag",
        DataType::Ascii(10),
        ReadWrite,
        None,
        (1, 0),
        "3.4.12/Table 3-13, p.3-61"
    ),
    entry!(
        101,
        1,
        "di_scanning",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.12/Table 3-13, p.3-61"
    ),
    entry!(
        101,
        3,
        "di_state",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.12/Table 3-13, p.3-61"
    ),
    entry!(
        101,
        6,
        "di_count",
        DataType::UInt32,
        ReadWrite,
        Some((0.0, 16000000.0)),
        (1, 0),
        "3.4.12/Table 3-13, p.3-61"
    ),
    entry!(
        103,
        0,
        "ai_tag",
        DataType::Ascii(10),
        ReadWrite,
        None,
        (1, 0),
        "3.4.14/Table 3-15, p.3-66"
    ),
    entry!(
        103,
        1,
        "ai_units_tag",
        DataType::Ascii(10),
        ReadWrite,
        None,
        (1, 0),
        "3.4.14/Table 3-15, p.3-66"
    ),
    entry!(
        103,
        2,
        "ai_scanning",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.14/Table 3-15, p.3-66"
    ),
    entry!(
        103,
        3,
        "ai_scan_period",
        DataType::Float,
        ReadWrite,
        Some((0.05, 43200.0)),
        (1, 0),
        "3.4.14/Table 3-15, p.3-66"
    ),
    entry!(
        103,
        4,
        "ai_scan_time",
        DataType::Float,
        ReadOnly,
        Some((0.0, 43200.0)),
        (1, 0),
        "3.4.14/Table 3-15, p.3-66"
    ),
    entry!(
        103,
        7,
        "ai_raw",
        DataType::UInt16,
        ReadOnly,
        Some((0.0, 65535.0)),
        (1, 0),
        "3.4.14/Table 3-15, p.3-66"
    ),
    entry!(
        103,
        21,
        "ai_value",
        DataType::Float,
        ReadOnly,
        None,
        (1, 0),
        "3.4.14/Table 3-15, p.3-67"
    ),
    entry!(
        103,
        32,
        "ai_alarm_bits",
        DataType::Binary,
        ReadOnly,
        None,
        (1, 0),
        "3.4.14/Table 3-15, p.3-68"
    ),
    entry!(
        109,
        0,
        "system_ai_tag",
        DataType::Ascii(10),
        ReadWrite,
        None,
        (1, 0),
        "3.4.20/Table 3-21, p.3-90"
    ),
    entry!(
        109,
        1,
        "system_ai_units_tag",
        DataType::Ascii(10),
        ReadWrite,
        None,
        (1, 0),
        "3.4.20/Table 3-21, p.3-90"
    ),
    entry!(
        109,
        4,
        "system_ai_scan_time",
        DataType::Float,
        ReadOnly,
        Some((1.0, 43200.0)),
        (1, 0),
        "3.4.20/Table 3-21, p.3-90"
    ),
    entry!(
        109,
        7,
        "system_ai_raw",
        DataType::UInt16,
        ReadWrite,
        Some((0.0, 65535.0)),
        (1, 0),
        "3.4.20/Table 3-21, p.3-91"
    ),
    entry!(
        109,
        12,
        "system_ai_value",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.20/Table 3-21, p.3-91"
    ),
    entry!(
        109,
        23,
        "system_ai_alarm_bits",
        DataType::Binary,
        ReadOnly,
        None,
        (1, 0),
        "3.4.20/Table 3-21, p.3-93"
    ),
];

/// Explicit ROC800 firmware identity; I/O existence comes from a generation-bound snapshot.
pub struct Roc800Catalog {
    firmware: FirmwareVersion,
    io: Option<IoSnapshot>,
}
impl Roc800Catalog {
    pub fn new(firmware: FirmwareVersion) -> Result<Self, Error> {
        // Conservative reviewed applicability window; future firmware requires an override.
        if firmware > FirmwareVersion::new(3, 60)? {
            return Err(Error::CapabilityUnavailable(
                "catalog firmware outside reviewed 1.00-3.60 window",
            ));
        }
        Ok(Self { firmware, io: None })
    }
    pub fn from_discovery(io: IoSnapshot) -> Result<Self, Error> {
        if io.configuration().device_type() != 6
            || io.configuration().mode() != crate::SystemMode::Run
        {
            return Err(Error::CapabilityUnavailable("ROC800 catalog identity"));
        }
        let mut catalog = Self::new(io.firmware())?;
        catalog.io = Some(io);
        Ok(catalog)
    }
    pub fn context(&self) -> SchemaContext {
        SchemaContext {
            device_family: Some("ROC800".into()),
            firmware: Some(self.firmware.to_string()),
        }
    }
    pub fn entries() -> &'static [CatalogEntry] {
        ENTRIES
    }
    pub fn entry(point_type: u8, parameter: u8) -> Option<&'static CatalogEntry> {
        ENTRIES
            .iter()
            .find(|entry| entry.point_type == point_type && entry.parameter == parameter)
    }
}
impl SchemaProvider for Roc800Catalog {
    fn generation(&self) -> Result<Option<u64>, Error> {
        Ok(self.io.as_ref().map(IoSnapshot::generation))
    }
    fn lookup(&self, context: &SchemaContext, tlp: Tlp) -> Result<Option<ParameterSchema>, Error> {
        if context.device_family.as_deref() != Some("ROC800")
            || context.firmware.as_deref() != Some(self.firmware.to_string().as_str())
        {
            return Err(Error::CapabilityUnavailable(
                "catalog requires matching explicit ROC800 firmware identity",
            ));
        }
        let Some(entry) = Self::entry(tlp.point_type, tlp.parameter) else {
            return Ok(None);
        };
        if self.firmware < FirmwareVersion::new(entry.introduced.0, entry.introduced.1)? {
            return Err(Error::CapabilityUnavailable(
                "parameter introduced after selected firmware",
            ));
        }
        let logical_range = if tlp.point_type == 91 {
            if tlp.logical != 0 {
                return Err(Error::InvalidRequest(
                    "system variables require logical zero",
                ));
            }
            Some((0, 0))
        } else {
            let io = self.io.as_ref().ok_or(Error::CapabilityUnavailable(
                "catalog requires I/O discovery",
            ))?;
            let point = io
                .by_location(tlp.logical)
                .filter(|point| point.point_type == tlp.point_type)
                .ok_or(Error::CapabilityUnavailable(
                    "point type absent at requested physical location",
                ))?;
            if tlp.point_type == 109 && point.logical_number > 4 {
                return Err(Error::CapabilityUnavailable(
                    "unreviewed system diagnostic point",
                ));
            }
            Some((point.location, point.location))
        };
        Ok(Some(ParameterSchema {
            data_type: entry.data_type,
            access: entry.access,
            logical_range,
            numeric_range: entry.numeric_range,
        }))
    }
}
