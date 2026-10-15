//! Finite, caller-driven RTD/TC/MVS polling against an independent local TCP fixture.
//! Usage: temperature_monitoring [polls=3] [interval_ms=100] [--demo-reconnect]
include!("support/discovery_fixture.rs");
#[path = "support/monitoring.rs"]
mod monitoring;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rocsteady::*;
    let mut args = std::env::args().skip(1);
    let polls = args
        .next()
        .map(|n| n.parse::<usize>())
        .transpose()?
        .unwrap_or(3);
    let interval = std::time::Duration::from_millis(
        args.next()
            .map(|n| n.parse::<u64>())
            .transpose()?
            .unwrap_or(100),
    );
    let demo = match args.next().as_deref() {
        None => false,
        Some("--demo-reconnect") => true,
        _ => return Err("expected --demo-reconnect as third argument".into()),
    };
    if args.next().is_some()
        || polls == 0
        || interval.is_zero()
        || tokio::time::Instant::now().checked_add(interval).is_none()
        || (demo && polls < 3)
    {
        return Err("usage: temperature_monitoring [positive polls] [positive interval_ms] [--demo-reconnect (at least 3 polls)]".into());
    }
    let peer = DiscoveryFixture::start(0).await;
    peer.peer
        .parameter_values
        .lock()
        .unwrap()
        .insert((106, 96, 37), vec![1]);
    let client = Client::tcp(
        &peer.endpoint,
        ClientConfig {
            profile: Arc::new(Roc800Profile {
                layout: IoLayout::Compatibility160,
            }),
            heartbeat_interval: None,
            recovery: etherbird::Config {
                retry_delay: std::time::Duration::from_millis(1),
                ..Default::default()
            },
            ..Default::default()
        },
    )?;
    let targets = [
        monitoring::MonitorTarget {
            point_type: 106,
            logical_number: 0,
            parameters: vec![0, 22, 37, 33],
        },
        monitoring::MonitorTarget {
            point_type: 107,
            logical_number: 0,
            parameters: vec![0, 9, 3, 4, 19],
        },
        monitoring::MonitorTarget {
            point_type: 108,
            logical_number: 0,
            parameters: vec![0, 3, 19, 35, 50, 10, 11, 68],
        },
    ];
    let mut successes = 0;
    let mut failures = 0;
    for attempt in 0..polls {
        // Completion-to-next-start pacing: no timer bursts, hidden background work or retry loop.
        if attempt > 0 {
            tokio::time::sleep(interval).await;
        }
        if demo && attempt == 1 {
            peer.peer.parameter_fault_at.store(
                peer.peer.parameter_reads.load(Ordering::SeqCst) + 1,
                Ordering::SeqCst,
            );
            peer.peer.parameter_fault.store(4, Ordering::SeqCst);
        }
        match monitoring::poll_once(&client, FirmwareVersion::new(3, 60)?, &targets).await {
            Ok(sample) => {
                successes += 1;
                println!(
                    "sample {}: generation {}, {} fields",
                    attempt + 1,
                    sample.generation,
                    sample.values.len()
                );
                for (tlp, value) in sample.tlps.iter().zip(&sample.values) {
                    let entry = Roc800Catalog::entry(tlp.point_type, tlp.parameter).unwrap();
                    let metadata = entry.metadata;
                    match value {
                        Value::UInt8(code) => println!(
                            "  {}: {} {:?}",
                            entry.key,
                            code,
                            metadata.enum_meaning(*code).map(|meaning| meaning.key)
                        ),
                        Value::Binary(raw) => println!(
                            "  {}: {:?}, uninterpreted 0x{:02x}",
                            entry.key,
                            metadata
                                .active_bits(*raw)
                                .map(|bit| bit.key)
                                .collect::<Vec<_>>(),
                            metadata.uninterpreted_bits(*raw)
                        ),
                        _ => println!("  {}: {:?}", entry.key, value),
                    }
                }
            }
            Err(failure) => {
                failures += 1;
                eprintln!(
                    "sample {} failed: {}; confirmed {} values in {} batches; next scheduled sample will re-discover",
                    attempt + 1,
                    failure.error,
                    failure.progress.values.len(),
                    failure.progress.completed_batches
                );
            }
        }
    }
    client.stop().await;
    if successes == 0 {
        return Err("no monitoring sample succeeded".into());
    }
    if demo {
        assert_eq!(failures, 1);
        assert_eq!(successes, polls - 1);
    }
    assert_eq!(peer.peer.parameter_reads.load(Ordering::SeqCst), polls); // No failed sample replay.
    Ok(())
}
