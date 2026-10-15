//! Optional display metadata for the audited I/O tables. Raw values stay intact.

/// A documented byte value. Unknown values have no label and remain readable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumMeaning {
    pub value: u8,
    pub key: &'static str,
}

/// Meaning of one set bit in a byte-valued parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitMeaning {
    pub mask: u8,
    pub key: &'static str,
}

/// Optional catalog annotations, separate from schema read planning.
/// Empty collections and None mean unspecified, not an exhaustive value constraint.
/// These annotations do not establish permission to write a parameter.
///
/// ```
/// use rocsteady::Roc800Catalog;
/// let alarms = Roc800Catalog::entry(101, 12).unwrap().metadata;
/// let raw = 0xa1;
/// let active: Vec<_> = alarms.active_bits(raw).map(|bit| bit.key).collect();
/// assert_eq!(active, ["status_on", "scanning_disabled"]);
/// assert_eq!(alarms.uninterpreted_bits(raw), 0x01); // Retain the unused bit.
/// assert!(Roc800Catalog::entry(101, 3).unwrap().metadata.enum_meaning(255).is_none());
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CatalogMetadata {
    /// Fixed units only; engineering units supplied by a device tag stay unspecified.
    pub units: Option<&'static str>,
    pub enumeration: &'static [EnumMeaning],
    pub bits: &'static [BitMeaning],
    /// Bits documented as unused; their raw values are still preserved.
    pub unused_bits: u8,
    /// Source conflict or condition that must be resolved before implementing writes.
    pub write_caveat: Option<&'static str>,
}
impl CatalogMetadata {
    pub const UNSPECIFIED: Self = Self {
        units: None,
        enumeration: &[],
        bits: &[],
        unused_bits: 0,
        write_caveat: None,
    };
    pub fn enum_meaning(self, raw: u8) -> Option<&'static EnumMeaning> {
        self.enumeration.iter().find(|meaning| meaning.value == raw)
    }
    pub fn active_bits(self, raw: u8) -> impl Iterator<Item = &'static BitMeaning> {
        self.bits
            .iter()
            .filter(move |meaning| raw & meaning.mask != 0)
    }
    /// Set bits with no supplied meaning, including documented unused bits.
    pub fn uninterpreted_bits(self, raw: u8) -> u8 {
        raw & !self
            .bits
            .iter()
            .fold(0, |mask, meaning| mask | meaning.mask)
    }
}

const ENABLED: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "disabled",
    },
    EnumMeaning {
        value: 1,
        key: "enabled",
    },
];
const STATE: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "off",
    },
    EnumMeaning {
        value: 1,
        key: "on",
    },
];
const CALIBRATION_MODE: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "current",
    },
    EnumMeaning {
        value: 1,
        key: "start",
    },
    EnumMeaning {
        value: 2,
        key: "calibrate",
    },
    EnumMeaning {
        value: 3,
        key: "restore_previous",
    },
    EnumMeaning {
        value: 4,
        key: "stop",
    },
];
const CALIBRATION_TYPE: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "none",
    },
    EnumMeaning {
        value: 1,
        key: "zero",
    },
    EnumMeaning {
        value: 2,
        key: "span",
    },
    EnumMeaning {
        value: 3,
        key: "midpoint_1",
    },
    EnumMeaning {
        value: 4,
        key: "midpoint_2",
    },
    EnumMeaning {
        value: 5,
        key: "midpoint_3",
    },
    EnumMeaning {
        value: 6,
        key: "offset",
    },
];
const AI_TYPE: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "12_bit",
    },
    EnumMeaning {
        value: 1,
        key: "16_bit",
    },
];
const DI_ALARMS: &[BitMeaning] = &[
    BitMeaning {
        mask: 0x20,
        key: "status_on",
    },
    BitMeaning {
        mask: 0x80,
        key: "scanning_disabled",
    },
];
const AI_ALARMS: &[BitMeaning] = &[
    BitMeaning {
        mask: 0x01,
        key: "low",
    },
    BitMeaning {
        mask: 0x02,
        key: "low_low",
    },
    BitMeaning {
        mask: 0x04,
        key: "high",
    },
    BitMeaning {
        mask: 0x08,
        key: "high_high",
    },
    BitMeaning {
        mask: 0x10,
        key: "rate",
    },
    BitMeaning {
        mask: 0x40,
        key: "point_fail",
    },
    BitMeaning {
        mask: 0x80,
        key: "scanning_disabled",
    },
];
const OUTPUT_SCAN: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "disabled",
    },
    EnumMeaning {
        value: 1,
        key: "automatic",
    },
    EnumMeaning {
        value: 2,
        key: "manual",
    },
];
const RESET_OUTPUT: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "last_value",
    },
    EnumMeaning {
        value: 1,
        key: "failsafe_value",
    },
];
const DO_TYPE: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "relay",
    },
    EnumMeaning {
        value: 1,
        key: "solid_state",
    },
    EnumMeaning {
        value: 3,
        key: "relay_6_point",
    },
];
const OUTPUT_INVERT: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "normal",
    },
    EnumMeaning {
        value: 1,
        key: "inverted",
    },
];
const DO_ALARMS: &[BitMeaning] = &[
    BitMeaning {
        mask: 0x20,
        key: "scanning_manual",
    },
    BitMeaning {
        mask: 0x80,
        key: "scanning_disabled",
    },
];
const AO_ALARMS: &[BitMeaning] = &[
    BitMeaning {
        mask: 0x20,
        key: "scanning_manual",
    },
    BitMeaning {
        mask: 0x40,
        key: "point_fail",
    },
    BitMeaning {
        mask: 0x80,
        key: "scanning_disabled",
    },
];
// Table 3-17, pp.3-72–74. Rate units depend on the device's units tag.
const PI_RATE_PERIOD: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "eu_per_second",
    },
    EnumMeaning {
        value: 1,
        key: "eu_per_minute",
    },
    EnumMeaning {
        value: 2,
        key: "eu_per_hour",
    },
    EnumMeaning {
        value: 3,
        key: "eu_per_day",
    },
];
const PI_CONVERSION: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "eu_per_pulse",
    },
    EnumMeaning {
        value: 1,
        key: "pulses_per_eu",
    },
];
const PI_EU_MODE: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "rate",
    },
    EnumMeaning {
        value: 1,
        key: "accumulator_maximum_rollover",
    },
    EnumMeaning {
        value: 2,
        key: "accumulator_entered_rollover",
    },
];
// Tables 3-18/19: labels preserve all documented codes; unknown bytes remain raw.
const RTD_ALPHA: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "alpha_0_00385",
    },
    EnumMeaning {
        value: 1,
        key: "alpha_0_00392",
    },
];
const TEMPERATURE_UNITS: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "fahrenheit",
    },
    EnumMeaning {
        value: 1,
        key: "celsius",
    },
    EnumMeaning {
        value: 2,
        key: "kelvin",
    },
    EnumMeaning {
        value: 3,
        key: "rankine",
    },
];
const RTD_UNITS: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "fahrenheit",
    },
    EnumMeaning {
        value: 1,
        key: "celsius",
    },
    EnumMeaning {
        value: 2,
        key: "kelvin",
    },
    EnumMeaning {
        value: 3,
        key: "rankine",
    },
    EnumMeaning {
        value: 4,
        key: "ohms",
    },
];
const TC_TYPE: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "type_j",
    },
    EnumMeaning {
        value: 1,
        key: "type_k",
    },
    EnumMeaning {
        value: 2,
        key: "type_b",
    },
    EnumMeaning {
        value: 3,
        key: "type_e",
    },
    EnumMeaning {
        value: 4,
        key: "type_r",
    },
    EnumMeaning {
        value: 5,
        key: "type_s",
    },
    EnumMeaning {
        value: 6,
        key: "type_t",
    },
    EnumMeaning {
        value: 8,
        key: "type_10",
    },
    EnumMeaning {
        value: 9,
        key: "type_c",
    },
    EnumMeaning {
        value: 10,
        key: "type_n",
    },
];
const RTD_CALIBRATION_TYPE: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "nothing",
    },
    EnumMeaning {
        value: 1,
        key: "zero",
    },
    EnumMeaning {
        value: 2,
        key: "span",
    },
    EnumMeaning {
        value: 3,
        key: "midpoint_1",
    },
    EnumMeaning {
        value: 4,
        key: "midpoint_2",
    },
    EnumMeaning {
        value: 5,
        key: "midpoint_3",
    },
    EnumMeaning {
        value: 6,
        key: "unused",
    },
];
// Table 3-20: source codes and masks, preserving gaps and all uninterpreted bits.
const MVS_POLL: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "off_scan",
    },
    EnumMeaning {
        value: 1,
        key: "normal",
    },
    EnumMeaning {
        value: 2,
        key: "input_freeze",
    },
    EnumMeaning {
        value: 4,
        key: "configuration",
    },
    EnumMeaning {
        value: 5,
        key: "set_tag_address",
    },
];
const MVS_UNITS: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "english",
    },
    EnumMeaning {
        value: 1,
        key: "metric",
    },
];
const MVS_WATER_REFERENCE: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "water_60_f",
    },
    EnumMeaning {
        value: 1,
        key: "water_68_f",
    },
];
const MVS_TAP: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "downstream",
    },
    EnumMeaning {
        value: 1,
        key: "upstream",
    },
];
const MVS_FAILURE: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "retain_last",
    },
    EnumMeaning {
        value: 1,
        key: "use_fault_value",
    },
];
const MVS_CALIBRATION_COMMAND: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "no_action",
    },
    EnumMeaning {
        value: 1,
        key: "calibrate_dp",
    },
    EnumMeaning {
        value: 2,
        key: "calibrate_sp",
    },
    EnumMeaning {
        value: 3,
        key: "calibrate_temperature",
    },
    EnumMeaning {
        value: 6,
        key: "save_calibration",
    },
    EnumMeaning {
        value: 7,
        key: "set_defaults",
    },
];
const MVS_CALIBRATION_TYPE: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "none",
    },
    EnumMeaning {
        value: 1,
        key: "zero",
    },
    EnumMeaning {
        value: 2,
        key: "span",
    },
    EnumMeaning {
        value: 3,
        key: "midpoint_1",
    },
    EnumMeaning {
        value: 4,
        key: "midpoint_2",
    },
    EnumMeaning {
        value: 5,
        key: "midpoint_3",
    },
    EnumMeaning {
        value: 6,
        key: "sensor_setup",
    },
    EnumMeaning {
        value: 7,
        key: "sensor_restore",
    },
];
const MVS_SENSOR_TYPE: &[EnumMeaning] = &[
    EnumMeaning {
        value: 0,
        key: "undefined",
    },
    EnumMeaning {
        value: 1,
        key: "4088_a",
    },
    EnumMeaning {
        value: 2,
        key: "4088_b",
    },
    EnumMeaning {
        value: 3,
        key: "3095",
    },
    EnumMeaning {
        value: 4,
        key: "mvs205",
    },
];
const MVS_SENSOR_ALARMS: &[BitMeaning] = &[
    BitMeaning {
        mask: 16,
        key: "input_freeze",
    },
    BitMeaning {
        mask: 32,
        key: "sensor_communication_failure",
    },
    BitMeaning {
        mask: 64,
        key: "rs485_communication_failure",
    },
    BitMeaning {
        mask: 128,
        key: "off_scan",
    },
];
const MVS_RANGE_BITS: &[BitMeaning] = &[
    BitMeaning {
        mask: 1,
        key: "dp_below_zero",
    },
    BitMeaning {
        mask: 2,
        key: "sp_below_zero",
    },
    BitMeaning {
        mask: 4,
        key: "temperature_below_zero",
    },
    BitMeaning {
        mask: 8,
        key: "dp_above_span",
    },
    BitMeaning {
        mask: 16,
        key: "sp_above_span",
    },
    BitMeaning {
        mask: 32,
        key: "temperature_above_span",
    },
];
const MVS_VALUE_ALARMS: &[BitMeaning] = &[
    BitMeaning {
        mask: 1,
        key: "low",
    },
    BitMeaning {
        mask: 4,
        key: "high",
    },
    BitMeaning {
        mask: 64,
        key: "point_fail",
    },
];
const CALIBRATION_CAVEAT: &str = "Table labels R/W, but the row note suggests R/O for external ROC Plus; write access is unresolved";

pub(super) const fn for_parameter(point_type: u8, parameter: u8) -> CatalogMetadata {
    let mut metadata = CatalogMetadata::UNSPECIFIED;
    metadata.enumeration = match (point_type, parameter) {
        (101, 1 | 4 | 5 | 9..=11) | (103, 2 | 6 | 22 | 29..=31 | 36) => ENABLED,
        (101, 3 | 15) | (102, 8 | 21 | 22 | 24) => STATE,
        (102 | 104, 2) => OUTPUT_SCAN,
        (102 | 104, 3..=5) | (102, 10..=13) => ENABLED,
        (102 | 104, 7) => RESET_OUTPUT,
        (102, 25) => DO_TYPE,
        (102, 26) => OUTPUT_INVERT,
        (105, 2 | 20..=22) => ENABLED,
        (105, 7) => PI_RATE_PERIOD,
        (105, 8) => PI_CONVERSION,
        (105, 11) => PI_EU_MODE,
        (103, 34) => CALIBRATION_MODE,
        (103, 35) => CALIBRATION_TYPE,
        (103, 38) => AI_TYPE,
        (106, 2 | 6 | 23 | 30..=32) | (107, 2 | 8 | 16..=18 | 21) => ENABLED,
        (106, 7) => RTD_ALPHA,
        (106, 35) => CALIBRATION_MODE,
        (106, 36) => RTD_CALIBRATION_TYPE,
        (106, 37) => RTD_UNITS,
        (107, 3) => TEMPERATURE_UNITS,
        (107, 4) => TC_TYPE,
        (108, 2) => MVS_POLL,
        (108, 3) => MVS_UNITS,
        (108, 4) => MVS_WATER_REFERENCE,
        (108, 5) => MVS_TAP,
        (108, 6) => MVS_FAILURE,
        (108, 9 | 25..=27 | 40..=42 | 55..=57 | 62 | 63) => ENABLED,
        (108, 59) => MVS_CALIBRATION_COMMAND,
        (108, 60) => MVS_CALIBRATION_TYPE,
        (108, 68) => MVS_SENSOR_TYPE,
        _ => &[],
    };
    metadata.units = match (point_type, parameter) {
        (101, 2 | 7 | 8 | 13 | 14) | (103, 3 | 4 | 33) => Some("s"),
        (103, 5) => Some("%"),
        (103, 39) => Some("mA"),
        (102, 14..=17 | 23) | (105, 3) => Some("s"),
        (105, 4 | 6) => Some("pulses"),
        (105, 27) => Some("pulses/s"),
        (106, 3 | 4 | 34) | (107, 5 | 6) => Some("s"),
        (106, 5) | (107, 7) => Some("%"),
        (108, 8) => Some("V"),
        _ => None,
    };
    match (point_type, parameter) {
        (101, 12) => {
            metadata.bits = DI_ALARMS;
            metadata.unused_bits = 0x5f;
        }
        // Tables 3-15/3-17 define the same seven alarm bits and unused bit 5.
        (103, 32) | (105, 23) | (106, 33) | (107, 19) => {
            metadata.bits = AI_ALARMS;
            metadata.unused_bits = 0x20;
        }
        (102, 6) => {
            metadata.bits = DO_ALARMS;
            metadata.unused_bits = 0x5f;
        }
        (104, 6) => {
            metadata.bits = AO_ALARMS;
            metadata.unused_bits = 0x1f;
        }
        (105, 9) => {
            metadata.write_caveat = Some(
                "Conversion value excludes zero; inclusive numeric_range cannot represent a nonzero float constraint",
            );
        }
        (102, 14) => {
            metadata.write_caveat = Some(
                "Time-on range depends on module: DO 0.002-43200 s; DOR 0.05-43200 s; no universal numeric range selected",
            );
        }
        (102, 15) => {
            metadata.write_caveat = Some(
                "Cycle time must be strictly greater than zero and at most 43200 s; inclusive numeric_range cannot represent the open lower bound",
            );
        }
        (102, 25) => {
            metadata.write_caveat = Some(
                "Printed range 0-1 conflicts with documented relay-6-point value 3; raw value and all documented labels retained",
            );
        }
        (102, 26) => {
            metadata.write_caveat = Some(
                "DO table names an ACIO channel; hardware applicability requires evidence before writes",
            );
        }
        (104, 7) => {
            metadata.write_caveat = Some(
                "Failsafe prose references nonexistent parameter 22 and calls parameter 13 EU value; read width/access are unambiguous, write behavior needs review",
            );
        }
        (103, 19 | 34 | 35) => {
            metadata.write_caveat = Some(CALIBRATION_CAVEAT);
        }
        (106, 2 | 7) => {
            metadata.write_caveat = Some(
                "Source access token R/W_LOG retained without inferring its logging procedure; temperature writes unavailable",
            )
        }
        (106, 3 | 5 | 6) => {
            metadata.write_caveat = Some(
                "Source access token R/W_CNDL retained without inferring its condition; readable width is clear; temperature writes unavailable",
            )
        }
        (106, 20) => {
            metadata.write_caveat = Some(
                "Set-value range depends on selected units and alpha; row note suggests R/O for external ROC Plus; no universal range selected",
            )
        }
        (106, 35) => {
            metadata.write_caveat = Some(
                "Calibration-mode changes are not event logged; calibration workflow and temperature writes unavailable",
            )
        }
        (106, 36) => {
            metadata.write_caveat = Some(
                "Row note suggests R/O for external ROC Plus; calibration type 6 is unused, not offset",
            )
        }
        (107, 4) => {
            metadata.write_caveat = Some(
                "Noncontiguous sensor codes exclude 7; original 5-point TC module only supports J/K; no universal numeric range selected",
            )
        }
        (107, 10 | 13..=15) => {
            metadata.write_caveat = Some(
                "Source prose copies RTD parameter references, including clipping parameter 23 absent from this table; no clipping or conversion behavior inferred",
            )
        }
        (107, 22) => {
            metadata.write_caveat = Some(
                "Source repeats the Failsafe Mode name on a FLOAT row; description identifies a failsafe value; raw read only",
            )
        }
        (108, 10) => {
            metadata.bits = MVS_SENSOR_ALARMS;
            metadata.unused_bits = 0x0f;
        }
        (108, 11) => {
            metadata.bits = MVS_RANGE_BITS;
            metadata.unused_bits = 0xc0;
        }
        (108, 28 | 43 | 58) => {
            metadata.bits = MVS_VALUE_ALARMS;
            metadata.unused_bits = 0xba;
        }
        (108, 2) => {
            metadata.write_caveat = Some(
                "Poll-mode codes are noncontiguous (0,1,2,4,5); no universal inclusive range selected",
            )
        }
        (108, 59) => {
            metadata.write_caveat = Some(
                "Printed range 0-6 conflicts with documented set-defaults code 7 and code gaps; commands remain read-only through this catalog",
            )
        }
        (108, 64) => {
            metadata.write_caveat = Some(
                "SP zero-shift prose names inches of water while other SP rows name PSI; no fixed units or conversion inferred",
            )
        }
        (108, 65) => {
            metadata.write_caveat = Some(
                "Module-type codes 0 standard MVS, 1 smart MVS, 2 3095 are stored as FLOAT; no byte coercion or numeric filter",
            )
        }
        (108, 67) => {
            metadata.write_caveat = Some(
                "4088 configuration counter is invalid for MVS205; explicit per-location 4088 assertion required before reads",
            )
        }
        _ => {}
    }
    metadata
}
