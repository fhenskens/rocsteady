//! Local simulator only. Set once, then explicitly observe the resulting calendar.
#[cfg(feature = "simulator")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rocsteady::*;
    let server =
        simulator::Simulator::bind("127.0.0.1:0".parse()?, simulator::Model::default()).await?;
    let client = Client::tcp(
        server.endpoint().to_string(),
        ClientConfig {
            heartbeat_interval: None,
            ..Default::default()
        },
    )?;
    let value = ClockSet::new(2024, 2, 29, 23, 58, 59)?;
    match client.set_clock(value).await {
        Ok(WriteOutcome::Acknowledged) => {
            let observed = client.read_clock().await?;
            assert_eq!(
                (
                    observed.year,
                    observed.month,
                    observed.day,
                    observed.weekday
                ),
                (2024, 2, 29, 5)
            );
            println!("Observed clock: {observed:?}");
        }
        Ok(_) => unreachable!("successful set is acknowledged"),
        Err(failure) => {
            eprintln!(
                "{:?}: {}; reconcile before another explicit set",
                failure.outcome, failure.error
            );
            return Err(failure.into());
        }
    }
    client.stop().await;
    server.stop().await?;
    Ok(())
}
#[cfg(not(feature = "simulator"))]
fn main() {
    eprintln!("Run with --features simulator");
}
