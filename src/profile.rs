//! Explicit protocol selection. Profiles cannot replace transport framing or ownership.
use crate::{Address, Clock, DecodeError, Error, Operation, SystemConfiguration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    Verified,
    Provisional,
    UserSupplied,
    Unavailable,
}

/// Applicability is a caller assertion, never inferred from successful parsing.
#[derive(Debug, Clone, Copy)]
pub struct ProfileMetadata {
    pub name: &'static str,
    pub applicability: &'static str,
    pub assumptions: &'static str,
    pub evidence: &'static str,
}

/// Limits may tighten, but cannot relax, the framework's ordinary unicast limits.
#[derive(Debug, Clone, Copy)]
pub struct PayloadLimits {
    pub request: usize,
    pub response: usize,
}
impl Default for PayloadLimits {
    fn default() -> Self {
        Self {
            request: 255,
            response: 255,
        }
    }
}

pub trait ProtocolProfile: Send + Sync + 'static {
    fn metadata(&self) -> ProfileMetadata;
    /// Availability of outgoing request services, not device-originated signals/errors.
    fn capability(&self, opcode: u8) -> Capability;
    fn limits(&self, _opcode: u8) -> PayloadLimits {
        PayloadLimits::default()
    }
    /// Only source correlation is replaceable. Destination, opcode, CRC, lengths,
    /// unicast direction and cancellation remain framework checks.
    fn accepts_source(&self, expected: Address, actual: Address) -> bool {
        expected == actual
    }
    fn decode_clock(&self, payload: &[u8]) -> Result<Clock, DecodeError> {
        ClockCodec.decode(&(), payload)
    }
    fn decode_configuration(&self, payload: &[u8]) -> Result<SystemConfiguration, DecodeError> {
        ConfigurationCodec.decode(&(), payload)
    }
    fn io_layout(&self) -> Option<crate::IoLayout> {
        None
    }
    fn decode_io_points(
        &self,
        request: &crate::IoPointRequest,
        payload: &[u8],
    ) -> Result<crate::IoPointReply, DecodeError> {
        crate::IoPointCodec.decode(request, payload)
    }
    fn decode_point_types(
        &self,
        request: &crate::PointTypeRequest,
        payload: &[u8],
    ) -> Result<crate::PointTypePage, DecodeError> {
        crate::PointTypeCodec.decode(request, payload)
    }
    fn decode_alarms(
        &self,
        request: &crate::LogRequest,
        payload: &[u8],
    ) -> Result<crate::AlarmPage, DecodeError> {
        crate::AlarmCodec.decode(request, payload)
    }
    fn decode_events(
        &self,
        request: &crate::LogRequest,
        payload: &[u8],
    ) -> Result<crate::EventPage, DecodeError> {
        crate::EventCodec.decode(request, payload)
    }
    fn decode_history(
        &self,
        request: &crate::HistoryRequest,
        payload: &[u8],
    ) -> Result<crate::HistoryPage, DecodeError> {
        crate::HistoryCodec.decode(request, payload)
    }
    fn decode_multi_history(
        &self,
        request: &crate::MultiHistoryRequest,
        payload: &[u8],
    ) -> Result<crate::MultiHistoryPage, DecodeError> {
        crate::MultiHistoryCodec.decode(request, payload)
    }
    fn decode_day_indices(
        &self,
        request: &crate::DayIndicesRequest,
        payload: &[u8],
    ) -> Result<crate::DayIndices, DecodeError> {
        crate::DayIndicesCodec.decode(request, payload)
    }
    fn decode_history_day(
        &self,
        request: &crate::DayHistoryRequest,
        payload: &[u8],
    ) -> Result<crate::DayHistoryReply, DecodeError> {
        crate::DayHistoryCodec.decode(request, payload)
    }
    fn decode_configured_history(
        &self,
        request: &crate::ConfiguredHistoryRequest,
        payload: &[u8],
    ) -> Result<crate::ConfiguredHistoryPoints, DecodeError> {
        crate::ConfiguredHistoryCodec.decode(request, payload)
    }
    fn decode_selected_history(
        &self,
        request: &crate::SelectedHistoryRequest,
        payload: &[u8],
    ) -> Result<crate::SelectedHistoryPage, DecodeError> {
        crate::SelectedHistoryCodec.decode(request, payload)
    }
    fn decode_min_max(
        &self,
        request: &crate::MinMaxRequest,
        payload: &[u8],
    ) -> Result<crate::MinMaxReply, DecodeError> {
        crate::MinMaxCodec(crate::RawOccurrenceCodec).decode(request, payload)
    }
    fn decode_history_tags(
        &self,
        request: &crate::HistoryTagsRequest,
        payload: &[u8],
    ) -> Result<crate::HistoryTagsReply, DecodeError> {
        crate::HistoryTagsCodec.decode(request, payload)
    }
    /// Section 3.2 sequential physical positions. Override explicitly for a known alternative.
    fn io_location(&self, layout: crate::IoLayout, response_index: usize) -> Result<u8, Error> {
        if response_index >= layout.len() {
            return Err(Error::InvalidRequest("I/O index outside layout"));
        }
        Ok(response_index as u8)
    }
    fn validate_io_layout(
        &self,
        configuration: &SystemConfiguration,
        firmware: crate::FirmwareVersion,
        layout: crate::IoLayout,
    ) -> Result<(), Error> {
        if configuration.device_type() != 6 || configuration.mode() != crate::SystemMode::Run {
            return Err(Error::CapabilityUnavailable("running ROC800 discovery"));
        }
        if firmware.major() >= 2 {
            if configuration.logical_compatibility_raw() > 2 {
                return Err(Error::CapabilityUnavailable(
                    "unknown reported I/O layout status",
                ));
            }
            if configuration.logical_compatibility_raw() != layout.status() {
                return Err(Error::DiscoveryChanged(
                    "reported layout differs from selected layout",
                ));
            }
        } else if layout != crate::IoLayout::Compatibility160 {
            return Err(Error::InvalidRequest(
                "pre-2.00 firmware requires the 160-entry layout",
            ));
        }
        Ok(())
    }
}
impl std::fmt::Debug for dyn ProtocolProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.metadata().fmt(f)
    }
}

/// Existing ordinary-unicast codecs; other services require caller-owned Operations.
pub struct DefaultProfile;
impl ProtocolProfile for DefaultProfile {
    fn metadata(&self) -> ProfileMetadata {
        ProfileMetadata {
            name: "roc-plus-default",
            applicability: "ordinary unicast; firmware applicability is not automatically discovered",
            assumptions: "explicit addresses and caller-supplied parameter types; no routed framing",
            evidence: "tests/protocol.rs, tests/configuration.rs, tests/integration.rs, tests/discovery.rs, tests/logs.rs, tests/history_metadata.rs, tests/history.rs, tests/multi_history.rs, tests/day_history.rs, tests/history_information.rs, tests/writes.rs, tests/arbitrary_writes.rs, tests/clock_set.rs, tests/transactions.rs, tests/configurable_tables.rs; no hardware evidence",
        }
    }
    fn capability(&self, opcode: u8) -> Capability {
        match opcode {
            6 | 7 | 8 | 10 | 11 | 17 | 50 | 100 | 108 | 135 | 136 | 137 | 138 | 139 | 118 | 119
            | 166 | 167 | 180 | 181 | 225 => Capability::Verified,
            105 => Capability::Provisional, // Raw occurrence regions; DEC-4.01 remains open.
            206 => Capability::Provisional, // Raw transaction pages; DEC-5.05–07 remain open.
            24 | 224 | 255 => Capability::Unavailable,
            _ => Capability::UserSupplied,
        }
    }
}

/// Named explicit layout preset. No discovery-derived firmware/security selection.
pub struct Roc800Profile {
    pub layout: crate::IoLayout,
}
impl ProtocolProfile for Roc800Profile {
    fn metadata(&self) -> ProfileMetadata {
        ProfileMetadata {
            name: "roc800-explicit-io-layout",
            applicability: "ROC800 with independently established firmware and explicit 160/240/224-entry layout",
            assumptions: "section 3.2 sequential physical positions; contradictory expanded-layout diagrams are not autodetected",
            evidence: "D301180X012 sections 2.9/3.2; tests/discovery.rs; no hardware evidence",
        }
    }
    fn capability(&self, opcode: u8) -> Capability {
        DefaultProfile.capability(opcode)
    }
    fn io_layout(&self) -> Option<crate::IoLayout> {
        Some(self.layout)
    }
}

pub(crate) fn validate_profile_request(
    profile: &dyn ProtocolProfile,
    opcode: u8,
    data: &[u8],
) -> Result<(), Error> {
    validate_profile_request_size(profile, opcode, data.len())
}
pub(crate) fn validate_profile_request_size(
    profile: &dyn ProtocolProfile,
    opcode: u8,
    size: usize,
) -> Result<(), Error> {
    crate::protocol::validate_request(opcode, size)?;
    if profile.capability(opcode) == Capability::Unavailable {
        return Err(Error::CapabilityUnavailable("selected profile service"));
    }
    if size > profile.limits(opcode).request.min(255) {
        return Err(Error::InvalidRequest(
            "request exceeds selected profile limit",
        ));
    }
    Ok(())
}

pub(crate) fn validate_expected_response(
    profile: &dyn ProtocolProfile,
    opcode: u8,
    size: usize,
) -> Result<(), Error> {
    if size > profile.limits(opcode).response.min(255) {
        return Err(Error::InvalidRequest(
            "expected response exceeds selected profile limit",
        ));
    }
    Ok(())
}

pub(crate) fn validate_parameter_read(
    profile: &dyn ProtocolProfile,
    parameters: &[crate::Parameter],
) -> Result<(), Error> {
    let budget = crate::protocol::parameter_budget(parameters)?;
    validate_profile_request_size(profile, 180, budget.request)?;
    validate_expected_response(profile, 180, budget.response)
}

/// A codec receives the original typed request, including selectors/echo context.
pub trait ReplyCodec<Q>: Send + Sync + 'static {
    type Output: Send + 'static;
    fn decode(&self, request: &Q, payload: &[u8]) -> Result<Self::Output, DecodeError>;
}

/// Use predefined and caller-owned codecs through the same managed Operation API.
pub struct DecodedOperation<Q, C> {
    pub request: Q,
    pub opcode: u8,
    pub payload: Vec<u8>,
    pub codec: C,
}
impl<Q: Send + Sync + 'static, C: ReplyCodec<Q>> Operation for DecodedOperation<Q, C> {
    type Output = C::Output;
    fn encode(&self) -> Result<(u8, Vec<u8>), Error> {
        Ok((self.opcode, self.payload.clone()))
    }
    fn decode(&self, payload: &[u8]) -> Result<Self::Output, DecodeError> {
        self.codec.decode(&self.request, payload)
    }
}

pub struct ClockCodec;
impl ReplyCodec<()> for ClockCodec {
    type Output = Clock;
    fn decode(&self, _: &(), payload: &[u8]) -> Result<Clock, DecodeError> {
        Clock::decode(payload).map_err(|_| DecodeError("invalid clock reply"))
    }
}

pub struct ConfigurationCodec;
impl ReplyCodec<()> for ConfigurationCodec {
    type Output = SystemConfiguration;
    fn decode(&self, _: &(), payload: &[u8]) -> Result<SystemConfiguration, DecodeError> {
        SystemConfiguration::decode(payload).map_err(|_| DecodeError("invalid configuration reply"))
    }
}
