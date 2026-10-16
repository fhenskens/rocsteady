//! Explicit raw transaction fixture; interpretation is selected by the caller.
#[cfg(feature = "simulator")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rocsteady::*;
    use simulator::{HistoryMetadataModel, Model, Simulator, TransactionSegmentModel};
    let mut history = HistoryMetadataModel::default();
    history.transactions.insert(
        13,
        TransactionSegmentModel {
            description: *b"EXAMPLE TX",
            payload_size: 7,
            records: vec![(42, 123456, vec![3, 0x34, 0x12, 9, b'A', 0, b'Z'])],
            list_page_size: 1,
            data_page_size: 2,
        },
    );
    let server =
        Simulator::bind_with_history("127.0.0.1:0".parse()?, Model::default(), history).await?;
    let client = Client::tcp(
        server.endpoint().to_string(),
        ClientConfig {
            heartbeat_interval: None,
            ..Default::default()
        },
    )?;
    let segment = TransactionSegment::new(13, FirmwareVersion::new(3, 50)?)?;
    let inventory = client
        .transaction_inventory(TransactionListRequest::new(segment, 0), 4)
        .await?;
    let page = &inventory.pages[0];
    let entry = &page.entries[0];
    match client
        .read_transaction(segment, entry.number, page.payload_size, 8)
        .await
    {
        Ok(record) => {
            assert!(record.complete);
            // Fixture uses tagged values. Neither this selection nor successful parsing
            // proves that a physical controller uses this disputed layout.
            let fields =
                decode_transaction_fields(&record.bytes, TransactionFieldLayout::TaggedValues)?;
            assert_eq!(fields.len(), 2);
            println!(
                "Transaction {}, raw creation TIME {}: {fields:?}",
                entry.number, entry.created
            );
        }
        Err(failure) => {
            eprintln!(
                "Confirmed {} bytes; next offset {:?}: {}",
                failure.progress.bytes.len(),
                failure.progress.next_offset,
                failure.error
            );
            // Recovery never resumes the failed read. Establish current inventory/identity
            // before deciding whether to start a new explicit read.
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
