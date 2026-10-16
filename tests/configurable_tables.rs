//! D301180X012 §§2.5–6/Table 2-5/6 and §3.4.10/Table 3-11, pp.2-12/3-56–57.
use rocsteady::*;
use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
fn fw() -> FirmwareVersion {
    FirmwareVersion::new(3, 60).unwrap()
}
fn crc(b: &[u8]) -> u16 {
    let mut c = 0u16;
    for b in b {
        c ^= u16::from(*b);
        for _ in 0..8 {
            c = if c & 1 == 0 {
                c >> 1
            } else {
                (c >> 1) ^ 0xa001
            };
        }
    }
    c
}
fn entries() -> [Tlp; 44] {
    let mut e = [Tlp::new(0, 0, 0); 44];
    e[..4].copy_from_slice(&[
        Tlp::new(91, 0, 2),
        Tlp::new(101, 16, 1),
        Tlp::new(101, 16, 2),
        Tlp::new(91, 0, 2),
    ]);
    e
}
fn definition(table: u8, bits: u32, entries: &[Tlp; 44]) -> Vec<u8> {
    let mut b = vec![99, table, 45, 0];
    b.extend(bits.to_le_bytes());
    for t in entries {
        b.extend([t.point_type, t.logical, t.parameter]);
    }
    b
}
fn def() -> Vec<u8> {
    definition(15, 0x7fc00001, &entries())
}
fn values() -> Vec<Value> {
    vec![
        Value::Ascii("ABCDEFGHIJKLMNOPQRST".into()),
        Value::UInt8(1),
        Value::Float(1.5),
        Value::Ascii("ABCDEFGHIJKLMNOPQRST".into()),
    ]
}
fn data() -> Vec<u8> {
    let mut b = b"ABCDEFGHIJKLMNOPQRST".to_vec();
    b.extend([1, 0, 0, 0xc0, 0x3f]);
    b.extend(b"ABCDEFGHIJKLMNOPQRST");
    b
}
fn read_reply(start: u8, count: u8, data: &[u8]) -> Vec<u8> {
    let mut b = vec![15, start, count, 1, 0, 0xc0, 0x7f];
    b.extend(data);
    b
}
struct Fields;
impl SchemaProvider for Fields {
    fn lookup(&self, _: &SchemaContext, t: Tlp) -> Result<Option<ParameterSchema>, Error> {
        let (kind, range) = match (t.point_type, t.parameter) {
            (91, 2) => (DataType::Ascii(20), None),
            (101, 1 | 4 | 5) => (DataType::UInt8, Some((0., 1.))),
            (101, 2) => (DataType::Float, Some((0., 43200.))),
            _ => return Ok(None),
        };
        Ok(Some(ParameterSchema {
            data_type: kind,
            access: Access::ReadWrite,
            logical_range: Some((t.logical, t.logical)),
            numeric_range: range,
        }))
    }
    fn lookup_write(&self, c: &SchemaContext, t: Tlp) -> Result<Option<ParameterSchema>, Error> {
        self.lookup(c, t)
    }
}
struct Tight;
impl ProtocolProfile for Tight {
    fn metadata(&self) -> ProfileMetadata {
        DefaultProfile.metadata()
    }
    fn capability(&self, op: u8) -> Capability {
        DefaultProfile.capability(op)
    }
    fn limits(&self, op: u8) -> PayloadLimits {
        match op {
            10 => PayloadLimits {
                request: 3,
                response: 27,
            },
            11 => PayloadLimits {
                request: 23,
                response: 0,
            },
            _ => PayloadLimits::default(),
        }
    }
}
#[derive(Clone)]
enum Reply {
    Data(Vec<u8>),
    Device,
    Disconnect,
    Stall,
    Wait(Arc<tokio::sync::Notify>, Vec<u8>),
}
type Steps = Arc<Mutex<VecDeque<(u8, Vec<u8>, Reply)>>>;
struct Peer {
    endpoint: String,
    queue: Steps,
    calls: Arc<Mutex<Vec<u8>>>,
    connections: Arc<AtomicUsize>,
    seen: Arc<tokio::sync::Notify>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Peer {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl Peer {
    async fn start(steps: Vec<(u8, Vec<u8>, Reply)>) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = listener.local_addr().unwrap().to_string();
        let queue = Arc::new(Mutex::new(VecDeque::from(steps)));
        let calls = Arc::new(Mutex::new(Vec::new()));
        let connections = Arc::new(AtomicUsize::new(0));
        let seen = Arc::new(tokio::sync::Notify::new());
        let (q, n, conn, notice) = (
            queue.clone(),
            calls.clone(),
            connections.clone(),
            seen.clone(),
        );
        let task = tokio::spawn(async move {
            let mut tasks = tokio::task::JoinSet::new();
            loop {
                tokio::select! {
                 Some(result)=tasks.join_next(),if !tasks.is_empty()=>{result.unwrap();},
                 accepted=listener.accept()=>{let(mut socket,_)=accepted.unwrap();conn.fetch_add(1,Ordering::SeqCst);let(q,n,notice)=(q.clone(),n.clone(),notice.clone());tasks.spawn(async move{loop{
                  let mut h=[0;6];if socket.read_exact(&mut h).await.is_err(){break;}assert_eq!(&h[..4],&[1,2,1,0]);let mut b=vec![0;usize::from(h[5])+2];socket.read_exact(&mut b).await.unwrap();let mut wire=h.to_vec();wire.extend(&b[..b.len()-2]);assert_eq!(crc(&wire).to_le_bytes(),b[b.len()-2..]);
                  let(opcode,data)=if h[4]==7{(7,vec![1,2,3,8,10,0xea,7,5])}else{
                    let(op,expected,reply)=q.lock().unwrap().pop_front().expect("unexpected table request/replay");assert_eq!((h[4],&b[..b.len()-2]),(op,expected.as_slice()));n.lock().unwrap().push(op);notice.notify_one();
                    match reply{Reply::Data(data)=>(op,data),Reply::Device=>(255,vec![25,6,20,8]),Reply::Disconnect=>return,Reply::Stall=>std::future::pending().await,Reply::Wait(release,data)=>{release.notified().await;(op,data)}}
                  };
                  let mut reply=vec![1,0,1,2,opcode,data.len() as u8];reply.extend(data);reply.extend(crc(&reply).to_le_bytes());socket.write_all(&reply).await.unwrap();
                 }});}
                }
            }
        });
        Self {
            endpoint,
            queue,
            calls,
            connections,
            seen,
            task,
        }
    }
    fn client(&self, profile: Arc<dyn ProtocolProfile>) -> Client {
        Client::tcp(
            &self.endpoint,
            ClientConfig {
                profile,
                heartbeat_interval: None,
                ..Default::default()
            },
        )
        .unwrap()
    }
}
fn metadata(bytes: Vec<u8>) -> (u8, Vec<u8>, Reply) {
    (167, vec![99, 15, 45, 0], Reply::Data(bytes))
}
fn discovery() -> Vec<(u8, Vec<u8>, Reply)> {
    vec![metadata(def()), metadata(def())]
}
#[test]
fn table_definition_and_read_wire_literals_and_bounds() {
    // REQ-099-DEFINITION: all 45 parameters, first FLOAT plus 44 TLPs; no float ordering.
    for table in [0, 15] {
        for bits in [0, 0x80000000, 0x7fc00001, 0x7f800000, 0xffffffff] {
            let q = TableDefinitionRequest::new(table).unwrap();
            assert_eq!(q.payload(), [99, table, 45, 0]);
            let mut e = [Tlp::new(0, 0, 0); 44];
            for (i, t) in e.iter_mut().enumerate() {
                *t = Tlp::new(91 + i as u8, i as u8, 255 - i as u8);
            }
            let bytes = definition(table, bits, &e);
            let decoded = q.decode_reply(&bytes).unwrap();
            assert_eq!(decoded.entries, e);
            assert_eq!(decoded.revision.bits(), bits);
            for n in 0..bytes.len() {
                assert!(q.decode_reply(&bytes[..n]).is_err());
            }
            let mut trailing = bytes.clone();
            trailing.push(0);
            assert!(q.decode_reply(&trailing).is_err());
            for i in 0..4 {
                let mut bad = bytes.clone();
                bad[i] ^= 1;
                assert!(q.decode_reply(&bad).is_err());
            }
        }
    }
    assert!(TableDefinitionRequest::new(16).is_err());
    // REQ-010-WIRE: echoed table/start/count, raw FLOAT revision, schema-ordered data.
    let q = TableReadBatch::new(
        15,
        0,
        TableRevision::from_bits(0x7fc00001),
        vec![
            DataType::Ascii(20),
            DataType::UInt8,
            DataType::Float,
            DataType::Ascii(20),
        ],
    )
    .unwrap();
    let reply = read_reply(0, 4, &data());
    assert_eq!(q.payload(), [15, 0, 4]);
    assert_eq!(q.decode_reply(&reply).unwrap().values, values());
    for n in 0..reply.len() {
        assert!(q.decode_reply(&reply[..n]).is_err());
    }
    for i in 0..3 {
        let mut bad = reply.clone();
        bad[i] ^= 1;
        assert!(q.decode_reply(&bad).is_err());
    }
    let mut changed = reply.clone();
    changed[3] ^= 1;
    assert!(matches!(
        q.decode_reply(&changed),
        Err(Error::DiscoveryChanged(_))
    ));
    let mut new_width = changed[..8].to_vec();
    new_width[7] = 255;
    assert!(matches!(
        q.decode_reply(&new_width),
        Err(Error::DiscoveryChanged(_))
    ));
    let mut extra = reply;
    extra.push(0);
    assert!(q.decode_reply(&extra).is_err());
    // REQ-010-RANGE: boundaries 0–15, 0–43, 1–44 and start+count<=44.
    for (table, start, count, valid) in [
        (0, 0, 44, true),
        (15, 43, 1, true),
        (16, 0, 1, false),
        (0, 44, 1, false),
        (0, 43, 2, false),
        (0, 0, 0, false),
        (0, 0, 45, false),
    ] {
        assert_eq!(
            TableReadBatch::new(
                table,
                start,
                TableRevision::from_bits(0),
                vec![DataType::UInt8; count]
            )
            .is_ok(),
            valid
        );
    }
    assert!(
        TableReadBatch::new(0, 0, TableRevision::from_bits(0), vec![DataType::Ascii(0)]).is_err()
    );
    assert!(
        TableReadBatch::new(
            0,
            0,
            TableRevision::from_bits(0),
            vec![DataType::Ascii(230); 2]
        )
        .is_err()
    );
}
#[tokio::test]
async fn table_mixed_reads_and_writes_match_independent_literals_and_budgets() {
    // REQ-010-WIRE / REQ-011-WIRE / POL-TABLE-BUDGET: independently packed mixed
    // widths and repeated mapped TLP, with both unsplit and tightly bounded batches.
    for tight in [false, true] {
        let mut steps = discovery();
        steps.push(metadata(def()));
        if tight {
            steps.extend([
                (
                    10,
                    vec![15, 0, 1],
                    Reply::Data(read_reply(0, 1, &data()[..20])),
                ),
                (
                    10,
                    vec![15, 1, 2],
                    Reply::Data(read_reply(1, 2, &data()[20..25])),
                ),
                (
                    10,
                    vec![15, 3, 1],
                    Reply::Data(read_reply(3, 1, &data()[25..])),
                ),
            ]);
        } else {
            steps.push((10, vec![15, 0, 4], Reply::Data(read_reply(0, 4, &data()))));
        }
        steps.push(metadata(def()));
        let chunks = if tight {
            vec![
                (0, 1, data()[..20].to_vec()),
                (1, 2, data()[20..25].to_vec()),
                (3, 1, data()[25..].to_vec()),
            ]
        } else {
            vec![(0, 4, data())]
        };
        for (start, count, bytes) in &chunks {
            steps.push(metadata(def()));
            let mut payload = vec![15, *start, *count];
            payload.extend(bytes);
            steps.push((11, payload, Reply::Data(vec![])));
        }
        let peer = Peer::start(steps).await;
        let client = peer.client(if tight {
            Arc::new(Tight)
        } else {
            Arc::new(DefaultProfile)
        });
        let table = client.discover_configurable_table(15, fw()).await.unwrap();
        let read = TableReadPlan::build(
            &table,
            &Fields,
            &SchemaContext::default(),
            0,
            4,
            client.profile(),
        )
        .unwrap();
        assert_eq!(read.batches().len(), chunks.len());
        let result = client.read_configurable_table(read).await.unwrap();
        assert!(result.complete);
        assert_eq!(
            result
                .batches
                .into_iter()
                .flat_map(|b| b.values)
                .collect::<Vec<_>>(),
            values()
        );
        let write = table
            .write_plan(
                &Fields,
                &SchemaContext::default(),
                0,
                &values(),
                client.profile(),
            )
            .unwrap();
        assert_eq!(write.batches().len(), chunks.len());
        for (batch, (start, count, bytes)) in write.batches().iter().zip(chunks) {
            let mut literal = vec![15, start, count];
            literal.extend(bytes);
            assert_eq!(batch.opcode(), 11);
            assert_eq!(batch.payload(), literal);
        }
        assert_eq!(
            client.write_plan(write).await.unwrap().batches,
            vec![WriteOutcome::Acknowledged; if tight { 3 } else { 1 }]
        );
        assert!(peer.queue.lock().unwrap().is_empty());
        client.stop().await;
    }
}
#[tokio::test]
async fn table_planning_preflight_validates_values_ranges_and_budgets() {
    let peer = Peer::start(discovery()).await;
    let client = peer.client(Arc::new(DefaultProfile));
    let table = client.discover_configurable_table(15, fw()).await.unwrap();
    // POL-TABLE-PREFLIGHT: reject the entire input for invalid ranges, unknown
    // slots, wrong value types, numeric bounds/nonfinite values or invalid text.
    for (case, start, vals) in [
        ("empty input", 0, vec![]),
        ("range overflow", 43, vec![Value::UInt8(0); 2]),
        ("unknown enum", 1, vec![Value::UInt8(2)]),
        ("wrong type", 1, vec![Value::Binary(1)]),
        ("below minimum", 2, vec![Value::Float(-1.)]),
        ("above maximum", 2, vec![Value::Float(43201.)]),
        ("NaN", 2, vec![Value::Float(f32::NAN)]),
        ("infinity", 2, vec![Value::Float(f32::INFINITY)]),
        ("wrong text width", 0, vec![Value::Ascii("short".into())]),
        ("NUL text", 0, vec![Value::Ascii("\0".repeat(20))]),
        ("unconfigured slot", 4, vec![Value::UInt8(0)]),
    ] {
        assert!(
            table
                .write_plan(
                    &Fields,
                    &SchemaContext::default(),
                    start,
                    &vals,
                    client.profile()
                )
                .is_err(),
            "{case}: start {start}"
        );
    }
    // POL-TABLE-PREFLIGHT: reads apply the same nonempty configured-range rule.
    for (start, count) in [(0, 0), (43, 2), (44, 1), (4, 1)] {
        assert!(
            TableReadPlan::build(
                &table,
                &Fields,
                &SchemaContext::default(),
                start,
                count,
                client.profile()
            )
            .is_err(),
            "read range: start {start}, count {count}"
        );
    }
    // POL-TABLE-PREFLIGHT: readable metadata alone never opts into writes.
    struct ReadOnly;
    impl SchemaProvider for ReadOnly {
        fn lookup(&self, c: &SchemaContext, t: Tlp) -> Result<Option<ParameterSchema>, Error> {
            Fields.lookup(c, t)
        }
    }
    assert!(
        table
            .write_plan(
                &ReadOnly,
                &SchemaContext::default(),
                1,
                &[Value::UInt8(1)],
                client.profile()
            )
            .is_err()
    );
    // POL-TABLE-PREFLIGHT: provider generation must match the bound snapshot.
    struct Bound(u64);
    impl SchemaProvider for Bound {
        fn generation(&self) -> Result<Option<u64>, Error> {
            Ok(Some(self.0))
        }
        fn lookup(&self, c: &SchemaContext, t: Tlp) -> Result<Option<ParameterSchema>, Error> {
            Fields.lookup(c, t)
        }
    }
    assert!(
        TableReadPlan::build(
            &table,
            &Bound(u64::MAX),
            &SchemaContext::default(),
            0,
            1,
            client.profile()
        )
        .is_err()
    );
    // POL-TABLE-BUDGET / POL-TABLE-PREFLIGHT: seven reply header bytes and
    // three request header bytes must fit together with each indivisible value.
    struct Limits {
        read: usize,
        write: usize,
        available: bool,
    }
    impl ProtocolProfile for Limits {
        fn metadata(&self) -> ProfileMetadata {
            DefaultProfile.metadata()
        }
        fn capability(&self, op: u8) -> Capability {
            if op == 10 && !self.available {
                Capability::Unavailable
            } else {
                DefaultProfile.capability(op)
            }
        }
        fn limits(&self, op: u8) -> PayloadLimits {
            match op {
                10 => PayloadLimits {
                    request: 3,
                    response: self.read,
                },
                11 => PayloadLimits {
                    request: self.write,
                    response: 0,
                },
                _ => PayloadLimits::default(),
            }
        }
    }
    for (case, read, write, available) in [
        ("exact fit", 8, 4, true),
        ("indivisible value exceeds budget", 7, 3, true),
        ("read unavailable", 8, 4, false),
    ] {
        let profile = Limits {
            read,
            write,
            available,
        };
        let read_plan =
            TableReadPlan::build(&table, &Fields, &SchemaContext::default(), 1, 1, &profile);
        assert_eq!(read_plan.is_ok(), read == 8 && available, "read: {case}");
        if let Ok(plan) = read_plan {
            assert_eq!(plan.batches()[0].response_size(), 8);
        }
        let write_plan = table.write_plan(
            &Fields,
            &SchemaContext::default(),
            1,
            &[Value::UInt8(1)],
            &profile,
        );
        assert_eq!(write_plan.is_ok(), write == 4, "write: {case}");
        if let Ok(plan) = write_plan {
            assert_eq!(plan.batches()[0].payload(), [15, 1, 1, 1]);
        }
    }

    // POL-TABLE-PREFLIGHT: all planning above leaves only the discovery requests.
    assert_eq!(peer.calls.lock().unwrap().as_slice(), [167, 167]);
    client.stop().await;
}
#[tokio::test]
async fn table_changed_mapping_version_and_postcheck_preserve_explicit_read_progress() {
    // POL-TABLE-CHANGE: no revision increment is assumed; full mapping differences
    // invalidate snapshots, and confirmed read batches survive a failed postcheck.
    for mode in [0, 1, 2, 3] {
        let mut changed = entries();
        changed[0] = Tlp::new(91, 0, 1);
        let changed = definition(15, 0x7fc00001, &changed);
        let mut steps = discovery();
        if mode == 0 {
            steps.push(metadata(changed));
        } else if mode == 1 {
            steps.push(metadata(def()));
            let mut b = read_reply(0, 4, &data());
            b[3] ^= 1;
            steps.push((10, vec![15, 0, 4], Reply::Data(b)));
        } else {
            steps.push(metadata(def()));
            steps.push((10, vec![15, 0, 4], Reply::Data(read_reply(0, 4, &data()))));
            steps.push(if mode == 2 {
                metadata(changed)
            } else {
                (167, vec![99, 15, 45, 0], Reply::Disconnect)
            });
        }
        let peer = Peer::start(steps).await;
        let client = peer.client(Arc::new(DefaultProfile));
        let table = client.discover_configurable_table(15, fw()).await.unwrap();
        let plan = TableReadPlan::build(
            &table,
            &Fields,
            &SchemaContext::default(),
            0,
            4,
            client.profile(),
        )
        .unwrap();
        let failure = client.read_configurable_table(plan).await.unwrap_err();
        assert!(!failure.progress.complete);
        assert_eq!(
            failure.progress.batches.len(),
            if mode >= 2 { 1 } else { 0 }
        );
        if mode < 3 {
            assert!(matches!(
                failure.error,
                etherbird::Error::Operation(Error::DiscoveryChanged(_))
            ));
        }
        client.read_clock().await.unwrap();
        assert!(peer.queue.lock().unwrap().is_empty());
        client.stop().await;
    }
}
#[tokio::test]
async fn table_write_failures_and_rechecks_retain_acknowledged_prefix_without_replay() {
    // REQ-011-ACK / POL-TABLE-WRITE-OUTCOME: table checks precede each batch;
    // malformed/lost acknowledgements stay uncertain, raw OP-255 is preserved.
    for mode in [0, 1, 2, 3] {
        let mut steps = discovery();
        steps.extend([
            metadata(def()),
            (
                11,
                {
                    let mut p = vec![15, 0, 1];
                    p.extend(&data()[..20]);
                    p
                },
                Reply::Data(vec![]),
            ),
        ]);
        if mode == 3 {
            let mut changed = entries();
            changed[1] = Tlp::new(101, 16, 4);
            steps.push(metadata(definition(15, 0x7fc00001, &changed)));
        } else {
            steps.push(metadata(def()));
            let mut p = vec![15, 1, 2];
            p.extend(&data()[20..25]);
            steps.push((
                11,
                p,
                match mode {
                    0 => Reply::Disconnect,
                    1 => Reply::Data(vec![0]),
                    _ => Reply::Device,
                },
            ));
        }
        let peer = Peer::start(steps).await;
        let client = peer.client(Arc::new(Tight));
        let table = client.discover_configurable_table(15, fw()).await.unwrap();
        let plan = table
            .write_plan(
                &Fields,
                &SchemaContext::default(),
                0,
                &values(),
                client.profile(),
            )
            .unwrap();
        let e = client.write_plan(plan).await.unwrap_err();
        assert_eq!(
            e.progress.batches,
            [
                WriteOutcome::Acknowledged,
                match mode {
                    2 => WriteOutcome::DeviceError,
                    3 => WriteOutcome::NotAttempted,
                    _ => WriteOutcome::Uncertain,
                },
                WriteOutcome::NotAttempted
            ]
        );
        if mode == 2 {
            let etherbird::Error::Operation(Error::Device(errors)) = e.error else {
                panic!()
            };
            assert_eq!(
                errors
                    .iter()
                    .map(|e| (e.code, e.offset))
                    .collect::<Vec<_>>(),
                [(25, 6), (20, 8)]
            );
        }
        client.read_clock().await.unwrap();
        assert_eq!(
            peer.calls
                .lock()
                .unwrap()
                .iter()
                .filter(|o| **o == 11)
                .count(),
            if mode == 3 { 1 } else { 2 }
        );
        assert!(peer.queue.lock().unwrap().is_empty());
        client.stop().await;
    }
}
#[tokio::test]
async fn table_discovery_changes_and_stale_or_foreign_snapshots_fail_before_table_io() {
    // POL-TABLE-GENERATION / POL-TABLE-DISCOVERY: two independent observations;
    // old/foreign handles cannot read or write through a replacement generation.
    let mut changed = def();
    changed[4] ^= 1;
    let peer = Peer::start(vec![metadata(def()), metadata(changed)]).await;
    let client = peer.client(Arc::new(DefaultProfile));
    assert!(matches!(
        client.discover_configurable_table(15, fw()).await,
        Err(etherbird::Error::Operation(Error::DiscoveryChanged(_)))
    ));
    client.stop().await;
    let mut steps = discovery();
    steps.push((254, vec![], Reply::Disconnect));
    let peer = Peer::start(steps).await;
    let client = peer.client(Arc::new(DefaultProfile));
    let table = client.discover_configurable_table(15, fw()).await.unwrap();
    let other = peer.client(Arc::new(DefaultProfile));
    other.connected().await.unwrap();
    let plan = table
        .write_plan(
            &Fields,
            &SchemaContext::default(),
            1,
            &[Value::UInt8(1)],
            other.profile(),
        )
        .unwrap();
    assert_eq!(
        other.write_plan(plan).await.unwrap_err().progress.batches,
        [WriteOutcome::NotAttempted]
    );
    let plan = TableReadPlan::build(
        &table,
        &Fields,
        &SchemaContext::default(),
        1,
        1,
        other.profile(),
    )
    .unwrap();
    assert!(
        other
            .read_configurable_table(plan)
            .await
            .unwrap_err()
            .progress
            .batches
            .is_empty()
    );
    other.stop().await;
    client.request(254, vec![]).await.unwrap_err();
    client.read_clock().await.unwrap();
    let plan = table
        .write_plan(
            &Fields,
            &SchemaContext::default(),
            1,
            &[Value::UInt8(1)],
            client.profile(),
        )
        .unwrap();
    assert_eq!(
        client.write_plan(plan).await.unwrap_err().progress.batches,
        [WriteOutcome::NotAttempted]
    );
    // POL-TABLE-GENERATION: replacement rejects old read snapshots as well as writes.
    let plan = TableReadPlan::build(
        &table,
        &Fields,
        &SchemaContext::default(),
        1,
        1,
        client.profile(),
    )
    .unwrap();
    let failure = client.read_configurable_table(plan).await.unwrap_err();
    assert!(failure.progress.batches.is_empty());
    assert!(!failure.progress.complete);
    assert_eq!(peer.calls.lock().unwrap().as_slice(), [167, 167, 254]);
    client.stop().await;
}

#[test]
fn table_read_values_keep_scalar_widths_unknown_codes_and_ieee_bits() {
    // REQ-010-VALUES: schema widths, independent numeric/TLP/time literals and raw IEEE payloads.
    let kinds = vec![
        DataType::UInt8,
        DataType::Int8,
        DataType::UInt16,
        DataType::Int16,
        DataType::UInt32,
        DataType::Int32,
        DataType::Float,
        DataType::Double,
        DataType::Binary,
        DataType::Tlp,
        DataType::Time,
    ];
    let raw = vec![
        vec![255],
        vec![128],
        vec![0x34, 0x12],
        vec![0, 128],
        vec![255; 4],
        vec![0, 0, 0, 128],
        vec![1, 0, 0xc0, 0x7f],
        vec![0, 0, 0, 0, 0, 0, 0xf0, 0x7f],
        vec![0xff],
        vec![91, 0, 2],
        vec![255; 4],
    ];
    let q = TableReadBatch::new(15, 0, TableRevision::from_bits(0x7fc00001), kinds).unwrap();
    let bytes = read_reply(0, 11, &raw.concat());
    let reply = q.decode_reply(&bytes).unwrap();
    assert_eq!(
        reply
            .values
            .iter()
            .map(|v| v.encode().unwrap())
            .collect::<Vec<_>>(),
        raw
    );
}
#[tokio::test]
async fn table_profile_budget_and_invalid_identity_fail_before_connect_or_write() {
    // POL-TABLE-PREFLIGHT: receiving-client profile limits cover the definition checks
    // and payload, including plans constructed against a different profile.
    struct Limits(u8);
    impl ProtocolProfile for Limits {
        fn metadata(&self) -> ProfileMetadata {
            DefaultProfile.metadata()
        }
        fn capability(&self, op: u8) -> Capability {
            if self.0 == 0 && op == 11 {
                Capability::Unavailable
            } else {
                DefaultProfile.capability(op)
            }
        }
        fn limits(&self, op: u8) -> PayloadLimits {
            if self.0 == 1 && op == 167 {
                PayloadLimits {
                    request: 4,
                    response: 139,
                }
            } else {
                PayloadLimits::default()
            }
        }
    }
    for mode in [0, 1] {
        let peer = Peer::start(discovery()).await;
        let client = peer.client(Arc::new(DefaultProfile));
        let table = client.discover_configurable_table(15, fw()).await.unwrap();
        let blocked = peer.client(Arc::new(Limits(mode)));
        let plan = table
            .write_plan(
                &Fields,
                &SchemaContext::default(),
                1,
                &[Value::UInt8(1)],
                client.profile(),
            )
            .unwrap();
        assert_eq!(
            blocked.write_plan(plan).await.unwrap_err().progress.batches,
            [WriteOutcome::NotAttempted]
        );
        assert_eq!(peer.connections.load(Ordering::SeqCst), 1);
        blocked.stop().await;
        client.stop().await;
    }
    let peer = Peer::start(vec![]).await;
    let client = peer.client(Arc::new(DefaultProfile));
    assert!(client.discover_configurable_table(16, fw()).await.is_err());
    assert!(
        client
            .discover_configurable_table(0, FirmwareVersion::new(3, 61).unwrap())
            .await
            .is_err()
    );
    assert_eq!(peer.connections.load(Ordering::SeqCst), 0);
    client.stop().await;
}
#[tokio::test]
async fn table_deadlines_abort_and_stop_do_not_resume_or_replay_pending_work() {
    // POL-TABLE-CANCEL: both read and write workflows stall at the selected exchange,
    // retain correct uncertainty/progress, and never continue after managed replacement.
    for write in [false, true] {
        for mode in [0, 1, 2] {
            let mut steps = discovery();
            steps.push(metadata(def()));
            steps.push(if write {
                (11, vec![15, 1, 1, 1], Reply::Stall)
            } else {
                (10, vec![15, 1, 1], Reply::Stall)
            });
            let peer = Peer::start(steps).await;
            let client = Client::tcp(
                &peer.endpoint,
                ClientConfig {
                    heartbeat_interval: None,
                    call_timeout: Duration::from_millis(80),
                    ..Default::default()
                },
            )
            .unwrap();
            let table = client.discover_configurable_table(15, fw()).await.unwrap();
            let c = client.clone();
            let task = tokio::spawn(async move {
                if write {
                    let plan = table
                        .write_plan(
                            &Fields,
                            &SchemaContext::default(),
                            1,
                            &[Value::UInt8(1)],
                            c.profile(),
                        )
                        .unwrap();
                    let e = c.write_plan(plan).await.unwrap_err();
                    assert_eq!(e.progress.batches, [WriteOutcome::Uncertain]);
                } else {
                    let plan = TableReadPlan::build(
                        &table,
                        &Fields,
                        &SchemaContext::default(),
                        1,
                        1,
                        c.profile(),
                    )
                    .unwrap();
                    let e = c.read_configurable_table(plan).await.unwrap_err();
                    assert!(e.progress.batches.is_empty());
                    assert!(!e.progress.complete);
                }
            });
            tokio::time::timeout(Duration::from_secs(2), async {
                while peer.calls.lock().unwrap().len() < 4 {
                    peer.seen.notified().await;
                }
            })
            .await
            .unwrap();
            if mode == 1 {
                task.abort();
                assert!(task.await.unwrap_err().is_cancelled());
            } else {
                if mode == 2 {
                    client.stop().await;
                }
                task.await.unwrap();
            }
            if mode != 2 {
                let mut states = client.subscribe_state();
                tokio::time::timeout(Duration::from_secs(5), async {
                    loop {
                        states.borrow_and_update();
                        if peer.connections.load(Ordering::SeqCst) >= 2
                            && client.state() == ResourceState::Connected
                        {
                            break;
                        }
                        states.changed().await.unwrap();
                    }
                })
                .await
                .unwrap();
                client.read_clock().await.unwrap();
            }
            assert_eq!(peer.calls.lock().unwrap().len(), 4);
            assert!(peer.queue.lock().unwrap().is_empty());
            client.stop().await;
        }
    }
}
#[tokio::test]
async fn table_exclusive_workflow_and_queued_write_cancellation_preserve_send_gate() {
    // POL-TABLE-OWNERSHIP / POL-TABLE-CANCEL: queued clones cannot enter between
    // definition check and table read; a dropped queued OP-11 plan never sends later.
    use std::future::Future;
    let release = Arc::new(tokio::sync::Notify::new());
    let mut steps = discovery();
    steps.push((
        167,
        vec![99, 15, 45, 0],
        Reply::Wait(release.clone(), def()),
    ));
    steps.push((10, vec![15, 1, 1], Reply::Data(read_reply(1, 1, &[1]))));
    steps.push(metadata(def()));
    steps.push((
        10,
        vec![15, 3, 1],
        Reply::Data(read_reply(3, 1, &data()[25..])),
    ));
    let peer = Peer::start(steps).await;
    let client = peer.client(Arc::new(DefaultProfile));
    let table = client.discover_configurable_table(15, fw()).await.unwrap();
    let read = TableReadPlan::build(
        &table,
        &Fields,
        &SchemaContext::default(),
        1,
        1,
        client.profile(),
    )
    .unwrap();
    let c = client.clone();
    let task = tokio::spawn(async move { c.read_configurable_table(read).await });
    tokio::time::timeout(Duration::from_secs(2), async {
        while peer.calls.lock().unwrap().len() < 3 {
            peer.seen.notified().await;
        }
    })
    .await
    .unwrap();
    let write = table
        .write_plan(
            &Fields,
            &SchemaContext::default(),
            1,
            &[Value::UInt8(0)],
            client.profile(),
        )
        .unwrap();
    let mut queued = Box::pin(client.write_plan(write));
    std::future::poll_fn(|cx| {
        assert!(queued.as_mut().poll(cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    drop(queued);
    let mut competing = Box::pin(
        client.read_table_batch(
            TableReadBatch::new(
                15,
                3,
                table.definition().revision,
                vec![DataType::Ascii(20)],
            )
            .unwrap(),
        ),
    );
    std::future::poll_fn(|cx| {
        assert!(competing.as_mut().poll(cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    release.notify_one();
    assert!(task.await.unwrap().unwrap().complete);
    competing.await.unwrap();
    client.read_clock().await.unwrap();
    assert_eq!(
        peer.calls.lock().unwrap().as_slice(),
        [167, 167, 167, 10, 167, 10]
    );
    assert!(peer.queue.lock().unwrap().is_empty());
    client.stop().await;
}
#[cfg(feature = "simulator")]
#[tokio::test]
async fn table_simulator_reads_writes_last_slot_and_retains_effect_after_lost_ack() {
    // REQ-099-SIMULATOR / REQ-011-SIMULATOR / POL-TABLE-NO-REPLAY: explicit map
    // and values; zero-based slot 43 maps PT-99 parameter 44. Injected failure targets
    // OP-11 after dispatch, not its preceding definition check.
    use simulator::{Fault, Model, Simulator};
    for firmware in [FirmwareVersion::new(1, 0).unwrap(), fw()] {
        let mut model = Model::default();
        model.io_point_types[16] = 101;
        model
            .parameters
            .insert(Tlp::new(101, 16, 1), Value::UInt8(0));
        model
            .parameters
            .insert(Tlp::new(101, 16, 2), Value::Float(0.));
        model.parameters.insert(
            Tlp::new(91, 0, 2),
            Value::Ascii("INITIAL STATION     ".into()),
        );
        let mut e = entries();
        e[43] = Tlp::new(101, 16, 1);
        model.configurable_tables.lock().unwrap().insert(
            15,
            ConfigurableTableDefinition {
                table: 15,
                revision: TableRevision::from_bits(0x7fc00001),
                entries: e,
            },
        );
        let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), model)
            .await
            .unwrap();
        let client = Client::tcp(
            server.endpoint().to_string(),
            ClientConfig {
                heartbeat_interval: None,
                ..Default::default()
            },
        )
        .unwrap();
        let table = client
            .discover_configurable_table(15, firmware)
            .await
            .unwrap();
        let write = table
            .write_plan(
                &Fields,
                &SchemaContext::default(),
                0,
                &values(),
                client.profile(),
            )
            .unwrap();
        assert_eq!(
            client.write_plan(write).await.unwrap().batches,
            [WriteOutcome::Acknowledged]
        );
        let read = TableReadPlan::build(
            &table,
            &Fields,
            &SchemaContext::default(),
            0,
            4,
            client.profile(),
        )
        .unwrap();
        let result = client.read_configurable_table(read).await.unwrap();
        assert_eq!(result.batches[0].values, values());
        assert!(result.complete);
        let write = table
            .write_plan(
                &Fields,
                &SchemaContext::default(),
                43,
                &[Value::UInt8(0)],
                client.profile(),
            )
            .unwrap();
        let before = server.stats().requests;
        server.inject_for(11, Fault::Disconnect);
        assert_eq!(
            client.write_plan(write).await.unwrap_err().progress.batches,
            [WriteOutcome::Uncertain]
        );
        assert_eq!(
            client
                .read_parameters(&[Parameter::new(Tlp::new(101, 16, 1), DataType::UInt8)])
                .await
                .unwrap(),
            [Value::UInt8(0)]
        );
        assert_eq!(server.stats().requests - before, 4); // Check, write, replacement setup, explicit read.
        client.stop().await;
        server.stop().await.unwrap();
    }
}
#[cfg(feature = "simulator")]
#[test]
fn table_simulator_rejects_malformed_ranges_and_retains_fixture_partial_effects() {
    // REQ-011-SIMULATOR: apply valid mapped values in order, retain earlier effect
    // after later rejection. Error offsets are fixture table slots, not hardware evidence.
    use simulator::Model;
    let mut model = Model::default();
    model.io_point_types[16] = 101;
    model
        .parameters
        .insert(Tlp::new(101, 16, 1), Value::UInt8(0));
    model
        .parameters
        .insert(Tlp::new(101, 16, 2), Value::Float(0.));
    model.configurable_tables.lock().unwrap().insert(
        15,
        ConfigurableTableDefinition {
            table: 15,
            revision: TableRevision::from_bits(0),
            entries: entries(),
        },
    );
    let mut auth = false;
    for (opcode, data) in [
        (10, vec![]),
        (10, vec![16, 0, 1]),
        (10, vec![15, 44, 1]),
        (10, vec![15, 43, 2]),
        (10, vec![15, 0, 0]),
        (11, vec![15, 1, 1]),
        (11, vec![15, 1, 1, 2]),
        (11, vec![15, 2, 1, 0, 0, 0xc0, 0x7f]),
    ] {
        let response = model
            .respond(
                &protocol::Frame::new(Address::new(1, 2), Address::new(1, 0), opcode, data),
                &mut auth,
            )
            .unwrap();
        assert_eq!(response.opcode, 255);
    }
    let response = model
        .respond(
            &protocol::Frame::new(
                Address::new(1, 2),
                Address::new(1, 0),
                11,
                vec![15, 1, 2, 1, 0, 0, 0xc0, 0x7f],
            ),
            &mut auth,
        )
        .unwrap();
    assert_eq!((response.opcode, response.data), (255, vec![4, 2]));
    assert_eq!(
        model
            .written_parameters
            .lock()
            .unwrap()
            .get(&Tlp::new(101, 16, 1)),
        Some(&Value::UInt8(1))
    );
    assert!(
        !model
            .written_parameters
            .lock()
            .unwrap()
            .contains_key(&Tlp::new(101, 16, 2))
    );
}

#[tokio::test]
async fn table_standalone_reply_validation_and_revision_change_connection_rules() {
    // REQ-010-VALIDATE / POL-TABLE-RETIREMENT: wire errors retire, observed revision
    // changes and valid device errors remain healthy. Revision checks occur after
    // wire validation, so a legitimate metadata change is not mislabeled disconnect.
    for mode in [0, 1, 2, 3] {
        let q = TableReadBatch::new(
            15,
            1,
            TableRevision::from_bits(0x7fc00001),
            vec![DataType::UInt8],
        )
        .unwrap();
        let mut changed = read_reply(1, 1, &[1]);
        changed[3] ^= 1;
        let step = if mode == 0 {
            (167, vec![99, 15, 45, 0], Reply::Data(vec![0; 139]))
        } else {
            (
                10,
                vec![15, 1, 1],
                match mode {
                    1 => Reply::Data(vec![0]),
                    2 => Reply::Data(changed),
                    _ => Reply::Device,
                },
            )
        };
        let peer = Peer::start(vec![step]).await;
        let stream = tokio::net::TcpStream::connect(&peer.endpoint)
            .await
            .unwrap();
        let session = Session::new(
            Box::new(stream),
            Address::new(1, 0),
            Address::new(1, 2),
            Duration::from_secs(1),
            protocol::CrcMode::Required,
        )
        .unwrap();
        if mode == 0 {
            assert!(
                session
                    .read_table_definition(TableDefinitionRequest::new(15).unwrap())
                    .await
                    .is_err()
            );
        } else {
            let error = session.read_table_batch(q).await.unwrap_err();
            if mode == 2 {
                assert!(matches!(error, Error::DiscoveryChanged(_)));
            }
        }
        assert_eq!(session.is_connected(), mode >= 2);
        session.disconnect().await;
    }
}
#[tokio::test]
async fn table_write_metadata_failure_before_send_is_not_attempted() {
    // POL-TABLE-BEFORE-SEND: a failed metadata exchange can retire the channel but
    // cannot make OP-11 uncertain, since the write send gate was never passed.
    for reply in [Reply::Disconnect, Reply::Device, Reply::Data(vec![0])] {
        let mut steps = discovery();
        steps.push((167, vec![99, 15, 45, 0], reply));
        let peer = Peer::start(steps).await;
        let client = peer.client(Arc::new(DefaultProfile));
        let table = client.discover_configurable_table(15, fw()).await.unwrap();
        let plan = table
            .write_plan(
                &Fields,
                &SchemaContext::default(),
                1,
                &[Value::UInt8(1)],
                client.profile(),
            )
            .unwrap();
        assert_eq!(
            client.write_plan(plan).await.unwrap_err().progress.batches,
            [WriteOutcome::NotAttempted]
        );
        client.read_clock().await.unwrap();
        assert!(!peer.calls.lock().unwrap().contains(&11));
        client.stop().await;
    }
}

#[tokio::test]
async fn table_target_scope_and_read_schema_constraints_are_preflighted() {
    let mut map = [Tlp::new(101, 16, 1); 44];
    let cases = [
        (0, false),
        (90, false),
        (91, true),
        (98, true),
        (99, false),
        (100, true),
        (135, true),
        (136, false),
        (139, false),
        (255, false),
    ];
    for (i, (point_type, _)) in cases.iter().enumerate() {
        map[i] = Tlp::new(*point_type, 16, 1);
    }
    let raw = definition(15, 0x7fc00001, &map);
    let peer = Peer::start(vec![metadata(raw.clone()), metadata(raw)]).await;
    let client = peer.client(Arc::new(DefaultProfile));
    let table = client.discover_configurable_table(15, fw()).await.unwrap();
    // REQ-099-TARGET-SCOPE: preserve raw definitions even for unsupported codes.
    assert_eq!(table.definition().entries, map);
    struct Schema(Option<ParameterSchema>);
    impl SchemaProvider for Schema {
        fn lookup(&self, _: &SchemaContext, _: Tlp) -> Result<Option<ParameterSchema>, Error> {
            Ok(self.0)
        }
        fn lookup_write(
            &self,
            c: &SchemaContext,
            t: Tlp,
        ) -> Result<Option<ParameterSchema>, Error> {
            self.lookup(c, t)
        }
    }
    let schema = ParameterSchema {
        data_type: DataType::UInt8,
        access: Access::ReadWrite,
        logical_range: Some((16, 16)),
        numeric_range: None,
    };
    let context = SchemaContext::default();
    // REQ-099-TARGET-SCOPE: Table 3-11 permits 91–98 and 100–135;
    // both planners reject the gaps/endpoints outside those source ranges.
    for (i, (point_type, valid)) in cases.iter().enumerate() {
        let provider = Schema(Some(schema));
        assert_eq!(
            TableReadPlan::build(&table, &provider, &context, i as u8, 1, client.profile()).is_ok(),
            *valid,
            "target point type {point_type}, slot {i}"
        );
        assert_eq!(
            table
                .write_plan(
                    &provider,
                    &context,
                    i as u8,
                    &[Value::UInt8(1)],
                    client.profile()
                )
                .is_ok(),
            *valid,
            "target point type {point_type}, slot {i}"
        );
    }
    // POL-TABLE-PREFLIGHT: read access is independent of explicit write opt-in.
    for access in [
        Access::ReadOnly,
        Access::ReadWrite,
        Access::WriteOnly,
        Access::Conditional,
    ] {
        let provider = Schema(Some(ParameterSchema { access, ..schema }));
        assert_eq!(
            TableReadPlan::build(&table, &provider, &context, 10, 1, client.profile()).is_ok(),
            matches!(access, Access::ReadOnly | Access::ReadWrite),
            "read access {access:?}"
        );
    }
    // POL-TABLE-PREFLIGHT: reject inverted/excluding bounds; absent bounds
    // permit the provider's schema to describe this location.
    for logical_range in [
        Some((17, 16)),
        Some((0, 15)),
        Some((17, 255)),
        None,
        Some((0, 255)),
    ] {
        let provider = Schema(Some(ParameterSchema {
            logical_range,
            ..schema
        }));
        assert_eq!(
            TableReadPlan::build(&table, &provider, &context, 10, 1, client.profile()).is_ok(),
            logical_range.is_none() || logical_range == Some((0, 255)),
            "logical range {logical_range:?}"
        );
    }
    // POL-TABLE-PREFLIGHT: missing schemas and zero widths cannot be planned.
    for provider in [
        Schema(None),
        Schema(Some(ParameterSchema {
            data_type: DataType::Ascii(0),
            ..schema
        })),
    ] {
        assert!(
            TableReadPlan::build(&table, &provider, &context, 10, 1, client.profile()).is_err()
        );
    }
    // POL-TABLE-PREFLIGHT: schema/scope decisions never emit table requests.
    assert_eq!(peer.calls.lock().unwrap().as_slice(), [167, 167]);
    client.stop().await;
}

#[tokio::test]
async fn table_failed_second_read_batch_retains_only_confirmed_prefix_without_replay() {
    // POL-TABLE-READ-PROGRESS / POL-TABLE-NO-REPLAY: malformed, rejected,
    // disconnected and changed-version second batches neither discard the first
    // batch nor start the third batch/postcheck after failure or reconnection.
    let mut changed = read_reply(1, 2, &[1, 0, 0, 0xc0, 0x3f]);
    changed[3] ^= 1;
    for reply in [
        Reply::Data(vec![]),
        Reply::Device,
        Reply::Disconnect,
        Reply::Data(changed),
    ] {
        let mut steps = discovery();
        steps.extend([
            metadata(def()),
            (
                10,
                vec![15, 0, 1],
                Reply::Data(read_reply(0, 1, b"ABCDEFGHIJKLMNOPQRST")),
            ),
            (10, vec![15, 1, 2], reply),
        ]);
        let peer = Peer::start(steps).await;
        let client = peer.client(Arc::new(Tight));
        let table = client.discover_configurable_table(15, fw()).await.unwrap();
        let plan = TableReadPlan::build(
            &table,
            &Fields,
            &SchemaContext::default(),
            0,
            4,
            client.profile(),
        )
        .unwrap();
        assert_eq!(plan.batches().len(), 3);
        let failure = client.read_configurable_table(plan).await.unwrap_err();
        assert!(!failure.progress.complete);
        assert_eq!(failure.progress.batches.len(), 1);
        assert_eq!(failure.progress.batches[0].start, 0);
        assert_eq!(
            failure.progress.batches[0].values,
            [Value::Ascii("ABCDEFGHIJKLMNOPQRST".into())]
        );
        client.read_clock().await.unwrap();
        assert!(peer.queue.lock().unwrap().is_empty());
        assert_eq!(
            peer.calls.lock().unwrap().as_slice(),
            [167, 167, 167, 10, 10]
        );
        client.stop().await;
    }
}
