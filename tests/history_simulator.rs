#![cfg(feature = "simulator")]
use rocsteady::{
    Client, ClientConfig, ConfiguredHistoryPolicy, Error, HistoryKind, HistoryRange,
    HistoryRequest, HistorySamples,
    simulator::{HistoryMetadataModel, Model, Simulator},
};

// REQ-135-SIMULATOR: Table 2-18 explicit four-byte words and current indices,
// every selector, index-only replies, configured ring boundary and zero/missing data.
#[tokio::test]
async fn simulator_reads_all_selectors_and_explicit_retention_boundaries() {
    let mut history = HistoryMetadataModel::default();
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
    for kind in 0..=4 {
        history.history_samples.insert(
            (0, 7, kind),
            if kind >= 3 {
                times.clone()
            } else {
                values.clone()
            },
        );
        history.history_sample_indices.insert((0, kind), 4);
    }
    let server =
        Simulator::bind_with_history("127.0.0.1:0".parse().unwrap(), Model::default(), history)
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
    let policy = ConfiguredHistoryPolicy::new(10, 200, 5, 5, 5).unwrap();
    for kind in [
        HistoryKind::Minute,
        HistoryKind::Periodic,
        HistoryKind::Daily,
        HistoryKind::PeriodicTimestamps,
        HistoryKind::DailyTimestamps,
    ] {
        let page = client
            .read_history(HistoryRequest::with_policy(0, 7, kind, 0, 5, &policy).unwrap())
            .await
            .unwrap();
        assert_eq!(page.current_index, 4);
        if kind.is_timestamp() {
            assert_eq!(
                page.samples,
                HistorySamples::Timestamps(vec![1, 0x12345678, u32::MAX, 0, 0x80000000])
            );
        } else {
            let HistorySamples::Values(v) = page.samples else {
                panic!("FLOAT expected")
            };
            assert_eq!(
                v.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                [0x3f800000, 0xc0000000, 0x7fc01234, 0x80000000, 0x7f800000]
            );
        }
        let probe = client
            .read_history(HistoryRequest::with_policy(0, 7, kind, 4, 0, &policy).unwrap())
            .await
            .unwrap();
        assert!(probe.samples.is_empty());
        assert_eq!(probe.current_index, 4);
        let progress = client
            .read_history_range(HistoryRange::with_policy(0, 7, kind, 4, 5, &policy).unwrap())
            .await
            .unwrap();
        assert!(progress.complete);
        assert_eq!(
            progress
                .pages
                .iter()
                .map(|p| (p.request.start(), p.samples.len()))
                .collect::<Vec<_>>(),
            [(4, 1), (0, 4)]
        );
        assert_eq!(progress.next_index, Some(4));
        if kind.is_timestamp() {
            assert_eq!(
                progress.pages[0].samples,
                HistorySamples::Timestamps(vec![0x80000000])
            );
            assert_eq!(
                progress.pages[1].samples,
                HistorySamples::Timestamps(vec![1, 0x12345678, u32::MAX, 0])
            );
        } else {
            let HistorySamples::Values(first) = &progress.pages[0].samples else {
                panic!("FLOAT expected")
            };
            let HistorySamples::Values(rest) = &progress.pages[1].samples else {
                panic!("FLOAT expected")
            };
            assert_eq!(first[0].to_bits(), 0x7f800000);
            assert_eq!(
                rest.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                [0x3f800000, 0xc0000000, 0x7fc01234, 0x80000000]
            );
        }
    }
    // Unknown periodic retention is not guessed: the controller's short reply stops the window.
    let short = client
        .read_history_range(HistoryRange::new(0, 7, HistoryKind::Periodic, 4, 2).unwrap())
        .await
        .unwrap();
    assert!(short.short_page && !short.complete);
    assert_eq!(short.remaining, 1);
    for result in [
        client
            .read_history(HistoryRequest::new(0, 8, HistoryKind::Periodic, 0, 1).unwrap())
            .await
            .map(|_| ()),
        client
            .read_history(HistoryRequest::new(0, 7, HistoryKind::Periodic, 5, 0).unwrap())
            .await
            .map(|_| ()),
        client
            .request(135, vec![0, 7, 5, 0, 0, 1])
            .await
            .map(|_| ()),
        client
            .request(135, vec![0, 7, 1, 0, 0, 61])
            .await
            .map(|_| ()),
        client.request(135, vec![0, 7, 1, 0, 0]).await.map(|_| ()),
    ] {
        assert!(matches!(
            result,
            Err(etherbird::Error::Operation(Error::Device(_)))
        ));
        client.read_clock().await.unwrap();
    }
    assert_eq!(server.stats().connections, 1);
    client.stop().await;
    server.stop().await.unwrap();
}
