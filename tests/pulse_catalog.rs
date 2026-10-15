//! Requirements for the complete PT-105 read slice of D301180X012 (October 2022).
include!("../examples/support/discovery_fixture.rs");
use rocsteady::{
    Access, Client, ClientConfig, DataType, Error, FirmwareVersion, IoLayout, ReadPlan,
    Roc800Catalog, Roc800Profile, SchemaContext, SchemaProvider, Tlp, Value,
};
use std::time::Duration;

fn client(peer: &DiscoveryFixture, layout: IoLayout) -> Client {
    Client::tcp(
        &peer.endpoint,
        ClientConfig {
            profile: Arc::new(Roc800Profile { layout }),
            heartbeat_interval: None,
            response_timeout: Duration::from_secs(1),
            call_timeout: Duration::from_secs(3),
            recovery: etherbird::Config {
                retry_delay: Duration::from_millis(1),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap()
}

// Expected types/values are independent of CatalogEntry and the peer's byte literals.
fn values() -> Vec<Value> {
    vec![
        Value::Ascii("PI SAMPLE ".into()),
        Value::Ascii("m3        ".into()),
        Value::UInt8(1),
        Value::Float(1.0),
        Value::UInt32(16000000),
        Value::UInt8(23),
        Value::UInt32(u32::MAX),
        Value::UInt8(3),
        Value::UInt8(1),
        Value::Float(1.0),
        Value::Float(12.5),
        Value::UInt8(2),
        Value::Float(12.5),
        Value::Float(12.5),
        Value::Float(12.5),
        Value::Float(12.5),
        Value::Float(12.5),
        Value::Float(12.5),
        Value::Float(12.5),
        Value::Float(12.5),
        Value::UInt8(1),
        Value::UInt8(1),
        Value::UInt8(1),
        Value::Binary(255),
        Value::Float(12.5),
        Value::Float(12.5),
        Value::Float(12.5),
        Value::Float(12.5),
    ]
}

// REQ-PI-TABLE: §3.4.16, Table 3-17, pp.3-72–74: all 28 field widths,
// source access, introduction versions, constraints, fixed units and source references.
#[test]
fn pulse_table_accounts_for_every_read_field_and_constraint() {
    assert_eq!(
        Roc800Catalog::entries()
            .iter()
            .filter(|e| e.point_type == 105)
            .count(),
        28
    );
    assert!(Roc800Catalog::entry(105, 28).is_none());
    for p in 0..28 {
        let entry = Roc800Catalog::entry(105, p).unwrap();
        let ty = match p {
            0 | 1 => DataType::Ascii(10),
            2 | 5 | 7 | 8 | 11 | 20..=22 => DataType::UInt8,
            4 | 6 => DataType::UInt32,
            23 => DataType::Binary,
            _ => DataType::Float,
        };
        assert_eq!(entry.data_type, ty, "PT-105/{p}");
        assert_eq!(
            entry.access,
            if matches!(p, 6 | 10 | 23..=27) {
                Access::ReadOnly
            } else {
                Access::ReadWrite
            }
        );
        assert_eq!(entry.introduced, (1, 0));
        assert_eq!(
            entry.numeric_range,
            match p {
                2 | 8 | 20..=22 => Some((0.0, 1.0)),
                3 => Some((0.05, 43200.0)),
                4 => Some((0.0, 16000000.0)),
                5 => Some((0.0, 23.0)),
                6 => Some((0.0, 4294967295.0)),
                7 => Some((0.0, 3.0)),
                11 => Some((0.0, 2.0)),
                27 => Some((0.0, f32::MAX as f64)),
                _ => None,
            }
        );
        assert_eq!(
            entry.metadata.units,
            match p {
                3 => Some("s"),
                4 | 6 => Some("pulses"),
                27 => Some("pulses/s"),
                _ => None,
            }
        );
        assert_eq!(entry.metadata.write_caveat.is_some(), p == 9);
        assert_eq!(
            entry.source,
            match p {
                0..=10 => "3.4.16/Table 3-17, p.3-72",
                11..=23 => "3.4.16/Table 3-17, p.3-73",
                _ => "3.4.16/Table 3-17, p.3-74",
            }
        );
    }
    // A nonzero float domain is not an inclusive interval; no invented positive-only bound.
    assert!(
        Roc800Catalog::entry(105, 9)
            .unwrap()
            .metadata
            .write_caveat
            .unwrap()
            .contains("excludes zero")
    );
}

// REQ-PI-ENUMS: Table 3-17 /2, /7, /8, /11, /20–22 (pp.3-72–73).
// Interpret every documented code and leave every other byte unlabeled.
#[test]
fn pulse_enumerations_preserve_unknown_codes() {
    for (p, labels) in [
        (2, &["disabled", "enabled"][..]),
        (
            7,
            &[
                "eu_per_second",
                "eu_per_minute",
                "eu_per_hour",
                "eu_per_day",
            ][..],
        ),
        (8, &["eu_per_pulse", "pulses_per_eu"][..]),
        (
            11,
            &[
                "rate",
                "accumulator_maximum_rollover",
                "accumulator_entered_rollover",
            ][..],
        ),
        (20, &["disabled", "enabled"][..]),
        (21, &["disabled", "enabled"][..]),
        (22, &["disabled", "enabled"][..]),
    ] {
        let meta = Roc800Catalog::entry(105, p).unwrap().metadata;
        assert_eq!(meta.enumeration.len(), labels.len());
        for code in 0..=255 {
            assert_eq!(
                meta.enum_meaning(code).map(|m| m.key),
                labels.get(usize::from(code)).copied()
            );
        }
    }
}

// REQ-PI-ALARMS: Table 3-17 /23.0–23.7, pp.3-73–74: bit 5 is unused;
// all seven documented meanings and every raw combination must survive.
#[test]
fn pulse_alarm_bits_match_source_and_retain_unused_bit() {
    let meta = Roc800Catalog::entry(105, 23).unwrap().metadata;
    let meanings = [
        (1, "low"),
        (2, "low_low"),
        (4, "high"),
        (8, "high_high"),
        (16, "rate"),
        (64, "point_fail"),
        (128, "scanning_disabled"),
    ];
    assert_eq!(meta.unused_bits, 0x20);
    assert_eq!(meta.bits.len(), 7);
    for raw in 0..=255 {
        let expected: Vec<_> = meanings
            .iter()
            .filter(|(mask, _)| raw & mask != 0)
            .map(|(_, key)| *key)
            .collect();
        assert_eq!(
            meta.active_bits(raw).map(|m| m.key).collect::<Vec<_>>(),
            expected
        );
        assert_eq!(meta.uninterpreted_bits(raw), raw & 0x20);
    }
}

// REQ-PI-READS: Table 3-17, all fields introduced 1.00; §3.2 physical TLPs;
// Table 2-25. Cover baseline and reviewed upper firmware, all applicable layouts,
// independent literal/TCP values, and whole-batch rejection before any I/O.
#[tokio::test]
async fn pulse_reads_every_field_at_firmware_and_location_boundaries() {
    for (status, layout, stride) in [
        (0, IoLayout::Compatibility160, 16),
        (1, IoLayout::Compatibility240, 16),
        (2, IoLayout::Expanded224, 8),
    ] {
        let peer = DiscoveryFixture::start(status).await;
        let client = client(&peer, layout);
        for (major, minor) in if status == 0 {
            vec![(1, 0), (2, 0), (3, 60)]
        } else {
            vec![(2, 0), (3, 60)]
        } {
            let io = client
                .discover_io(FirmwareVersion::new(major, minor).unwrap())
                .await
                .unwrap();
            assert_eq!(
                io.by_logical(105, 0).unwrap().tlp(27).unwrap(),
                Tlp::new(105, 5 * stride, 27)
            );
            assert_eq!(io.configuration().point_count(105), Some(1));
            let catalog = Roc800Catalog::from_discovery(io).unwrap();
            let tlps: Vec<_> = (0..28).map(|p| Tlp::new(105, 5 * stride, p)).collect();
            assert_eq!(
                client
                    .read_schema(&catalog, &catalog.context(), &tlps)
                    .await
                    .unwrap(),
                values()
            );
            let reads = peer.peer.parameter_reads.load(Ordering::SeqCst);
            for invalid in [
                Tlp::new(105, 0, 10),
                Tlp::new(105, 4 * stride, 10),
                Tlp::new(105, 255, 10),
                Tlp::new(105, 5 * stride, 28),
            ] {
                let err = client
                    .read_schema(&catalog, &catalog.context(), &[tlps[0], invalid])
                    .await
                    .unwrap_err();
                assert!(err.progress.values.is_empty());
                assert_eq!(peer.peer.parameter_reads.load(Ordering::SeqCst), reads);
            }
            assert!(catalog.lookup(&SchemaContext::default(), tlps[0]).is_err());
        }
        assert_eq!(
            client
                .read_point_types(rocsteady::PointTypeRequest::new(105, 1).unwrap())
                .await
                .unwrap()
                .get(105),
            Some(rocsteady::PointTypeClassification::Roc)
        );
        assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
        client.stop().await;
    }
    let unbound = Roc800Catalog::new(FirmwareVersion::new(1, 0).unwrap()).unwrap();
    assert!(
        unbound
            .lookup(&unbound.context(), Tlp::new(105, 80, 10))
            .is_err()
    );
    assert!(Roc800Catalog::new(FirmwareVersion::new(3, 61).unwrap()).is_err());
}

// POL-PI-RETIREMENT: Table 2-25 + managed ownership contract. Malformed
// count/echo/width/length or disconnect retires; valid rejection stays healthy.
// Failed calls never replay; old pulse plans require fresh discovery after replacement.
#[tokio::test]
async fn pulse_malformed_replies_rejections_and_replacements_have_explicit_outcomes() {
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer, IoLayout::Compatibility160);
    let tlps = [Tlp::new(105, 80, 4), Tlp::new(105, 80, 27)];
    let mut connections = 1;
    for fault in [3, 1, 2, 6, 7, 8, 4] {
        let catalog = Roc800Catalog::from_discovery(
            client
                .discover_io(FirmwareVersion::new(1, 0).unwrap())
                .await
                .unwrap(),
        )
        .unwrap();
        let plan = ReadPlan::build(&catalog, &catalog.context(), &tlps, client.profile()).unwrap();
        let reads = peer.peer.parameter_reads.load(Ordering::SeqCst);
        peer.peer.fault_opcode.store(180, Ordering::SeqCst);
        peer.peer.fault.store(fault, Ordering::SeqCst);
        let err = client
            .read_schema(&catalog, &catalog.context(), &tlps)
            .await
            .unwrap_err();
        assert!(err.progress.values.is_empty());
        if fault == 3 {
            assert!(matches!(
                err.error,
                etherbird::Error::Operation(Error::Device(_))
            ));
            assert_eq!(
                client.read_plan(plan).await.unwrap(),
                [Value::UInt32(16000000), Value::Float(12.5)]
            );
            assert_eq!(peer.peer.parameter_reads.load(Ordering::SeqCst), reads + 2);
        } else {
            if fault != 4 {
                assert!(matches!(
                    err.error,
                    etherbird::Error::Operation(Error::InvalidResponse(_))
                ));
            }
            client.read_clock().await.unwrap();
            connections += 1;
            assert!(matches!(
                client.read_plan(plan).await.unwrap_err().error,
                etherbird::Error::Operation(Error::StaleGeneration { .. })
            ));
            assert!(
                client
                    .read_schema(&catalog, &catalog.context(), &tlps)
                    .await
                    .is_err()
            );
            assert_eq!(peer.peer.parameter_reads.load(Ordering::SeqCst), reads + 1);
        }
        assert_eq!(peer.peer.connections.load(Ordering::SeqCst), connections);
    }
    let catalog = Roc800Catalog::from_discovery(
        client
            .discover_io(FirmwareVersion::new(1, 0).unwrap())
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        client
            .read_schema(&catalog, &catalog.context(), &tlps)
            .await
            .unwrap(),
        [Value::UInt32(16000000), Value::Float(12.5)]
    );
    client.stop().await;
}

// REQ-PI-RAW: Table 3-17; ranges describe metadata, not a filter on device
// observations. Do not recalculate converted totals, rollover or rate locally.
#[tokio::test]
async fn pulse_raw_values_preserve_unknown_codes_out_of_range_counts_and_float_bits() {
    let peer = DiscoveryFixture::start(0).await;
    peer.peer.parameter_values.lock().unwrap().extend([
        ((105, 80, 4), vec![255; 4]),
        ((105, 80, 5), vec![255]),
        ((105, 80, 7), vec![255]),
        ((105, 80, 8), vec![254]),
        ((105, 80, 11), vec![253]),
        ((105, 80, 23), vec![0x20]),
        ((105, 80, 9), vec![0; 4]), // nonzero write constraint must not prevent reading zero
        ((105, 80, 10), vec![0x34, 0x12, 0xc0, 0x7f]), // NaN payload
        ((105, 80, 24), vec![0, 0, 0x80, 0xff]), // -infinity
        ((105, 80, 25), vec![0, 0, 0, 0x80]), // negative zero
        ((105, 80, 26), vec![0, 0, 0x80, 0xbf]), // -1.0
    ]);
    let client = client(&peer, IoLayout::Compatibility160);
    let catalog = Roc800Catalog::from_discovery(
        client
            .discover_io(FirmwareVersion::new(1, 0).unwrap())
            .await
            .unwrap(),
    )
    .unwrap();
    let tlps: Vec<_> = [4, 5, 7, 8, 11, 23, 9, 10, 24, 25, 26]
        .into_iter()
        .map(|p| Tlp::new(105, 80, p))
        .collect();
    let values = client
        .read_schema(&catalog, &catalog.context(), &tlps)
        .await
        .unwrap();
    assert_eq!(
        &values[..6],
        &[
            Value::UInt32(u32::MAX),
            Value::UInt8(255),
            Value::UInt8(255),
            Value::UInt8(254),
            Value::UInt8(253),
            Value::Binary(0x20)
        ]
    );
    for (i, bits) in [
        (6, 0),
        (7, 0x7fc01234),
        (8, 0xff800000),
        (9, 0x80000000),
        (10, 0xbf800000),
    ] {
        let Value::Float(value) = values[i] else {
            panic!("expected raw FLOAT")
        };
        assert_eq!(value.to_bits(), bits);
    }
    client.stop().await;
}
