#![cfg(feature = "simulator")]
include!("../examples/support/history_literals.rs");
use rocsteady::{
    CalendarOccurrenceCodec, Client, ClientConfig, Error, HistoryTagsRequest, MinMaxCodec,
    MinMaxRequest, OccurrenceTime,
    simulator::{HistoryMetadataModel, Model, Simulator},
};

// REQ-HISTORY-SIMULATOR: Tables 2-14/2-15. Explicit record bodies/tag bytes;
// missing points and invalid requests are healthy device errors, not guessed values.
#[tokio::test]
async fn simulator_metadata_reads_missing_points_and_invalid_requests() {
    let model = Model::default();
    let mut history = HistoryMetadataModel::default();
    history
        .history_min_max
        .insert((0, 7), min_max_literal(0, 7)[2..].try_into().unwrap());
    history.history_tags.insert((0, 7), *b"FLOW      ");
    history.history_tags.insert((0, 2), *b"PAD\0      ");
    history.history_periodic_indices.insert(0, u16::MAX);
    let server = Simulator::bind_with_history("127.0.0.1:0".parse().unwrap(), model, history)
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
    let q = MinMaxRequest::new(0, 7).unwrap();
    let r = client.read_min_max(q).await.unwrap();
    assert_eq!(r.raw, min_max_literal(0, 7));
    assert_eq!(r.current, 12.5);
    assert!(matches!(r.today.minimum_time, OccurrenceTime::Raw(_)));
    assert!(matches!(
        client
            .read_min_max_with_codec(q, MinMaxCodec(CalendarOccurrenceCodec))
            .await
            .unwrap()
            .today
            .minimum_time,
        OccurrenceTime::Calendar { .. }
    ));
    let tags = client
        .read_history_tags(HistoryTagsRequest::new(0, vec![7, 2, 7]).unwrap())
        .await
        .unwrap();
    assert_eq!(tags.periodic_index, u16::MAX);
    assert_eq!(tags.tags[1].tag.0, *b"PAD\0      ");
    for result in [
        client
            .read_min_max(MinMaxRequest::new(0, 8).unwrap())
            .await
            .map(|_| ()),
        client
            .read_history_tags(HistoryTagsRequest::new(0, vec![8]).unwrap())
            .await
            .map(|_| ()),
        client.request(105, vec![0]).await.map(|_| ()),
        client.request(108, vec![0, 0]).await.map(|_| ()),
        client.request(108, vec![0, 21]).await.map(|_| ()),
        client.request(108, vec![0, 1, 200]).await.map(|_| ()),
    ] {
        assert!(matches!(
            result,
            Err(etherbird::Error::Operation(Error::Device(_)))
        ));
        client.read_clock().await.unwrap();
    }
    client.stop().await;
    server.stop().await.unwrap();
}
