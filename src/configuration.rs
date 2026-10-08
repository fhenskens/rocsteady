//! Opcode 6 discovery, based on D301180X012 Table 2-2 (October 2022).
use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemMode {
    FirmwareUpdate,
    Run,
    Unknown(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalCompatibility {
    SixteenPointsNineSlots,
    SixteenPointsFourteenSlots,
    EightPointsTwentySevenSlots,
    Unknown(u8),
}

/// Configuration and point counts reported by the device, without inferring
/// firmware version or an authentication algorithm from the port security byte.
/// Reserved bytes and future extensions are retained verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemConfiguration {
    bytes: Vec<u8>,
}

impl SystemConfiguration {
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        // Original through point 137: 97 bytes; firmware 1.20 adds point 138.
        // Revision >= 1 includes counts through point 255: 215 bytes.
        if bytes.len() < 97 || bytes.len() > 255 {
            return Err(Error::InvalidResponse(
                "invalid system configuration length",
            ));
        }
        if bytes[5] >= 1 && bytes.len() < 215 {
            return Err(Error::InvalidResponse(
                "truncated extended system configuration",
            ));
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn mode(&self) -> SystemMode {
        match self.bytes[0] {
            0 => SystemMode::FirmwareUpdate,
            1 => SystemMode::Run,
            value => SystemMode::Unknown(value),
        }
    }

    /// Undefined in firmware update mode; unknown modes are also left uninterpreted.
    pub fn communication_port(&self) -> Option<u16> {
        (self.mode() == SystemMode::Run).then(|| u16::from_le_bytes([self.bytes[1], self.bytes[2]]))
    }

    /// Raw port security mode. This alone does not select a login procedure.
    pub fn security_access_mode(&self) -> u8 {
        self.bytes[3]
    }

    /// Raw byte; reserved before firmware 2.00. Discovery does not report firmware.
    pub fn logical_compatibility_raw(&self) -> u8 {
        self.bytes[4]
    }

    /// Interpret only with an independently established ROC Plus firmware major
    /// version. None means applicability is unknown or the field is reserved.
    /// The configuration revision and response length are not firmware versions.
    pub fn logical_compatibility(
        &self,
        firmware_major: Option<u16>,
    ) -> Option<LogicalCompatibility> {
        if !firmware_major.is_some_and(|major| major >= 2) {
            return None;
        }
        Some(match self.logical_compatibility_raw() {
            0 => LogicalCompatibility::SixteenPointsNineSlots,
            1 => LogicalCompatibility::SixteenPointsFourteenSlots,
            2 => LogicalCompatibility::EightPointsTwentySevenSlots,
            value => LogicalCompatibility::Unknown(value),
        })
    }

    pub fn revision(&self) -> u8 {
        self.bytes[5]
    }
    /// Defined by firmware 3.00 and later: 0 = Series 2, 1 = Series 1.
    /// Older firmware reserves this byte, so it is not proof of device series.
    pub fn roc_subtype(&self) -> u8 {
        self.bytes[6]
    }
    /// Device type code; 6 identifies ROC800, 11 identifies DL8000.
    pub fn device_type(&self) -> u8 {
        self.bytes[18]
    }

    /// None means the response does not describe this point type; Some(0)
    /// means it explicitly reports no logical points of that type.
    pub fn point_count(&self, point_type: u8) -> Option<u8> {
        if point_type < 60 {
            return None;
        }
        let last = if self.revision() >= 1 {
            255
        } else if self.bytes.len() >= 98 {
            138
        } else {
            137
        };
        if point_type > last {
            return None;
        }
        self.bytes.get(19 + usize::from(point_type - 60)).copied()
    }

    pub fn extensions(&self) -> &[u8] {
        let end = if self.revision() >= 1 {
            215
        } else {
            self.bytes.len().min(98)
        };
        &self.bytes[end..]
    }
}
