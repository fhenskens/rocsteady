#![cfg(feature = "simulator")]
use rocsteady::{
    Client, ClientConfig, FirmwareVersion, IoLayout, PointTypeClassification, PointTypeRequest,
    Roc800Catalog, Roc800Profile, Tlp, Value,
    simulator::{Model, Simulator},
};
use std::sync::Arc;

#[tokio::test]
async fn simulator_supports_each_discovery_layout_classification_and_catalog_reads() {
    for (layout, stride) in [
        (IoLayout::Compatibility160, 16),
        (IoLayout::Compatibility240, 16),
        (IoLayout::Expanded224, 8),
    ] {
        let mut model = Model::default();
        model.set_io_layout(layout);
        model.io_point_types[usize::from(stride)] = 103;
        model
            .parameters
            .insert(Tlp::new(103, stride, 21), Value::Float(42.5));
        model
            .parameters
            .insert(Tlp::new(109, 0, 12), Value::Float(12.0));
        model
            .point_types
            .insert(60, PointTypeClassification::UserDefined);
        model
            .point_types
            .insert(61, PointTypeClassification::Unknown(42));
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
        let page = client
            .read_point_types(PointTypeRequest::new(60, 2).unwrap())
            .await
            .unwrap();
        assert_eq!(
            page.classifications(),
            [
                PointTypeClassification::UserDefined,
                PointTypeClassification::Unknown(42)
            ]
        );
        assert_eq!(client.scan_point_types(0, 256).await.unwrap().len(), 2);
        let io = client
            .discover_io(FirmwareVersion::new(3, 0).unwrap())
            .await
            .unwrap();
        assert_eq!(
            io.by_logical(103, 0).unwrap().tlp(21).unwrap(),
            Tlp::new(103, stride, 21)
        );
        let catalog = Roc800Catalog::from_discovery(io).unwrap();
        assert_eq!(
            client
                .read_schema(
                    &catalog,
                    &catalog.context(),
                    &[Tlp::new(103, stride, 21), Tlp::new(109, 0, 12)]
                )
                .await
                .unwrap(),
            [Value::Float(42.5), Value::Float(12.0)]
        );
        client.stop().await;
        server.stop().await.unwrap();
    }
}

#[tokio::test]
async fn simulator_reads_all_clock_catalog_parameters_with_explicit_model_values() {
    // Table 3-48 types; values are explicit fixture inputs, not an emulated clock
    // or DST calculation. Keep TIME and UINT32 distinct despite equal widths.
    let values = [
        Value::UInt8(0),
        Value::UInt8(59),
        Value::UInt8(23),
        Value::UInt8(29),
        Value::UInt8(2),
        Value::UInt16(2024),
        Value::UInt8(5),
        Value::Time(0),
        Value::UInt8(0),
        Value::UInt32(999999),
        Value::UInt8(2),
        Value::UInt8(1),
        Value::UInt8(6),
        Value::UInt8(3),
        Value::Time(0x12345678),
        Value::UInt8(2),
        Value::UInt8(1),
        Value::UInt8(56),
        Value::UInt8(11),
        Value::Time(u32::MAX),
    ];
    let mut model = Model::default();
    let tlps: Vec<_> = (0..20)
        .map(|parameter| Tlp::new(136, 0, parameter))
        .collect();
    model
        .parameters
        .extend(tlps.iter().copied().zip(values.iter().cloned()));
    let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), model)
        .await
        .unwrap();
    let client = Client::tcp(
        server.endpoint().to_string(),
        ClientConfig {
            heartbeat_interval: None,
            ..Default::default()
        },
    )
    .unwrap();
    let catalog = Roc800Catalog::new(FirmwareVersion::new(3, 40).unwrap()).unwrap();
    assert_eq!(
        client
            .read_schema(&catalog, &catalog.context(), &tlps)
            .await
            .unwrap(),
        values
    );
    assert_eq!(
        client
            .read_point_types(PointTypeRequest::new(136, 1).unwrap())
            .await
            .unwrap()
            .classifications(),
        [PointTypeClassification::Roc]
    );
    client.stop().await;
    server.stop().await.unwrap();
}

#[tokio::test]
async fn simulator_rejects_invalid_selector_command_and_ranges_as_healthy_device_errors() {
    let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), Model::default())
        .await
        .unwrap();
    let client = Client::tcp(
        server.endpoint().to_string(),
        ClientConfig {
            heartbeat_interval: None,
            ..Default::default()
        },
    )
    .unwrap();
    for (opcode, data) in [
        (50, vec![2]),
        (50, vec![]),
        (50, vec![0, 0]),
        (100, vec![10, 0, 1]),
        (100, vec![11, 255, 2]),
        (100, vec![11, 0, 246]),
        (100, vec![11]),
    ] {
        assert!(matches!(
            client.request(opcode, data).await,
            Err(etherbird::Error::Operation(rocsteady::Error::Device(_)))
        ));
    }
    assert!(
        client
            .read_point_types(PointTypeRequest::new(255, 0).unwrap())
            .await
            .unwrap()
            .classifications()
            .is_empty()
    );
    client.read_clock().await.unwrap();
    assert_eq!(server.stats().connections, 1);
    client.stop().await;
    server.stop().await.unwrap();
}

#[tokio::test]
async fn simulator_reads_foundation_fields_with_port_and_module_ordinals() {
    let tlps = [
        Tlp::new(91, 0, 23),
        Tlp::new(95, 0, 1),
        Tlp::new(95, 1, 36),
        Tlp::new(95, 5, 9),
        Tlp::new(139, 0, 0),
        Tlp::new(139, 26, 3),
    ];
    let values = [
        Value::Float(37.5),
        Value::UInt16(3),
        Value::UInt16(65535),
        Value::UInt8(255),
        Value::UInt8(36),
        Value::Ascii("2.02      ".into()),
    ];
    let mut model = Model::default();
    model
        .parameters
        .extend(tlps.iter().copied().zip(values.iter().cloned()));
    let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), model)
        .await
        .unwrap();
    let client = Client::tcp(
        server.endpoint().to_string(),
        ClientConfig {
            heartbeat_interval: None,
            ..Default::default()
        },
    )
    .unwrap();
    let catalog = Roc800Catalog::new(FirmwareVersion::new(2, 2).unwrap()).unwrap();
    assert_eq!(
        client
            .read_schema(&catalog, &catalog.context(), &tlps)
            .await
            .unwrap(),
        values
    );
    for pt in [95, 139] {
        assert_eq!(
            client
                .read_point_types(PointTypeRequest::new(pt, 1).unwrap())
                .await
                .unwrap()
                .classifications(),
            [PointTypeClassification::Roc]
        );
    }
    client.stop().await;
    server.stop().await.unwrap();
}

#[tokio::test]
async fn simulator_reads_all_input_fields_using_explicit_source_typed_values() {
    // Tables 3-13/3-15: fixture types are literal and independent of the catalog.
    let di = [
        Value::Ascii("DI SAMPLE ".into()),
        Value::UInt8(1),
        Value::Float(0.3),
        Value::UInt8(1),
        Value::UInt8(0),
        Value::UInt8(0),
        Value::UInt32(16000000),
        Value::Float(1000000.0),
        Value::Float(0.0),
        Value::UInt8(1),
        Value::UInt8(0),
        Value::UInt8(1),
        Value::Binary(0xff),
        Value::Float(0.05),
        Value::Float(0.05),
        Value::UInt8(1),
    ];
    let ai = [
        Value::Ascii("AI SAMPLE ".into()),
        Value::Ascii("kPa       ".into()),
        Value::UInt8(1),
        Value::Float(1.0),
        Value::Float(1.0),
        Value::UInt8(3),
        Value::UInt8(0),
        Value::UInt16(819),
        Value::UInt16(819),
        Value::UInt16(4095),
        Value::UInt16(4095),
        Value::UInt16(4095),
        Value::UInt16(4095),
        Value::Float(0.0),
        Value::Float(100.0),
        Value::Float(100.0),
        Value::Float(100.0),
        Value::Float(100.0),
        Value::Float(-1.0),
        Value::Float(0.0),
        Value::Float(12.5),
        Value::Float(12.5),
        Value::UInt8(1),
        Value::Float(-20.0),
        Value::Float(-10.0),
        Value::Float(110.0),
        Value::Float(120.0),
        Value::Float(5.0),
        Value::Float(2.0),
        Value::UInt8(1),
        Value::UInt8(1),
        Value::UInt8(1),
        Value::Binary(0xff),
        Value::Float(3600.0),
        Value::UInt8(0),
        Value::UInt8(0),
        Value::UInt8(1),
        Value::Float(-10.0),
        Value::UInt8(0),
        Value::Float(4.0),
    ];
    for (layout, stride) in [
        (IoLayout::Compatibility160, 16),
        (IoLayout::Compatibility240, 16),
        (IoLayout::Expanded224, 8),
    ] {
        let mut model = Model::default();
        model.set_io_layout(layout);
        model.io_point_types[stride as usize] = 103;
        model.io_point_types[2 * stride as usize] = 101;
        let tlps: Vec<_> = (0..16)
            .map(|p| Tlp::new(101, 2 * stride, p))
            .chain((0..40).map(|p| Tlp::new(103, stride, p)))
            .collect();
        let expected: Vec<_> = di.iter().chain(&ai).cloned().collect();
        model
            .parameters
            .extend(tlps.iter().copied().zip(expected.iter().cloned()));
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
                .discover_io(FirmwareVersion::new(3, 10).unwrap())
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
