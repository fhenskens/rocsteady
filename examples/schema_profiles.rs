//! External profile/schema implementations, bounded reads and explicit applicability.
include!("support/profile_fixture.rs");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = ProfileFixture::start().await;
    let client = rocsteady::Client::tcp(
        &fixture.endpoint,
        rocsteady::ClientConfig {
            profile: Arc::new(FixtureProfile),
            heartbeat_interval: None,
            ..Default::default()
        },
    )?;
    let result = client
        .read_schema(
            &FixtureSchema,
            &fixture_identity(),
            &[
                rocsteady::Tlp::new(200, 0, 1),
                rocsteady::Tlp::new(200, 0, 2),
                rocsteady::Tlp::new(200, 1, 1),
            ],
        )
        .await;
    client.stop().await;
    println!("{:?}: {:?}", client.profile().metadata(), result?);
    Ok(())
}
