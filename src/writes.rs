//! Schema-aware OP-166/181 writes. Acknowledgements do not establish atomicity or persistence.
use crate::{
    Access, Client, Error, ManagedContext, ProtocolProfile, SchemaContext, SchemaProvider, Tlp,
    Value,
};
use std::sync::{Arc, Mutex};

/// Conservative OP-181 request-payload policy, including count, TLPs and values.
/// Section 2.23's 240-byte *response* wording does not establish a request limit.
/// This ceiling is software policy, not a verified device boundary (DEC-1.03).
pub const MAX_ARBITRARY_WRITE_REQUEST: usize = 240;

/// Immutable OP-166 or OP-181 batch. Inspect opcode before interpreting the payload.
#[derive(Debug, Clone)]
pub struct WriteBatch {
    opcode: u8,
    payload: Vec<u8>,
}
impl WriteBatch {
    pub fn opcode(&self) -> u8 {
        self.opcode
    }
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
    fn validate(&self, profile: &dyn ProtocolProfile) -> Result<(), Error> {
        crate::profile::validate_profile_request(profile, self.opcode, &self.payload)?;
        crate::profile::validate_expected_response(profile, self.opcode, 0)
    }
}
/// All metadata and values are checked before any batch can be sent. Order and duplicates survive.
#[derive(Debug, Clone)]
pub struct WritePlan {
    batches: Vec<WriteBatch>,
    generation: Option<u64>,
    table_guard: Option<crate::ConfigurableTable>,
}
impl WritePlan {
    // Shared once-only outcome/send-gate machinery for single state-changing services.
    pub(crate) fn single(opcode: u8, payload: Vec<u8>) -> Self {
        Self {
            batches: vec![WriteBatch { opcode, payload }],
            generation: None,
            table_guard: None,
        }
    }
    pub(crate) fn configurable_table(
        snapshot: crate::ConfigurableTable,
        payloads: Vec<Vec<u8>>,
    ) -> Self {
        Self {
            batches: payloads
                .into_iter()
                .map(|payload| WriteBatch {
                    opcode: 11,
                    payload,
                })
                .collect(),
            generation: Some(snapshot.generation()),
            table_guard: Some(snapshot),
        }
    }
    pub fn batches(&self) -> &[WriteBatch] {
        &self.batches
    }
    /// Build contiguous OP-166 batches, each with at most 230 value bytes.
    pub fn build(
        provider: &dyn SchemaProvider,
        context: &SchemaContext,
        writes: &[(Tlp, Value)],
        profile: &dyn ProtocolProfile,
    ) -> Result<Self, Error> {
        Self::build_for(provider, context, writes, profile, 166)
    }
    /// Build OP-181 batches across arbitrary points/parameters, retaining order and duplicates.
    /// Uses the explicit conservative 240-byte request policy, tightened by the profile.
    pub fn build_arbitrary(
        provider: &dyn SchemaProvider,
        context: &SchemaContext,
        writes: &[(Tlp, Value)],
        profile: &dyn ProtocolProfile,
    ) -> Result<Self, Error> {
        Self::build_for(provider, context, writes, profile, 181)
    }
    fn build_for(
        provider: &dyn SchemaProvider,
        context: &SchemaContext,
        writes: &[(Tlp, Value)],
        profile: &dyn ProtocolProfile,
        opcode: u8,
    ) -> Result<Self, Error> {
        if writes.is_empty() {
            return Err(Error::InvalidRequest("empty schema write"));
        }
        let generation = provider.generation()?;
        let arbitrary = opcode == 181;
        let limit = profile.limits(opcode).request.min(if arbitrary {
            MAX_ARBITRARY_WRITE_REQUEST
        } else {
            234
        });
        let mut batches: Vec<WriteBatch> = Vec::new();
        let mut previous: Option<Tlp> = None;
        for (tlp, value) in writes {
            let bytes = encode_schema_value(provider, context, *tlp, value)?;
            // A single OP-181 tuple costs count + TLP + value, independently of OP-166.
            if bytes.is_empty() || bytes.len() + 4 > limit {
                return Err(Error::InvalidRequest(
                    "write cannot fit selected service/profile budget",
                ));
            }
            let contiguous = previous.is_some_and(|p| {
                p.point_type == tlp.point_type
                    && p.logical == tlp.logical
                    && p.parameter.checked_add(1) == Some(tlp.parameter)
            });
            let split = batches.last().is_none_or(|batch| {
                batch.payload.len() + bytes.len() + if arbitrary { 3 } else { 0 } > limit
                    || batch.payload[if arbitrary { 0 } else { 2 }] == 255
            });
            if split || (!arbitrary && !contiguous) {
                batches.push(WriteBatch {
                    opcode,
                    payload: if arbitrary {
                        vec![0]
                    } else {
                        vec![tlp.point_type, tlp.logical, 0, tlp.parameter]
                    },
                });
            }
            let batch = batches.last_mut().unwrap();
            batch.payload[if arbitrary { 0 } else { 2 }] += 1;
            if arbitrary {
                batch
                    .payload
                    .extend([tlp.point_type, tlp.logical, tlp.parameter]);
            }
            batch.payload.extend(bytes);
            batch.validate(profile)?;
            previous = Some(*tlp);
        }
        Ok(Self {
            batches,
            generation,
            table_guard: None,
        })
    }
}
pub(crate) fn encode_schema_value(
    provider: &dyn SchemaProvider,
    context: &SchemaContext,
    tlp: Tlp,
    value: &Value,
) -> Result<Vec<u8>, Error> {
    let schema = provider
        .lookup_write(context, tlp)?
        .ok_or(Error::CapabilityUnavailable("writable parameter schema"))?;
    if !matches!(schema.access, Access::ReadWrite | Access::WriteOnly) {
        return Err(Error::InvalidRequest(
            "schema does not establish writable access",
        ));
    }
    if let Some((lo, hi)) = schema.logical_range
        && (lo > hi || !(lo..=hi).contains(&tlp.logical))
    {
        return Err(Error::InvalidRequest("logical number outside schema range"));
    }
    if value.data_type() != schema.data_type {
        return Err(Error::InvalidRequest(
            "write type or width differs from schema",
        ));
    }
    if let Some((lo, hi)) = schema.numeric_range {
        let n = match value {
            Value::UInt8(v) | Value::Binary(v) => f64::from(*v),
            Value::Int8(v) => f64::from(*v),
            Value::Int16(v) => f64::from(*v),
            Value::UInt16(v) => f64::from(*v),
            Value::Int32(v) => f64::from(*v),
            Value::UInt32(v) | Value::Time(v) => f64::from(*v),
            Value::Float(v) => f64::from(*v),
            Value::Double(v) => *v,
            _ => return Err(Error::InvalidRequest("numeric bounds on nonnumeric write")),
        };
        if !lo.is_finite() || !hi.is_finite() || lo > hi || !n.is_finite() || n < lo || n > hi {
            return Err(Error::InvalidRequest("write outside schema numeric range"));
        }
    }
    value.encode()
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteOutcome {
    NotAttempted,
    Acknowledged,
    /// Device error received. Some parameters may have changed; raw errors remain available.
    DeviceError,
    /// Exchange was permitted; no valid acknowledgement or device error was observed.
    Uncertain,
}
#[derive(Debug, Clone)]
pub struct WriteProgress {
    pub batches: Vec<WriteOutcome>,
}
#[derive(Debug, thiserror::Error)]
#[error("schema write failed: {error}")]
pub struct SchemaWriteError {
    pub progress: WriteProgress,
    #[source]
    pub error: etherbird::Error<Error>,
}
struct State {
    closed: bool,
    progress: WriteProgress,
}
struct Gate(Arc<Mutex<State>>);
impl Drop for Gate {
    fn drop(&mut self) {
        self.0.lock().unwrap().closed = true;
    }
}
struct PlannedWrite {
    plan: WritePlan,
    state: Arc<Mutex<State>>,
}
#[etherbird::async_trait]
impl crate::Workflow for PlannedWrite {
    type Output = ();
    async fn run(self, context: &mut ManagedContext<'_>) -> Result<(), Error> {
        if let Some(expected) = self.plan.generation
            && expected != context.generation()
        {
            return Err(Error::StaleGeneration {
                expected,
                actual: context.generation(),
            });
        }
        for (i, batch) in self.plan.batches.into_iter().enumerate() {
            batch.validate(context.profile())?;
            if let Some(table) = &self.plan.table_guard {
                table.verify(context).await?;
            }
            {
                let mut state = self.state.lock().unwrap();
                if state.closed {
                    return Err(Error::Disconnected);
                }
                state.progress.batches[i] = WriteOutcome::Uncertain;
            }
            let result = context
                .exchange_operation(crate::operations::write_batch(batch.opcode, batch.payload))
                .await;
            let mut state = self.state.lock().unwrap();
            state.progress.batches[i] = match &result {
                Ok(()) => WriteOutcome::Acknowledged,
                Err(Error::Device(_)) => WriteOutcome::DeviceError,
                Err(e) if e.kind() == crate::ErrorKind::LocalValidation => {
                    WriteOutcome::NotAttempted
                }
                Err(_) => WriteOutcome::Uncertain,
            };
            result?;
        }
        Ok(())
    }
}
impl Client {
    /// Execute once under exclusive ownership. Recovery never replays a write or resumes a plan.
    /// Dropping the future closes the send gate; already permitted effects remain uncertain.
    pub async fn write_plan(&self, plan: WritePlan) -> Result<WriteProgress, SchemaWriteError> {
        let initial = WriteProgress {
            batches: vec![WriteOutcome::NotAttempted; plan.batches.len()],
        };
        for batch in &plan.batches {
            batch
                .validate(self.profile())
                .map_err(|error| SchemaWriteError {
                    progress: initial.clone(),
                    error: etherbird::Error::Operation(error),
                })?;
        }
        if let Some(table) = &plan.table_guard {
            crate::operations::table_definition(
                crate::TableDefinitionRequest::new(table.definition().table).map_err(|error| {
                    SchemaWriteError {
                        progress: initial.clone(),
                        error: etherbird::Error::Operation(error),
                    }
                })?,
            )
            .validate(self.profile())
            .map_err(|error| SchemaWriteError {
                progress: initial.clone(),
                error: etherbird::Error::Operation(error),
            })?;
        }
        let state = Arc::new(Mutex::new(State {
            closed: false,
            progress: initial,
        }));
        let gate = Gate(state.clone());
        let result = self
            .workflow(PlannedWrite {
                plan,
                state: state.clone(),
            })
            .await;
        drop(gate);
        let progress = state.lock().unwrap().progress.clone();
        result
            .map(|()| progress.clone())
            .map_err(|error| SchemaWriteError { progress, error })
    }
}
