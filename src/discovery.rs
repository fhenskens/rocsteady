//! D301180X012 (October 2022), Tables 2-12/2-13 and sections 3.1-3.2.
use crate::{
    DecodeError, Error, ManagedContext, Operation, ReplyCodec, SystemConfiguration, Workflow,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FirmwareVersion {
    major: u8,
    minor: u8,
}
impl FirmwareVersion {
    pub const fn new(major: u8, minor: u8) -> Result<Self, Error> {
        if major == 0 || minor > 99 {
            return Err(Error::InvalidRequest("invalid firmware version"));
        }
        Ok(Self { major, minor })
    }
    pub const fn major(self) -> u8 {
        self.major
    }
    pub const fn minor(self) -> u8 {
        self.minor
    }
}
impl std::fmt::Display for FirmwareVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{:02}", self.major, self.minor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointTypeClassification {
    UserProgram(u8),
    UserDefined,
    Roc,
    Absent,
    Unknown(u8),
}
impl PointTypeClassification {
    pub const fn from_byte(byte: u8) -> Self {
        match byte {
            0..=7 => Self::UserProgram(byte),
            253 => Self::UserDefined,
            254 => Self::Roc,
            255 => Self::Absent,
            _ => Self::Unknown(byte),
        }
    }
    pub const fn as_byte(self) -> u8 {
        match self {
            Self::UserProgram(v) | Self::Unknown(v) => v,
            Self::UserDefined => 253,
            Self::Roc => 254,
            Self::Absent => 255,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointTypeRequest {
    start: u8,
    count: u8,
}
impl PointTypeRequest {
    pub fn new(start: u8, count: u8) -> Result<Self, Error> {
        if count > 245 || u16::from(start) + u16::from(count) > 256 {
            return Err(Error::InvalidRequest("invalid point-type range"));
        }
        Ok(Self { start, count })
    }
    pub const fn start(self) -> u8 {
        self.start
    }
    pub const fn count(self) -> u8 {
        self.count
    }
    pub fn payload(self) -> Vec<u8> {
        vec![11, self.start, self.count]
    }
    pub fn response_size(self) -> usize {
        3 + usize::from(self.count)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointTypePage {
    request: PointTypeRequest,
    classifications: Vec<PointTypeClassification>,
}
impl PointTypePage {
    pub fn request(&self) -> PointTypeRequest {
        self.request
    }
    pub fn classifications(&self) -> &[PointTypeClassification] {
        &self.classifications
    }
    pub fn get(&self, point_type: u8) -> Option<PointTypeClassification> {
        self.classifications
            .get(usize::from(point_type.checked_sub(self.request.start)?))
            .copied()
    }
}
pub struct PointTypeCodec;
impl ReplyCodec<PointTypeRequest> for PointTypeCodec {
    type Output = PointTypePage;
    fn decode(
        &self,
        request: &PointTypeRequest,
        bytes: &[u8],
    ) -> Result<PointTypePage, DecodeError> {
        if bytes.len() != request.response_size()
            || bytes.get(..3) != Some(request.payload().as_slice())
        {
            return Err(DecodeError(
                "point-type count/command/start echo or length mismatch",
            ));
        }
        Ok(PointTypePage {
            request: *request,
            classifications: bytes[3..]
                .iter()
                .copied()
                .map(PointTypeClassification::from_byte)
                .collect(),
        })
    }
}
impl Operation for PointTypeRequest {
    type Output = PointTypePage;
    fn encode(&self) -> Result<(u8, Vec<u8>), Error> {
        Ok((100, self.payload()))
    }
    fn decode(&self, bytes: &[u8]) -> Result<PointTypePage, DecodeError> {
        PointTypeCodec.decode(self, bytes)
    }
}

/// Explicit wire layout; no length-based autodetection or PT-91/50 conflation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoLayout {
    Compatibility160,
    Compatibility240,
    Expanded224,
}
impl IoLayout {
    pub const fn len(self) -> usize {
        match self {
            Self::Compatibility160 => 160,
            Self::Compatibility240 => 240,
            Self::Expanded224 => 224,
        }
    }
    pub const fn is_empty(self) -> bool {
        false
    }
    pub const fn status(self) -> u8 {
        match self {
            Self::Compatibility160 => 0,
            Self::Compatibility240 => 1,
            Self::Expanded224 => 2,
        }
    }
    pub const fn points_per_slot(self) -> u8 {
        match self {
            Self::Compatibility160 | Self::Compatibility240 => 16,
            Self::Expanded224 => 8,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoSelector {
    PointTypes,
    LogicalNumbers,
}
impl IoSelector {
    pub const fn as_byte(self) -> u8 {
        match self {
            Self::PointTypes => 0,
            Self::LogicalNumbers => 1,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoPointRequest {
    pub layout: IoLayout,
    pub selector: IoSelector,
}
impl IoPointRequest {
    pub fn payload(self) -> Vec<u8> {
        vec![self.selector.as_byte()]
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IoPointReply {
    request: IoPointRequest,
    bytes: Vec<u8>,
}
impl IoPointReply {
    pub fn request(&self) -> IoPointRequest {
        self.request
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}
pub struct IoPointCodec;
impl ReplyCodec<IoPointRequest> for IoPointCodec {
    type Output = IoPointReply;
    fn decode(&self, request: &IoPointRequest, bytes: &[u8]) -> Result<IoPointReply, DecodeError> {
        if bytes.len() != request.layout.len() {
            return Err(DecodeError("I/O layout length mismatch"));
        }
        Ok(IoPointReply {
            request: *request,
            bytes: bytes.to_vec(),
        })
    }
}
impl Operation for IoPointRequest {
    type Output = IoPointReply;
    fn encode(&self) -> Result<(u8, Vec<u8>), Error> {
        Ok((50, self.payload()))
    }
    fn decode(&self, bytes: &[u8]) -> Result<IoPointReply, DecodeError> {
        IoPointCodec.decode(self, bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoPoint {
    pub response_index: usize,
    pub location: u8,
    pub point_type: u8,
    pub logical_number: u8,
}
impl IoPoint {
    /// Only 101-109 have established physical-location TLP semantics in section 3.2.
    pub fn tlp(self, parameter: u8) -> Result<crate::Tlp, Error> {
        if !(101..=109).contains(&self.point_type) {
            return Err(Error::CapabilityUnavailable(
                "I/O point TLP location semantics",
            ));
        }
        Ok(crate::Tlp::new(self.point_type, self.location, parameter))
    }
}
#[derive(Debug, Clone)]
pub struct IoSnapshot {
    generation: u64,
    firmware: FirmwareVersion,
    layout: IoLayout,
    configuration: SystemConfiguration,
    points: Vec<IoPoint>,
    point_types: Vec<u8>,
    logical_numbers: Vec<u8>,
}
impl IoSnapshot {
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn firmware(&self) -> FirmwareVersion {
        self.firmware
    }
    pub fn layout(&self) -> IoLayout {
        self.layout
    }
    pub fn configuration(&self) -> &SystemConfiguration {
        &self.configuration
    }
    pub fn points(&self) -> &[IoPoint] {
        &self.points
    }
    pub fn point_types(&self) -> &[u8] {
        &self.point_types
    }
    pub fn logical_numbers(&self) -> &[u8] {
        &self.logical_numbers
    }
    pub fn by_location(&self, location: u8) -> Option<IoPoint> {
        self.points.iter().find(|p| p.location == location).copied()
    }
    pub fn by_logical(&self, point_type: u8, logical_number: u8) -> Result<IoPoint, Error> {
        let mut points = self
            .points
            .iter()
            .filter(|p| p.point_type == point_type && p.logical_number == logical_number);
        let first = points
            .next()
            .copied()
            .ok_or(Error::CapabilityUnavailable("discovered I/O point"))?;
        if points.next().is_some() {
            return Err(Error::InvalidResponse("ambiguous I/O logical number"));
        }
        Ok(first)
    }
}

/// Repeated selector/configuration checks detect observed changes, not atomicity.
pub struct IoDiscovery {
    pub firmware: FirmwareVersion,
    pub layout: IoLayout,
}
#[etherbird::async_trait]
impl Workflow for IoDiscovery {
    type Output = IoSnapshot;
    async fn run(self, context: &mut ManagedContext<'_>) -> Result<IoSnapshot, Error> {
        let before = context.read_configuration().await?;
        context
            .profile()
            .validate_io_layout(&before, self.firmware, self.layout)?;
        let types_request = IoPointRequest {
            layout: self.layout,
            selector: IoSelector::PointTypes,
        };
        let logical_request = IoPointRequest {
            layout: self.layout,
            selector: IoSelector::LogicalNumbers,
        };
        let types = context.read_io_points(types_request).await?;
        let logical = context.read_io_points(logical_request).await?;
        let types_again = context.read_io_points(types_request).await?;
        let logical_again = context.read_io_points(logical_request).await?;
        let after = context.read_configuration().await?;
        if before != after || types != types_again || logical != logical_again {
            return Err(Error::DiscoveryChanged(
                "configuration or I/O selectors changed during discovery",
            ));
        }
        let mut points = Vec::new();
        let mut locations = std::collections::BTreeSet::new();
        let mut logicals = std::collections::BTreeSet::new();
        for (index, (&point_type, &logical_number)) in
            types.bytes.iter().zip(&logical.bytes).enumerate()
        {
            if point_type == 0 {
                continue;
            }
            let location = context.profile().io_location(self.layout, index)?;
            if !locations.insert(location)
                || ((101..=109).contains(&point_type)
                    && !logicals.insert((point_type, logical_number)))
            {
                return Err(Error::InvalidResponse(
                    "duplicate I/O location or logical number",
                ));
            }
            points.push(IoPoint {
                response_index: index,
                location,
                point_type,
                logical_number,
            });
        }
        Ok(IoSnapshot {
            generation: context.generation(),
            firmware: self.firmware,
            layout: self.layout,
            configuration: before,
            points,
            point_types: types.bytes,
            logical_numbers: logical.bytes,
        })
    }
}

pub(crate) struct ProfileIoOperation {
    pub request: IoPointRequest,
    pub profile: std::sync::Arc<dyn crate::ProtocolProfile>,
}
impl Operation for ProfileIoOperation {
    type Output = IoPointReply;
    fn encode(&self) -> Result<(u8, Vec<u8>), Error> {
        crate::profile::validate_expected_response(
            self.profile.as_ref(),
            50,
            self.request.layout.len(),
        )?;
        Ok((50, self.request.payload()))
    }
    fn decode(&self, payload: &[u8]) -> Result<IoPointReply, DecodeError> {
        self.profile.decode_io_points(&self.request, payload)
    }
}
pub(crate) struct ProfilePointTypeOperation {
    pub request: PointTypeRequest,
    pub profile: std::sync::Arc<dyn crate::ProtocolProfile>,
}
impl Operation for ProfilePointTypeOperation {
    type Output = PointTypePage;
    fn encode(&self) -> Result<(u8, Vec<u8>), Error> {
        crate::profile::validate_expected_response(
            self.profile.as_ref(),
            100,
            self.request.response_size(),
        )?;
        Ok((100, self.request.payload()))
    }
    fn decode(&self, payload: &[u8]) -> Result<PointTypePage, DecodeError> {
        self.profile.decode_point_types(&self.request, payload)
    }
}

pub(crate) struct PointTypeScan {
    pub requests: Vec<PointTypeRequest>,
}
impl PointTypeScan {
    pub fn new(start: u8, count: u16, profile: &dyn crate::ProtocolProfile) -> Result<Self, Error> {
        if count > 256 || u16::from(start) + count > 256 {
            return Err(Error::InvalidRequest("invalid point-type scan range"));
        }
        crate::profile::validate_profile_request(profile, 100, &[11, start, 0])?;
        let limit = profile.limits(100).response.min(255);
        if limit < 3 {
            return Err(Error::InvalidRequest(
                "point-type reply cannot fit profile limits",
            ));
        }
        let capacity = (limit - 3).min(245) as u16;
        if count > 0 && capacity == 0 {
            return Err(Error::InvalidRequest(
                "point-type classification cannot fit profile limits",
            ));
        }
        let mut requests = Vec::new();
        let mut remaining = count;
        let mut cursor = u16::from(start);
        if count == 0 {
            requests.push(PointTypeRequest::new(start, 0)?);
        }
        while remaining > 0 {
            let count = remaining.min(capacity);
            requests.push(PointTypeRequest::new(cursor as u8, count as u8)?);
            remaining -= count;
            cursor += count;
        }
        Ok(Self { requests })
    }
}
#[etherbird::async_trait]
impl Workflow for PointTypeScan {
    type Output = Vec<PointTypePage>;
    async fn run(self, context: &mut ManagedContext<'_>) -> Result<Self::Output, Error> {
        let mut pages = Vec::new();
        for request in self.requests {
            pages.push(context.read_point_types(request).await?);
        }
        Ok(pages)
    }
}
