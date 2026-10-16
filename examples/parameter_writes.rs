//! Local fixture only: contiguous and arbitrary writes, then explicit read-back.
#[cfg(feature = "simulator")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rocsteady::*;
    let mut model = simulator::Model::default();
    model.io_point_types[16] = 101;
    model.io_point_types[32] = 103;
    let simulator = simulator::Simulator::bind("127.0.0.1:0".parse()?, model).await?;
    let client = Client::tcp(
        simulator.endpoint().to_string(),
        ClientConfig {
            heartbeat_interval: None,
            profile: std::sync::Arc::new(Roc800Profile {
                layout: IoLayout::Compatibility160,
            }),
            ..Default::default()
        },
    )?;
    let catalog = Roc800Catalog::new(FirmwareVersion::new(3, 60)?)?;
    let target = Tlp::new(91, 0, 2);
    let value = Value::Ascii("Example station     ".into());
    let plan = WritePlan::build(
        &catalog,
        &catalog.context(),
        &[(target, value.clone())],
        client.profile(),
    )?;
    let progress = client.write_plan(plan).await?;
    assert_eq!(progress.batches, [WriteOutcome::Acknowledged]);
    assert_eq!(
        client
            .read_parameters(&[Parameter::new(target, DataType::Ascii(20))])
            .await?,
        [value]
    );
    let io = client.discover_io(FirmwareVersion::new(3, 60)?).await?;
    let di = io.by_logical(101, 0)?;
    let ai = io.by_logical(103, 0)?;
    let catalog = Roc800Catalog::from_discovery(io)?;
    let writes = [
        (ai.tlp(0)?, Value::Ascii("FLOW TAG  ".into())),
        (di.tlp(2)?, Value::Float(0.5)),
        (di.tlp(4)?, Value::UInt8(1)),  // Invert input interpretation.
        (di.tlp(5)?, Value::UInt8(0)),  // Disable latching.
        (di.tlp(9)?, Value::UInt8(1)),  // Permit alarm-log generation.
        (di.tlp(10)?, Value::UInt8(0)), // Disable SRBX on clear.
        (di.tlp(11)?, Value::UInt8(1)), // Request SRBX on set; no automatic polling/ack.
        (di.tlp(13)?, Value::Float(0.125)), // Scan period in seconds.
        (ai.tlp(1)?, Value::Ascii("m3/h      ".into())),
    ];
    let plan = WritePlan::build_arbitrary(&catalog, &catalog.context(), &writes, client.profile())?;
    assert_eq!(plan.batches()[0].opcode(), 181);
    match client.write_plan(plan).await {
        Ok(progress) => assert_eq!(progress.batches, [WriteOutcome::Acknowledged]),
        Err(failure) => {
            eprintln!(
                "Write outcomes: {:?}; reconcile device state before any new write",
                failure.progress.batches
            );
            return Err(failure.into());
        }
    }
    let tlps: Vec<_> = writes.iter().map(|(tlp, _)| *tlp).collect();
    assert_eq!(
        client
            .read_schema(&catalog, &catalog.context(), &tlps)
            .await?,
        writes
            .iter()
            .map(|(_, value)| value.clone())
            .collect::<Vec<_>>()
    );
    client.stop().await;
    simulator.stop().await?;
    Ok(())
}
#[cfg(not(feature = "simulator"))]
fn main() {
    eprintln!("Run with --features simulator");
}
