//! Source-audited table facts; applicability and lookup live in the provider.
use super::*;

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
pub(super) const ENTRIES: &[CatalogEntry] = &[
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
    // PT-136/17's printed 1-56 range conflicts with PT-136/12's week model.
    // Keep its byte readable, without selecting an unverified write constraint.
    entry!(
        136,
        0,
        "clock_seconds",
        DataType::UInt8,
        ReadOnly,
        Some((0.0, 59.0)),
        (1, 0),
        "3.4.47/Table 3-48, p.3-200"
    ),
    entry!(
        136,
        1,
        "clock_minutes",
        DataType::UInt8,
        ReadOnly,
        Some((0.0, 59.0)),
        (1, 0),
        "3.4.47/Table 3-48, p.3-200"
    ),
    entry!(
        136,
        2,
        "clock_hours",
        DataType::UInt8,
        ReadOnly,
        Some((0.0, 23.0)),
        (1, 0),
        "3.4.47/Table 3-48, p.3-200"
    ),
    entry!(
        136,
        3,
        "clock_day",
        DataType::UInt8,
        ReadOnly,
        Some((1.0, 31.0)),
        (1, 0),
        "3.4.47/Table 3-48, p.3-200"
    ),
    entry!(
        136,
        4,
        "clock_month",
        DataType::UInt8,
        ReadOnly,
        Some((1.0, 12.0)),
        (1, 0),
        "3.4.47/Table 3-48, p.3-200"
    ),
    entry!(
        136,
        5,
        "clock_year",
        DataType::UInt16,
        ReadOnly,
        Some((2000.0, 2104.0)),
        (1, 0),
        "3.4.47/Table 3-48, p.3-200"
    ),
    entry!(
        136,
        6,
        "clock_weekday",
        DataType::UInt8,
        ReadOnly,
        Some((1.0, 7.0)),
        (1, 0),
        "3.4.47/Table 3-48, p.3-200"
    ),
    entry!(
        136,
        7,
        "clock_time",
        DataType::Time,
        ReadOnly,
        None,
        (1, 0),
        "3.4.47/Table 3-48, p.3-200"
    ),
    entry!(
        136,
        8,
        "clock_dst_enabled",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.47/Table 3-48, p.3-200"
    ),
    entry!(
        136,
        9,
        "clock_microseconds",
        DataType::UInt32,
        ReadOnly,
        Some((0.0, 999999.0)),
        (1, 20),
        "3.4.47/Table 3-48, p.3-200"
    ),
    entry!(
        136,
        10,
        "clock_dst_start_hour",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 23.0)),
        (3, 40),
        "3.4.47/Table 3-48, p.3-200"
    ),
    entry!(
        136,
        11,
        "clock_dst_start_weekday",
        DataType::UInt8,
        ReadWrite,
        Some((1.0, 7.0)),
        (3, 40),
        "3.4.47/Table 3-48, p.3-200"
    ),
    entry!(
        136,
        12,
        "clock_dst_start_week",
        DataType::UInt8,
        ReadWrite,
        Some((1.0, 6.0)),
        (3, 40),
        "3.4.47/Table 3-48, p.3-200"
    ),
    entry!(
        136,
        13,
        "clock_dst_start_month",
        DataType::UInt8,
        ReadWrite,
        Some((1.0, 12.0)),
        (3, 40),
        "3.4.47/Table 3-48, p.3-201"
    ),
    entry!(
        136,
        14,
        "clock_dst_start_time",
        DataType::Time,
        ReadOnly,
        None,
        (3, 40),
        "3.4.47/Table 3-48, p.3-201"
    ),
    entry!(
        136,
        15,
        "clock_dst_end_hour",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 23.0)),
        (3, 40),
        "3.4.47/Table 3-48, p.3-201"
    ),
    entry!(
        136,
        16,
        "clock_dst_end_weekday",
        DataType::UInt8,
        ReadWrite,
        Some((1.0, 7.0)),
        (3, 40),
        "3.4.47/Table 3-48, p.3-201"
    ),
    entry!(
        136,
        17,
        "clock_dst_end_week",
        DataType::UInt8,
        ReadWrite,
        None,
        (3, 40),
        "3.4.47/Table 3-48, p.3-201; disputed week range"
    ),
    entry!(
        136,
        18,
        "clock_dst_end_month",
        DataType::UInt8,
        ReadWrite,
        Some((1.0, 12.0)),
        (3, 40),
        "3.4.47/Table 3-48, p.3-201"
    ),
    entry!(
        136,
        19,
        "clock_dst_end_time",
        DataType::Time,
        ReadOnly,
        None,
        (3, 40),
        "3.4.47/Table 3-48, p.3-201"
    ),
];
