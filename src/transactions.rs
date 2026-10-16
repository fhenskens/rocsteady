//! OP-206 §2.26/Table 2-29, D301180X012 October 2022, pp.2-42–43.
//! Raw pages are the default. Field framing is an explicit provisional choice (DEC-5).
use crate::{DecodeError, Error, FirmwareVersion, Operation, Tlp, Value};
use std::sync::{Arc, Mutex};

/// Explicit caller-established transaction segment and firmware. No discovery or
/// segment geometry is inferred. The reviewed firmware window is 3.50–3.60.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionSegment {
    segment: u8,
    firmware: FirmwareVersion,
}
impl TransactionSegment {
    pub fn new(segment: u8, firmware: FirmwareVersion) -> Result<Self, Error> {
        if firmware < FirmwareVersion::new(3, 50)? || firmware > FirmwareVersion::new(3, 60)? {
            return Err(Error::CapabilityUnavailable(
                "transactions require reviewed firmware 3.50–3.60",
            ));
        }
        Ok(Self { segment, firmware })
    }
    pub fn segment(self) -> u8 {
        self.segment
    }
    pub fn firmware(self) -> FirmwareVersion {
        self.firmware
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionListRequest {
    pub segment: TransactionSegment,
    pub offset: u16,
}
impl TransactionListRequest {
    pub fn new(segment: TransactionSegment, offset: u16) -> Self {
        Self { segment, offset }
    }
    pub fn payload(self) -> Vec<u8> {
        let mut b = vec![1, self.segment.segment];
        b.extend(self.offset.to_le_bytes());
        b
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionEntry {
    pub number: u16,
    pub created: u32,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionListPage {
    pub request: TransactionListRequest,
    pub more: bool,
    /// Raw fixed-width text; no printable-ASCII assumption or trimming.
    pub description: [u8; 10],
    pub payload_size: u16,
    pub entries: Vec<TransactionEntry>,
    pub raw: Vec<u8>,
}
impl TransactionListPage {
    pub fn decode(request: TransactionListRequest, b: &[u8]) -> Result<Self, DecodeError> {
        if b.len() < 15
            || b.len() > 255
            || b[0] != 1
            || b[2] > 1
            || b.len() != 15 + usize::from(b[1]) * 6
        {
            return Err(DecodeError(
                "transaction list command/count/flag/length mismatch",
            ));
        }
        if b[2] == 1 && b[1] == 0 {
            return Err(DecodeError(
                "transaction list continuation made no progress",
            ));
        }
        Ok(Self {
            request,
            more: b[2] == 1,
            description: b[3..13].try_into().unwrap(),
            payload_size: u16::from_le_bytes([b[13], b[14]]),
            entries: b[15..]
                .as_chunks::<6>()
                .0
                .iter()
                .map(|v| TransactionEntry {
                    number: u16::from_le_bytes([v[0], v[1]]),
                    created: u32::from_le_bytes(v[2..6].try_into().unwrap()),
                })
                .collect(),
            raw: b.to_vec(),
        })
    }
}
impl Operation for TransactionListRequest {
    type Output = TransactionListPage;
    fn encode(&self) -> Result<(u8, Vec<u8>), Error> {
        Ok((206, self.payload()))
    }
    fn decode(&self, b: &[u8]) -> Result<Self::Output, DecodeError> {
        TransactionListPage::decode(*self, b)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionDataRequest {
    pub segment: TransactionSegment,
    pub number: u16,
    pub offset: u16,
}
impl TransactionDataRequest {
    pub fn new(segment: TransactionSegment, number: u16, offset: u16) -> Self {
        Self {
            segment,
            number,
            offset,
        }
    }
    /// Table 2-29 places the byte offset at full-frame offset 10 (payload offset 4).
    pub fn payload(self) -> Vec<u8> {
        let mut b = vec![2, self.segment.segment];
        b.extend(self.number.to_le_bytes());
        b.extend(self.offset.to_le_bytes());
        b
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionDataPage {
    pub request: TransactionDataRequest,
    pub more: bool,
    pub bytes: Vec<u8>,
    pub raw: Vec<u8>,
}
impl TransactionDataPage {
    pub fn decode(request: TransactionDataRequest, b: &[u8]) -> Result<Self, DecodeError> {
        // The size byte counts everything below it, including the continuation flag.
        if b.len() < 3 || b.len() > 255 || b[0] != 2 || usize::from(b[1]) != b.len() - 2 || b[2] > 1
        {
            return Err(DecodeError(
                "transaction data command/size/flag/length mismatch",
            ));
        }
        if b[2] == 1 && b.len() == 3 {
            return Err(DecodeError(
                "transaction data continuation made no progress",
            ));
        }
        Ok(Self {
            request,
            more: b[2] == 1,
            bytes: b[3..].to_vec(),
            raw: b.to_vec(),
        })
    }
}
impl Operation for TransactionDataRequest {
    type Output = TransactionDataPage;
    fn encode(&self) -> Result<(u8, Vec<u8>), Error> {
        Ok((206, self.payload()))
    }
    fn decode(&self, b: &[u8]) -> Result<Self::Output, DecodeError> {
        TransactionDataPage::decode(*self, b)
    }
}

/// Neither layout resolves the contradictory TLP wording. Select only with
/// applicable evidence. Raw records never require selecting either interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionFieldLayout {
    TaggedValues,
    TlpTaggedValues,
}
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionValue {
    Scalar(Value),
    String(Vec<u8>),
}
#[derive(Debug, Clone, PartialEq)]
pub struct TransactionField {
    pub tlp: Option<Tlp>,
    pub tag: u8,
    pub raw: Vec<u8>,
    pub value: TransactionValue,
}
/// Decode one reassembled record. Unknown tags/truncation return an error without
/// guessing a next boundary; the caller retains the original record independently.
pub fn decode_transaction_fields(
    bytes: &[u8],
    layout: TransactionFieldLayout,
) -> Result<Vec<TransactionField>, DecodeError> {
    use crate::DataType::*;
    let mut rest = bytes;
    let mut fields = Vec::new();
    while !rest.is_empty() {
        let tlp = if layout == TransactionFieldLayout::TlpTaggedValues {
            let b = rest
                .get(..3)
                .ok_or(DecodeError("truncated transaction field TLP"))?;
            rest = &rest[3..];
            Some(crate::Tlp::new(b[0], b[1], b[2]))
        } else {
            None
        };
        let (&tag, tail) = rest
            .split_first()
            .ok_or(DecodeError("missing transaction field tag"))?;
        rest = tail;
        // This vocabulary is specific to Table 2-29, not event type tags.
        let kind = match tag {
            1 => UInt8,
            2 => Int8,
            3 => UInt16,
            4 => Int16,
            5 => UInt32,
            6 => Int32,
            7 => Float,
            8 => Double,
            9 => Ascii(3),
            10 => Ascii(7),
            11 => Ascii(10),
            12 => Ascii(20),
            14 => Ascii(30),
            15 => Ascii(40),
            17 => Binary,
            18 => Tlp,
            20 => Time,
            _ => {
                return Err(DecodeError(
                    "unknown transaction type tag; width unavailable",
                ));
            }
        };
        let raw = rest
            .get(..kind.width())
            .ok_or(DecodeError("truncated transaction field value"))?;
        rest = &rest[kind.width()..];
        let value = if matches!(kind, Ascii(_)) {
            TransactionValue::String(raw.to_vec())
        } else {
            TransactionValue::Scalar(
                Value::decode(kind, raw).map_err(|_| DecodeError("invalid transaction scalar"))?,
            )
        };
        fields.push(TransactionField {
            tlp,
            tag,
            raw: raw.to_vec(),
            value,
        });
    }
    Ok(fields)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransactionInventoryProgress {
    pub pages: Vec<TransactionListPage>,
    pub next_offset: Option<u16>,
    pub complete: bool,
}
#[derive(Debug, thiserror::Error)]
#[error("transaction inventory failed: {error}")]
pub struct TransactionInventoryError {
    pub progress: TransactionInventoryProgress,
    #[source]
    pub error: etherbird::Error<Error>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransactionReadProgress {
    pub pages: Vec<TransactionDataPage>,
    pub bytes: Vec<u8>,
    pub next_offset: Option<u16>,
    pub complete: bool,
}
#[derive(Debug, thiserror::Error)]
#[error("transaction read failed: {error}")]
pub struct TransactionReadError {
    pub progress: TransactionReadProgress,
    #[source]
    pub error: etherbird::Error<Error>,
}
struct InventoryRead {
    request: TransactionListRequest,
    max_pages: usize,
    progress: Arc<Mutex<TransactionInventoryProgress>>,
}
#[etherbird::async_trait]
impl crate::Workflow for InventoryRead {
    type Output = ();
    async fn run(self, c: &mut crate::ManagedContext<'_>) -> Result<(), Error> {
        let mut q = self.request;
        for _ in 0..self.max_pages {
            let page = c.list_transactions(q).await?;
            let mut p = self.progress.lock().unwrap();
            if p.pages.first().is_some_and(|first| {
                first.description != page.description || first.payload_size != page.payload_size
            }) {
                return Err(Error::HistoryChanged(
                    "transaction segment metadata changed",
                ));
            }
            let more = page.more;
            let next = if more {
                Some(q.offset.checked_add(page.entries.len() as u16).ok_or(
                    Error::InvalidResponse("transaction inventory offset overflow"),
                )?)
            } else {
                None
            };
            p.pages.push(page);
            p.next_offset = next;
            if let Some(offset) = next {
                q.offset = offset;
            } else {
                p.complete = true;
                return Ok(());
            }
        }
        Err(Error::InvalidRequest(
            "transaction inventory page budget exhausted",
        ))
    }
}
struct DataRead {
    request: TransactionDataRequest,
    payload_size: u16,
    max_pages: usize,
    progress: Arc<Mutex<TransactionReadProgress>>,
}
#[etherbird::async_trait]
impl crate::Workflow for DataRead {
    type Output = ();
    async fn run(self, c: &mut crate::ManagedContext<'_>) -> Result<(), Error> {
        let mut q = self.request;
        for _ in 0..self.max_pages {
            let page = c.read_transaction_page(q).await?;
            let mut p = self.progress.lock().unwrap();
            let end = usize::from(q.offset) + page.bytes.len();
            if end > usize::from(self.payload_size)
                || (page.more && end >= usize::from(self.payload_size))
                || (!page.more && end != usize::from(self.payload_size))
            {
                return Err(Error::InvalidResponse(
                    "transaction continuation/length disagrees with declared payload size",
                ));
            }
            p.bytes.extend(&page.bytes);
            let more = page.more;
            p.pages.push(page);
            p.next_offset = if more { Some(end as u16) } else { None };
            if !more {
                p.complete = true;
                return Ok(());
            }
            q.offset = end as u16;
        }
        Err(Error::InvalidRequest(
            "transaction data page budget exhausted",
        ))
    }
}
impl crate::Client {
    pub async fn list_transactions(
        &self,
        q: TransactionListRequest,
    ) -> Result<TransactionListPage, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::transaction_list(q))
            .await
    }
    pub async fn read_transaction_page(
        &self,
        q: TransactionDataRequest,
    ) -> Result<TransactionDataPage, etherbird::Error<Error>> {
        self.execute_prepared(crate::operations::transaction_data(q))
            .await
    }
    /// Inventory in device storage order, without rollover sorting or gap-free claims.
    pub async fn transaction_inventory(
        &self,
        q: TransactionListRequest,
        max_pages: usize,
    ) -> Result<TransactionInventoryProgress, TransactionInventoryError> {
        let p = Arc::new(Mutex::new(TransactionInventoryProgress {
            next_offset: Some(q.offset),
            ..Default::default()
        }));
        if max_pages == 0 || max_pages > 65536 {
            return Err(TransactionInventoryError {
                progress: p.lock().unwrap().clone(),
                error: etherbird::Error::Operation(Error::InvalidRequest(
                    "transaction page budget must be 1–65536",
                )),
            });
        }
        crate::operations::transaction_list(q)
            .validate(self.profile())
            .map_err(|error| TransactionInventoryError {
                progress: p.lock().unwrap().clone(),
                error: etherbird::Error::Operation(error),
            })?;
        let result = self
            .workflow(InventoryRead {
                request: q,
                max_pages,
                progress: p.clone(),
            })
            .await;
        let progress = p.lock().unwrap().clone();
        result
            .map(|()| progress.clone())
            .map_err(|error| TransactionInventoryError { progress, error })
    }
    /// Reassemble from offset zero using caller-supplied command-1 payload size.
    /// One exclusive workflow, bounded pages, no retry/resume across generations.
    pub async fn read_transaction(
        &self,
        segment: TransactionSegment,
        number: u16,
        payload_size: u16,
        max_pages: usize,
    ) -> Result<TransactionReadProgress, TransactionReadError> {
        let p = Arc::new(Mutex::new(TransactionReadProgress {
            next_offset: Some(0),
            ..Default::default()
        }));
        if max_pages == 0 || max_pages > 65536 {
            return Err(TransactionReadError {
                progress: p.lock().unwrap().clone(),
                error: etherbird::Error::Operation(Error::InvalidRequest(
                    "transaction page budget must be 1–65536",
                )),
            });
        }
        crate::operations::transaction_data(TransactionDataRequest::new(segment, number, 0))
            .validate(self.profile())
            .map_err(|error| TransactionReadError {
                progress: p.lock().unwrap().clone(),
                error: etherbird::Error::Operation(error),
            })?;
        let result = self
            .workflow(DataRead {
                request: TransactionDataRequest::new(segment, number, 0),
                payload_size,
                max_pages,
                progress: p.clone(),
            })
            .await;
        let progress = p.lock().unwrap().clone();
        result
            .map(|()| progress.clone())
            .map_err(|error| TransactionReadError { progress, error })
    }
}
impl crate::Session {
    pub async fn list_transactions(
        &self,
        q: TransactionListRequest,
    ) -> Result<TransactionListPage, Error> {
        self.request_operation(crate::operations::transaction_list(q))
            .await
    }
    pub async fn read_transaction_page(
        &self,
        q: TransactionDataRequest,
    ) -> Result<TransactionDataPage, Error> {
        self.request_operation(crate::operations::transaction_data(q))
            .await
    }
}
impl crate::ManagedContext<'_> {
    pub async fn list_transactions(
        &mut self,
        q: TransactionListRequest,
    ) -> Result<TransactionListPage, Error> {
        self.exchange_operation(crate::operations::transaction_list(q))
            .await
    }
    pub async fn read_transaction_page(
        &mut self,
        q: TransactionDataRequest,
    ) -> Result<TransactionDataPage, Error> {
        self.exchange_operation(crate::operations::transaction_data(q))
            .await
    }
}
