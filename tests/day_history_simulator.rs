#![cfg(feature = "simulator")]
use rocsteady::{
    Client, ClientConfig, ConfiguredHistoryPolicy, DayHistoryRequest, DayHistoryTarget,
    DayIndicesRequest, Error, HistoryDay, HistorySamples, IndexedDayHistoryRequest,
    simulator::{DayHistoryData, HistoryMetadataModel, Model, Simulator},
};
use std::sync::Arc;
// REQ-DAY-SIMULATOR: Tables 2-20/21 explicit raw words; absent day and oversized
// whole-day rejection; indexed fallback works only through explicit caller selection.
#[tokio::test]
async fn simulator_reads_values_timestamps_missing_days_and_oversized_indexed_alternative() {
    let mut h = HistoryMetadataModel::default();
    let date = HistoryDay::new(15, 10).unwrap();
    h.day_indices.insert((0, 15, 10), [4, 0, 3, 0, 2, 0, 2, 0]);
    h.day_indices
        .insert((0, 16, 10), [0, 0, 62, 0, 255, 255, 0, 0]);
    let periodic = vec![
        [0, 0, 0x80, 0x3f],
        [0, 0, 0, 0xc0],
        [0x34, 0x12, 0xc0, 0x7f],
    ];
    let daily = vec![[0, 0, 0, 0x80], [0, 0, 0x80, 0x7f]];
    let pt = vec![[0x78, 0x56, 0x34, 0x12], [255; 4], [0; 4]];
    let dt = vec![[1, 0, 0, 0], [0, 0, 0, 0x80]];
    h.day_history.insert(
        (0, 7, 15, 10),
        DayHistoryData {
            periodic: periodic.clone(),
            daily: daily.clone(),
        },
    );
    h.day_history.insert(
        (0, 255, 15, 10),
        DayHistoryData {
            periodic: pt.clone(),
            daily: dt.clone(),
        },
    );
    h.day_history.insert(
        (0, 7, 16, 10),
        DayHistoryData {
            periodic: vec![[0, 0, 0x48, 0x41]; 62],
            daily: vec![],
        },
    );
    h.day_history.insert(
        (0, 255, 16, 10),
        DayHistoryData {
            periodic: vec![[0x78, 0x56, 0x34, 0x12]; 62],
            daily: vec![],
        },
    );
    for (kind, words) in [
        (1, vec![[0, 0, 0x48, 0x41]; 62]),
        (3, vec![[0x78, 0x56, 0x34, 0x12]; 62]),
    ] {
        h.history_samples.insert((0, 7, kind), words);
        h.history_sample_indices.insert((0, kind), 61);
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
    let indices = c
        .read_day_indices(DayIndicesRequest::new(0, date).unwrap())
        .await
        .unwrap();
    assert_eq!(indices.periodic_start, 4);
    assert_eq!(indices.daily_count, 2);
    let values = c
        .read_history_day(DayHistoryRequest::new(0, date, DayHistoryTarget::Values(7)).unwrap())
        .await
        .unwrap();
    let HistorySamples::Values(p) = values.periodic else {
        panic!()
    };
    let HistorySamples::Values(d) = values.daily else {
        panic!()
    };
    assert_eq!(
        p.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        [0x3f800000, 0xc0000000, 0x7fc01234]
    );
    assert_eq!(
        d.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        [0x80000000, 0x7f800000]
    );
    let times = c
        .read_history_day(DayHistoryRequest::new(0, date, DayHistoryTarget::Timestamps).unwrap())
        .await
        .unwrap();
    assert_eq!(
        times.periodic,
        HistorySamples::Timestamps(vec![0x12345678, u32::MAX, 0])
    );
    assert_eq!(times.daily, HistorySamples::Timestamps(vec![1, 0x80000000]));
    let missing = HistoryDay::new(17, 10).unwrap();
    assert!(matches!(
        c.read_day_indices(DayIndicesRequest::new(0, missing).unwrap())
            .await,
        Err(etherbird::Error::Operation(Error::Device(_)))
    ));
    assert!(matches!(
        c.read_history_day(
            DayHistoryRequest::new(0, missing, DayHistoryTarget::Values(7)).unwrap()
        )
        .await,
        Err(etherbird::Error::Operation(Error::Device(_)))
    ));
    let date = HistoryDay::new(16, 10).unwrap();
    for target in [DayHistoryTarget::Values(7), DayHistoryTarget::Timestamps] {
        let q = DayHistoryRequest::new(0, date, target).unwrap();
        assert!(matches!(
            c.read_history_day(q).await,
            Err(etherbird::Error::Operation(Error::Device(_)))
        ));
        let indexed = IndexedDayHistoryRequest::new(
            q,
            Arc::new(ConfiguredHistoryPolicy::new(10, 200, 60, 100, 3).unwrap()),
        )
        .with_timestamp_point(7);
        let p = c.read_history_day_indexed(indexed).await.unwrap();
        assert!(p.complete);
        assert!(p.daily_pages.is_empty());
        assert_eq!(p.periodic_remaining, Some(0));
        assert_eq!(
            p.periodic_pages
                .iter()
                .map(|p| p.samples.len())
                .collect::<Vec<_>>(),
            [60, 2]
        );
        assert_eq!(p.periodic_next_index, Some(62));
        assert_eq!(p.indices.as_ref().unwrap().raw_daily_start, 65535);
        assert_eq!(p.indices.as_ref().unwrap().daily_start, None);
        let expected = if target == DayHistoryTarget::Timestamps {
            HistorySamples::Timestamps(vec![0x12345678; 2])
        } else {
            HistorySamples::Values(vec![12.5; 2])
        };
        assert_eq!(p.periodic_pages[1].samples, expected);
    }
    c.read_clock().await.unwrap();
    c.stop().await;
    server.stop().await.unwrap();
}
