//! Schema lookup, write opt-in metadata and bounded opcode-180 read planning.
use crate::{DataType, Error, Parameter, ProtocolProfile, Tlp, Value};
use std::sync::{Arc, Mutex};

/// Caller-established identity. Unknown identity stays absent rather than guessed.
#[derive(Debug, Clone, Default)]
pub struct SchemaContext {
    pub device_family: Option<String>,
    pub firmware: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    ReadOnly,
    ReadWrite,
    WriteOnly,
    /// Provider must resolve the condition before returning usable metadata.
    Conditional,
}
#[derive(Debug, Clone, Copy)]
pub struct ParameterSchema {
    pub data_type: DataType,
    pub access: Access,
    pub logical_range: Option<(u8, u8)>,
    /// Inclusive write bounds; the read planner does not interpret value bounds.
    pub numeric_range: Option<(f64, f64)>,
}
pub trait SchemaProvider: Send + Sync + 'static {
    /// Discovery-derived metadata must remain in the connection generation that verified it.
    fn generation(&self) -> Result<Option<u64>, Error> {
        Ok(None)
    }
    /// Write-specific metadata. Providers opt in after resolving all write constraints.
    /// Read metadata alone never authorizes a write.
    fn lookup_write(
        &self,
        _context: &SchemaContext,
        _tlp: Tlp,
    ) -> Result<Option<ParameterSchema>, Error> {
        Ok(None)
    }
    /// None means missing; Err means unsupported/invalid applicability and stops fallback.
    fn lookup(&self, context: &SchemaContext, tlp: Tlp) -> Result<Option<ParameterSchema>, Error>;
}
/// First matching provider wins. Put caller overrides before built-in catalogs.
pub struct SchemaChain(pub Vec<Arc<dyn SchemaProvider>>);
impl SchemaProvider for SchemaChain {
    fn generation(&self) -> Result<Option<u64>, Error> {
        let mut bound = None;
        for provider in &self.0 {
            if let Some(generation) = provider.generation()? {
                if let Some(previous) = bound
                    && previous != generation
                {
                    return Err(Error::StaleGeneration {
                        expected: previous,
                        actual: generation,
                    });
                }
                bound = Some(generation);
            }
        }
        Ok(bound)
    }
    fn lookup_write(
        &self,
        context: &SchemaContext,
        tlp: Tlp,
    ) -> Result<Option<ParameterSchema>, Error> {
        for provider in &self.0 {
            if let Some(schema) = provider.lookup_write(context, tlp)? {
                return Ok(Some(schema));
            }
        }
        Ok(None)
    }
    fn lookup(&self, context: &SchemaContext, tlp: Tlp) -> Result<Option<ParameterSchema>, Error> {
        for provider in &self.0 {
            if let Some(schema) = provider.lookup(context, tlp)? {
                return Ok(Some(schema));
            }
        }
        Ok(None)
    }
}
/// Immutable batches preserve order and duplicates. Build before admission.
#[derive(Debug, Clone)]
pub struct ReadPlan {
    pub(crate) batches: Vec<Vec<Parameter>>,
    generation: Option<u64>,
}
impl ReadPlan {
    pub fn build(
        provider: &dyn SchemaProvider,
        context: &SchemaContext,
        tlps: &[Tlp],
        profile: &dyn ProtocolProfile,
    ) -> Result<Self, Error> {
        if tlps.is_empty() {
            return Err(Error::InvalidRequest("empty schema read"));
        }
        crate::profile::validate_profile_request(profile, 180, &[])?;
        let limits = profile.limits(180);
        let request_limit = limits.request.min(255);
        let response_limit = limits.response.min(crate::protocol::MAX_PARAMETER_RESPONSE);
        let mut batches = Vec::new();
        let mut batch = Vec::new();
        let mut budget = crate::protocol::ParameterBudget::EMPTY;
        for &tlp in tlps {
            let schema = provider
                .lookup(context, tlp)?
                .ok_or(Error::CapabilityUnavailable("parameter schema"))?;
            if !matches!(schema.access, Access::ReadOnly | Access::ReadWrite) {
                return Err(Error::InvalidRequest(
                    "schema does not establish readable access",
                ));
            }
            if let Some((start, end)) = schema.logical_range
                && (start > end || !(start..=end).contains(&tlp.logical))
            {
                return Err(Error::InvalidRequest("logical number outside schema range"));
            }
            let parameter = Parameter::new(tlp, schema.data_type);
            let single =
                crate::protocol::ParameterBudget::EMPTY.with_parameter(schema.data_type)?;
            if !single.fits(request_limit, response_limit) {
                return Err(Error::InvalidRequest(
                    "parameter cannot fit selected profile limits",
                ));
            }
            let next = budget.with_parameter(schema.data_type)?;
            if !next.fits(request_limit, response_limit) {
                batches.push(std::mem::take(&mut batch));
                budget = single;
            } else {
                budget = next;
            }
            batch.push(parameter);
        }
        if !batch.is_empty() {
            batches.push(batch);
        }
        Ok(Self {
            batches,
            generation: provider.generation()?,
        })
    }
    pub fn batches(&self) -> &[Vec<Parameter>] {
        &self.batches
    }
}

/// Only fully decoded batches are confirmed; the failed batch has no returned values.
#[derive(Debug, Default)]
pub struct ReadProgress {
    pub values: Vec<Value>,
    pub completed_batches: usize,
}
#[derive(Debug, thiserror::Error)]
#[error("schema read failed: {error}")]
pub struct SchemaReadError {
    pub progress: ReadProgress,
    #[source]
    pub error: etherbird::Error<Error>,
}
pub(crate) struct PlannedRead {
    pub plan: ReadPlan,
    pub progress: Arc<Mutex<ReadProgress>>,
}
#[etherbird::async_trait]
impl crate::Workflow for PlannedRead {
    type Output = ();
    async fn run(self, context: &mut crate::ManagedContext<'_>) -> Result<(), Error> {
        if let Some(expected) = self.plan.generation
            && expected != context.generation()
        {
            return Err(Error::StaleGeneration {
                expected,
                actual: context.generation(),
            });
        }
        for batch in self.plan.batches {
            let values = context
                .exchange_operation(crate::operations::parameters(&batch)?)
                .await?;
            let mut progress = self.progress.lock().unwrap();
            progress.values.extend(values);
            progress.completed_batches += 1;
        }
        Ok(())
    }
}
