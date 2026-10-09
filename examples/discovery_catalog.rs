//! Specification-derived discovery and parameter reads without manually supplied types.
include!("support/discovery_fixture.rs");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rocsteady::{
        Client, ClientConfig, FirmwareVersion, IoLayout, Roc800Catalog, Roc800Profile, Tlp,
    };
    let peer = DiscoveryFixture::start(0).await;
    let client = Client::tcp(
        &peer.endpoint,
        ClientConfig {
            profile: Arc::new(Roc800Profile {
                layout: IoLayout::Compatibility160,
            }),
            heartbeat_interval: None,
            ..Default::default()
        },
    )?;
    let result = async {
        let pages = client.scan_point_types(0, 256).await?;
        println!("point-type discovery: {} pages", pages.len());
        // This version is an explicit fixture identity, not inferred from opcode 6.
        let io = client.discover_io(FirmwareVersion::new(3, 0)?).await?;
        let ai = io.by_logical(103, 0)?.tlp(21)?;
        let di = io.by_logical(101, 0)?.tlp(3)?;
        let battery = io.by_logical(109, 0)?.tlp(12)?;
        let catalog = Roc800Catalog::from_discovery(io)?;
        let values = client
            .read_schema(
                &catalog,
                &catalog.context(),
                &[Tlp::new(91, 0, 3), ai, di, battery],
            )
            .await?;
        Ok::<_, Box<dyn std::error::Error>>(values)
    }
    .await;
    client.stop().await;
    println!("catalog values: {:?}", result?);
    Ok(())
}
