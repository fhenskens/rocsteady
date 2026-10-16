//! OP-10/11 and PT-99: D301180X012 §§2.5–6, §3.4.10, Tables 2-5/6, 3-11.
use crate::{
    Access, DataType, DecodeError, Error, FirmwareVersion, GenerationValue, ManagedContext,
    Operation, ProtocolProfile, SchemaContext, SchemaProvider, Tlp, Value, WritePlan,
};
use std::sync::{Arc, Mutex};

/// Exact IEEE revision identity, not a sortable number or automatic change counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableRevision(u32);
impl TableRevision {
    pub fn from_bits(bits: u32) -> Self {
        Self(bits)
    }
    pub fn bits(self) -> u32 {
        self.0
    }
    pub fn as_float(self) -> f32 {
        f32::from_bits(self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurableTableDefinition {
    pub table: u8,
    pub revision: TableRevision,
    pub entries: [Tlp; 44],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableDefinitionRequest {
    table: u8,
}
impl TableDefinitionRequest {
    pub fn new(table: u8) -> Result<Self, Error> {
        if table > 15 {
            return Err(Error::InvalidRequest("table number must be 0–15"));
        }
        Ok(Self { table })
    }
    pub fn table(self) -> u8 {
        self.table
    }
    pub fn payload(self) -> Vec<u8> {
        vec![99, self.table, 45, 0]
    }
    pub fn decode_reply(self, b: &[u8]) -> Result<ConfigurableTableDefinition, Error> {
        if b.len() != 140 || b[..4] != [99, self.table, 45, 0] {
            return Err(Error::InvalidResponse(
                "PT-99 definition echo/length mismatch",
            ));
        }
        let revision = TableRevision::from_bits(u32::from_le_bytes(b[4..8].try_into().unwrap()));
        let mut entries = [Tlp::new(0, 0, 0); 44];
        for (entry, v) in entries.iter_mut().zip(b[8..].as_chunks::<3>().0) {
            *entry = Tlp::new(v[0], v[1], v[2]);
        }
        Ok(ConfigurableTableDefinition {
            table: self.table,
            revision,
            entries,
        })
    }
}
impl Operation for TableDefinitionRequest {
    type Output = ConfigurableTableDefinition;
    fn encode(&self) -> Result<(u8, Vec<u8>), Error> {
        Ok((167, self.payload()))
    }
    fn decode(&self, b: &[u8]) -> Result<Self::Output, DecodeError> {
        self.decode_reply(b)
            .map_err(|_| DecodeError("invalid PT-99 definition"))
    }
}
struct Snapshot {
    definition: ConfigurableTableDefinition,
    bound: GenerationValue<()>,
}
/// Immutable, per-client/generation table snapshot. Discovery compares two full
/// definitions. Repeated equality detects observed changes, not an atomic snapshot.
#[derive(Clone)]
pub struct ConfigurableTable(Arc<Snapshot>);
impl std::fmt::Debug for ConfigurableTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigurableTable")
            .field("table", &self.0.definition.table)
            .field("generation", &self.generation())
            .field("revision", &self.0.definition.revision)
            .finish()
    }
}
impl ConfigurableTable {
    pub fn definition(&self) -> &ConfigurableTableDefinition {
        &self.0.definition
    }
    pub fn generation(&self) -> u64 {
        self.0.bound.generation()
    }
    pub(crate) async fn verify(&self, c: &mut ManagedContext<'_>) -> Result<(), Error> {
        self.0.bound.get(c)?;
        let current = c
            .read_table_definition(TableDefinitionRequest::new(self.0.definition.table)?)
            .await?;
        if current != self.0.definition {
            return Err(Error::DiscoveryChanged(
                "configurable table mapping/revision changed",
            ));
        }
        Ok(())
    }
    fn selected(&self, start: u8, count: usize) -> Result<&[Tlp], Error> {
        validate_range(self.0.definition.table, start, count)?;
        let entries = &self.0.definition.entries[usize::from(start)..usize::from(start) + count];
        if entries.iter().any(|t| *t == Tlp::new(0, 0, 0)) {
            return Err(Error::CapabilityUnavailable(
                "selected table location is unconfigured",
            ));
        }
        if entries
            .iter()
            .any(|t| !matches!(t.point_type, 91..=98 | 100..=135))
        {
            return Err(Error::CapabilityUnavailable(
                "table target outside Table 3-11 reviewed point types",
            ));
        }
        Ok(entries)
    }
    fn check_provider(&self, p: &dyn SchemaProvider) -> Result<(), Error> {
        if let Some(expected) = p.generation()?
            && expected != self.generation()
        {
            return Err(Error::StaleGeneration {
                expected,
                actual: self.generation(),
            });
        }
        Ok(())
    }
    /// Validate every mapped value before admission. Rechecks the full definition
    /// before each OP-11 batch under the shared write lease/send gate. PT-99
    /// configuration changes themselves are not enabled by this interface.
    pub fn write_plan(
        &self,
        p: &dyn SchemaProvider,
        ctx: &SchemaContext,
        start: u8,
        values: &[Value],
        profile: &dyn ProtocolProfile,
    ) -> Result<WritePlan, Error> {
        self.check_provider(p)?;
        let tlps = self.selected(start, values.len())?;
        validate_definition_budget(profile, self.0.definition.table)?;
        let limit = profile.limits(11).request.min(255);
        crate::profile::validate_profile_request_size(profile, 11, 3)?;
        crate::profile::validate_expected_response(profile, 11, 0)?;
        let mut payloads: Vec<Vec<u8>> = Vec::new();
        for (i, (tlp, value)) in tlps.iter().zip(values).enumerate() {
            let bytes = crate::writes::encode_schema_value(p, ctx, *tlp, value)?;
            if bytes.len() + 3 > limit {
                return Err(Error::InvalidRequest(
                    "table write value exceeds profile budget",
                ));
            }
            if payloads
                .last()
                .is_none_or(|b| b.len() + bytes.len() > limit)
            {
                payloads.push(vec![self.0.definition.table, start + i as u8, 0]);
            }
            let batch = payloads.last_mut().unwrap();
            batch[2] += 1;
            batch.extend(bytes);
        }
        Ok(WritePlan::configurable_table(self.clone(), payloads))
    }
}
fn validate_range(table: u8, start: u8, count: usize) -> Result<(), Error> {
    if table > 15 || start > 43 || count == 0 || count > 44 || usize::from(start) + count > 44 {
        return Err(Error::InvalidRequest(
            "table range requires table 0–15, start 0–43, count 1–44 and end <=44",
        ));
    }
    Ok(())
}
fn validate_definition_budget(profile: &dyn ProtocolProfile, table: u8) -> Result<(), Error> {
    crate::operations::table_definition(TableDefinitionRequest::new(table)?).validate(profile)
}
#[derive(Debug, Clone)]
pub struct TableReadBatch {
    table: u8,
    start: u8,
    revision: TableRevision,
    types: Vec<DataType>,
}
impl TableReadBatch {
    /// Caller-supplied metadata operation; higher-level TableReadPlan additionally
    /// binds metadata to discovery and rechecks the complete map before/after reads.
    pub fn new(
        table: u8,
        start: u8,
        revision: TableRevision,
        types: Vec<DataType>,
    ) -> Result<Self, Error> {
        validate_range(table, start, types.len())?;
        let mut size = 7;
        for kind in &types {
            size += kind.checked_width()?;
        }
        if size > 255 {
            return Err(Error::InvalidRequest(
                "table read response exceeds unicast budget",
            ));
        }
        Ok(Self {
            table,
            start,
            revision,
            types,
        })
    }
    pub fn payload(&self) -> Vec<u8> {
        vec![self.table, self.start, self.types.len() as u8]
    }
    pub fn response_size(&self) -> usize {
        7 + self.types.iter().map(|t| t.width()).sum::<usize>()
    }
    pub fn decode_reply(&self, b: &[u8]) -> Result<TableReadReply, Error> {
        let reply = self.decode_wire(b)?;
        self.validate_revision(&reply)?;
        Ok(reply)
    }
    pub(crate) fn validate_revision(&self, reply: &TableReadReply) -> Result<(), Error> {
        if reply.revision != self.revision {
            return Err(Error::DiscoveryChanged("table read revision changed"));
        }
        Ok(())
    }
    pub(crate) fn decode_wire(&self, b: &[u8]) -> Result<TableReadReply, Error> {
        if b.len() < 7 || b.len() > 255 || b[..3] != self.payload() {
            return Err(Error::InvalidResponse("table read echo/length mismatch"));
        }
        let revision = TableRevision::from_bits(u32::from_le_bytes(b[3..7].try_into().unwrap()));
        // Changed revisions may imply different widths. Retain the valid raw
        // envelope without interpreting values under the obsolete schema. The
        // caller reports DiscoveryChanged after completing the wire guard.
        if revision != self.revision {
            return Ok(TableReadReply {
                table: self.table,
                start: self.start,
                revision,
                values: Vec::new(),
                raw: b.to_vec(),
            });
        }
        if b.len() != self.response_size() {
            return Err(Error::InvalidResponse("table read value length mismatch"));
        }
        let mut offset = 7;
        let mut values = Vec::new();
        for kind in &self.types {
            values.push(Value::decode(*kind, &b[offset..offset + kind.width()])?);
            offset += kind.width();
        }
        Ok(TableReadReply {
            table: self.table,
            start: self.start,
            revision,
            values,
            raw: b.to_vec(),
        })
    }
}
impl Operation for TableReadBatch {
    type Output = TableReadReply;
    fn encode(&self) -> Result<(u8, Vec<u8>), Error> {
        Ok((10, self.payload()))
    }
    fn decode(&self, b: &[u8]) -> Result<Self::Output, DecodeError> {
        self.decode_reply(b)
            .map_err(|_| DecodeError("table read echo/length/version/value mismatch"))
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct TableReadReply {
    pub table: u8,
    pub start: u8,
    pub revision: TableRevision,
    pub values: Vec<Value>,
    pub raw: Vec<u8>,
}
#[derive(Debug, Clone)]
pub struct TableReadPlan {
    snapshot: ConfigurableTable,
    batches: Vec<TableReadBatch>,
}
impl TableReadPlan {
    pub fn build(
        snapshot: &ConfigurableTable,
        p: &dyn SchemaProvider,
        ctx: &SchemaContext,
        start: u8,
        count: u8,
        profile: &dyn ProtocolProfile,
    ) -> Result<Self, Error> {
        snapshot.check_provider(p)?;
        let tlps = snapshot.selected(start, usize::from(count))?;
        validate_definition_budget(profile, snapshot.0.definition.table)?;
        crate::profile::validate_profile_request_size(profile, 10, 3)?;
        let limit = profile.limits(10).response.min(255);
        let mut batches: Vec<TableReadBatch> = Vec::new();
        for (i, tlp) in tlps.iter().enumerate() {
            let schema = p
                .lookup(ctx, *tlp)?
                .ok_or(Error::CapabilityUnavailable("mapped table read schema"))?;
            if !matches!(schema.access, Access::ReadOnly | Access::ReadWrite) {
                return Err(Error::CapabilityUnavailable(
                    "mapped parameter is not readable",
                ));
            }
            if let Some((lo, hi)) = schema.logical_range
                && (lo > hi || !(lo..=hi).contains(&tlp.logical))
            {
                return Err(Error::InvalidRequest(
                    "mapped table location outside schema bounds",
                ));
            }
            let width = schema.data_type.checked_width()?;
            if 7 + width > limit {
                return Err(Error::InvalidRequest(
                    "table read value exceeds profile budget",
                ));
            }
            if batches
                .last()
                .is_none_or(|b| b.response_size() + width > limit)
            {
                batches.push(TableReadBatch {
                    table: snapshot.0.definition.table,
                    start: start + i as u8,
                    revision: snapshot.0.definition.revision,
                    types: Vec::new(),
                });
            }
            batches.last_mut().unwrap().types.push(schema.data_type);
        }
        Ok(Self {
            snapshot: snapshot.clone(),
            batches,
        })
    }
    pub fn batches(&self) -> &[TableReadBatch] {
        &self.batches
    }
}
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TableReadProgress {
    pub batches: Vec<TableReadReply>,
    pub complete: bool,
}
#[derive(Debug, thiserror::Error)]
#[error("configurable table read failed: {error}")]
pub struct TableReadError {
    pub progress: TableReadProgress,
    #[source]
    pub error: etherbird::Error<Error>,
}
struct Discover {
    request: TableDefinitionRequest,
}
#[etherbird::async_trait]
impl crate::Workflow for Discover {
    type Output = ConfigurableTable;
    async fn run(self, c: &mut ManagedContext<'_>) -> Result<Self::Output, Error> {
        let first = c.read_table_definition(self.request).await?;
        let second = c.read_table_definition(self.request).await?;
        if first != second {
            return Err(Error::DiscoveryChanged(
                "configurable table changed during discovery",
            ));
        }
        Ok(ConfigurableTable(Arc::new(Snapshot {
            definition: first,
            bound: c.bind(()),
        })))
    }
}
struct Read {
    plan: TableReadPlan,
    progress: Arc<Mutex<TableReadProgress>>,
}
#[etherbird::async_trait]
impl crate::Workflow for Read {
    type Output = ();
    async fn run(self, c: &mut ManagedContext<'_>) -> Result<(), Error> {
        self.plan.snapshot.verify(c).await?;
        for batch in self.plan.batches {
            let reply = c.read_table_batch(batch).await?;
            self.progress.lock().unwrap().batches.push(reply);
        }
        self.plan.snapshot.verify(c).await?;
        self.progress.lock().unwrap().complete = true;
        Ok(())
    }
}
impl crate::Client {
    /// Explicit ROC800 firmware applicability, reviewed 1.00–3.60. No automatic
    /// table installation, access or persistent identity is inferred.
    pub async fn discover_configurable_table(
        &self,
        table: u8,
        firmware: FirmwareVersion,
    ) -> Result<ConfigurableTable, etherbird::Error<Error>> {
        if firmware > FirmwareVersion::new(3, 60).map_err(etherbird::Error::Operation)? {
            return Err(etherbird::Error::Operation(Error::CapabilityUnavailable(
                "table firmware outside reviewed 1.00–3.60 window",
            )));
        }
        let request = TableDefinitionRequest::new(table).map_err(etherbird::Error::Operation)?;
        validate_definition_budget(self.profile(), table).map_err(etherbird::Error::Operation)?;
        self.workflow(Discover { request }).await
    }
    pub async fn read_table_definition(
        &self,
        q: TableDefinitionRequest,
    ) -> Result<ConfigurableTableDefinition, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::table_definition(q))
            .await
    }
    pub async fn read_table_batch(
        &self,
        b: TableReadBatch,
    ) -> Result<TableReadReply, etherbird::Error<Error>> {
        let reply = self
            .execute_prepared(crate::operations::table_read(b.clone()))
            .await?;
        b.validate_revision(&reply)
            .map_err(etherbird::Error::Operation)?;
        Ok(reply)
    }
    pub async fn read_configurable_table(
        &self,
        plan: TableReadPlan,
    ) -> Result<TableReadProgress, TableReadError> {
        let p = Arc::new(Mutex::new(TableReadProgress::default()));
        validate_definition_budget(self.profile(), plan.snapshot.0.definition.table)
            .and_then(|()| {
                for batch in &plan.batches {
                    crate::operations::table_read(batch.clone()).validate(self.profile())?;
                }
                Ok(())
            })
            .map_err(|error| TableReadError {
                progress: TableReadProgress::default(),
                error: etherbird::Error::Operation(error),
            })?;
        let result = self
            .workflow(Read {
                plan,
                progress: p.clone(),
            })
            .await;
        let progress = p.lock().unwrap().clone();
        result
            .map(|()| progress.clone())
            .map_err(|error| TableReadError { progress, error })
    }
}
impl crate::Session {
    pub async fn read_table_definition(
        &self,
        q: TableDefinitionRequest,
    ) -> Result<ConfigurableTableDefinition, Error> {
        self.request_operation(crate::operations::table_definition(q))
            .await
    }
    pub async fn read_table_batch(&self, b: TableReadBatch) -> Result<TableReadReply, Error> {
        let reply = self
            .request_operation(crate::operations::table_read(b.clone()))
            .await?;
        b.validate_revision(&reply)?;
        Ok(reply)
    }
}
impl ManagedContext<'_> {
    pub async fn read_table_definition(
        &mut self,
        q: TableDefinitionRequest,
    ) -> Result<ConfigurableTableDefinition, Error> {
        self.exchange_operation(crate::operations::table_definition(q))
            .await
    }
    pub async fn read_table_batch(&mut self, b: TableReadBatch) -> Result<TableReadReply, Error> {
        let reply = self
            .exchange_operation(crate::operations::table_read(b.clone()))
            .await?;
        b.validate_revision(&reply)?;
        Ok(reply)
    }
}
