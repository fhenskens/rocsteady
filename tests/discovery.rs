include!("../examples/support/discovery_fixture.rs");
use rocsteady::{
    Access, Capability, Client, ClientConfig, DataType, DecodeError, Error, FirmwareVersion,
    IoLayout, IoPointCodec, IoPointRequest, IoSelector, PointTypeClassification, PointTypeCodec,
    PointTypeRequest, ProtocolProfile, ReplyCodec, Roc800Catalog, Roc800Profile, SchemaContext,
    SchemaProvider, Tlp, Value,
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
fn firmware() -> FirmwareVersion {
    FirmwareVersion::new(3, 0).unwrap()
}

#[test]
fn table_2_13_literals_validate_command_start_count_and_classifications() {
    let request = PointTypeRequest::new(60, 5).unwrap();
    assert_eq!(request.payload(), [11, 60, 5]);
    let literal = [11, 60, 5, 0, 7, 253, 254, 255];
    let page = PointTypeCodec.decode(&request, &literal).unwrap();
    assert_eq!(
        page.classifications(),
        [
            PointTypeClassification::UserProgram(0),
            PointTypeClassification::UserProgram(7),
            PointTypeClassification::UserDefined,
            PointTypeClassification::Roc,
            PointTypeClassification::Absent
        ]
    );
    assert_eq!(page.get(59), None);
    assert_eq!(page.get(65), None);
    assert_eq!(page.get(63), Some(PointTypeClassification::Roc));
    assert_eq!(
        PointTypeCodec
            .decode(&PointTypeRequest::new(255, 1).unwrap(), &[11, 255, 1, 42])
            .unwrap()
            .classifications(),
        [PointTypeClassification::Unknown(42)]
    );
    assert!(
        PointTypeCodec
            .decode(&PointTypeRequest::new(255, 0).unwrap(), &[11, 255, 0])
            .unwrap()
            .classifications()
            .is_empty()
    );
    for length in 0..literal.len() {
        assert!(PointTypeCodec.decode(&request, &literal[..length]).is_err());
    }
    let mut trailing = literal.to_vec();
    trailing.push(0);
    assert!(PointTypeCodec.decode(&request, &trailing).is_err());
    for index in 0..3 {
        let mut bad = literal;
        bad[index] ^= 1;
        assert!(PointTypeCodec.decode(&request, &bad).is_err());
    }
    for (start, count) in [(0, 246), (255, 2), (12, 245)] {
        assert!(PointTypeRequest::new(start, count).is_err());
    }
    assert!(PointTypeRequest::new(11, 245).is_ok());
}
#[test]
fn table_2_12_explicit_selector_lengths_preserve_absent_unknown_and_diagnostic_bytes() {
    for (layout, length) in [
        (IoLayout::Compatibility160, 160),
        (IoLayout::Compatibility240, 240),
        (IoLayout::Expanded224, 224),
    ] {
        for selector in [IoSelector::PointTypes, IoSelector::LogicalNumbers] {
            let request = IoPointRequest { layout, selector };
            assert_eq!(
                request.payload(),
                [if selector == IoSelector::PointTypes {
                    0
                } else {
                    1
                }]
            );
            let mut bytes = vec![0; length];
            bytes[..5].copy_from_slice(&[109, 109, 109, 109, 109]);
            bytes[length - 1] = 200;
            assert_eq!(
                IoPointCodec.decode(&request, &bytes).unwrap().as_bytes(),
                bytes
            );
            assert!(IoPointCodec.decode(&request, &bytes[..length - 1]).is_err());
            bytes.push(0);
            assert!(IoPointCodec.decode(&request, &bytes).is_err());
        }
    }
}
#[tokio::test]
async fn independent_tcp_pages_cover_256_types_and_zero_count_without_wrapping() {
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer, IoLayout::Compatibility160);
    let pages = client.scan_point_types(0, 256).await.unwrap();
    assert_eq!(
        pages
            .iter()
            .map(|p| (p.request().start(), p.request().count()))
            .collect::<Vec<_>>(),
        [(0, 245), (245, 11)]
    );
    assert_eq!(pages[0].get(60), Some(PointTypeClassification::UserDefined));
    assert_eq!(pages[0].get(61), Some(PointTypeClassification::Unknown(42)));
    assert_eq!(pages[0].get(91), Some(PointTypeClassification::Roc));
    assert_eq!(pages[1].get(255), Some(PointTypeClassification::Absent));
    assert!(
        client
            .read_point_types(PointTypeRequest::new(255, 0).unwrap())
            .await
            .unwrap()
            .classifications()
            .is_empty()
    );
    assert!(client.scan_point_types(255, 1).await.is_ok());
    let requests = peer.peer.requests.lock().unwrap().clone();
    assert!(requests.contains(&(100, vec![11, 0, 245])));
    assert!(requests.contains(&(100, vec![11, 245, 11])));
    client.stop().await;
}
struct BudgetProfile;
impl ProtocolProfile for BudgetProfile {
    fn metadata(&self) -> rocsteady::ProfileMetadata {
        rocsteady::DefaultProfile.metadata()
    }
    fn capability(&self, opcode: u8) -> Capability {
        rocsteady::DefaultProfile.capability(opcode)
    }
    fn limits(&self, opcode: u8) -> rocsteady::PayloadLimits {
        if opcode == 100 {
            rocsteady::PayloadLimits {
                request: 3,
                response: 5,
            }
        } else if opcode == 50 {
            rocsteady::PayloadLimits {
                request: 1,
                response: 159,
            }
        } else {
            rocsteady::PayloadLimits::default()
        }
    }
}
#[tokio::test]
async fn profile_budgets_split_pages_and_reject_impossible_io_before_channel_use() {
    let peer = DiscoveryFixture::start(0).await;
    let client = Client::tcp(
        &peer.endpoint,
        ClientConfig {
            profile: Arc::new(BudgetProfile),
            heartbeat_interval: None,
            ..Default::default()
        },
    )
    .unwrap();
    client.connected().await.unwrap();
    let pages = client.scan_point_types(250, 6).await.unwrap();
    assert_eq!(
        pages
            .iter()
            .map(|p| p.request().count())
            .collect::<Vec<_>>(),
        [2, 2, 2]
    );
    let before = peer.peer.requests.lock().unwrap().len();
    assert!(
        client
            .read_point_types(PointTypeRequest::new(0, 3).unwrap())
            .await
            .is_err()
    );
    assert!(
        client
            .discover_io_with_layout(firmware(), IoLayout::Compatibility160)
            .await
            .is_err()
    );
    assert!(client.scan_point_types(1, u16::MAX).await.is_err());
    assert!(client.scan_point_types(255, 2).await.is_err());
    assert_eq!(peer.peer.requests.lock().unwrap().len(), before);
    client.stop().await;
}
#[tokio::test]
async fn all_layouts_map_logicals_to_physical_tlps_and_preserve_unknown_points() {
    for (status, layout, stride, last) in [
        (0, IoLayout::Compatibility160, 16, 159),
        (1, IoLayout::Compatibility240, 16, 239),
        (2, IoLayout::Expanded224, 8, 223),
    ] {
        let peer = DiscoveryFixture::start(status).await;
        let client = client(&peer, layout);
        let snapshot = client.discover_io(firmware()).await.unwrap();
        assert_eq!(snapshot.point_types().len(), last + 1);
        assert_eq!(snapshot.logical_numbers().len(), last + 1);
        assert_eq!(snapshot.points().len(), 8); // Five CPU diagnostics, two field points, unknown.
        assert!(snapshot.by_location(5).is_none());
        assert_eq!(
            snapshot.by_logical(103, 0).unwrap().tlp(21).unwrap(),
            Tlp::new(103, stride, 21)
        );
        assert_eq!(
            snapshot.by_logical(101, 0).unwrap().tlp(3).unwrap(),
            Tlp::new(101, 2 * stride, 3)
        );
        assert_eq!(snapshot.by_logical(109, 4).unwrap().location, 4);
        let unknown = snapshot.by_location(last as u8).unwrap();
        assert_eq!(unknown.point_type, 200);
        assert_eq!(unknown.logical_number, 254);
        assert!(unknown.tlp(0).is_err());
        assert_eq!(
            peer.peer
                .requests
                .lock()
                .unwrap()
                .iter()
                .filter(|(op, _)| *op == 50)
                .map(|(_, data)| data.clone())
                .collect::<Vec<_>>(),
            [vec![0], vec![1], vec![0], vec![1]]
        );
        client.stop().await;
    }
}
#[tokio::test]
async fn observed_type_logical_or_identity_change_rejects_snapshot_without_retiring_healthy_peer() {
    for change in 0..3 {
        let peer = DiscoveryFixture::start(0).await;
        let client = client(&peer, IoLayout::Compatibility160);
        match change {
            0 => peer.peer.change_type_at.store(2, Ordering::SeqCst),
            1 => peer.peer.change_logical_at.store(2, Ordering::SeqCst),
            _ => peer.peer.change_config_at.store(2, Ordering::SeqCst),
        }
        assert!(matches!(
            client.discover_io(firmware()).await,
            Err(etherbird::Error::Operation(Error::DiscoveryChanged(_)))
        ));
        client.read_clock().await.unwrap();
        assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
        client.stop().await;
    }
}
#[tokio::test]
async fn malformed_service_replies_retire_but_valid_errors_preserve_connection() {
    for (opcode, fault) in [
        (100, 1),
        (100, 2),
        (100, 6),
        (100, 7),
        (100, 8),
        (50, 1),
        (50, 6),
    ] {
        let peer = DiscoveryFixture::start(0).await;
        let client = client(&peer, IoLayout::Compatibility160);
        client.connected().await.unwrap();
        peer.peer.fault_opcode.store(opcode, Ordering::SeqCst);
        peer.peer.fault.store(fault, Ordering::SeqCst);
        let result = if opcode == 100 {
            client
                .read_point_types(PointTypeRequest::new(91, 1).unwrap())
                .await
                .map(|_| ())
        } else {
            client
                .read_io_points(IoPointRequest {
                    layout: IoLayout::Compatibility160,
                    selector: IoSelector::PointTypes,
                })
                .await
                .map(|_| ())
        };
        assert!(matches!(
            result,
            Err(etherbird::Error::Operation(Error::InvalidResponse(_)))
        ));
        client.read_clock().await.unwrap();
        assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
        client.stop().await;
    }
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer, IoLayout::Compatibility160);
    client.connected().await.unwrap();
    for opcode in [50, 100] {
        peer.peer.fault_opcode.store(opcode, Ordering::SeqCst);
        peer.peer.fault.store(3, Ordering::SeqCst);
        let result = if opcode == 100 {
            client
                .read_point_types(PointTypeRequest::new(91, 1).unwrap())
                .await
                .map(|_| ())
        } else {
            client.discover_io(firmware()).await.map(|_| ())
        };
        assert!(matches!(
            result,
            Err(etherbird::Error::Operation(Error::Device(_)))
        ));
        client.read_clock().await.unwrap();
        assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
    }
    client.stop().await;
}
#[tokio::test]
async fn configured_layout_mismatch_fails_without_guessing_or_trying_another_layout() {
    let peer = DiscoveryFixture::start(2).await;
    let client = client(&peer, IoLayout::Compatibility160);
    assert!(matches!(
        client.discover_io(firmware()).await,
        Err(etherbird::Error::Operation(Error::DiscoveryChanged(_)))
    ));
    assert_eq!(peer.peer.type_queries.load(Ordering::SeqCst), 0);
    client.read_clock().await.unwrap();
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
    client.stop().await;
}

fn expected_catalog_value(pt: u8, p: u8) -> Value {
    match (pt, p) {
        (91, 0) => Value::UInt8(1),
        (91, 1) => Value::UInt8(2),
        (91, 50) => Value::UInt8(0),
        (91, 8 | 9) => Value::UInt16(450),
        (91, 41) => Value::UInt32(0x12345678),
        (91, 42) => Value::UInt32(65536),
        (91, 2) => Value::Ascii("Example ROC Station ".into()),
        (91, 3) => Value::Ascii("W68000 Ver 3.00     ".into()),
        (91, 4) => Value::Ascii("Oct 08,2022 00:00   ".into()),
        (91, 5) => Value::Ascii("Example Vendor      ".into()),
        (91, 6) => Value::Ascii("ROC809              ".into()),
        (101, 0) => Value::Ascii("DI SAMPLE ".into()),
        (101, 1 | 3) => Value::UInt8(1),
        (101, 6) => Value::UInt32(0x01020304),
        (103, 0) => Value::Ascii("AI SAMPLE ".into()),
        (103, 1) => Value::Ascii("kPa       ".into()),
        (103, 2) => Value::UInt8(1),
        (103, 3 | 4) => Value::Float(1.0),
        (103, 7) => Value::UInt16(0x1234),
        (103, 21) => Value::Float(12.5),
        (103, 32) => Value::Binary(0xa5),
        (109, 0) => Value::Ascii("Battery   ".into()),
        (109, 1) => Value::Ascii("Volts     ".into()),
        (109, 4) => Value::Float(1.0),
        (109, 7) => Value::UInt16(0x5678),
        (109, 12) => Value::Float(12.0),
        (109, 23) => Value::Binary(0x81),
        (136, 0 | 1) => Value::UInt8(59),
        (136, 2 | 10 | 15) => Value::UInt8(23),
        (136, 3) => Value::UInt8(31),
        (136, 4 | 13 | 18) => Value::UInt8(12),
        (136, 5) => Value::UInt16(2104),
        (136, 6 | 11 | 16) => Value::UInt8(7),
        (136, 7) => Value::Time(u32::MAX),
        (136, 8) => Value::UInt8(1),
        (136, 9) => Value::UInt32(999999),
        (136, 12) => Value::UInt8(6),
        (136, 14) => Value::Time(0x12345678),
        (136, 17) => Value::UInt8(56),
        (136, 19) => Value::Time(0),
        _ => panic!("unexpected catalog row"),
    }
}
#[tokio::test]
async fn every_catalog_row_reads_independent_literal_values_over_tcp() {
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer, IoLayout::Compatibility160);
    let snapshot = client
        .discover_io(FirmwareVersion::new(3, 40).unwrap())
        .await
        .unwrap();
    let catalog = Roc800Catalog::from_discovery(snapshot).unwrap();
    let mut tlps = Vec::new();
    let mut expected = Vec::new();
    let mut keys = std::collections::BTreeSet::new();
    for entry in Roc800Catalog::entries() {
        assert!(keys.insert((entry.point_type, entry.parameter)));
        assert!(!entry.source.is_empty());
        let logical = match entry.point_type {
            91 | 109 | 136 => 0,
            101 => 32,
            103 => 16,
            _ => panic!("unreviewed row"),
        };
        tlps.push(Tlp::new(entry.point_type, logical, entry.parameter));
        let value = expected_catalog_value(entry.point_type, entry.parameter);
        assert_eq!(entry.data_type, value.data_type());
        assert_eq!(
            entry.data_type.width(),
            catalog_literal(entry.point_type, entry.parameter)
                .unwrap()
                .len()
        );
        // Independently audited table access, including unlike PT-103/7 and PT-109/7.
        let access = match (entry.point_type, entry.parameter) {
            (91, 0..=2 | 50)
            | (101, _)
            | (103, 0..=3)
            | (109, 0 | 1 | 7 | 12)
            | (136, 8 | 10..=13 | 15..=18) => Access::ReadWrite,
            _ => Access::ReadOnly,
        };
        assert_eq!(entry.access, access);
        assert_eq!(
            entry.introduced,
            match (entry.point_type, entry.parameter) {
                (91, 41 | 42) => (1, 20),
                (91, 50) => (2, 0),
                (136, 9) => (1, 20),
                (136, 10..=19) => (3, 40),
                _ => (1, 0),
            }
        );
        expected.push(value);
    }
    assert_eq!(tlps.len(), 50);
    assert_eq!(
        client
            .read_schema(&catalog, &catalog.context(), &tlps)
            .await
            .unwrap(),
        expected
    );
    assert!(peer.peer.parameter_reads.load(Ordering::SeqCst) >= 2);
    client.stop().await;
}
#[test]
fn catalog_identity_version_introductions_and_omissions_are_explicit() {
    assert!(FirmwareVersion::new(0, 0).is_err());
    assert!(FirmwareVersion::new(3, 100).is_err());
    assert!(Roc800Catalog::new(FirmwareVersion::new(3, 61).unwrap()).is_err());
    let old = Roc800Catalog::new(FirmwareVersion::new(1, 0).unwrap()).unwrap();
    assert!(old.lookup(&old.context(), Tlp::new(91, 0, 41)).is_err());
    assert!(old.lookup(&old.context(), Tlp::new(91, 0, 50)).is_err());
    assert!(
        old.lookup(&SchemaContext::default(), Tlp::new(91, 0, 0))
            .is_err()
    );
    assert!(
        old.lookup(&old.context(), Tlp::new(91, 0, 7))
            .unwrap()
            .is_none()
    ); // Reserved/omitted, never a guessed width.
    assert!(old.lookup(&old.context(), Tlp::new(103, 16, 21)).is_err()); // No I/O existence snapshot.
    let v120 = Roc800Catalog::new(FirmwareVersion::new(1, 20).unwrap()).unwrap();
    assert_eq!(
        v120.lookup(&v120.context(), Tlp::new(91, 0, 41))
            .unwrap()
            .unwrap()
            .data_type,
        DataType::UInt32
    );
    assert!(v120.lookup(&old.context(), Tlp::new(91, 0, 0)).is_err());
}

#[test]
fn clock_catalog_firmware_ranges_and_singleton_location_follow_table_3_48() {
    let ranges = [
        Some((0.0, 59.0)),
        Some((0.0, 59.0)),
        Some((0.0, 23.0)),
        Some((1.0, 31.0)),
        Some((1.0, 12.0)),
        Some((2000.0, 2104.0)),
        Some((1.0, 7.0)),
        None,
        Some((0.0, 1.0)),
        Some((0.0, 999999.0)),
        Some((0.0, 23.0)),
        Some((1.0, 7.0)),
        Some((1.0, 6.0)),
        Some((1.0, 12.0)),
        None,
        Some((0.0, 23.0)),
        Some((1.0, 7.0)),
        None,
        Some((1.0, 12.0)),
        None,
    ];
    // Test both sides of each introduction boundary, independently of metadata.
    for (major, minor, last) in [(1, 0, 8), (1, 19, 8), (1, 20, 9), (3, 39, 9), (3, 40, 19)] {
        let catalog = Roc800Catalog::new(FirmwareVersion::new(major, minor).unwrap()).unwrap();
        for parameter in 0..20 {
            let result = catalog.lookup(&catalog.context(), Tlp::new(136, 0, parameter));
            if parameter > last {
                assert!(matches!(result, Err(Error::CapabilityUnavailable(_))));
            } else {
                let schema = result.unwrap().unwrap();
                assert_eq!(schema.numeric_range, ranges[usize::from(parameter)]);
                assert_eq!(schema.logical_range, Some((0, 0)));
                assert!(matches!(
                    catalog.lookup(&catalog.context(), Tlp::new(136, 1, parameter)),
                    Err(Error::InvalidRequest(_))
                ));
            }
        }
        assert!(
            catalog
                .lookup(&catalog.context(), Tlp::new(136, 0, 20))
                .unwrap()
                .is_none()
        );
    }
}

#[tokio::test]
async fn clock_catalog_reads_without_io_discovery_and_rejects_unavailable_batches_locally() {
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer, IoLayout::Compatibility160);
    let old = Roc800Catalog::new(FirmwareVersion::new(1, 19).unwrap()).unwrap();
    for tlps in [
        vec![Tlp::new(136, 0, 0), Tlp::new(136, 0, 9)],
        vec![Tlp::new(136, 0, 10)],
        vec![Tlp::new(136, 1, 0)],
        vec![Tlp::new(136, 0, 20)],
    ] {
        assert!(
            client
                .read_schema(&old, &old.context(), &tlps)
                .await
                .is_err()
        );
    }
    assert_eq!(peer.peer.parameter_reads.load(Ordering::SeqCst), 0);
    let catalog = Roc800Catalog::new(FirmwareVersion::new(3, 40).unwrap()).unwrap();
    // Order and duplicates survive; time is a raw u32 and the disputed week
    // byte is not narrowed into an assumed DST calendar rule.
    let tlps = [
        Tlp::new(136, 0, 7),
        Tlp::new(136, 0, 5),
        Tlp::new(136, 0, 9),
        Tlp::new(136, 0, 17),
        Tlp::new(136, 0, 7),
    ];
    assert_eq!(
        client
            .read_schema(&catalog, &catalog.context(), &tlps)
            .await
            .unwrap(),
        [
            Value::Time(u32::MAX),
            Value::UInt16(2104),
            Value::UInt32(999999),
            Value::UInt8(56),
            Value::Time(u32::MAX)
        ]
    );
    assert!(peer.peer.requests.lock().unwrap().contains(&(
        180,
        vec![5, 136, 0, 7, 136, 0, 5, 136, 0, 9, 136, 0, 17, 136, 0, 7]
    )));
    assert_eq!(peer.peer.type_queries.load(Ordering::SeqCst), 0);
    assert_eq!(peer.peer.logical_queries.load(Ordering::SeqCst), 0);
    client.stop().await;
}

#[tokio::test]
async fn clock_schema_malformed_reply_retires_but_device_rejection_keeps_connection() {
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer, IoLayout::Compatibility160);
    let catalog = Roc800Catalog::new(FirmwareVersion::new(3, 40).unwrap()).unwrap();
    peer.peer.fault_opcode.store(180, Ordering::SeqCst);
    peer.peer.fault.store(3, Ordering::SeqCst);
    let tlps = [Tlp::new(136, 0, 5), Tlp::new(136, 0, 7)];
    let rejected = client
        .read_schema(&catalog, &catalog.context(), &tlps)
        .await
        .unwrap_err();
    assert!(matches!(
        rejected.error,
        etherbird::Error::Operation(Error::Device(_))
    ));
    assert!(rejected.progress.values.is_empty());
    client
        .read_schema(&catalog, &catalog.context(), &tlps)
        .await
        .unwrap();
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
    peer.peer.fault.store(1, Ordering::SeqCst);
    let malformed = client
        .read_schema(&catalog, &catalog.context(), &tlps)
        .await
        .unwrap_err();
    assert!(malformed.progress.values.is_empty());
    assert_eq!(malformed.progress.completed_batches, 0);
    assert_eq!(
        client
            .read_schema(&catalog, &catalog.context(), &tlps)
            .await
            .unwrap(),
        [Value::UInt16(2104), Value::Time(u32::MAX)]
    );
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
    // Exactly one exchange per explicit call, with no failed-read replay.
    assert_eq!(peer.peer.parameter_reads.load(Ordering::SeqCst), 4);
    client.stop().await;
}
#[tokio::test]
async fn invalid_locations_fail_locally_and_replacement_invalidates_catalog_and_plans() {
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer, IoLayout::Compatibility160);
    let io = client.discover_io(firmware()).await.unwrap();
    let generation = io.generation();
    let catalog = Roc800Catalog::from_discovery(io).unwrap();
    for tlp in [
        Tlp::new(103, 0, 21),
        Tlp::new(103, 32, 21),
        Tlp::new(101, 16, 3),
        Tlp::new(91, 1, 0),
        Tlp::new(136, 1, 5),
    ] {
        assert!(
            client
                .read_schema(&catalog, &catalog.context(), &[tlp])
                .await
                .is_err()
        );
    }
    assert_eq!(peer.peer.parameter_reads.load(Ordering::SeqCst), 0);
    let plan = rocsteady::ReadPlan::build(
        &catalog,
        &catalog.context(),
        &[Tlp::new(103, 16, 21), Tlp::new(136, 0, 5)],
        client.profile(),
    )
    .unwrap();
    peer.peer.fault_opcode.store(7, Ordering::SeqCst);
    peer.peer.fault.store(4, Ordering::SeqCst);
    assert!(client.read_clock().await.is_err());
    client.read_clock().await.unwrap();
    let error = client.read_plan(plan).await.unwrap_err();
    assert!(
        matches!(error.error,etherbird::Error::Operation(Error::StaleGeneration {expected,..}) if expected==generation)
    );
    assert!(
        client
            .read_schema(&catalog, &catalog.context(), &[Tlp::new(136, 0, 5)])
            .await
            .is_err()
    );
    assert_eq!(peer.peer.parameter_reads.load(Ordering::SeqCst), 0);
    let fresh =
        Roc800Catalog::from_discovery(client.discover_io(firmware()).await.unwrap()).unwrap();
    assert_eq!(
        client
            .read_schema(&fresh, &fresh.context(), &[Tlp::new(103, 16, 21)])
            .await
            .unwrap(),
        [Value::Float(12.5)]
    );
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
    client.stop().await;
}
#[tokio::test]
async fn disconnect_during_discovery_returns_once_and_does_not_resume() {
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer, IoLayout::Compatibility160);
    client.connected().await.unwrap();
    peer.peer.fault_opcode.store(50, Ordering::SeqCst);
    peer.peer.fault.store(4, Ordering::SeqCst);
    assert!(client.discover_io(firmware()).await.is_err());
    client.read_clock().await.unwrap();
    assert_eq!(peer.peer.type_queries.load(Ordering::SeqCst), 1);
    assert_eq!(peer.peer.logical_queries.load(Ordering::SeqCst), 0);
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
    client.stop().await;
}
#[tokio::test]
async fn canceled_discovery_retires_generation_and_cannot_finish_late() {
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer, IoLayout::Compatibility160);
    client.connected().await.unwrap();
    peer.peer.fault_opcode.store(50, Ordering::SeqCst);
    peer.peer.fault.store(5, Ordering::SeqCst);
    let clone = client.clone();
    let work = tokio::spawn(async move { clone.discover_io(firmware()).await });
    tokio::time::timeout(Duration::from_secs(1), async {
        while peer.peer.type_queries.load(Ordering::SeqCst) == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    work.abort();
    assert!(work.await.unwrap_err().is_cancelled());
    tokio::time::timeout(Duration::from_secs(1), async {
        while peer.peer.connections.load(Ordering::SeqCst) < 2 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    client.connected().await.unwrap();
    client.read_clock().await.unwrap();
    assert_eq!(peer.peer.logical_queries.load(Ordering::SeqCst), 0);
    client.stop().await;
}

struct ShiftedProfile;
impl ProtocolProfile for ShiftedProfile {
    fn metadata(&self) -> rocsteady::ProfileMetadata {
        rocsteady::DefaultProfile.metadata()
    }
    fn capability(&self, opcode: u8) -> Capability {
        rocsteady::DefaultProfile.capability(opcode)
    }
    fn io_layout(&self) -> Option<IoLayout> {
        Some(IoLayout::Compatibility160)
    }
    fn io_location(&self, _: IoLayout, index: usize) -> Result<u8, Error> {
        Ok(if index == 16 { 17 } else { index as u8 })
    }
    fn decode_point_types(
        &self,
        request: &PointTypeRequest,
        payload: &[u8],
    ) -> Result<rocsteady::PointTypePage, DecodeError> {
        let mut payload = payload.to_vec();
        if payload.len() > 3 {
            payload[3] = 253;
        }
        PointTypeCodec.decode(request, &payload)
    }
}
#[tokio::test]
async fn explicit_mapping_and_disputed_decoder_are_replaceable_through_public_profile() {
    let peer = DiscoveryFixture::start(0).await;
    let client = Client::tcp(
        &peer.endpoint,
        ClientConfig {
            profile: Arc::new(ShiftedProfile),
            heartbeat_interval: None,
            ..Default::default()
        },
    )
    .unwrap();
    let io = client.discover_io(firmware()).await.unwrap();
    assert_eq!(io.by_logical(103, 0).unwrap().location, 17);
    assert_eq!(
        client
            .read_point_types(PointTypeRequest::new(91, 1).unwrap())
            .await
            .unwrap()
            .get(91),
        Some(PointTypeClassification::UserDefined)
    );
    client.stop().await;
}
