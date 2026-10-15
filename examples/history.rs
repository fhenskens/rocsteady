//! OP-135 values/timestamps and explicit bounded progress; no controller required.
include!("support/discovery_fixture.rs");
use rocsteady::{
    Client, ClientConfig, ConfiguredHistoryPolicy, HistoryKind, HistoryRange, HistoryRequest,
    ProseMinuteHistoryPolicy,
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
        let page = client
            .read_history(HistoryRequest::new(0, 7, HistoryKind::Periodic, 0, 3)?)
            .await?;
        println!(
            "values={:?}, reported index={}",
            page.samples, page.current_index
        );
        let times = client
            .read_history(HistoryRequest::new(
                0,
                7,
                HistoryKind::PeriodicTimestamps,
                0,
                3,
            )?)
            .await?;
        println!("raw epoch seconds={:?}", times.samples);
        // Known default minute capacity: split 58,59 then 0,1,2 without guessing an alias for 60.
        let progress = client
            .read_history_range(HistoryRange::new(0, 7, HistoryKind::Minute, 58, 5)?)
            .await?;
        println!(
            "pages={}, next={:?}, remaining={}, complete={}",
            progress.pages.len(),
            progress.next_index,
            progress.remaining,
            progress.complete
        );
        // Explicit fixture geometry. Real applications obtain these capacities from device configuration.
        let policy = ConfiguredHistoryPolicy::new(10, 200, 60, 5, 3)?;
        let configured = client
            .read_history_range(HistoryRange::with_policy(
                0,
                7,
                HistoryKind::Periodic,
                4,
                5,
                &policy,
            )?)
            .await?;
        println!("configured boundary pages={}", configured.pages.len());
        // A provisional alternative selected explicitly; fixture success is not firmware evidence.
        client
            .read_history(HistoryRequest::with_policy(
                0,
                7,
                HistoryKind::Minute,
                60,
                1,
                &ProseMinuteHistoryPolicy,
            )?)
            .await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    }
    .await;
    client.stop().await;
    result
}
