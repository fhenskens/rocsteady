//! Table-based day/month defaults, explicit caller interpretation and indexed fallback.
include!("support/discovery_fixture.rs");
use rocsteady::{
    Client, ClientConfig, ConfiguredHistoryPolicy, DayHistoryCodec, DayHistoryReply,
    DayHistoryRequest, DayHistoryTarget, DayIndicesRequest, DecodeError, HistoryDay,
    IndexedDayHistoryRequest, ReplyCodec,
};
/// Caller-supplied archival label; the device reply itself does not establish a year.
struct LabelledDay {
    year: u16,
}
impl ReplyCodec<DayHistoryRequest> for LabelledDay {
    type Output = (u16, DayHistoryReply);
    fn decode(&self, q: &DayHistoryRequest, b: &[u8]) -> Result<Self::Output, DecodeError> {
        Ok((self.year, DayHistoryCodec.decode(q, b)?))
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
        let date = HistoryDay::new(29, 2)?;
        let indices = client
            .read_day_indices(DayIndicesRequest::new(0, date)?)
            .await?;
        println!(
            "periodic start={}, count={}; daily start={:?}, count={}",
            indices.periodic_start,
            indices.periodic_count,
            indices.daily_start,
            indices.daily_count
        );
        let request = DayHistoryRequest::new(0, date, DayHistoryTarget::Values(7))?;
        let values = client.read_history_day(request).await?;
        println!("periodic={:?}; daily={:?}", values.periodic, values.daily);
        let timestamps = DayHistoryRequest::new(0, date, DayHistoryTarget::Timestamps)?;
        println!(
            "times={:?}",
            client.read_history_day(timestamps).await?.periodic
        );
        let (label, _) = client
            .read_history_day_with_codec(request, LabelledDay { year: 2024 })
            .await?;
        println!("caller-asserted year label={label}; not inferred from the wire");
        // Explicit alternative for a day too large for OP-138. This fixture has known
        // geometry; real callers obtain capacities/point applicability independently.
        let policy = Arc::new(ConfiguredHistoryPolicy::new(10, 200, 60, 5, 3)?);
        for plan in [
            IndexedDayHistoryRequest::new(request, policy.clone()),
            IndexedDayHistoryRequest::new(timestamps, policy).with_timestamp_point(7),
        ] {
            match client.read_history_day_indexed(plan).await {
                Ok(p) => println!(
                    "complete={}, periodic pages={}, daily pages={}, mapping changed={}",
                    p.complete,
                    p.periodic_pages.len(),
                    p.daily_pages.len(),
                    p.mapping_changed
                ),
                Err(e) => {
                    eprintln!(
                        "{}; periodic remaining={:?}, daily remaining={:?}",
                        e, e.progress.periodic_remaining, e.progress.daily_remaining
                    );
                    return Err(e.into());
                }
            }
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    }
    .await;
    client.stop().await;
    result
}
