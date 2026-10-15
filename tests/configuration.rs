use rocsteady::{LogicalCompatibility, SystemConfiguration, SystemMode};

// Independent Table 2-2 offsets, expressed as payload offsets (wire offset - 6).
fn original() -> Vec<u8> {
    let mut bytes = vec![0; 98];
    bytes[..7].copy_from_slice(&[1, 0x34, 0x12, 3, 2, 0, 1]);
    bytes[18] = 6;
    bytes[19] = 7; // point 60
    bytes[97] = 9; // point 138
    bytes
}

#[test]
// REQ-006-ORIGINAL: Table 2-2. See docs/REQUIREMENTS.md.
fn original_and_pre_120_discovery_have_distinct_available_counts() {
    let bytes = original();
    let value = SystemConfiguration::decode(&bytes).unwrap();
    assert_eq!(value.mode(), SystemMode::Run);
    assert_eq!(value.communication_port(), Some(0x1234));
    assert_eq!(value.security_access_mode(), 3);
    assert_eq!(
        value.logical_compatibility(Some(2)),
        Some(LogicalCompatibility::EightPointsTwentySevenSlots)
    );
    assert_eq!(value.roc_subtype(), 1);
    assert_eq!(value.device_type(), 6);
    assert_eq!(value.point_count(59), None);
    assert_eq!(value.point_count(60), Some(7));
    assert_eq!(value.point_count(137), Some(0));
    assert_eq!(value.point_count(138), Some(9));
    assert_eq!(value.point_count(139), None);
    assert_eq!(value.as_bytes(), bytes);
    assert_eq!(
        SystemConfiguration::decode(&bytes[..97])
            .unwrap()
            .point_count(138),
        None
    );
    for length in 0..97 {
        assert!(SystemConfiguration::decode(&bytes[..length]).is_err());
    }
}

#[test]
// REQ-006-EXTENDED: Table 2-2. See docs/REQUIREMENTS.md.
fn extended_counts_and_unknown_fields_are_preserved() {
    let mut bytes = original();
    bytes.resize(215, 0);
    bytes[5] = 1;
    bytes[98] = 17; // point 139
    bytes[214] = 23; // point 255
    bytes[7] = 0xab; // reserved
    let value = SystemConfiguration::decode(&bytes).unwrap();
    assert_eq!(value.point_count(139), Some(17));
    assert_eq!(value.point_count(255), Some(23));
    for length in 97..215 {
        assert!(SystemConfiguration::decode(&bytes[..length]).is_err());
    }
    bytes[0] = 99;
    bytes[4] = 99;
    bytes[5] = 2;
    bytes[18] = 99;
    bytes.extend([0xde, 0xad]);
    let value = SystemConfiguration::decode(&bytes).unwrap();
    assert_eq!(value.mode(), SystemMode::Unknown(99));
    assert_eq!(value.communication_port(), None);
    assert_eq!(
        value.logical_compatibility(Some(3)),
        Some(LogicalCompatibility::Unknown(99))
    );
    assert_eq!(value.device_type(), 99);
    assert_eq!(value.extensions(), [0xde, 0xad]);
    assert_eq!(value.as_bytes(), bytes);
    bytes[0] = 0;
    assert_eq!(
        SystemConfiguration::decode(&bytes)
            .unwrap()
            .communication_port(),
        None
    );
    assert!(SystemConfiguration::decode(&[0; 256]).is_err());
}

#[test]
// POL-006-FIRMWARE: DEC-8; configuration contract. See docs/REQUIREMENTS.md.
fn layout_interpretation_requires_independent_firmware_applicability() {
    for (revision, length) in [(0, 98), (1, 215), (2, 217)] {
        let mut bytes = original();
        bytes.resize(length, 0);
        bytes[5] = revision;
        for raw in [0, 1, 2, 99] {
            bytes[4] = raw;
            let configuration = SystemConfiguration::decode(&bytes).unwrap();
            assert_eq!(configuration.logical_compatibility_raw(), raw);
            assert_eq!(configuration.logical_compatibility(None), None);
            assert_eq!(configuration.logical_compatibility(Some(1)), None);
            let expected = match raw {
                0 => LogicalCompatibility::SixteenPointsNineSlots,
                1 => LogicalCompatibility::SixteenPointsFourteenSlots,
                2 => LogicalCompatibility::EightPointsTwentySevenSlots,
                value => LogicalCompatibility::Unknown(value),
            };
            assert_eq!(configuration.logical_compatibility(Some(2)), Some(expected));
            assert_eq!(configuration.as_bytes(), bytes);
        }
    }
}
