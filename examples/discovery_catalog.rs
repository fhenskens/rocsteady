//! Specification-derived discovery and parameter reads without manually supplied types.
include!("support/discovery_fixture.rs");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rocsteady::{
        Client, ClientConfig, FirmwareVersion, IoLayout, Roc800Catalog, Roc800Profile, Tlp, Value,
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
        let io = client.discover_io(FirmwareVersion::new(3, 40)?).await?;
        let ai_point = io.by_logical(103, 0)?;
        let di_point = io.by_logical(101, 0)?;
        let ai = ai_point.tlp(21)?;
        let di = di_point.tlp(3)?;
        let battery = io.by_logical(109, 0)?.tlp(12)?;
        let do_point = io.by_logical(102, 0)?;
        let ao_point = io.by_logical(104, 0)?;
        let pulse_point = io.by_logical(105, 0)?;
        let catalog = Roc800Catalog::from_discovery(io)?;
        let values = client
            .read_schema(
                &catalog,
                &catalog.context(),
                &[
                    Tlp::new(91, 0, 3),
                    Tlp::new(136, 0, 5),
                    Tlp::new(136, 0, 7),
                    Tlp::new(91, 0, 23),
                    Tlp::new(95, 1, 36),
                    Tlp::new(139, 26, 3),
                    ai,
                    di,
                    battery,
                ],
            )
            .await?;
        // Read configuration, value and alarms together; neither this read nor
        // a multi-batch plan claims an atomic device snapshot.
        let input_tlps = [
            di_point.tlp(2)?,
            di_point.tlp(3)?,
            di_point.tlp(12)?,
            ai_point.tlp(5)?,
            ai_point.tlp(21)?,
            ai_point.tlp(32)?,
            ai_point.tlp(39)?,
        ];
        let inputs = client
            .read_schema(&catalog, &catalog.context(), &input_tlps)
            .await?;
        for (tlp, value) in input_tlps.iter().zip(&inputs) {
            let entry = Roc800Catalog::entry(tlp.point_type, tlp.parameter).unwrap();
            println!("{}: {:?} {:?}", entry.key, value, entry.metadata.units);
            if let Value::Binary(raw) = value {
                println!(
                    "  active: {:?}, uninterpreted bits: {:#04x}",
                    entry
                        .metadata
                        .active_bits(*raw)
                        .map(|bit| bit.key)
                        .collect::<Vec<_>>(),
                    entry.metadata.uninterpreted_bits(*raw)
                );
            }
        }
        // Read configured modes and physical outputs, retaining raw codes/bits.
        let output_tlps = [
            do_point.tlp(2)?,
            do_point.tlp(24)?,
            do_point.tlp(25)?,
            do_point.tlp(6)?,
            ao_point.tlp(2)?,
            ao_point.tlp(13)?,
            ao_point.tlp(16)?,
            ao_point.tlp(6)?,
        ];
        let outputs = client
            .read_schema(&catalog, &catalog.context(), &output_tlps)
            .await?;
        for (tlp, value) in output_tlps.iter().zip(outputs) {
            let entry = Roc800Catalog::entry(tlp.point_type, tlp.parameter).unwrap();
            println!("{}: {:?}", entry.key, value);
            if let Value::UInt8(raw) = value {
                println!(
                    "  meaning: {:?}",
                    entry.metadata.enum_meaning(raw).map(|m| m.key)
                );
            }
        }
        // Rates and totals are controller-reported values; no local conversion or rollover.
        let pulse_tlps = [
            pulse_point.tlp(4)?,
            pulse_point.tlp(6)?,
            pulse_point.tlp(7)?,
            pulse_point.tlp(10)?,
            pulse_point.tlp(13)?,
            pulse_point.tlp(23)?,
            pulse_point.tlp(24)?,
            pulse_point.tlp(25)?,
            pulse_point.tlp(26)?,
            pulse_point.tlp(27)?,
        ];
        let pulses = client
            .read_schema(&catalog, &catalog.context(), &pulse_tlps)
            .await?;
        for (tlp, value) in pulse_tlps.iter().zip(pulses) {
            let entry = Roc800Catalog::entry(tlp.point_type, tlp.parameter).unwrap();
            println!("{}: {:?} {:?}", entry.key, value, entry.metadata.units);
        }
        Ok::<_, Box<dyn std::error::Error>>(values)
    }
    .await;
    client.stop().await;
    println!("catalog values: {:?}", result?);
    Ok(())
}
