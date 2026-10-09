//! Wire codec and typed read operations, independently usable without networking.
use crate::Error;

pub const HEADER_LEN: usize = 6;
pub const OVERHEAD: usize = 8;
pub const MAX_DATA_LEN: usize = u8::MAX as usize;
pub const MAX_PARAMETER_RESPONSE: usize = 240;

/// Unit/group pair. Codec accepts broadcast/reserved addresses; client does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address {
    pub unit: u8,
    pub group: u8,
}
impl Address {
    pub const fn new(unit: u8, group: u8) -> Self {
        Self { unit, group }
    }
    pub fn validate_unicast(self) -> Result<(), Error> {
        if self.unit == 0 || self == Self::new(240, 240) {
            return Err(Error::InvalidConfig(
                "broadcast/reserved address is not a unicast endpoint",
            ));
        }
        Ok(())
    }
}

/// Ethernet still carries CRC bytes. Ignore is an explicit compatibility option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrcMode {
    #[default]
    Required,
    Ignore,
}

/// Reflected CRC-16/ARC: polynomial 0x8005, seed zero, no final XOR.
pub fn crc16(bytes: &[u8]) -> u16 {
    let mut crc = 0_u16;
    for &byte in bytes {
        crc ^= u16::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xa001
            } else {
                crc >> 1
            };
        }
    }
    crc
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub destination: Address,
    pub source: Address,
    pub opcode: u8,
    pub data: Vec<u8>,
}
impl Frame {
    pub fn new(destination: Address, source: Address, opcode: u8, data: Vec<u8>) -> Self {
        Self {
            destination,
            source,
            opcode,
            data,
        }
    }
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        let len = u8::try_from(self.data.len())
            .map_err(|_| Error::InvalidRequest("payload exceeds 255 bytes"))?;
        let mut bytes = Vec::with_capacity(self.data.len() + OVERHEAD);
        bytes.extend([
            self.destination.unit,
            self.destination.group,
            self.source.unit,
            self.source.group,
            self.opcode,
            len,
        ]);
        bytes.extend_from_slice(&self.data);
        bytes.extend(crc16(&bytes).to_le_bytes());
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8], crc_mode: CrcMode) -> Result<Self, Error> {
        if bytes.len() < OVERHEAD {
            return Err(Error::InvalidFrame("truncated header/checksum"));
        }
        if bytes.len() != usize::from(bytes[5]) + OVERHEAD {
            return Err(Error::InvalidFrame("declared length does not match frame"));
        }
        let end = bytes.len() - 2;
        let expected = crc16(&bytes[..end]);
        let received = u16::from_le_bytes([bytes[end], bytes[end + 1]]);
        if crc_mode == CrcMode::Required && expected != received {
            return Err(Error::Crc { expected, received });
        }
        Ok(Self::new(
            Address::new(bytes[0], bytes[1]),
            Address::new(bytes[2], bytes[3]),
            bytes[4],
            bytes[HEADER_LEN..end].to_vec(),
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clock {
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u16,
    pub weekday: u8,
}
impl Clock {
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 8 {
            return Err(Error::InvalidResponse("clock must be eight bytes"));
        }
        let value = Self {
            second: bytes[0],
            minute: bytes[1],
            hour: bytes[2],
            day: bytes[3],
            month: bytes[4],
            year: u16::from_le_bytes([bytes[5], bytes[6]]),
            weekday: bytes[7],
        };
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<(), Error> {
        let leap = self.year.is_multiple_of(4)
            && (!self.year.is_multiple_of(100) || self.year.is_multiple_of(400));
        let days = match self.month {
            2 => {
                if leap {
                    29
                } else {
                    28
                }
            }
            4 | 6 | 9 | 11 => 30,
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            _ => 0,
        };
        if self.second > 59
            || self.minute > 59
            || self.hour > 23
            || self.day == 0
            || self.day > days
            || self.year == 0
            || !(1..=7).contains(&self.weekday)
        {
            return Err(Error::InvalidResponse("invalid clock fields"));
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        self.validate()?;
        let year = self.year.to_le_bytes();
        Ok(vec![
            self.second,
            self.minute,
            self.hour,
            self.day,
            self.month,
            year[0],
            year[1],
            self.weekday,
        ])
    }
}

/// Legacy three-character operator identity and numeric password.
/// Enhanced-security firmware login is intentionally not implemented yet.
#[derive(Clone, PartialEq, Eq)]
pub struct Login {
    operator: [u8; 3],
    password: u16,
    access_level: Option<u8>,
}
impl std::fmt::Debug for Login {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Login")
            .field("operator", &self.operator)
            .field("password", &"[redacted]")
            .field("access_level", &self.access_level)
            .finish()
    }
}
impl Login {
    pub fn new(operator: [u8; 3], password: u16, access_level: Option<u8>) -> Result<Self, Error> {
        if !operator.iter().all(|b| (0x20..=0x7e).contains(b)) {
            return Err(Error::InvalidRequest(
                "operator ID must contain three printable ASCII bytes",
            ));
        }
        Ok(Self {
            operator,
            password,
            access_level,
        })
    }
    pub fn payload(&self) -> Vec<u8> {
        let mut data = self.operator.to_vec();
        data.extend(self.password.to_le_bytes());
        if let Some(level) = self.access_level {
            data.push(level);
        }
        data
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tlp {
    pub point_type: u8,
    pub logical: u8,
    pub parameter: u8,
}
impl Tlp {
    pub const fn new(point_type: u8, logical: u8, parameter: u8) -> Self {
        Self {
            point_type,
            logical,
            parameter,
        }
    }
    pub const fn bytes(self) -> [u8; 3] {
        [self.point_type, self.logical, self.parameter]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Binary,
    Ascii(usize),
    Int8,
    Int16,
    Int32,
    UInt8,
    UInt16,
    UInt32,
    Float,
    Double,
    Tlp,
    Time,
}
impl DataType {
    pub const fn width(self) -> usize {
        match self {
            Self::Binary | Self::Int8 | Self::UInt8 => 1,
            Self::Int16 | Self::UInt16 => 2,
            Self::Int32 | Self::UInt32 | Self::Float | Self::Time => 4,
            Self::Double => 8,
            Self::Tlp => 3,
            Self::Ascii(n) => n,
        }
    }
    fn checked_width(self) -> Result<usize, Error> {
        let n = self.width();
        if n == 0 || n > MAX_PARAMETER_RESPONSE {
            return Err(Error::InvalidRequest("invalid parameter width"));
        }
        Ok(n)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Binary(u8),
    Ascii(String),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    Float(f32),
    Double(f64),
    Tlp(Tlp),
    Time(u32),
}
impl Value {
    pub fn data_type(&self) -> DataType {
        match self {
            Self::Binary(_) => DataType::Binary,
            Self::Ascii(v) => DataType::Ascii(v.len()),
            Self::Int8(_) => DataType::Int8,
            Self::Int16(_) => DataType::Int16,
            Self::Int32(_) => DataType::Int32,
            Self::UInt8(_) => DataType::UInt8,
            Self::UInt16(_) => DataType::UInt16,
            Self::UInt32(_) => DataType::UInt32,
            Self::Float(_) => DataType::Float,
            Self::Double(_) => DataType::Double,
            Self::Tlp(_) => DataType::Tlp,
            Self::Time(_) => DataType::Time,
        }
    }
    pub fn decode(kind: DataType, bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != kind.checked_width()? {
            return Err(Error::InvalidResponse("parameter width mismatch"));
        }
        // Array conversions are safe after the exact width check.
        Ok(match kind {
            DataType::Binary => Self::Binary(bytes[0]),
            DataType::Ascii(_) => {
                if !bytes.iter().all(|b| (0x20..=0x7e).contains(b)) {
                    return Err(Error::InvalidResponse("non-printable ASCII parameter"));
                }
                Self::Ascii(
                    String::from_utf8(bytes.to_vec())
                        .map_err(|_| Error::InvalidResponse("invalid ASCII"))?,
                )
            }
            DataType::Int8 => Self::Int8(bytes[0] as i8),
            DataType::UInt8 => Self::UInt8(bytes[0]),
            DataType::Int16 => Self::Int16(i16::from_le_bytes(bytes.try_into().unwrap())),
            DataType::UInt16 => Self::UInt16(u16::from_le_bytes(bytes.try_into().unwrap())),
            DataType::Int32 => Self::Int32(i32::from_le_bytes(bytes.try_into().unwrap())),
            DataType::UInt32 => Self::UInt32(u32::from_le_bytes(bytes.try_into().unwrap())),
            DataType::Float => Self::Float(f32::from_le_bytes(bytes.try_into().unwrap())),
            DataType::Double => Self::Double(f64::from_le_bytes(bytes.try_into().unwrap())),
            DataType::Tlp => Self::Tlp(Tlp::new(bytes[0], bytes[1], bytes[2])),
            DataType::Time => Self::Time(u32::from_le_bytes(bytes.try_into().unwrap())),
        })
    }
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        if let Self::Ascii(text) = self
            && (text.is_empty() || !text.bytes().all(|b| (0x20..=0x7e).contains(&b)))
        {
            return Err(Error::InvalidRequest(
                "ASCII value must be nonempty printable characters",
            ));
        }
        Ok(match self {
            Self::Binary(v) | Self::UInt8(v) => vec![*v],
            Self::Int8(v) => vec![*v as u8],
            Self::Ascii(v) => v.as_bytes().to_vec(),
            Self::Tlp(v) => v.bytes().to_vec(),
            Self::Int16(v) => v.to_le_bytes().to_vec(),
            Self::UInt16(v) => v.to_le_bytes().to_vec(),
            Self::Int32(v) => v.to_le_bytes().to_vec(),
            Self::UInt32(v) | Self::Time(v) => v.to_le_bytes().to_vec(),
            Self::Float(v) => v.to_le_bytes().to_vec(),
            Self::Double(v) => v.to_le_bytes().to_vec(),
        })
    }
}

/// The response does not include type tags; callers provide widths from their device schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parameter {
    pub tlp: Tlp,
    pub data_type: DataType,
}
impl Parameter {
    pub const fn new(tlp: Tlp, data_type: DataType) -> Self {
        Self { tlp, data_type }
    }
}
pub fn parameter_request(parameters: &[Parameter]) -> Result<Vec<u8>, Error> {
    let count =
        u8::try_from(parameters.len()).map_err(|_| Error::InvalidRequest("too many parameters"))?;
    if count == 0 {
        return Err(Error::InvalidRequest("empty parameter request"));
    }
    let mut response_size = 1_usize;
    for p in parameters {
        response_size += 3 + p.data_type.checked_width()?;
    }
    if response_size > MAX_PARAMETER_RESPONSE {
        return Err(Error::InvalidRequest(
            "parameter response would exceed 240 bytes",
        ));
    }
    let mut bytes = vec![count];
    for p in parameters {
        bytes.extend(p.tlp.bytes());
    }
    Ok(bytes)
}
pub fn decode_parameters(parameters: &[Parameter], bytes: &[u8]) -> Result<Vec<Value>, Error> {
    parameter_request(parameters)?;
    if bytes.first().copied() != Some(parameters.len() as u8) {
        return Err(Error::InvalidResponse("parameter count mismatch"));
    }
    let mut cursor = 1;
    let mut values = Vec::with_capacity(parameters.len());
    for p in parameters {
        if bytes.get(cursor..cursor + 3) != Some(p.tlp.bytes().as_slice()) {
            return Err(Error::InvalidResponse("TLP echo mismatch"));
        }
        cursor += 3;
        let end = cursor + p.data_type.width();
        let value = bytes
            .get(cursor..end)
            .ok_or(Error::InvalidResponse("truncated parameter"))?;
        values.push(Value::decode(p.data_type, value)?);
        cursor = end;
    }
    if cursor != bytes.len() {
        return Err(Error::InvalidResponse("trailing parameter data"));
    }
    Ok(values)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterBlock {
    pub point_type: u8,
    pub logical: u8,
    pub start: u8,
    types: Vec<DataType>,
}
impl ParameterBlock {
    pub(crate) fn response_size(&self) -> usize {
        4 + self.types.iter().map(|kind| kind.width()).sum::<usize>()
    }
    pub fn new(
        point_type: u8,
        logical: u8,
        start: u8,
        types: Vec<DataType>,
    ) -> Result<Self, Error> {
        if types.is_empty() || usize::from(start) + types.len() > 256 {
            return Err(Error::InvalidRequest("invalid contiguous parameter range"));
        }
        let mut size = 0;
        for t in &types {
            size += t.checked_width()?;
        }
        if size > 230 {
            return Err(Error::InvalidRequest(
                "block response would exceed 230 data bytes",
            ));
        }
        Ok(Self {
            point_type,
            logical,
            start,
            types,
        })
    }
    pub fn payload(&self) -> Vec<u8> {
        vec![
            self.point_type,
            self.logical,
            self.types.len() as u8,
            self.start,
        ]
    }
    pub fn decode(&self, bytes: &[u8]) -> Result<Vec<Value>, Error> {
        if bytes.get(..4) != Some(self.payload().as_slice()) {
            return Err(Error::InvalidResponse("block header mismatch"));
        }
        let mut cursor = 4;
        let mut values = Vec::with_capacity(self.types.len());
        for &kind in &self.types {
            let end = cursor + kind.width();
            values.push(Value::decode(
                kind,
                bytes
                    .get(cursor..end)
                    .ok_or(Error::InvalidResponse("truncated block"))?,
            )?);
            cursor = end;
        }
        if cursor != bytes.len() {
            return Err(Error::InvalidResponse("trailing block data"));
        }
        Ok(values)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceError {
    pub code: u8,
    pub offset: u8,
}
impl DeviceError {
    pub fn decode(bytes: &[u8]) -> Result<Vec<Self>, Error> {
        if bytes.is_empty() || !bytes.len().is_multiple_of(2) {
            return Err(Error::InvalidResponse("malformed device error list"));
        }
        Ok(bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| Self {
                code: b[0],
                offset: b[1],
            })
            .collect())
    }
}
