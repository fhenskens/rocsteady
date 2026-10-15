//! Example-owned polling policy. Each call explicitly refreshes discovery; it never retries.
use rocsteady::{
    Client, Error, FirmwareVersion, ReadProgress, Roc800Catalog, SchemaReadError, Tlp, Value,
};

pub struct MonitorTarget {
    pub point_type: u8,
    /// Per-type logical ordinal, resolved anew on every sample.
    pub logical_number: u8,
    pub parameters: Vec<u8>,
}
#[derive(Debug)]
pub struct MonitorSample {
    pub generation: u64,
    pub tlps: Vec<Tlp>,
    pub values: Vec<Value>,
}
fn local(error: Error) -> SchemaReadError {
    SchemaReadError {
        progress: ReadProgress::default(),
        error: etherbird::Error::Operation(error),
    }
}
/// One explicit sample: discover, resolve physical locations, plan and read once.
/// A replacement between discovery and read is reported as a failed sample, not retried.
/// Callers own intervals, attempt limits, stop/cancellation and whether to call again.
/// Returned read errors retain confirmed values/batches; discovery errors have empty progress.
pub async fn poll_once(
    client: &Client,
    firmware: FirmwareVersion,
    targets: &[MonitorTarget],
) -> Result<MonitorSample, SchemaReadError> {
    if targets.is_empty() || targets.iter().any(|t| t.parameters.is_empty()) {
        return Err(local(Error::InvalidRequest(
            "monitoring needs nonempty targets and parameters",
        )));
    }
    let io = client
        .discover_io(firmware)
        .await
        .map_err(|error| SchemaReadError {
            progress: ReadProgress::default(),
            error,
        })?;
    let generation = io.generation();
    let mut tlps = Vec::new();
    for target in targets {
        let point = io
            .by_logical(target.point_type, target.logical_number)
            .map_err(local)?;
        for &parameter in &target.parameters {
            tlps.push(point.tlp(parameter).map_err(local)?);
        }
    }
    let catalog = Roc800Catalog::from_discovery(io).map_err(local)?;
    let values = client
        .read_schema(&catalog, &catalog.context(), &tlps)
        .await?;
    Ok(MonitorSample {
        generation,
        tlps,
        values,
    })
}
