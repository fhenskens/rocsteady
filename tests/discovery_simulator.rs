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
