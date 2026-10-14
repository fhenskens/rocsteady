//! Optional display metadata for the audited input tables. Raw values stay intact.

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
const CALIBRATION_CAVEAT: &str = "Table labels R/W, but the row note suggests R/O for external ROC Plus; write access is unresolved";

pub(super) const fn for_parameter(point_type: u8, parameter: u8) -> CatalogMetadata {
    let mut metadata = CatalogMetadata::UNSPECIFIED;
    metadata.enumeration = match (point_type, parameter) {
        (101, 1 | 4 | 5 | 9..=11) | (103, 2 | 6 | 22 | 29..=31 | 36) => ENABLED,
        (101, 3 | 15) => STATE,
        (103, 34) => CALIBRATION_MODE,
        (103, 35) => CALIBRATION_TYPE,
        (103, 38) => AI_TYPE,
        _ => &[],
    };
    metadata.units = match (point_type, parameter) {
        (101, 2 | 7 | 8 | 13 | 14) | (103, 3 | 4 | 33) => Some("s"),
        (103, 5) => Some("%"),
        (103, 39) => Some("mA"),
        _ => None,
    };
    match (point_type, parameter) {
        (101, 12) => {
            metadata.bits = DI_ALARMS;
            metadata.unused_bits = 0x5f;
        }
        (103, 32) => {
            metadata.bits = AI_ALARMS;
            metadata.unused_bits = 0x20;
        }
        (103, 19 | 34 | 35) => {
            metadata.write_caveat = Some(CALIBRATION_CAVEAT);
        }
        _ => {}
    }
    metadata
}
