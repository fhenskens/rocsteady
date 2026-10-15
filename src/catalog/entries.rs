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
            metadata: metadata::for_parameter($pt, $p),
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
        91,
        10,
        "pid_capacity",
        DataType::UInt8,
        ReadOnly,
        Some((0.0, 16.0)),
        (1, 0),
        "3.4.4/Table 3-5, p.3-34"
    ),
    entry!(
        91,
        11,
        "meter_capacity",
        DataType::UInt8,
        ReadOnly,
        Some((2.0, 12.0)),
        (1, 0),
        "3.4.4/Table 3-5, p.3-35"
    ),
    entry!(
        91,
        12,
        "fst_capacity",
        DataType::UInt8,
        ReadOnly,
        Some((6.0, 6.0)),
        (1, 0),
        "3.4.4/Table 3-5, p.3-35"
    ),
    entry!(
        91,
        13,
        "event_index",
        DataType::UInt16,
        ReadOnly,
        Some((0.0, 449.0)),
        (1, 0),
        "3.4.4/Table 3-5, p.3-35"
    ),
    entry!(
        91,
        14,
        "alarm_index",
        DataType::UInt16,
        ReadOnly,
        Some((0.0, 449.0)),
        (1, 0),
        "3.4.4/Table 3-5, p.3-35"
    ),
    entry!(
        91,
        22,
        "configuration_write_complete",
        DataType::UInt8,
        ReadOnly,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.4/Table 3-5, p.3-35"
    ),
    entry!(
        91,
        23,
        "cpu_loading",
        DataType::Float,
        ReadOnly,
        Some((0.0, 100.0)),
        (1, 0),
        "3.4.4/Table 3-5, p.3-35"
    ),
    entry!(
        91,
        37,
        "boot_identification",
        DataType::Ascii(20),
        ReadOnly,
        None,
        (1, 0),
        "3.4.4/Table 3-5, p.3-37"
    ),
    entry!(
        91,
        38,
        "boot_build_time",
        DataType::Ascii(20),
        ReadOnly,
        None,
        (1, 0),
        "3.4.4/Table 3-5, p.3-37"
    ),
    entry!(
        91,
        43,
        "initialization_count",
        DataType::UInt16,
        ReadWrite,
        Some((0.0, 65535.0)),
        (1, 20),
        "3.4.4/Table 3-5, p.3-37"
    ),
    entry!(
        91,
        44,
        "warm_start_count",
        DataType::UInt16,
        ReadWrite,
        Some((0.0, 65535.0)),
        (1, 20),
        "3.4.4/Table 3-5, p.3-37"
    ),
    entry!(
        91,
        45,
        "cold_start_count",
        DataType::UInt16,
        ReadWrite,
        Some((0.0, 65535.0)),
        (1, 20),
        "3.4.4/Table 3-5, p.3-37"
    ),
    entry!(
        91,
        46,
        "power_cycle_count",
        DataType::UInt16,
        ReadWrite,
        Some((0.0, 65535.0)),
        (1, 20),
        "3.4.4/Table 3-5, p.3-37"
    ),
    entry!(
        91,
        47,
        "last_power_down",
        DataType::Time,
        ReadOnly,
        None,
        (1, 30),
        "3.4.4/Table 3-5, p.3-37"
    ),
    entry!(
        91,
        48,
        "last_power_up",
        DataType::Time,
        ReadOnly,
        None,
        (1, 30),
        "3.4.4/Table 3-5, p.3-37"
    ),
    entry!(
        95,
        0,
        "port_tag",
        DataType::Ascii(10),
        ReadWrite,
        None,
        (1, 0),
        "3.4.6/Table 3-7, p.3-44"
    ),
    entry!(
        95,
        1,
        "port_baud_generator",
        DataType::UInt16,
        ReadWrite,
        Some((0.0, 3.0)),
        (1, 0),
        "3.4.6/Table 3-7, p.3-44"
    ),
    entry!(
        95,
        2,
        "port_stop_bits",
        DataType::UInt8,
        ReadWrite,
        Some((1.0, 2.0)),
        (1, 0),
        "3.4.6/Table 3-7, p.3-44"
    ),
    entry!(
        95,
        3,
        "port_data_bits",
        DataType::UInt8,
        ReadWrite,
        Some((7.0, 8.0)),
        (1, 0),
        "3.4.6/Table 3-7, p.3-44"
    ),
    entry!(
        95,
        4,
        "port_parity",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 2.0)),
        (1, 0),
        "3.4.6/Table 3-7, p.3-44"
    ),
    entry!(
        95,
        5,
        "port_module_type",
        DataType::UInt8,
        ReadOnly,
        None,
        (1, 0),
        "3.4.6/Table 3-7, p.3-45"
    ),
    entry!(
        95,
        6,
        "port_store_forward",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.6/Table 3-7, p.3-45"
    ),
    entry!(
        95,
        7,
        "port_key_on_delay",
        DataType::Float,
        ReadWrite,
        Some((0.0, f32::MAX as f64)),
        (1, 0),
        "3.4.6/Table 3-7, p.3-45"
    ),
    entry!(
        95,
        8,
        "port_key_off_delay",
        DataType::Float,
        ReadWrite,
        Some((0.0, f32::MAX as f64)),
        (1, 0),
        "3.4.6/Table 3-7, p.3-45"
    ),
    entry!(
        95,
        9,
        "port_modem_status",
        DataType::UInt8,
        ReadOnly,
        Some((0.0, 255.0)),
        (1, 0),
        "3.4.6/Table 3-7, p.3-45"
    ),
    entry!(
        95,
        36,
        "port_receive_count",
        DataType::UInt16,
        ReadWrite,
        Some((0.0, 65535.0)),
        (1, 0),
        "3.4.6/Table 3-7, p.3-47"
    ),
    entry!(
        95,
        37,
        "port_last_message",
        DataType::Time,
        ReadOnly,
        None,
        (1, 0),
        "3.4.6/Table 3-7, p.3-47"
    ),
    entry!(
        95,
        42,
        "port_transmit_count",
        DataType::UInt16,
        ReadWrite,
        Some((0.0, 65535.0)),
        (1, 0),
        "3.4.6/Table 3-7, p.3-47"
    ),
    entry!(
        95,
        44,
        "port_security_status",
        DataType::UInt8,
        ReadWrite,
        None,
        (1, 0),
        "3.4.6/Table 3-7, p.3-48"
    ),
    entry!(
        139,
        0,
        "module_type",
        DataType::UInt8,
        ReadOnly,
        None,
        (2, 2),
        "3.4.50/Table 3-51, p.3-210"
    ),
    entry!(
        139,
        1,
        "module_mode",
        DataType::UInt8,
        ReadOnly,
        None,
        (2, 2),
        "3.4.50/Table 3-51, p.3-210"
    ),
    entry!(
        139,
        2,
        "module_health",
        DataType::UInt8,
        ReadOnly,
        Some((0.0, 2.0)),
        (2, 2),
        "3.4.50/Table 3-51, p.3-210"
    ),
    entry!(
        139,
        3,
        "module_boot_version",
        DataType::Ascii(10),
        ReadOnly,
        None,
        (2, 2),
        "3.4.50/Table 3-51, p.3-210"
    ),
    entry!(
        139,
        4,
        "module_boot_part",
        DataType::Ascii(20),
        ReadOnly,
        None,
        (2, 2),
        "3.4.50/Table 3-51, p.3-210"
    ),
    entry!(
        139,
        8,
        "module_flash_build",
        DataType::Ascii(20),
        ReadOnly,
        None,
        (2, 2),
        "3.4.50/Table 3-51, p.3-211"
    ),
    entry!(
        139,
        9,
        "module_specific_data",
        DataType::Ascii(20),
        ReadOnly,
        None,
        (2, 2),
        "3.4.50/Table 3-51, p.3-211"
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
        101,
        2,
        "di_filter",
        DataType::Float,
        ReadWrite,
        Some((0.0, 43200.0)),
        (1, 0),
        "3.4.12/Table 3-13, p.3-61"
    ),
    entry!(
        101,
        4,
        "di_invert",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.12/Table 3-13, p.3-61"
    ),
    entry!(
        101,
        5,
        "di_latch",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.12/Table 3-13, p.3-61"
    ),
    entry!(
        101,
        7,
        "di_on_time",
        DataType::Float,
        ReadWrite,
        Some((0.0, 1000000.0)),
        (1, 0),
        "3.4.12/Table 3-13, p.3-61"
    ),
    entry!(
        101,
        8,
        "di_off_time",
        DataType::Float,
        ReadWrite,
        Some((0.0, 1000000.0)),
        (1, 0),
        "3.4.12/Table 3-13, p.3-61"
    ),
    entry!(
        101,
        9,
        "di_alarming",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.12/Table 3-13, p.3-61"
    ),
    entry!(
        101,
        10,
        "di_srbx_clear",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.12/Table 3-13, p.3-62"
    ),
    entry!(
        101,
        11,
        "di_srbx_set",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.12/Table 3-13, p.3-62"
    ),
    entry!(
        101,
        12,
        "di_alarm_bits",
        DataType::Binary,
        ReadOnly,
        None,
        (1, 0),
        "3.4.12/Table 3-13, p.3-62"
    ),
    entry!(
        101,
        13,
        "di_scan_period",
        DataType::Float,
        ReadWrite,
        Some((0.004, 43200.0)),
        (1, 0),
        "3.4.12/Table 3-13, p.3-62"
    ),
    entry!(
        101,
        14,
        "di_scan_time",
        DataType::Float,
        ReadOnly,
        Some((0.0, 43200.0)),
        (1, 0),
        "3.4.12/Table 3-13, p.3-62"
    ),
    entry!(
        101,
        15,
        "di_physical_state",
        DataType::UInt8,
        ReadOnly,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.12/Table 3-13, p.3-62"
    ),
    entry!(
        103,
        5,
        "ai_filter",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 99.0)),
        (1, 0),
        "3.4.14/Table 3-15, p.3-66"
    ),
    entry!(
        103,
        6,
        "ai_averaging",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.14/Table 3-15, p.3-66"
    ),
    entry!(
        103,
        8,
        "ai_zero_raw",
        DataType::UInt16,
        ReadWrite,
        Some((0.0, 65535.0)),
        (1, 0),
        "3.4.14/Table 3-15, p.3-66"
    ),
    entry!(
        103,
        9,
        "ai_midpoint_1_raw",
        DataType::UInt16,
        ReadWrite,
        Some((0.0, 65535.0)),
        (1, 0),
        "3.4.14/Table 3-15, p.3-66"
    ),
    entry!(
        103,
        10,
        "ai_midpoint_2_raw",
        DataType::UInt16,
        ReadWrite,
        Some((0.0, 65535.0)),
        (1, 0),
        "3.4.14/Table 3-15, p.3-67"
    ),
    entry!(
        103,
        11,
        "ai_midpoint_3_raw",
        DataType::UInt16,
        ReadWrite,
        Some((0.0, 65535.0)),
        (1, 0),
        "3.4.14/Table 3-15, p.3-67"
    ),
    entry!(
        103,
        12,
        "ai_span_raw",
        DataType::UInt16,
        ReadWrite,
        Some((0.0, 65535.0)),
        (1, 0),
        "3.4.14/Table 3-15, p.3-67"
    ),
    entry!(
        103,
        13,
        "ai_zero_eu",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.14/Table 3-15, p.3-67"
    ),
    entry!(
        103,
        14,
        "ai_midpoint_1_eu",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.14/Table 3-15, p.3-67"
    ),
    entry!(
        103,
        15,
        "ai_midpoint_2_eu",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.14/Table 3-15, p.3-67"
    ),
    entry!(
        103,
        16,
        "ai_midpoint_3_eu",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.14/Table 3-15, p.3-67"
    ),
    entry!(
        103,
        17,
        "ai_span_eu",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.14/Table 3-15, p.3-67"
    ),
    entry!(
        103,
        18,
        "ai_offset",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.14/Table 3-15, p.3-67"
    ),
    entry!(
        103,
        19,
        "ai_calibration_set",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.14/Table 3-15, p.3-67"
    ),
    entry!(
        103,
        20,
        "ai_calibration_value",
        DataType::Float,
        ReadOnly,
        None,
        (1, 0),
        "3.4.14/Table 3-15, p.3-67"
    ),
    entry!(
        103,
        23,
        "ai_low_low_limit",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.14/Table 3-15, p.3-67"
    ),
    entry!(
        103,
        24,
        "ai_low_limit",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.14/Table 3-15, p.3-67"
    ),
    entry!(
        103,
        25,
        "ai_high_limit",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.14/Table 3-15, p.3-67"
    ),
    entry!(
        103,
        26,
        "ai_high_high_limit",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.14/Table 3-15, p.3-67"
    ),
    entry!(
        103,
        27,
        "ai_rate_limit",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.14/Table 3-15, p.3-68"
    ),
    entry!(
        103,
        28,
        "ai_deadband",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.14/Table 3-15, p.3-68"
    ),
    entry!(
        103,
        22,
        "ai_clipping",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.14/Table 3-15, p.3-67"
    ),
    entry!(
        103,
        29,
        "ai_alarming",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.14/Table 3-15, p.3-68"
    ),
    entry!(
        103,
        30,
        "ai_srbx_clear",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.14/Table 3-15, p.3-68"
    ),
    entry!(
        103,
        31,
        "ai_srbx_set",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.14/Table 3-15, p.3-68"
    ),
    entry!(
        103,
        33,
        "ai_calibration_timer",
        DataType::Float,
        ReadOnly,
        Some((0.0, 3600.0)),
        (1, 0),
        "3.4.14/Table 3-15, p.3-69"
    ),
    entry!(
        103,
        34,
        "ai_calibration_mode",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 4.0)),
        (1, 0),
        "3.4.14/Table 3-15, p.3-69"
    ),
    entry!(
        103,
        35,
        "ai_calibration_type",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 6.0)),
        (1, 0),
        "3.4.14/Table 3-15, p.3-69"
    ),
    entry!(
        103,
        36,
        "ai_failsafe_mode",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (3, 0),
        "3.4.14/Table 3-15, p.3-69"
    ),
    entry!(
        103,
        37,
        "ai_failsafe_value",
        DataType::Float,
        ReadWrite,
        None,
        (3, 0),
        "3.4.14/Table 3-15, p.3-69"
    ),
    entry!(
        103,
        38,
        "ai_module_type",
        DataType::UInt8,
        ReadOnly,
        Some((0.0, 1.0)),
        (3, 0),
        "3.4.14/Table 3-15, p.3-69"
    ),
    entry!(
        103,
        39,
        "ai_milliamps",
        DataType::Float,
        ReadOnly,
        Some((4.0, 20.0)),
        (3, 10),
        "3.4.14/Table 3-15, p.3-69"
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
    // Tables 3-14 and 3-16: every output read field; no write codecs.
    entry!(
        102,
        0,
        "do_tag",
        DataType::Ascii(10),
        ReadWrite,
        None,
        (1, 0),
        "3.4.13/Table 3-14, p.3-63"
    ),
    entry!(
        102,
        1,
        "do_units_tag",
        DataType::Ascii(10),
        ReadWrite,
        None,
        (1, 0),
        "3.4.13/Table 3-14, p.3-63"
    ),
    entry!(
        102,
        2,
        "do_scanning_mode",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 2.0)),
        (1, 0),
        "3.4.13/Table 3-14, p.3-63"
    ),
    entry!(
        102,
        3,
        "do_alarming",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.13/Table 3-14, p.3-63"
    ),
    entry!(
        102,
        4,
        "do_srbx_on_clear",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.13/Table 3-14, p.3-63"
    ),
    entry!(
        102,
        5,
        "do_srbx_on_set",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.13/Table 3-14, p.3-63"
    ),
    entry!(
        102,
        6,
        "do_alarm_code",
        DataType::Binary,
        ReadOnly,
        None,
        (1, 0),
        "3.4.13/Table 3-14, p.3-63"
    ),
    entry!(
        102,
        7,
        "do_failsafe_on_reset",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.13/Table 3-14, p.3-64"
    ),
    entry!(
        102,
        8,
        "do_auto_output",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.13/Table 3-14, p.3-64"
    ),
    entry!(
        102,
        9,
        "do_accumulated_value",
        DataType::UInt32,
        ReadWrite,
        Some((0.0, 4294967295.0)),
        (1, 0),
        "3.4.13/Table 3-14, p.3-64"
    ),
    entry!(
        102,
        10,
        "do_momentary_mode",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.13/Table 3-14, p.3-64"
    ),
    entry!(
        102,
        11,
        "do_momentary_active",
        DataType::UInt8,
        ReadOnly,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.13/Table 3-14, p.3-64"
    ),
    entry!(
        102,
        12,
        "do_toggle_mode",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.13/Table 3-14, p.3-64"
    ),
    entry!(
        102,
        13,
        "do_timed_output_mode",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.13/Table 3-14, p.3-64"
    ),
    entry!(
        102,
        14,
        "do_time_on",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.13/Table 3-14, p.3-64"
    ),
    entry!(
        102,
        15,
        "do_cycle_time",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.13/Table 3-14, p.3-65"
    ),
    entry!(
        102,
        16,
        "do_low_reading_time",
        DataType::Float,
        ReadWrite,
        Some((0.0, 43200.0)),
        (1, 0),
        "3.4.13/Table 3-14, p.3-65"
    ),
    entry!(
        102,
        17,
        "do_high_reading_time",
        DataType::Float,
        ReadWrite,
        Some((0.0, 43200.0)),
        (1, 0),
        "3.4.13/Table 3-14, p.3-65"
    ),
    entry!(
        102,
        18,
        "do_low_reading_eu",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.13/Table 3-14, p.3-65"
    ),
    entry!(
        102,
        19,
        "do_high_reading_eu",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.13/Table 3-14, p.3-65"
    ),
    entry!(
        102,
        20,
        "do_eu_value",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.13/Table 3-14, p.3-65"
    ),
    entry!(
        102,
        21,
        "do_manual_output",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.13/Table 3-14, p.3-65"
    ),
    entry!(
        102,
        22,
        "do_failsafe_output",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.13/Table 3-14, p.3-65"
    ),
    entry!(
        102,
        23,
        "do_max_scan_period",
        DataType::Float,
        ReadOnly,
        Some((0.0, f32::MAX as f64)),
        (1, 0),
        "3.4.13/Table 3-14, p.3-65"
    ),
    entry!(
        102,
        24,
        "do_physical_output",
        DataType::UInt8,
        ReadOnly,
        Some((0.0, 1.0)),
        (1, 10),
        "3.4.13/Table 3-14, p.3-65"
    ),
    entry!(
        102,
        25,
        "do_type",
        DataType::UInt8,
        ReadOnly,
        None,
        (1, 10),
        "3.4.13/Table 3-14, p.3-65"
    ),
    entry!(
        102,
        26,
        "do_invert_output_mode",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (3, 1),
        "3.4.13/Table 3-14, p.3-65"
    ),
    entry!(
        104,
        0,
        "ao_tag",
        DataType::Ascii(10),
        ReadWrite,
        None,
        (1, 0),
        "3.4.15/Table 3-16, p.3-70"
    ),
    entry!(
        104,
        1,
        "ao_units_tag",
        DataType::Ascii(10),
        ReadWrite,
        None,
        (1, 0),
        "3.4.15/Table 3-16, p.3-70"
    ),
    entry!(
        104,
        2,
        "ao_scanning_mode",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 2.0)),
        (1, 0),
        "3.4.15/Table 3-16, p.3-70"
    ),
    entry!(
        104,
        3,
        "ao_alarming",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.15/Table 3-16, p.3-70"
    ),
    entry!(
        104,
        4,
        "ao_srbx_on_clear",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.15/Table 3-16, p.3-70"
    ),
    entry!(
        104,
        5,
        "ao_srbx_on_set",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.15/Table 3-16, p.3-70"
    ),
    entry!(
        104,
        6,
        "ao_alarm_code",
        DataType::Binary,
        ReadOnly,
        None,
        (1, 0),
        "3.4.15/Table 3-16, p.3-70"
    ),
    entry!(
        104,
        7,
        "ao_failsafe_on_reset",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.15/Table 3-16, p.3-71"
    ),
    entry!(
        104,
        8,
        "ao_zero_raw",
        DataType::UInt16,
        ReadWrite,
        Some((0.0, 65535.0)),
        (1, 0),
        "3.4.15/Table 3-16, p.3-71"
    ),
    entry!(
        104,
        9,
        "ao_span_raw",
        DataType::UInt16,
        ReadWrite,
        Some((0.0, 65535.0)),
        (1, 0),
        "3.4.15/Table 3-16, p.3-71"
    ),
    entry!(
        104,
        10,
        "ao_zero_eu",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.15/Table 3-16, p.3-71"
    ),
    entry!(
        104,
        11,
        "ao_span_eu",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.15/Table 3-16, p.3-71"
    ),
    entry!(
        104,
        12,
        "ao_auto_value",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.15/Table 3-16, p.3-71"
    ),
    entry!(
        104,
        13,
        "ao_raw_da_output",
        DataType::UInt16,
        ReadOnly,
        Some((0.0, 65535.0)),
        (1, 0),
        "3.4.15/Table 3-16, p.3-71"
    ),
    entry!(
        104,
        14,
        "ao_manual_value",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.15/Table 3-16, p.3-71"
    ),
    entry!(
        104,
        15,
        "ao_failsafe_value",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.15/Table 3-16, p.3-71"
    ),
    entry!(
        104,
        16,
        "ao_physical_value",
        DataType::Float,
        ReadOnly,
        None,
        (1, 0),
        "3.4.15/Table 3-16, p.3-71"
    ),
    // Table 3-17: every pulse-input read field.
    entry!(
        105,
        0,
        "pi_tag",
        DataType::Ascii(10),
        ReadWrite,
        None,
        (1, 0),
        "3.4.16/Table 3-17, p.3-72"
    ),
    entry!(
        105,
        1,
        "pi_units_tag",
        DataType::Ascii(10),
        ReadWrite,
        None,
        (1, 0),
        "3.4.16/Table 3-17, p.3-72"
    ),
    entry!(
        105,
        2,
        "pi_scanning",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.16/Table 3-17, p.3-72"
    ),
    entry!(
        105,
        3,
        "pi_scan_period",
        DataType::Float,
        ReadWrite,
        Some((0.05, 43200.0)),
        (1, 0),
        "3.4.16/Table 3-17, p.3-72"
    ),
    entry!(
        105,
        4,
        "pi_accumulated_value",
        DataType::UInt32,
        ReadWrite,
        Some((0.0, 16000000.0)),
        (1, 0),
        "3.4.16/Table 3-17, p.3-72"
    ),
    entry!(
        105,
        5,
        "pi_contract_hour",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 23.0)),
        (1, 0),
        "3.4.16/Table 3-17, p.3-72"
    ),
    entry!(
        105,
        6,
        "pi_pulses_for_day",
        DataType::UInt32,
        ReadOnly,
        Some((0.0, 4294967295.0)),
        (1, 0),
        "3.4.16/Table 3-17, p.3-72"
    ),
    entry!(
        105,
        7,
        "pi_current_rate_period",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 3.0)),
        (1, 0),
        "3.4.16/Table 3-17, p.3-72"
    ),
    entry!(
        105,
        8,
        "pi_conversion",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.16/Table 3-17, p.3-72"
    ),
    entry!(
        105,
        9,
        "pi_conversion_value",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.16/Table 3-17, p.3-72"
    ),
    entry!(
        105,
        10,
        "pi_current_rate",
        DataType::Float,
        ReadOnly,
        None,
        (1, 0),
        "3.4.16/Table 3-17, p.3-72"
    ),
    entry!(
        105,
        11,
        "pi_eu_value_mode",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 2.0)),
        (1, 0),
        "3.4.16/Table 3-17, p.3-73"
    ),
    entry!(
        105,
        12,
        "pi_rollover_maximum",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.16/Table 3-17, p.3-73"
    ),
    entry!(
        105,
        13,
        "pi_eu_value",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.16/Table 3-17, p.3-73"
    ),
    entry!(
        105,
        14,
        "pi_low_low_alarm_eu",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.16/Table 3-17, p.3-73"
    ),
    entry!(
        105,
        15,
        "pi_low_alarm_eu",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.16/Table 3-17, p.3-73"
    ),
    entry!(
        105,
        16,
        "pi_high_alarm_eu",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.16/Table 3-17, p.3-73"
    ),
    entry!(
        105,
        17,
        "pi_high_high_alarm_eu",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.16/Table 3-17, p.3-73"
    ),
    entry!(
        105,
        18,
        "pi_rate_alarm_eu",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.16/Table 3-17, p.3-73"
    ),
    entry!(
        105,
        19,
        "pi_alarm_deadband",
        DataType::Float,
        ReadWrite,
        None,
        (1, 0),
        "3.4.16/Table 3-17, p.3-73"
    ),
    entry!(
        105,
        20,
        "pi_alarming",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.16/Table 3-17, p.3-73"
    ),
    entry!(
        105,
        21,
        "pi_srbx_on_clear",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.16/Table 3-17, p.3-73"
    ),
    entry!(
        105,
        22,
        "pi_srbx_on_set",
        DataType::UInt8,
        ReadWrite,
        Some((0.0, 1.0)),
        (1, 0),
        "3.4.16/Table 3-17, p.3-73"
    ),
    entry!(
        105,
        23,
        "pi_alarm_code",
        DataType::Binary,
        ReadOnly,
        None,
        (1, 0),
        "3.4.16/Table 3-17, p.3-73"
    ),
    entry!(
        105,
        24,
        "pi_today_total",
        DataType::Float,
        ReadOnly,
        None,
        (1, 0),
        "3.4.16/Table 3-17, p.3-74"
    ),
    entry!(
        105,
        25,
        "pi_yesterday_total",
        DataType::Float,
        ReadOnly,
        None,
        (1, 0),
        "3.4.16/Table 3-17, p.3-74"
    ),
    entry!(
        105,
        26,
        "pi_corrected_accumulation",
        DataType::Float,
        ReadOnly,
        None,
        (1, 0),
        "3.4.16/Table 3-17, p.3-74"
    ),
    entry!(
        105,
        27,
        "pi_frequency",
        DataType::Float,
        ReadOnly,
        Some((0.0, f32::MAX as f64)),
        (1, 0),
        "3.4.16/Table 3-17, p.3-74"
    ),
];
