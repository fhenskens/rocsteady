// External consumer example: public rocsteady and published Etherbird APIs only.
// This clock-based fixture is NOT an enhanced-security algorithm.
include!("support/managed_fixture.rs");

#[tokio::main]
async fn main() {
    let (_, client) = fixture(Duration::from_secs(2));
    client.connected().await.unwrap();
    assert_eq!(client.operation(FixtureClock).await.unwrap(), 2026);
    let generation = client.workflow(Pair).await.unwrap();
    println!(
        "fixture setup, typed operation and two-exchange workflow passed (generation {generation})"
    );
    client.stop().await;
}
