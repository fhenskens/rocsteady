use rocsteady::Error;
use rocsteady::protocol::{
    Address, Clock, CrcMode, DataType, DeviceError, Frame, Login, Parameter, ParameterBlock, Tlp,
    Value, crc16, decode_parameters, parameter_request,
};

fn host() -> Address {
    Address::new(1, 0)
}
fn device() -> Address {
    Address::new(1, 2)
}

#[test]
fn crc_matches_manual_chapter_four_vector() {
    // D301180X012, October 2022, p. 4-1: independent published vector.
    let message = [1, 2, 1, 0, 17, 3, b'M', b'O', b'C'];
    assert_eq!(crc16(&message), 0x1885);
    assert_eq!(crc16(b"123456789"), 0xbb3d);
    assert_eq!(crc16(&[]), 0);
}

#[test]
fn published_srbx_and_ack_vectors_decode_and_encode_exactly() {
    // Chapter 6, p. 6-1: literals do not use the production encoder as oracle.
    for bytes in [
        vec![1, 0, 1, 2, 224, 0, 232, 45],
        vec![1, 2, 1, 0, 225, 2, 7, 0, 118, 17],
    ] {
        let frame = Frame::decode(&bytes, CrcMode::Required).unwrap();
        assert_eq!(frame.encode().unwrap(), bytes);
    }
}

#[test]
fn frame_length_and_crc_failures_are_rejected() {
    let valid = [1, 0, 1, 2, 224, 0, 232, 45];
    for n in 0..valid.len() {
        assert!(Frame::decode(&valid[..n], CrcMode::Required).is_err());
    }
    let mut extra = valid.to_vec();
    extra.push(0);
    assert!(Frame::decode(&extra, CrcMode::Required).is_err());
    for i in 0..valid.len() {
        let mut corrupt = valid;
        corrupt[i] ^= 1;
        assert!(Frame::decode(&corrupt, CrcMode::Required).is_err());
    }
    let mut unused_crc = valid;
    unused_crc[6..].fill(0);
    assert!(Frame::decode(&unused_crc, CrcMode::Ignore).is_ok());
}

#[test]
fn frame_lengths_are_bounded_by_the_wire_length_field() {
    for len in [0, 1, 240, 255] {
        let frame = Frame::new(device(), host(), 180, vec![0; len]);
        let bytes = frame.encode().unwrap();
        assert_eq!(bytes.len(), len + 8);
        assert_eq!(bytes[5] as usize, len);
        assert_eq!(Frame::decode(&bytes, CrcMode::Required).unwrap(), frame);
    }
    assert!(
        Frame::new(device(), host(), 180, vec![0; 256])
            .encode()
            .is_err()
    );
}

#[test]
fn clock_uses_uint16_year_not_obsolete_leap_year_byte() {
    // Table 2-3, p. 2-11 takes precedence over outdated Figure 1-2 illustration.
    let clock = Clock::decode(&[59, 58, 23, 29, 2, 0xe8, 0x07, 5]).unwrap();
    assert_eq!(clock.year, 2024);
    assert_eq!(clock.day, 29);
    assert_eq!(clock.encode().unwrap(), [59, 58, 23, 29, 2, 0xe8, 0x07, 5]);
    assert!(Clock::decode(&[0, 0, 0, 29, 2, 0xe9, 0x07, 1]).is_err());
    assert!(Clock::decode(&[0, 0, 0, 31, 4, 0xe8, 0x07, 1]).is_err());
    assert!(Clock::decode(&[60, 0, 0, 1, 1, 0xe8, 0x07, 1]).is_err());
    assert!(Clock::decode(&[0, 0, 0, 1, 1, 0xe8, 0x07, 0]).is_err());
    assert!(Clock::decode(&[0; 7]).is_err());
    assert!(Clock::decode(&[0; 9]).is_err());
}

#[test]
fn legacy_login_packs_password_little_endian_and_optional_access() {
    let basic = Login::new(*b"ABC", 0x1234, None).unwrap();
    assert_eq!(basic.payload(), [b'A', b'B', b'C', 0x34, 0x12]);
    assert_eq!(
        Login::new(*b"ABC", 0x1234, Some(2)).unwrap().payload(),
        [b'A', b'B', b'C', 0x34, 0x12, 2]
    );
    assert!(!format!("{basic:?}").contains("4660"));
    assert!(Login::new([0, b'A', b'B'], 0, None).is_err());
}

#[test]
fn opcode_180_request_has_count_and_ordered_tlps() {
    let params = [
        Parameter::new(Tlp::new(103, 16, 3), DataType::Float),
        Parameter::new(Tlp::new(136, 0, 5), DataType::UInt16),
    ];
    assert_eq!(
        parameter_request(&params).unwrap(),
        [2, 103, 16, 3, 136, 0, 5]
    );
    assert!(parameter_request(&[]).is_err());
    let large = vec![Parameter::new(Tlp::new(1, 0, 0), DataType::Ascii(30)); 8];
    assert!(parameter_request(&large).is_err()); // response >240 bytes
    assert!(parameter_request(&[Parameter::new(Tlp::new(1, 0, 0), DataType::Ascii(0))]).is_err());
}

#[test]
fn opcode_180_response_echoes_tlp_before_each_value() {
    let params = [
        Parameter::new(Tlp::new(103, 16, 3), DataType::Float),
        Parameter::new(Tlp::new(136, 0, 5), DataType::UInt16),
    ];
    let bytes = [2, 103, 16, 3, 0, 0, 0x48, 0x41, 136, 0, 5, 0xe8, 7];
    assert_eq!(
        decode_parameters(&params, &bytes).unwrap(),
        [Value::Float(12.5), Value::UInt16(2024)]
    );
    for len in 0..bytes.len() {
        assert!(decode_parameters(&params, &bytes[..len]).is_err());
    }
    let mut wrong = bytes;
    wrong[2] = 17;
    assert!(decode_parameters(&params, &wrong).is_err());
    let mut trailing = bytes.to_vec();
    trailing.push(0);
    assert!(decode_parameters(&params, &trailing).is_err());
}

#[test]
fn opcode_167_validates_count_range_and_echoed_block_header() {
    let block = ParameterBlock::new(136, 0, 5, vec![DataType::UInt16, DataType::UInt8]).unwrap();
    assert_eq!(block.payload(), [136, 0, 2, 5]);
    assert_eq!(
        block.decode(&[136, 0, 2, 5, 0xe8, 7, 5]).unwrap(),
        [Value::UInt16(2024), Value::UInt8(5)]
    );
    assert!(block.decode(&[136, 0, 2, 6, 0xe8, 7, 5]).is_err());
    assert!(ParameterBlock::new(136, 0, 255, vec![DataType::UInt8; 2]).is_err());
    assert!(ParameterBlock::new(136, 0, 0, vec![]).is_err());
}

#[test]
fn error_response_preserves_all_codes_and_offsets_including_unknown_codes() {
    assert_eq!(
        DeviceError::decode(&[2, 9, 250, 12]).unwrap(),
        [
            DeviceError { code: 2, offset: 9 },
            DeviceError {
                code: 250,
                offset: 12
            }
        ]
    );
    assert!(DeviceError::decode(&[]).is_err());
    assert!(DeviceError::decode(&[2]).is_err());
    assert!(matches!(
        Value::decode(DataType::UInt32, &[1]),
        Err(Error::InvalidResponse(_))
    ));
}

#[test]
fn all_supported_data_types_decode_independent_literals() {
    let cases = [
        (DataType::Binary, vec![0xa5], Value::Binary(0xa5)),
        (DataType::Int8, vec![0xff], Value::Int8(-1)),
        (DataType::Int16, vec![0xfe, 0xff], Value::Int16(-2)),
        (
            DataType::Int32,
            vec![0xfd, 0xff, 0xff, 0xff],
            Value::Int32(-3),
        ),
        (DataType::UInt8, vec![42], Value::UInt8(42)),
        (DataType::UInt16, vec![0x34, 0x12], Value::UInt16(0x1234)),
        (
            DataType::UInt32,
            vec![0x78, 0x56, 0x34, 0x12],
            Value::UInt32(0x12345678),
        ),
        (DataType::Float, vec![0, 0, 0x80, 0x3f], Value::Float(1.0)),
        (
            DataType::Double,
            vec![0, 0, 0, 0, 0, 0, 0xf0, 0x3f],
            Value::Double(1.0),
        ),
        (DataType::Time, vec![1, 0, 0, 0], Value::Time(1)),
        (
            DataType::Tlp,
            vec![103, 16, 3],
            Value::Tlp(Tlp::new(103, 16, 3)),
        ),
        (
            DataType::Ascii(3),
            b"A B".to_vec(),
            Value::Ascii("A B".into()),
        ),
    ];
    for (kind, bytes, value) in cases {
        assert_eq!(Value::decode(kind, &bytes).unwrap(), value);
        assert_eq!(value.encode().unwrap(), bytes);
        assert!(Value::decode(kind, &bytes[..bytes.len() - 1]).is_err());
    }
    assert!(Value::decode(DataType::Ascii(1), &[0xff]).is_err());
}
