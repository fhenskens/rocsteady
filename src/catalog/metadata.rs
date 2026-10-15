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
        _ => &[],
    };
    metadata.units = match (point_type, parameter) {
        (101, 2 | 7 | 8 | 13 | 14) | (103, 3 | 4 | 33) => Some("s"),
        (103, 5) => Some("%"),
        (103, 39) => Some("mA"),
        (102, 14..=17 | 23) | (105, 3) => Some("s"),
        (105, 4 | 6) => Some("pulses"),
        (105, 27) => Some("pulses/s"),
        _ => None,
    };
    match (point_type, parameter) {
        (101, 12) => {
            metadata.bits = DI_ALARMS;
            metadata.unused_bits = 0x5f;
        }
        // Tables 3-15/3-17 define the same seven alarm bits and unused bit 5.
        (103, 32) | (105, 23) => {
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
        _ => {}
    }
    metadata
}
