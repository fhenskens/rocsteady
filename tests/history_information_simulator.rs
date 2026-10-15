#![cfg(feature = "simulator")]
use rocsteady::{
    Client, ClientConfig, ConfiguredHistoryPolicy, ConfiguredHistoryRequest, Error, HistoryKind,
    SelectedHistoryRange, SelectedHistoryRequest,
    simulator::{HistoryMetadataModel, Model, Simulator},
};
// REQ-139-SIMULATOR: explicit independent words, inventory, every selector/flag,
// sparse/reordered/duplicated points, point 255 values, index-only, short and missing data.
#[tokio::test]
async fn simulator_inventory_and_selection_use_explicit_values_and_order() {
    let mut h = HistoryMetadataModel {
        history_max_point: 255,
        ..Default::default()
    };
    h.configured_history_points.insert(0, vec![7, 2, 255]);
    h.configured_history_points.insert(1, vec![]);
    h.configured_history_points.insert(2, (0..252).collect());
    h.configured_history_points.insert(3, (0..253).collect());
    let values = vec![
        [0, 0, 0x80, 0x3f],
        [0, 0, 0, 0xc0],
        [0x34, 0x12, 0xc0, 0x7f],
        [0, 0, 0, 0x80],
        [0, 0, 0x80, 0x7f],
    ];
    let times = vec![
        [1, 0, 0, 0],
        [0x78, 0x56, 0x34, 0x12],
        [255; 4],
        [0; 4],
        [0, 0, 0, 0x80],
    ];
    for kind in 0..=2 {
        h.multi_history_timestamps.insert((0, kind), times.clone());
        h.history_sample_indices.insert((0, kind), 4);
        h.history_samples.insert((0, 7, kind), values.clone());
        h.history_samples
            .insert((0, 2, kind), vec![[0, 0, 0, 0xc0]; 5]);
        h.history_samples
            .insert((0, 255, kind), vec![[0, 0, 0x7f, 0x43]; 5]);
    }
    let server = Simulator::bind_with_history("127.0.0.1:0".parse().unwrap(), Model::default(), h)
        .await
        .unwrap();
    let c = Client::tcp(
        server.endpoint().to_string(),
        ClientConfig {
            heartbeat_interval: None,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        c.read_configured_history(ConfiguredHistoryRequest::new(0).unwrap())
            .await
            .unwrap()
            .points,
        [7, 2, 255]
    );
    assert!(
        c.read_configured_history(ConfiguredHistoryRequest::new(1).unwrap())
            .await
            .unwrap()
            .points
            .is_empty()
    );
    assert_eq!(
        c.read_configured_history(ConfiguredHistoryRequest::new(2).unwrap())
            .await
            .unwrap()
            .points
            .len(),
        252
    );
    assert!(matches!(
        c.read_configured_history(ConfiguredHistoryRequest::new(3).unwrap())
            .await,
        Err(etherbird::Error::Operation(Error::Device(_)))
    ));
    let policy = ConfiguredHistoryPolicy::new(10, 256, 5, 5, 5).unwrap();
    for kind in [
        HistoryKind::Minute,
        HistoryKind::Periodic,
        HistoryKind::Daily,
    ] {
        for timestamps in [false, true] {
            let q =
                SelectedHistoryRequest::new(0, 0, kind, vec![7, 2, 7, 255], 5, timestamps).unwrap();
            let r = c.read_selected_history(q).await.unwrap();
            assert_eq!(r.current_index, 4);
            assert_eq!(
                r.periods
                    .iter()
                    .map(|r| r.values[0].to_bits())
                    .collect::<Vec<_>>(),
                [0x3f800000, 0xc0000000, 0x7fc01234, 0x80000000, 0x7f800000]
            );
            assert!(
                r.periods
                    .iter()
                    .all(|r| r.values[0].to_bits() == r.values[2].to_bits()
                        && r.values[1] == -2.0
                        && r.values[3] == 255.0)
            );
            assert_eq!(
                r.periods.iter().map(|r| r.timestamp).collect::<Vec<_>>(),
                if timestamps {
                    vec![
                        Some(1),
                        Some(0x12345678),
                        Some(u32::MAX),
                        Some(0),
                        Some(0x80000000),
                    ]
                } else {
                    vec![None; 5]
                }
            );
            let probe = c
                .read_selected_history(
                    SelectedHistoryRequest::new(0, 4, kind, vec![255], 0, timestamps).unwrap(),
                )
                .await
                .unwrap();
            assert!(probe.periods.is_empty());
            assert_eq!(probe.current_index, 4);
            let r = c
                .read_selected_history_range(
                    SelectedHistoryRange::with_policy(
                        0,
                        4,
                        kind,
                        vec![7, 2, 7],
                        5,
                        timestamps,
                        &policy,
                    )
                    .unwrap()
                    .with_inventory_check(true),
                )
                .await
                .unwrap();
            assert!(r.complete);
            assert!(r.inventory_after.is_some());
            assert_eq!(r.next_index, Some(4));
            assert_eq!(
                r.pages.iter().map(|p| p.periods.len()).collect::<Vec<_>>(),
                [1, 4]
            );
        }
    }
    let short = c
        .read_selected_history_range(
            SelectedHistoryRange::new(0, 4, HistoryKind::Periodic, vec![7, 2], 2, true).unwrap(),
        )
        .await
        .unwrap();
    assert!(short.short_page && !short.complete);
    assert_eq!(short.remaining, 1);
    assert!(matches!(
        c.read_selected_history(
            SelectedHistoryRequest::new(0, 0, HistoryKind::Periodic, vec![3], 1, false).unwrap()
        )
        .await,
        Err(etherbird::Error::Operation(Error::Device(_)))
    ));
    for payload in [
        vec![2, 0],
        vec![0, 0, 1],
        vec![1, 0, 0, 0, 3, 1, 0, 1, 7],
        vec![1, 0, 0, 0, 1, 1, 2, 1, 7],
        vec![1, 0, 0, 0, 1, 1, 0, 2, 7],
        vec![1, 0, 0, 0, 1, 61, 0, 1, 7],
    ] {
        assert!(matches!(
            c.request(139, payload).await,
            Err(etherbird::Error::Operation(Error::Device(_)))
        ));
    }
    c.read_clock().await.unwrap();
    c.stop().await;
    server.stop().await.unwrap();
}
