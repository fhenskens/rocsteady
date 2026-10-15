//! Configured-point discovery and selected history, using a hardware-free independent peer.
include!("support/discovery_fixture.rs");
use rocsteady::{
    Client, ClientConfig, ConfiguredHistoryPolicy, ConfiguredHistoryRequest, DecodeError,
    HistoryKind, ReplyCodec, SelectedHistoryCodec, SelectedHistoryRange, SelectedHistoryRequest,
};
struct ConsumerPeriods;
impl ReplyCodec<SelectedHistoryRequest> for ConsumerPeriods {
    type Output = usize;
    fn decode(&self, q: &SelectedHistoryRequest, b: &[u8]) -> Result<usize, DecodeError> {
        Ok(SelectedHistoryCodec.decode(q, b)?.periods.len())
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
        let configured = client
            .read_configured_history(ConfiguredHistoryRequest::new(0)?)
            .await?;
        println!("configured points={:?}", configured.points);
        // Explicit order and duplicates are retained. Point 255 is an ordinary
        // selected point here; it is not OP-138's timestamp sentinel.
        let request =
            SelectedHistoryRequest::new(0, 0, HistoryKind::Periodic, vec![255, 7, 255], 3, true)?;
        let page = client.read_selected_history(request.clone()).await?;
        for row in page.periods {
            println!(
                "time={:?}, points 255/7/255={:?}",
                row.timestamp, row.values
            );
        }
        println!(
            "consumer-decoded periods={}",
            client
                .read_selected_history_with_codec(request, ConsumerPeriods)
                .await?
        );
        let geometry = ConfiguredHistoryPolicy::new(10, 256, 60, 5, 3)?;
        let range = SelectedHistoryRange::with_policy(
            0,
            4,
            HistoryKind::Periodic,
            vec![7, 2],
            5,
            true,
            &geometry,
        )?
        .with_inventory_check(true);
        match client.read_selected_history_range(range).await {
            Ok(p) => println!(
                "pages={}, complete={}, remaining={}, inventory changed={}",
                p.pages.len(),
                p.complete,
                p.remaining,
                p.inventory_changed
            ),
            Err(e) => {
                eprintln!(
                    "{}; confirmed pages={}, next={:?}, remaining={}",
                    e,
                    e.progress.pages.len(),
                    e.progress.next_index,
                    e.progress.remaining
                );
                return Err(e.into());
            }
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    }
    .await;
    client.stop().await;
    result
}
