//! Local PT-99 discovery, mixed table reads/writes and explicit read-back.
#[cfg(feature = "simulator")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rocsteady::*;
    let mut model = simulator::Model::default();
    model.io_point_types[16] = 101;
    model.parameters.insert(
        Tlp::new(91, 0, 2),
        Value::Ascii("INITIAL STATION     ".into()),
    );
    model
        .parameters
        .insert(Tlp::new(101, 16, 1), Value::UInt8(0));
    model
        .parameters
        .insert(Tlp::new(101, 16, 2), Value::Float(0.3));
    let mut entries = [Tlp::new(0, 0, 0); 44];
    entries[..3].copy_from_slice(&[
        Tlp::new(91, 0, 2),
        Tlp::new(101, 16, 1),
        Tlp::new(101, 16, 2),
    ]);
    model.configurable_tables.lock().unwrap().insert(
        0,
        ConfigurableTableDefinition {
            table: 0,
            revision: TableRevision::from_bits(1.0f32.to_bits()),
            entries,
        },
    );
    let server = simulator::Simulator::bind("127.0.0.1:0".parse()?, model).await?;
    let client = Client::tcp(
        server.endpoint().to_string(),
        ClientConfig {
            profile: std::sync::Arc::new(Roc800Profile {
                layout: IoLayout::Compatibility160,
            }),
            heartbeat_interval: None,
            ..Default::default()
        },
    )?;
    let firmware = FirmwareVersion::new(3, 60)?;
    let catalog = Roc800Catalog::from_discovery(client.discover_io(firmware).await?)?;
    let table = client.discover_configurable_table(0, firmware).await?;
    let plan = TableReadPlan::build(&table, &catalog, &catalog.context(), 0, 3, client.profile())?;
    let before = client.read_configurable_table(plan).await?;
    println!("Before: {:?}", before.batches[0].values);
    let values = [
        Value::Ascii("Example station     ".into()),
        Value::UInt8(1),
        Value::Float(0.5),
    ];
    let plan = table.write_plan(&catalog, &catalog.context(), 0, &values, client.profile())?;
    let progress = client.write_plan(plan).await?;
    assert_eq!(progress.batches, [WriteOutcome::Acknowledged]);
    let plan = TableReadPlan::build(&table, &catalog, &catalog.context(), 0, 3, client.profile())?;
    let after = client.read_configurable_table(plan).await?;
    assert!(after.complete);
    assert_eq!(after.batches[0].values, values);
    println!("After: {:?}", after.batches[0].values);
    // A failed write is never replayed; reconcile explicitly before another write.
    // OP-11 does not carry a revision, so external map edits can still race a check.
    client.stop().await;
    server.stop().await?;
    Ok(())
}
#[cfg(not(feature = "simulator"))]
fn main() {
    eprintln!("Run with --features simulator");
}
