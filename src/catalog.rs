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
mod entries;
use entries::ENTRIES;

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
        let logical_range = if matches!(tlp.point_type, 91 | 136) {
            if tlp.logical != 0 {
                return Err(Error::InvalidRequest(
                    "singleton catalog point requires logical zero",
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
