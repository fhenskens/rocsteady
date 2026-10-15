//! Multi-point history with time-major rows and explicit partial progress; no controller required.
include!("support/discovery_fixture.rs");
use rocsteady::{
    Client, ClientConfig, ConfiguredHistoryPolicy, HistoryKind, MultiHistoryRange,
    MultiHistoryRequest,
};
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
        // Two contiguous points, in ascending point order, for three time periods.
        let page = client
            .read_multi_history(MultiHistoryRequest::new(
                0,
                0,
                HistoryKind::Periodic,
                7,
                2,
                3,
            )?)
            .await?;
        for row in page.periods {
            println!(
                "time={}, point7={}, point8={}",
                row.timestamp, row.values[0], row.values[1]
            );
        }
        // Caller-asserted fixture geometry; obtain real capacities from device configuration.
        let geometry = ConfiguredHistoryPolicy::new(10, 200, 60, 5, 3)?;
        let window =
            MultiHistoryRange::with_policy(0, 4, HistoryKind::Periodic, 7, 2, 5, &geometry)?;
        let progress = match client.read_multi_history_range(window).await {
            Ok(progress) => progress,
            Err(error) => {
                eprintln!(
                    "{}; confirmed pages={}, next={:?}, remaining={}",
                    error,
                    error.progress.pages.len(),
                    error.progress.next_index,
                    error.progress.remaining
                );
                return Err(error.into());
            }
        };
        println!(
            "pages={}, complete={}, short={}, index changed={}",
            progress.pages.len(),
            progress.complete,
            progress.short_page,
            progress.index_changed
        );
        // Incomplete/empty replies may indicate an invalid request. Inspect the result;
        // do not automatically resume or infer that the remaining periods are absent.
        Ok::<(), Box<dyn std::error::Error>>(())
    }
    .await;
    client.stop().await;
    result
}
