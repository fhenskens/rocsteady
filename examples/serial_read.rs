//! Explicitly opt in to hardware access: ROC_SERIAL_PORT and ROC_SERIAL_BAUD are required.
use rocsteady::{Client, ClientConfig, transport::SerialConnector};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("ROC_SERIAL_PORT")?;
    let baud = std::env::var("ROC_SERIAL_BAUD")?.parse()?;
    // Set parity/stop bits/flow control on this connector to match your channel.
    let connector = SerialConnector::new(port, baud);
    let client = Client::new(Arc::new(connector), ClientConfig::default())?;
    let result = client.read_clock().await;
    client.stop().await;
    println!("clock: {:?}", result?);
    Ok(())
}
