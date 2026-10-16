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
    /// Access column in the source table; metadata may record unresolved write notes.
    pub access: Access,
    pub numeric_range: Option<(f64, f64)>,
    pub introduced: (u8, u8),
    /// Optional interpretation; lookup still determines firmware/location applicability.
    pub metadata: CatalogMetadata,
    pub source: &'static str,
}
mod metadata;
pub use metadata::{BitMeaning, CatalogMetadata, EnumMeaning};
mod entries;
use entries::ENTRIES;

/// Explicit ROC800 firmware identity; I/O existence comes from a generation-bound snapshot.
///
/// PT-91/136 use logical zero, PT-95 uses port ordinals 0-5 (Ethernet is 1),
/// and PT-139 uses module ordinals 0-26. Static port/module metadata establishes
/// addressing and read widths, not installation, module mode or authorization.
/// The controller can reject a read. PT-101–109 require discovered physical positions.
pub struct Roc800Catalog {
    firmware: FirmwareVersion,
    io: Option<IoSnapshot>,
    mvs_4088: std::collections::BTreeSet<u8>,
}
impl Roc800Catalog {
    pub fn new(firmware: FirmwareVersion) -> Result<Self, Error> {
        // Conservative reviewed applicability window; future firmware requires an override.
        if firmware > FirmwareVersion::new(3, 60)? {
            return Err(Error::CapabilityUnavailable(
                "catalog firmware outside reviewed 1.00-3.60 window",
            ));
        }
        Ok(Self {
            firmware,
            io: None,
            mvs_4088: Default::default(),
        })
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
    /// Explicit caller assertion that this discovered physical PT-108 location is a
    /// 4088 A/B sensor, permitting its config-change counter (/67). This does not read
    /// /68, infer subtype from parsing, or establish persistent controller identity.
    /// The resulting catalog and its plans remain bound to the discovery generation.
    pub fn with_mvs_4088(mut self, location: u8) -> Result<Self, Error> {
        if self.firmware < FirmwareVersion::new(3, 60)? {
            return Err(Error::CapabilityUnavailable(
                "MVS 4088 counter requires firmware 3.60",
            ));
        }
        let io = self.io.as_ref().ok_or(Error::CapabilityUnavailable(
            "MVS 4088 assertion requires discovery",
        ))?;
        if !io
            .by_location(location)
            .is_some_and(|point| point.point_type == 108)
        {
            return Err(Error::InvalidRequest(
                "MVS 4088 assertion requires a discovered PT-108 location",
            ));
        }
        self.mvs_4088.insert(location);
        Ok(self)
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
    fn lookup_write(
        &self,
        context: &SchemaContext,
        tlp: Tlp,
    ) -> Result<Option<ParameterSchema>, Error> {
        // Audited R/W User fields only: Table 3-5 /2; Tables 3-13..17 tags,
        // plus Table 3-13 DI configuration /1–2, /4–5, /9–11, /13.
        // Both-updated status/counters and system-only fields remain excluded.
        // See docs/WRITES.md for the field-by-field write audit.
        // lookup retains the same explicit firmware and discovered physical-location gates.
        let schema = self.lookup(context, tlp)?;
        if matches!(
            (tlp.point_type, tlp.parameter),
            (91, 2) | (101, 0..=2 | 4..=5 | 9..=11 | 13) | (102..=105, 0..=1)
        ) {
            Ok(schema)
        } else {
            Err(Error::CapabilityUnavailable(
                "catalog parameter not audited for writes",
            ))
        }
    }

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
        let logical_range = if matches!(tlp.point_type, 91 | 136) {
            if tlp.logical != 0 {
                return Err(Error::InvalidRequest(
                    "singleton catalog point requires logical zero",
                ));
            }
            Some((0, 0))
        } else if tlp.point_type == 95 {
            // Table 3-7: six port ordinals; logical 1 is Ethernet.
            if tlp.logical > 5 {
                return Err(Error::InvalidRequest(
                    "communication port requires logical 0-5",
                ));
            }
            if tlp.logical == 1 && !matches!(tlp.parameter, 36 | 37 | 42 | 44) {
                return Err(Error::CapabilityUnavailable(
                    "parameter not valid for Ethernet port",
                ));
            }
            Some((0, 5))
        } else if tlp.point_type == 139 {
            // Table 3-51 uses module ordinals, not opcode-50 physical positions.
            if tlp.logical > 26 {
                return Err(Error::InvalidRequest("smart module requires logical 0-26"));
            }
            Some((0, 26))
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
        if tlp.point_type == 108 && tlp.parameter == 67 && !self.mvs_4088.contains(&tlp.logical) {
            return Err(Error::CapabilityUnavailable(
                "MVS config counter requires explicit 4088 sensor applicability",
            ));
        }
        Ok(Some(ParameterSchema {
            data_type: entry.data_type,
            access: entry.access,
            logical_range,
            numeric_range: entry.numeric_range,
        }))
    }
}
