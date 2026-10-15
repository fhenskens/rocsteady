#![cfg(feature = "simulator")]
use rocsteady::{
    Client, ClientConfig, ConfiguredHistoryPolicy, HistoryKind, MultiHistoryRange,
    MultiHistoryRequest,
    simulator::{HistoryMetadataModel, Model, Simulator},
};
// REQ-136-SIMULATOR: Table 2-19 independent timestamp/FLOAT words, every selector,
// contiguous points, zero periods/elements, short/missing data and ring wrapping.
#[tokio::test]
async fn simulator_multi_point_periods_preserve_order_bits_and_explicit_geometry() {
    let mut h = HistoryMetadataModel::default();
    for kind in 0..=2 {
        h.multi_history_timestamps
            .insert((0, kind), vec![[0x78, 0x56, 0x34, 0x12], [255; 4], [0; 4]]);
        h.history_samples.insert(
            (0, 7, kind),
            vec![
                [0, 0, 0x80, 0x3f],
                [0x34, 0x12, 0xc0, 0x7f],
                [0, 0, 0, 0x80],
            ],
        );
        h.history_samples.insert(
            (0, 8, kind),
            vec![[0, 0, 0, 0xc0], [0, 0, 0x80, 0x7f], [0, 0, 0x80, 0xff]],
        );
        h.history_sample_indices.insert((0, kind), 2);
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
    let policy = ConfiguredHistoryPolicy::new(10, 200, 3, 3, 3).unwrap();
    for kind in [
        HistoryKind::Minute,
        HistoryKind::Periodic,
        HistoryKind::Daily,
    ] {
        let page = c
            .read_multi_history(MultiHistoryRequest::new(0, 0, kind, 7, 2, 3).unwrap())
            .await
            .unwrap();
        assert_eq!(page.current_index, 2);
        assert_eq!(page.periods.len(), 3);
        assert_eq!(
            page.periods.iter().map(|p| p.timestamp).collect::<Vec<_>>(),
            [0x12345678, u32::MAX, 0]
        );
        assert_eq!(
            page.periods
                .iter()
                .flat_map(|p| p.values.iter().map(|v| v.to_bits()))
                .collect::<Vec<_>>(),
            [
                0x3f800000, 0xc0000000, 0x7fc01234, 0x7f800000, 0x80000000, 0xff800000
            ]
        );
        let progress = c
            .read_multi_history_range(
                MultiHistoryRange::with_policy(0, 2, kind, 7, 2, 3, &policy).unwrap(),
            )
            .await
            .unwrap();
        assert!(progress.complete);
        assert_eq!(progress.next_index, Some(2));
        assert_eq!(
            progress
                .pages
                .iter()
                .map(|p| (p.request.start(), p.periods.len()))
                .collect::<Vec<_>>(),
            [(2, 1), (0, 2)]
        );
        let probe = c
            .read_multi_history(MultiHistoryRequest::new(0, 0, kind, 7, 2, 0).unwrap())
            .await
            .unwrap();
        assert!(probe.periods.is_empty());
        assert_eq!(probe.current_index, 2);
        let short = c
            .read_multi_history_range(MultiHistoryRange::new(0, 2, kind, 7, 2, 2).unwrap())
            .await
            .unwrap();
        assert!(short.short_page && !short.complete);
        assert_eq!(short.remaining, 1);
        let missing = c
            .read_multi_history_range(MultiHistoryRange::new(0, 0, kind, 8, 2, 2).unwrap())
            .await
            .unwrap();
        assert!(missing.short_page && !missing.complete);
        assert_eq!(missing.remaining, 2);
        let times = c
            .read_multi_history(MultiHistoryRequest::new(0, 0, kind, 0, 0, 3).unwrap())
            .await
            .unwrap();
        assert!(times.periods.iter().all(|p| p.values.is_empty()));
    }
    // Table 2-19 invalid request uses a zero-element reply, not fabricated values.
    for payload in [
        vec![0, 0, 0, 3, 7, 2, 1],
        vec![0, 0, 0, 1, 7, 2, 21],
        vec![0, 3, 0, 1, 7, 2, 1],
    ] {
        let raw = c.request(136, payload).await.unwrap();
        assert_eq!(raw.len(), 6);
        assert_eq!(raw[5], 0);
    }
    c.read_clock().await.unwrap();
    c.stop().await;
    server.stop().await.unwrap();
}
