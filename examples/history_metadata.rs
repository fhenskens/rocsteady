//! Default raw times, named provisional alternatives, and caller-owned decoding.
include!("support/discovery_fixture.rs");
use rocsteady::{
    CalendarOccurrenceCodec, Client, ClientConfig, DecodeError, HistoryTagsRequest, MinMaxCodec,
    MinMaxRequest, PrefixedEpochOccurrenceCodec, ReplyCodec,
};
struct FirstByte;
impl ReplyCodec<()> for FirstByte {
    type Output = u8;
    fn decode(&self, _: &(), bytes: &[u8]) -> Result<u8, DecodeError> {
        bytes
            .first()
            .copied()
            .ok_or(DecodeError("missing occurrence"))
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let peer = DiscoveryFixture::start(0).await;
    let client = Client::tcp(
        &peer.endpoint,
        ClientConfig {
            heartbeat_interval: None,
            ..Default::default()
        },
    )?;
    let result = async {
        let request = MinMaxRequest::new(0, 7)?;
        let raw = client.read_min_max(request).await?;
        println!(
            "current={}, raw minimum time={:?}",
            raw.current, raw.today.minimum_time
        );
        // Explicit choices for synthetic fixture bytes, not automatic firmware selection.
        let calendar = client
            .read_min_max_with_codec(request, MinMaxCodec(CalendarOccurrenceCodec))
            .await?;
        let epoch = client
            .read_min_max_with_codec(request, MinMaxCodec(PrefixedEpochOccurrenceCodec))
            .await?;
        let custom = client
            .read_min_max_with_codec(request, MinMaxCodec(FirstByte))
            .await?;
        println!(
            "calendar={:?}, epoch={:?}, custom={}",
            calendar.today.minimum_time, epoch.today.minimum_time, custom.today.minimum_time
        );
        let tags = client
            .read_history_tags(HistoryTagsRequest::new(0, vec![7, 2, 7])?)
            .await?;
        println!(
            "reported periodic index={}, tags={:?}",
            tags.periodic_index, tags.tags
        );
        Ok::<_, Box<dyn std::error::Error>>(())
    }
    .await;
    client.stop().await;
    result
}
