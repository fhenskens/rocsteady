//! Source-audited read contracts: D301180X012 §§3.4.17/18, Tables 3-18/19, pp.3-75–81.
include!("../examples/support/discovery_fixture.rs");
use rocsteady::{
    Access, Client, ClientConfig, Error, FirmwareVersion, IoLayout, ReadPlan, Roc800Catalog,
    Roc800Profile, SchemaProvider, Tlp, Value, WritePlan,
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
fn value(pt: u8, p: u8) -> Value {
    match (pt, p) {
        (106, 0) => Value::Ascii("RTD SAMPLE".into()),
        (107, 0) => Value::Ascii("TC SAMPLE ".into()),
        (106 | 107, 1) => Value::Ascii("C         ".into()),
        (106, 2 | 6 | 7 | 23 | 30..=32) | (107, 2 | 3 | 8 | 16..=18 | 21) => Value::UInt8(1),
        (106, 3 | 4) | (107, 5 | 6) => Value::Float(1.0),
        (106, 5) | (107, 7) => Value::UInt8(99),
        (106, 8..=13) => Value::UInt16(0x1234),
        (106, 14..=22) => Value::Float(-10.0),
        (106, 24..=29) | (107, 9..=15 | 20 | 22) => Value::Float(12.5),
        (106, 33) | (107, 19) => Value::Binary(255),
        (106, 34) => Value::Float(3600.0),
        (106, 35 | 37) => Value::UInt8(4),
        (106, 36) => Value::UInt8(6),
        (107, 4) => Value::UInt8(10),
        _ => panic!("unexpected temperature row"),
    }
}
fn introduced(pt: u8, p: u8) -> (u8, u8) {
    match (pt, p) {
        (106, 19) => (3, 0),
        (107, 21 | 22) => (3, 50),
        _ => (1, 0),
    }
}

#[test]
fn temperature_tables_account_for_every_width_access_range_unit_and_caveat() {
    // REQ-TEMP-TABLE: all 38 RTD and 23 TC fields; metadata never authorizes writes.
    for (pt, count) in [(106, 38), (107, 23)] {
        assert_eq!(
            Roc800Catalog::entries()
                .iter()
                .filter(|e| e.point_type == pt)
                .count(),
            count as usize
        );
        assert!(Roc800Catalog::entry(pt, count).is_none());
        for p in 0..count {
            let e = Roc800Catalog::entry(pt, p).unwrap();
            assert_eq!(e.data_type, value(pt, p).data_type());
            assert_eq!(
                e.access,
                if matches!((pt, p), (106, 4 | 8..=18 | 21 | 33 | 34) | (107, 6 | 19)) {
                    Access::ReadOnly
                } else {
                    Access::ReadWrite
                }
            );
            assert_eq!(e.introduced, introduced(pt, p));
            assert_eq!(
                e.numeric_range,
                match (pt, p) {
                    (106, 2 | 6 | 7 | 23 | 30..=32) | (107, 2 | 8 | 16..=18 | 21) => Some((0., 1.)),
                    (106, 3) => Some((0.066, 43200.)),
                    (106, 4) => Some((0.05, 43200.)),
                    (106, 5) | (107, 7) => Some((0., 99.)),
                    (106, 8..=13) => Some((0., 65535.)),
                    (106, 34) => Some((0., 3600.)),
                    (106, 35 | 37) => Some((0., 4.)),
                    (106, 36) => Some((0., 6.)),
                    (107, 3) => Some((0., 3.)),
                    (107, 5) => Some((0.1, 43200.)),
                    (107, 6) => Some((0., 43200.)),
                    _ => None,
                }
            );
            assert_eq!(
                e.metadata.units,
                match (pt, p) {
                    (106, 3 | 4 | 34) | (107, 5 | 6) => Some("s"),
                    (106, 5) | (107, 7) => Some("%"),
                    _ => None,
                }
            );
            assert_eq!(
                e.metadata.write_caveat.is_some(),
                matches!(
                    (pt, p),
                    (106, 2 | 3 | 5 | 6 | 7 | 20 | 35 | 36) | (107, 4 | 10 | 13..=15 | 22)
                )
            );
            assert!(e.source.contains(if pt == 106 {
                "3.4.17/Table 3-18"
            } else {
                "3.4.18/Table 3-19"
            }));
        }
    }
    assert!(
        Roc800Catalog::entry(106, 3)
            .unwrap()
            .metadata
            .write_caveat
            .unwrap()
            .contains("R/W_CNDL")
    );
    assert!(
        Roc800Catalog::entry(106, 7)
            .unwrap()
            .metadata
            .write_caveat
            .unwrap()
            .contains("R/W_LOG")
    );
    assert!(
        Roc800Catalog::entry(106, 20)
            .unwrap()
            .metadata
            .write_caveat
            .unwrap()
            .contains("units and alpha")
    );
    assert!(
        Roc800Catalog::entry(107, 22)
            .unwrap()
            .metadata
            .write_caveat
            .unwrap()
            .contains("FLOAT")
    );
}
#[test]
fn temperature_enum_codes_and_alarm_bits_preserve_every_unknown_byte() {
    // REQ-TEMP-ENUMS / REQ-TEMP-ALARMS: source enums and BIN fields, including TC code gap 7.
    let enabled = [(0, "disabled"), (1, "enabled")];
    let alpha = [(0, "alpha_0_00385"), (1, "alpha_0_00392")];
    let units = [
        (0, "fahrenheit"),
        (1, "celsius"),
        (2, "kelvin"),
        (3, "rankine"),
    ];
    let rtd_units = [
        (0, "fahrenheit"),
        (1, "celsius"),
        (2, "kelvin"),
        (3, "rankine"),
        (4, "ohms"),
    ];
    let calibration = [
        (0, "current"),
        (1, "start"),
        (2, "calibrate"),
        (3, "restore_previous"),
        (4, "stop"),
    ];
    let cal_type = [
        (0, "nothing"),
        (1, "zero"),
        (2, "span"),
        (3, "midpoint_1"),
        (4, "midpoint_2"),
        (5, "midpoint_3"),
        (6, "unused"),
    ];
    let sensor = [
        (0, "type_j"),
        (1, "type_k"),
        (2, "type_b"),
        (3, "type_e"),
        (4, "type_r"),
        (5, "type_s"),
        (6, "type_t"),
        (8, "type_10"),
        (9, "type_c"),
        (10, "type_n"),
    ];
    for (pt, count) in [(106, 38), (107, 23)] {
        for p in 0..count {
            let expected: &[(u8, &str)] = match (pt, p) {
                (106, 2 | 6 | 23 | 30..=32) | (107, 2 | 8 | 16..=18 | 21) => &enabled,
                (106, 7) => &alpha,
                (106, 35) => &calibration,
                (106, 36) => &cal_type,
                (106, 37) => &rtd_units,
                (107, 3) => &units,
                (107, 4) => &sensor,
                _ => &[],
            };
            let m = Roc800Catalog::entry(pt, p).unwrap().metadata;
            assert_eq!(
                m.enumeration
                    .iter()
                    .map(|e| (e.value, e.key))
                    .collect::<Vec<_>>(),
                expected
            );
            for raw in 0..=255u8 {
                assert_eq!(
                    m.enum_meaning(raw).map(|v| v.key),
                    expected
                        .iter()
                        .find(|(v, _)| *v == raw)
                        .map(|(_, label)| *label)
                );
            }
        }
    }
    for (pt, p) in [(106, 33), (107, 19)] {
        let m = Roc800Catalog::entry(pt, p).unwrap().metadata;
        let expected = [
            (1, "low"),
            (2, "low_low"),
            (4, "high"),
            (8, "high_high"),
            (16, "rate"),
            (64, "point_fail"),
            (128, "scanning_disabled"),
        ];
        assert_eq!(
            m.bits.iter().map(|b| (b.mask, b.key)).collect::<Vec<_>>(),
            expected
        );
        assert_eq!(m.unused_bits, 0x20);
        for raw in 0..=255u8 {
            assert_eq!(
                m.active_bits(raw).map(|b| b.key).collect::<Vec<_>>(),
                expected
                    .iter()
                    .filter(|(bit, _)| raw & bit != 0)
                    .map(|(_, label)| *label)
                    .collect::<Vec<_>>()
            );
            assert_eq!(m.uninterpreted_bits(raw), raw & 0x20);
        }
    }
}
#[tokio::test]
async fn temperature_reads_every_field_with_firmware_location_and_write_gates() {
    // REQ-TEMP-READS / REQ-TEMP-FIRMWARE / POL-TEMP-NO-WRITES: all source fields over independent TCP.
    for (status, layout, stride) in [
        (0, IoLayout::Compatibility160, 16),
        (1, IoLayout::Compatibility240, 16),
        (2, IoLayout::Expanded224, 8),
    ] {
        let peer = DiscoveryFixture::start(status).await;
        let client = client(&peer, layout);
        for (major, minor) in [
            (1, 0),
            (2, 99),
            (3, 0),
            (3, 39),
            (3, 40),
            (3, 49),
            (3, 50),
            (3, 60),
        ] {
            if major == 1 && status != 0 {
                continue;
            }
            let firmware = FirmwareVersion::new(major, minor).unwrap();
            let io = client.discover_io(firmware).await.unwrap();
            for (pt, multiplier) in [(106, 6), (107, 7)] {
                assert_eq!(
                    io.by_logical(pt, 0).unwrap().tlp(0).unwrap(),
                    Tlp::new(pt, multiplier * stride, 0)
                );
                assert_eq!(io.configuration().point_count(pt), Some(1));
            }
            let catalog = Roc800Catalog::from_discovery(io).unwrap();
            for (pt, count, multiplier) in [(106, 38, 6), (107, 23, 7)] {
                let mut tlps = Vec::new();
                let mut expected = Vec::new();
                for p in 0..count {
                    let tlp = Tlp::new(pt, multiplier * stride, p);
                    if introduced(pt, p) <= (major, minor) {
                        tlps.push(tlp);
                        expected.push(value(pt, p));
                    } else {
                        assert!(catalog.lookup(&catalog.context(), tlp).is_err());
                    }
                    // Every temperature field is blocked for both write protocols, including tags.
                    if introduced(pt, p) <= (major, minor) {
                        assert!(catalog.lookup_write(&catalog.context(), tlp).is_err());
                        assert!(
                            WritePlan::build(
                                &catalog,
                                &catalog.context(),
                                &[(tlp, value(pt, p))],
                                client.profile()
                            )
                            .is_err()
                        );
                        assert!(
                            WritePlan::build_arbitrary(
                                &catalog,
                                &catalog.context(),
                                &[(tlp, value(pt, p))],
                                client.profile()
                            )
                            .is_err()
                        );
                    }
                }
                assert_eq!(
                    client
                        .read_schema(&catalog, &catalog.context(), &tlps)
                        .await
                        .unwrap(),
                    expected
                );
                let reads = peer.peer.parameter_reads.load(Ordering::SeqCst);
                for invalid in [
                    Tlp::new(pt, 0, 0),
                    Tlp::new(pt, stride, 0),
                    Tlp::new(pt, 255, 0),
                    Tlp::new(pt, multiplier * stride, count),
                ] {
                    assert!(
                        client
                            .read_schema(&catalog, &catalog.context(), &[tlps[0], invalid])
                            .await
                            .unwrap_err()
                            .progress
                            .values
                            .is_empty()
                    );
                }
                for p in 0..count {
                    if introduced(pt, p) > (major, minor) {
                        assert!(
                            client
                                .read_schema(
                                    &catalog,
                                    &catalog.context(),
                                    &[tlps[0], Tlp::new(pt, multiplier * stride, p)]
                                )
                                .await
                                .unwrap_err()
                                .progress
                                .values
                                .is_empty()
                        );
                    }
                }
                assert_eq!(peer.peer.parameter_reads.load(Ordering::SeqCst), reads);
                assert!(catalog.lookup(&Default::default(), tlps[0]).is_err());
            }
        }
        client.stop().await;
    }
    let unbound = Roc800Catalog::new(FirmwareVersion::new(3, 60).unwrap()).unwrap();
    for pt in [106, 107] {
        assert!(
            unbound
                .lookup(&unbound.context(), Tlp::new(pt, 0, 0))
                .is_err()
        );
    }
}
#[tokio::test]
async fn temperature_malformed_replies_and_replacements_preserve_lifecycle_contracts() {
    // POL-TEMP-RETIREMENT / POL-TEMP-NO-REPLAY: Table 2-25 and managed generation contract.
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer, IoLayout::Compatibility160);
    let tlps = [Tlp::new(106, 96, 22), Tlp::new(107, 112, 9)];
    let mut connections = 1;
    for fault in [3, 1, 2, 6, 7, 8, 4] {
        let catalog = Roc800Catalog::from_discovery(
            client
                .discover_io(FirmwareVersion::new(3, 60).unwrap())
                .await
                .unwrap(),
        )
        .unwrap();
        let plan = ReadPlan::build(&catalog, &catalog.context(), &tlps, client.profile()).unwrap();
        let reads = peer.peer.parameter_reads.load(Ordering::SeqCst);
        peer.peer.fault_opcode.store(180, Ordering::SeqCst);
        peer.peer.fault.store(fault, Ordering::SeqCst);
        let failure = client.read_plan(plan.clone()).await.unwrap_err();
        assert!(failure.progress.values.is_empty());
        if fault == 3 {
            assert!(matches!(
                failure.error,
                etherbird::Error::Operation(Error::Device(_))
            ));
            assert_eq!(
                client.read_plan(plan).await.unwrap(),
                [Value::Float(-10.0), Value::Float(12.5)]
            );
            assert_eq!(peer.peer.parameter_reads.load(Ordering::SeqCst), reads + 2);
        } else {
            client.read_clock().await.unwrap();
            connections += 1;
            assert!(matches!(
                client.read_plan(plan).await.unwrap_err().error,
                etherbird::Error::Operation(Error::StaleGeneration { .. })
            ));
            assert_eq!(peer.peer.parameter_reads.load(Ordering::SeqCst), reads + 1);
        }
        assert_eq!(peer.peer.connections.load(Ordering::SeqCst), connections);
    }
    client.stop().await;
}
#[tokio::test]
async fn temperature_raw_codes_unused_bits_and_ieee_payloads_remain_readable() {
    // REQ-TEMP-RAW: metadata ranges and unit labels never filter/convert observations.
    let peer = DiscoveryFixture::start(0).await;
    peer.peer.parameter_values.lock().unwrap().extend([
        ((106, 96, 7), vec![255]),
        ((106, 96, 37), vec![254]),
        ((107, 112, 4), vec![7]),
        ((106, 96, 33), vec![0x20]),
        ((107, 112, 19), vec![0xff]),
        ((106, 96, 22), vec![0x34, 0x12, 0xc0, 0x7f]),
        ((107, 112, 9), vec![0, 0, 0x80, 0xff]),
        ((106, 96, 3), vec![0, 0, 0, 0x80]),
        ((107, 112, 7), vec![255]),
    ]);
    let client = client(&peer, IoLayout::Compatibility160);
    let catalog = Roc800Catalog::from_discovery(
        client
            .discover_io(FirmwareVersion::new(3, 60).unwrap())
            .await
            .unwrap(),
    )
    .unwrap();
    let tlps = [
        Tlp::new(106, 96, 7),
        Tlp::new(106, 96, 37),
        Tlp::new(107, 112, 4),
        Tlp::new(106, 96, 33),
        Tlp::new(107, 112, 19),
        Tlp::new(106, 96, 22),
        Tlp::new(107, 112, 9),
        Tlp::new(106, 96, 3),
        Tlp::new(107, 112, 7),
    ];
    let values = client
        .read_schema(&catalog, &catalog.context(), &tlps)
        .await
        .unwrap();
    assert_eq!(
        &values[..5],
        &[
            Value::UInt8(255),
            Value::UInt8(254),
            Value::UInt8(7),
            Value::Binary(0x20),
            Value::Binary(255)
        ]
    );
    for (index, bits) in [(5, 0x7fc01234), (6, 0xff800000), (7, 0x80000000)] {
        let Value::Float(v) = values[index] else {
            panic!("FLOAT expected")
        };
        assert_eq!(v.to_bits(), bits);
    }
    assert_eq!(values[8], Value::UInt8(255));
    client.stop().await;
}

#[cfg(feature = "simulator")]
#[tokio::test]
async fn simulator_reads_all_temperature_fields_across_layouts() {
    // REQ-TEMP-SIMULATOR: independently specified values for every field; real loopback TCP.
    use rocsteady::simulator::{Model, Simulator};
    for layout in [
        IoLayout::Compatibility160,
        IoLayout::Compatibility240,
        IoLayout::Expanded224,
    ] {
        let mut model = Model::default();
        model.set_io_layout(layout);
        model.io_point_types[96] = 106;
        model.io_point_types[112] = 107;
        let mut tlps = Vec::new();
        let mut expected = Vec::new();
        for (pt, count, location) in [(106, 38, 96), (107, 23, 112)] {
            for p in 0..count {
                let tlp = Tlp::new(pt, location, p);
                let v = value(pt, p);
                model.parameters.insert(tlp, v.clone());
                tlps.push(tlp);
                expected.push(v);
            }
        }
        let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), model)
            .await
            .unwrap();
        let client = Client::tcp(
            server.endpoint().to_string(),
            ClientConfig {
                profile: Arc::new(Roc800Profile { layout }),
                heartbeat_interval: None,
                ..Default::default()
            },
        )
        .unwrap();
        let catalog = Roc800Catalog::from_discovery(
            client
                .discover_io(FirmwareVersion::new(3, 60).unwrap())
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            client
                .read_schema(&catalog, &catalog.context(), &tlps)
                .await
                .unwrap(),
            expected
        );
        client.stop().await;
        server.stop().await.unwrap();
    }
}

#[tokio::test]
async fn temperature_failed_second_batch_returns_confirmed_prefix_without_resume() {
    // POL-TEMP-PROGRESS: two Table 2-25 batches; malformed second batch never returns partial values.
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer, IoLayout::Compatibility160);
    let catalog = Roc800Catalog::from_discovery(
        client
            .discover_io(FirmwareVersion::new(3, 60).unwrap())
            .await
            .unwrap(),
    )
    .unwrap();
    let tlps: Vec<_> = (0..38)
        .map(|p| Tlp::new(106, 96, p))
        .chain((0..23).map(|p| Tlp::new(107, 112, p)))
        .collect();
    let plan = ReadPlan::build(&catalog, &catalog.context(), &tlps, client.profile()).unwrap();
    assert_eq!(
        plan.batches().iter().map(Vec::len).collect::<Vec<_>>(),
        [38, 23]
    );
    let reads = peer.peer.parameter_reads.load(Ordering::SeqCst);
    peer.peer
        .parameter_fault_at
        .store(reads + 2, Ordering::SeqCst);
    peer.peer.parameter_fault.store(1, Ordering::SeqCst);
    let failure = client.read_plan(plan.clone()).await.unwrap_err();
    assert_eq!(failure.progress.completed_batches, 1);
    assert_eq!(
        failure.progress.values,
        (0..38).map(|p| value(106, p)).collect::<Vec<_>>()
    );
    client.read_clock().await.unwrap();
    assert!(matches!(
        client.read_plan(plan).await.unwrap_err().error,
        etherbird::Error::Operation(Error::StaleGeneration { .. })
    ));
    assert_eq!(peer.peer.parameter_reads.load(Ordering::SeqCst), reads + 2);
    client.stop().await;
}
