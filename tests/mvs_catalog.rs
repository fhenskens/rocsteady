//! Independent evidence for D301180X012 §3.4.19/Table 3-20, printed pp.3-82–89.
include!("../examples/support/discovery_fixture.rs");
use rocsteady::{
    Access, Client, ClientConfig, DataType, Error, FirmwareVersion, IoLayout, ReadPlan,
    Roc800Catalog, Roc800Profile, SchemaProvider, Tlp, Value, WritePlan,
};
fn client(peer: &DiscoveryFixture, layout: IoLayout) -> Client {
    Client::tcp(
        &peer.endpoint,
        ClientConfig {
            profile: Arc::new(Roc800Profile { layout }),
            heartbeat_interval: None,
            call_timeout: std::time::Duration::from_secs(3),
            recovery: etherbird::Config {
                retry_delay: std::time::Duration::from_millis(1),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap()
}
fn value(p: u8) -> Value {
    match p {
        0 => Value::Ascii("MVS SAMPLE".into()),
        1 | 7 => Value::UInt8(255),
        2 => Value::UInt8(5),
        3..=6 | 9 | 25..=27 | 40..=42 | 55..=57 | 62 | 63 | 68 => Value::UInt8(1),
        10 | 11 | 28 | 43 | 58 => Value::Binary(255),
        8 => Value::Float(1.5),
        59 | 60 => Value::UInt8(7),
        65 => Value::Float(1.0),
        67 => Value::UInt16(0x1234),
        12..=24 | 29..=39 | 44..=54 | 61 | 64 | 66 => Value::Float(12.5),
        _ => panic!("unexpected MVS field"),
    }
}
fn version(p: u8) -> (u8, u8) {
    match p {
        64 | 65 => (2, 2),
        66 => (3, 20),
        67 | 68 => (3, 60),
        _ => (1, 0),
    }
}
#[test]
fn mvs_table_accounts_for_all_widths_access_versions_ranges_units_and_source_caveats() {
    // REQ-MVS-TABLE: every source field, including FLOAT /65, UINT16 /67 and discontinuous codes.
    assert_eq!(
        Roc800Catalog::entries()
            .iter()
            .filter(|e| e.point_type == 108)
            .count(),
        69
    );
    assert!(Roc800Catalog::entry(108, 69).is_none());
    for p in 0..69 {
        let e = Roc800Catalog::entry(108, p).unwrap();
        let ty = match p {
            0 => DataType::Ascii(10),
            1..=7 | 9 | 25..=27 | 40..=42 | 55..=57 | 59 | 60 | 62 | 63 | 68 => DataType::UInt8,
            10 | 11 | 28 | 43 | 58 => DataType::Binary,
            67 => DataType::UInt16,
            _ => DataType::Float,
        };
        assert_eq!(e.data_type, ty);
        assert_eq!(e.data_type, value(p).data_type());
        assert_eq!(
            e.access,
            if matches!(p,7|8|10|11|13..=18|20|28|29..=34|43|44..=49|58|65|67|68) {
                Access::ReadOnly
            } else {
                Access::ReadWrite
            }
        );
        assert_eq!(e.introduced, version(p));
        assert_eq!(
            e.numeric_range,
            match p {
                1 | 7 => Some((0., 255.)),
                3..=6 | 9 | 25..=27 | 40..=42 | 55..=57 | 62 | 63 => Some((0., 1.)),
                60 => Some((0., 7.)),
                67 => Some((0., 65535.)),
                68 => Some((0., 4.)),
                _ => None,
            }
        );
        assert_eq!(e.metadata.units, if p == 8 { Some("V") } else { None });
        assert_eq!(
            e.metadata.write_caveat.is_some(),
            matches!(p, 2 | 59 | 64 | 65 | 67)
        );
        assert!(e.source.contains("3.4.19/Table 3-20"));
    }
    assert!(
        Roc800Catalog::entry(108, 59)
            .unwrap()
            .metadata
            .write_caveat
            .unwrap()
            .contains("code 7")
    );
    assert!(
        Roc800Catalog::entry(108, 65)
            .unwrap()
            .metadata
            .write_caveat
            .unwrap()
            .contains("FLOAT")
    );
}
#[test]
fn mvs_enums_and_all_alarm_bytes_preserve_noncontiguous_and_unknown_values() {
    // REQ-MVS-ENUMS / REQ-MVS-BITS: Tables 3-20's five distinct BIN masks and documented codes.
    for p in 0..69 {
        let expected: &[(u8, &str)] = match p {
            2 => &[
                (0, "off_scan"),
                (1, "normal"),
                (2, "input_freeze"),
                (4, "configuration"),
                (5, "set_tag_address"),
            ],
            3 => &[(0, "english"), (1, "metric")],
            4 => &[(0, "water_60_f"), (1, "water_68_f")],
            5 => &[(0, "downstream"), (1, "upstream")],
            6 => &[(0, "retain_last"), (1, "use_fault_value")],
            9 | 25..=27 | 40..=42 | 55..=57 | 62 | 63 => &[(0, "disabled"), (1, "enabled")],
            59 => &[
                (0, "no_action"),
                (1, "calibrate_dp"),
                (2, "calibrate_sp"),
                (3, "calibrate_temperature"),
                (6, "save_calibration"),
                (7, "set_defaults"),
            ],
            60 => &[
                (0, "none"),
                (1, "zero"),
                (2, "span"),
                (3, "midpoint_1"),
                (4, "midpoint_2"),
                (5, "midpoint_3"),
                (6, "sensor_setup"),
                (7, "sensor_restore"),
            ],
            68 => &[
                (0, "undefined"),
                (1, "4088_a"),
                (2, "4088_b"),
                (3, "3095"),
                (4, "mvs205"),
            ],
            _ => &[],
        };
        let m = Roc800Catalog::entry(108, p).unwrap().metadata;
        assert_eq!(
            m.enumeration
                .iter()
                .map(|v| (v.value, v.key))
                .collect::<Vec<_>>(),
            expected
        );
        for raw in 0..=255u8 {
            assert_eq!(
                m.enum_meaning(raw).map(|v| v.key),
                expected.iter().find(|(v, _)| *v == raw).map(|(_, k)| *k)
            );
        }
    }
    for p in [10, 11, 28, 43, 58] {
        let (expected, unused): (&[(u8, &str)], u8) = match p {
            10 => (
                &[
                    (16, "input_freeze"),
                    (32, "sensor_communication_failure"),
                    (64, "rs485_communication_failure"),
                    (128, "off_scan"),
                ],
                0x0f,
            ),
            11 => (
                &[
                    (1, "dp_below_zero"),
                    (2, "sp_below_zero"),
                    (4, "temperature_below_zero"),
                    (8, "dp_above_span"),
                    (16, "sp_above_span"),
                    (32, "temperature_above_span"),
                ],
                0xc0,
            ),
            _ => (&[(1, "low"), (4, "high"), (64, "point_fail")], 0xba),
        };
        let m = Roc800Catalog::entry(108, p).unwrap().metadata;
        assert_eq!(m.unused_bits, unused);
        assert_eq!(
            m.bits.iter().map(|b| (b.mask, b.key)).collect::<Vec<_>>(),
            expected
        );
        for raw in 0..=255u8 {
            assert_eq!(
                m.active_bits(raw).map(|b| b.key).collect::<Vec<_>>(),
                expected
                    .iter()
                    .filter(|(bit, _)| raw & bit != 0)
                    .map(|(_, k)| *k)
                    .collect::<Vec<_>>()
            );
            assert_eq!(m.uninterpreted_bits(raw), raw & unused);
        }
    }
}
#[tokio::test]
async fn mvs_every_field_reads_at_firmware_boundaries_with_explicit_sensor_applicability() {
    // REQ-MVS-READS / POL-MVS-4088 / POL-MVS-NO-WRITES: no subtype inferred from firmware or parser success.
    for (status, layout, stride) in [
        (0, IoLayout::Compatibility160, 16),
        (1, IoLayout::Compatibility240, 16),
        (2, IoLayout::Expanded224, 8),
    ] {
        let peer = DiscoveryFixture::start(status).await;
        let client = client(&peer, layout);
        let location = 8 * stride;
        for (major, minor) in [(1, 0), (2, 1), (2, 2), (3, 19), (3, 20), (3, 59), (3, 60)] {
            if major == 1 && status != 0 {
                continue;
            }
            let io = client
                .discover_io(FirmwareVersion::new(major, minor).unwrap())
                .await
                .unwrap();
            assert_eq!(
                io.by_logical(108, 0).unwrap().tlp(19).unwrap(),
                Tlp::new(108, location, 19)
            );
            assert_eq!(io.configuration().point_count(108), Some(1));
            let basic = Roc800Catalog::from_discovery(io.clone()).unwrap();
            assert!(
                basic
                    .lookup(&basic.context(), Tlp::new(108, location, 67))
                    .is_err()
            );
            assert!(
                Roc800Catalog::from_discovery(io.clone())
                    .unwrap()
                    .with_mvs_4088(0)
                    .is_err()
            );
            let catalog = if (major, minor) >= (3, 60) {
                basic.with_mvs_4088(location).unwrap()
            } else {
                assert!(
                    Roc800Catalog::from_discovery(io)
                        .unwrap()
                        .with_mvs_4088(location)
                        .is_err()
                );
                basic
            };
            let mut tlps = Vec::new();
            let mut expected = Vec::new();
            for p in 0..69 {
                let tlp = Tlp::new(108, location, p);
                if version(p) <= (major, minor) {
                    tlps.push(tlp);
                    expected.push(value(p));
                } else {
                    assert!(catalog.lookup(&catalog.context(), tlp).is_err());
                }
                assert!(catalog.lookup_write(&catalog.context(), tlp).is_err());
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
                Tlp::new(108, 0, 0),
                Tlp::new(108, stride, 19),
                Tlp::new(108, 255, 50),
                Tlp::new(108, location, 69),
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
            for p in 0..69 {
                if version(p) > (major, minor) {
                    assert!(
                        client
                            .read_schema(
                                &catalog,
                                &catalog.context(),
                                &[tlps[0], Tlp::new(108, location, p)]
                            )
                            .await
                            .unwrap_err()
                            .progress
                            .values
                            .is_empty()
                    );
                }
            }
            assert!(
                WritePlan::build_arbitrary(
                    &catalog,
                    &catalog.context(),
                    &[(Tlp::new(108, location, 19), Value::Float(1.))],
                    client.profile()
                )
                .is_err()
            );
            assert_eq!(peer.peer.parameter_reads.load(Ordering::SeqCst), reads);
        }
        client.stop().await;
    }
    assert!(
        Roc800Catalog::new(FirmwareVersion::new(3, 60).unwrap())
            .unwrap()
            .with_mvs_4088(128)
            .is_err()
    );
}
#[tokio::test]
async fn mvs_partial_progress_and_rejections_preserve_generation_and_no_replay() {
    // POL-MVS-PROGRESS / POL-MVS-RETIREMENT: independent TCP, raw device errors vs malformed second batch.
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer, IoLayout::Compatibility160);
    let catalog = Roc800Catalog::from_discovery(
        client
            .discover_io(FirmwareVersion::new(3, 60).unwrap())
            .await
            .unwrap(),
    )
    .unwrap()
    .with_mvs_4088(128)
    .unwrap();
    let tlps: Vec<_> = (0..69).map(|p| Tlp::new(108, 128, p)).collect();
    let plan = ReadPlan::build(&catalog, &catalog.context(), &tlps, client.profile()).unwrap();
    assert_eq!(
        plan.batches().iter().map(Vec::len).collect::<Vec<_>>(),
        [39, 30]
    );
    let prefix = 39;
    let reads = peer.peer.parameter_reads.load(Ordering::SeqCst);
    peer.peer
        .parameter_fault_at
        .store(reads + 2, Ordering::SeqCst);
    peer.peer.parameter_fault.store(1, Ordering::SeqCst);
    let failure = client.read_plan(plan.clone()).await.unwrap_err();
    assert_eq!(failure.progress.completed_batches, 1);
    assert_eq!(
        failure.progress.values,
        (0..prefix as u8).map(value).collect::<Vec<_>>()
    );
    client.read_clock().await.unwrap();
    assert!(matches!(
        client.read_plan(plan).await.unwrap_err().error,
        etherbird::Error::Operation(Error::StaleGeneration { .. })
    ));
    assert_eq!(peer.peer.parameter_reads.load(Ordering::SeqCst), reads + 2);
    let basic = Roc800Catalog::from_discovery(
        client
            .discover_io(FirmwareVersion::new(3, 60).unwrap())
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(
        client
            .read_schema(&basic, &basic.context(), &[Tlp::new(108, 128, 67)])
            .await
            .is_err()
    ); // Assertion did not survive replacement.
    let counter = basic.with_mvs_4088(128).unwrap();
    peer.peer.fault_opcode.store(180, Ordering::SeqCst);
    peer.peer.fault.store(3, Ordering::SeqCst);
    assert!(matches!(
        client
            .read_schema(&counter, &counter.context(), &tlps[..2])
            .await
            .unwrap_err()
            .error,
        etherbird::Error::Operation(Error::Device(_))
    ));
    assert_eq!(
        client
            .read_schema(&counter, &counter.context(), &tlps[..2])
            .await
            .unwrap(),
        [value(0), value(1)]
    );
    client.stop().await;
}
#[tokio::test]
async fn mvs_unknown_codes_raw_float_type_and_unused_bits_remain_lossless() {
    // REQ-MVS-RAW: do not coerce FLOAT /65 to byte enums or invent unit conversions.
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer, IoLayout::Compatibility160);
    peer.peer.parameter_values.lock().unwrap().extend([
        ((108, 128, 2), vec![3]),
        ((108, 128, 3), vec![255]),
        ((108, 128, 59), vec![7]),
        ((108, 128, 68), vec![254]),
        ((108, 128, 65), vec![0x34, 0x12, 0xc0, 0x7f]),
        ((108, 128, 19), vec![0, 0, 0, 0x80]),
        ((108, 128, 35), vec![0, 0, 0x80, 0x7f]),
        ((108, 128, 10), vec![15]),
        ((108, 128, 11), vec![192]),
        ((108, 128, 28), vec![186]),
    ]);
    let catalog = Roc800Catalog::from_discovery(
        client
            .discover_io(FirmwareVersion::new(3, 60).unwrap())
            .await
            .unwrap(),
    )
    .unwrap();
    let fields = [2, 3, 59, 68, 65, 19, 35, 10, 11, 28];
    let values = client
        .read_schema(
            &catalog,
            &catalog.context(),
            &fields.map(|p| Tlp::new(108, 128, p)),
        )
        .await
        .unwrap();
    assert_eq!(
        &values[..4],
        &[
            Value::UInt8(3),
            Value::UInt8(255),
            Value::UInt8(7),
            Value::UInt8(254)
        ]
    );
    for (index, bits) in [(4, 0x7fc01234), (5, 0x80000000), (6, 0x7f800000)] {
        let Value::Float(v) = values[index] else {
            panic!("FLOAT expected")
        };
        assert_eq!(v.to_bits(), bits);
    }
    assert_eq!(
        &values[7..],
        &[Value::Binary(15), Value::Binary(192), Value::Binary(186)]
    );
    client.stop().await;
}
#[cfg(feature = "simulator")]
#[tokio::test]
async fn simulator_reads_all_mvs_fields_with_explicit_4088_context() {
    // REQ-MVS-SIMULATOR: all 69 source widths through actual TCP in every layout.
    use rocsteady::simulator::{Model, Simulator};
    for layout in [
        IoLayout::Compatibility160,
        IoLayout::Compatibility240,
        IoLayout::Expanded224,
    ] {
        let mut model = Model::default();
        model.set_io_layout(layout);
        model.io_point_types[128] = 108;
        for p in 0..69 {
            model.parameters.insert(Tlp::new(108, 128, p), value(p));
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
        .unwrap()
        .with_mvs_4088(128)
        .unwrap();
        assert_eq!(
            client
                .read_schema(
                    &catalog,
                    &catalog.context(),
                    &(0..69).map(|p| Tlp::new(108, 128, p)).collect::<Vec<_>>()
                )
                .await
                .unwrap(),
            (0..69).map(value).collect::<Vec<_>>()
        );
        client.stop().await;
        server.stop().await.unwrap();
    }
}
