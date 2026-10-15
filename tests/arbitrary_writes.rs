//! Independent request literals: D301180X012 §§2.20/2.23, Tables 2-23/26 and 3-13..17.
use rocsteady::*;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn crc(bytes: &[u8]) -> u16 {
    let mut n = 0u16;
    for &byte in bytes {
        n ^= u16::from(byte);
        for _ in 0..8 {
            n = if n & 1 != 0 {
                (n >> 1) ^ 0xa001
            } else {
                n >> 1
            };
        }
    }
    n
}
type ExpectedWrites = Arc<Mutex<VecDeque<(u8, Vec<u8>)>>>;
struct Peer {
    address: String,
    expected: ExpectedWrites,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Peer {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl Peer {
    async fn start(layout: IoLayout, expected: Vec<(u8, Vec<u8>)>) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let expected = Arc::new(Mutex::new(VecDeque::from(expected)));
        let queue = expected.clone();
        let task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            loop {
                let mut header = [0; 6];
                if stream.read_exact(&mut header).await.is_err() {
                    break;
                }
                assert_eq!(&header[..4], &[1, 2, 1, 0]);
                let mut body = vec![0; usize::from(header[5]) + 2];
                stream.read_exact(&mut body).await.unwrap();
                let payload = &body[..body.len() - 2];
                let mut request = header.to_vec();
                request.extend(payload);
                assert_eq!(crc(&request).to_le_bytes(), body[body.len() - 2..]);
                let data = match header[4] {
                    7 => vec![1, 2, 3, 8, 10, 0xea, 7, 5],
                    6 => {
                        let mut data = vec![0; 215];
                        data[0] = 1;
                        data[1] = 1;
                        data[4] = match layout {
                            IoLayout::Compatibility160 => 0,
                            IoLayout::Compatibility240 => 1,
                            IoLayout::Expanded224 => 2,
                        };
                        data[18] = 6;
                        data
                    }
                    50 => {
                        assert!(payload == [0] || payload == [1]);
                        let mut data = vec![0; layout.len()];
                        if payload == [0] {
                            for (i, pt) in (101..=105).enumerate() {
                                data[16 + i] = pt;
                            }
                        }
                        data
                    }
                    166 | 181 => {
                        let next = queue
                            .lock()
                            .unwrap()
                            .pop_front()
                            .expect("unexpected write/replay");
                        assert_eq!((header[4], payload), (next.0, next.1.as_slice()));
                        vec![]
                    }
                    other => panic!("unexpected opcode {other}"),
                };
                let mut response = vec![1, 0, 1, 2, header[4], data.len() as u8];
                response.extend(data);
                response.extend(crc(&response).to_le_bytes());
                stream.write_all(&response).await.unwrap();
            }
        });
        Self {
            address,
            expected,
            task,
        }
    }
    fn client(&self, layout: IoLayout) -> Client {
        Client::tcp(
            &self.address,
            ClientConfig {
                profile: Arc::new(Roc800Profile { layout }),
                heartbeat_interval: None,
                call_timeout: Duration::from_secs(2),
                ..Default::default()
            },
        )
        .unwrap()
    }
}
struct Schema(DataType);
impl SchemaProvider for Schema {
    fn lookup(&self, _: &SchemaContext, _: Tlp) -> Result<Option<ParameterSchema>, Error> {
        Ok(None)
    }
    fn lookup_write(&self, _: &SchemaContext, _: Tlp) -> Result<Option<ParameterSchema>, Error> {
        Ok(Some(ParameterSchema {
            data_type: self.0,
            access: Access::ReadWrite,
            logical_range: None,
            numeric_range: None,
        }))
    }
}
struct Limits(usize);
impl ProtocolProfile for Limits {
    fn metadata(&self) -> ProfileMetadata {
        DefaultProfile.metadata()
    }
    fn capability(&self, op: u8) -> Capability {
        DefaultProfile.capability(op)
    }
    fn limits(&self, op: u8) -> PayloadLimits {
        if op == 181 {
            PayloadLimits {
                request: self.0,
                response: 0,
            }
        } else {
            PayloadLimits::default()
        }
    }
}
#[test]
fn req_181_budget_counts_tlps_values_and_count_independently_of_166() {
    // POL-181-BUDGET: conservative 240-byte payload policy; Table 2-26 packing.
    let context = SchemaContext::default();
    let writes = [(Tlp::new(200, 4, 255), Value::Ascii("X".repeat(236)))];
    let plan = WritePlan::build_arbitrary(
        &Schema(DataType::Ascii(236)),
        &context,
        &writes,
        &DefaultProfile,
    )
    .unwrap();
    assert_eq!(plan.batches()[0].opcode(), 181);
    assert_eq!(plan.batches()[0].payload().len(), 240);
    assert_eq!(&plan.batches()[0].payload()[..4], &[1, 200, 4, 255]);
    assert!(
        WritePlan::build(
            &Schema(DataType::Ascii(236)),
            &context,
            &writes,
            &DefaultProfile
        )
        .is_err()
    );
    assert!(
        WritePlan::build_arbitrary(
            &Schema(DataType::Ascii(237)),
            &context,
            &[(Tlp::new(200, 0, 0), Value::Ascii("X".repeat(237)))],
            &DefaultProfile
        )
        .is_err()
    );
    assert!(
        WritePlan::build_arbitrary(
            &Schema(DataType::Ascii(236)),
            &context,
            &writes,
            &Limits(239)
        )
        .is_err()
    );
    let writes = vec![(Tlp::new(200, 0, 255), Value::UInt8(9)); 256];
    let plan =
        WritePlan::build_arbitrary(&Schema(DataType::UInt8), &context, &writes, &DefaultProfile)
            .unwrap();
    assert_eq!(
        plan.batches()
            .iter()
            .map(|b| b.payload()[0])
            .collect::<Vec<_>>(),
        [59, 59, 59, 59, 20]
    );
    assert!(plan.batches().iter().all(|b| b.payload().len() <= 240));
    let plan =
        WritePlan::build_arbitrary(&Schema(DataType::UInt8), &context, &writes[..3], &Limits(9))
            .unwrap();
    assert_eq!(
        plan.batches()
            .iter()
            .map(|b| b.payload().to_vec())
            .collect::<Vec<_>>(),
        [
            vec![2, 200, 0, 255, 9, 200, 0, 255, 9],
            vec![1, 200, 0, 255, 9]
        ]
    );
    assert!(
        WritePlan::build_arbitrary(&Schema(DataType::UInt8), &context, &writes[..1], &Limits(4))
            .is_err()
    );
}
#[tokio::test]
async fn req_181_mixed_width_literals_preserve_cross_point_order_and_duplicates() {
    // REQ-181-REQUEST / REQ-181-ACK: Table 2-26; independent little-endian literals.
    let writes = vec![
        (Tlp::new(200, 3, 255), Value::UInt16(0x1234)),
        (Tlp::new(201, 7, 0), Value::Float(1.5)),
        (Tlp::new(200, 3, 2), Value::Ascii("TEST".into())),
        (Tlp::new(200, 3, 255), Value::UInt16(0xabcd)),
    ];
    struct Mixed;
    impl SchemaProvider for Mixed {
        fn lookup(&self, _: &SchemaContext, _: Tlp) -> Result<Option<ParameterSchema>, Error> {
            Ok(None)
        }
        fn lookup_write(
            &self,
            c: &SchemaContext,
            t: Tlp,
        ) -> Result<Option<ParameterSchema>, Error> {
            Schema(match (t.point_type, t.parameter) {
                (200, 255) => DataType::UInt16,
                (201, 0) => DataType::Float,
                (200, 2) => DataType::Ascii(4),
                _ => return Ok(None),
            })
            .lookup_write(c, t)
        }
    }
    let peer = Peer::start(
        IoLayout::Compatibility160,
        vec![(
            181,
            vec![
                4, 200, 3, 255, 0x34, 0x12, 201, 7, 0, 0, 0, 0xc0, 0x3f, 200, 3, 2, b'T', b'E',
                b'S', b'T', 200, 3, 255, 0xcd, 0xab,
            ],
        )],
    )
    .await;
    let client = peer.client(IoLayout::Compatibility160);
    let plan =
        WritePlan::build_arbitrary(&Mixed, &SchemaContext::default(), &writes, client.profile())
            .unwrap();
    assert_eq!(
        client.write_plan(plan).await.unwrap().batches,
        [WriteOutcome::Acknowledged]
    );
    client.read_clock().await.unwrap(); // Empty ack preserved the same healthy connection.
    assert!(peer.expected.lock().unwrap().is_empty());
    client.stop().await;
}

fn catalog_writes() -> Vec<(Tlp, Value)> {
    let mut writes = vec![(
        Tlp::new(91, 0, 2),
        Value::Ascii("ABCDEFGHIJKLMNOPQRST".into()),
    )];
    for (i, pt) in (101..=105).enumerate() {
        writes.push((
            Tlp::new(pt, 16 + i as u8, 0),
            Value::Ascii("TAG       ".into()),
        ));
        if pt != 101 {
            writes.push((
                Tlp::new(pt, 16 + i as u8, 1),
                Value::Ascii("UNITS     ".into()),
            ));
        }
    }
    writes.extend([
        (Tlp::new(101, 16, 1), Value::UInt8(0)),
        (Tlp::new(101, 16, 2), Value::Float(1.5)),
    ]);
    writes
}
fn literal_catalog_payload() -> Vec<u8> {
    let mut payload = vec![12, 91, 0, 2];
    payload.extend(b"ABCDEFGHIJKLMNOPQRST");
    for (i, pt) in (101..=105).enumerate() {
        payload.extend([pt, 16 + i as u8, 0]);
        payload.extend(b"TAG       ");
        if pt != 101 {
            payload.extend([pt, 16 + i as u8, 1]);
            payload.extend(b"UNITS     ");
        }
    }
    payload.extend([101, 16, 1, 0, 101, 16, 2, 0, 0, 0xc0, 0x3f]);
    payload
}
#[tokio::test]
async fn req_write_catalog_literals_and_local_constraints_cover_all_audited_fields() {
    // REQ-WRITE-CATALOG: Tables 3-5/13..17. Twelve fields including existing station name.
    for layout in [
        IoLayout::Compatibility160,
        IoLayout::Compatibility240,
        IoLayout::Expanded224,
    ] {
        let peer = Peer::start(layout, vec![(181, literal_catalog_payload())]).await;
        let client = peer.client(layout);
        let io = client
            .discover_io(FirmwareVersion::new(3, 60).unwrap())
            .await
            .unwrap();
        let catalog = Roc800Catalog::from_discovery(io).unwrap();
        let writes = catalog_writes();
        for (tlp, value) in &writes {
            let schema = catalog
                .lookup_write(&catalog.context(), *tlp)
                .unwrap()
                .unwrap();
            assert_eq!(schema.data_type, value.data_type());
            assert_eq!(schema.access, Access::ReadWrite);
            assert_eq!(schema.logical_range, Some((tlp.logical, tlp.logical)));
            assert_eq!(
                schema.numeric_range,
                match (tlp.point_type, tlp.parameter) {
                    (101, 1) => Some((0., 1.)),
                    (101, 2) => Some((0., 43200.)),
                    _ => None,
                }
            );
        }
        for (tlp, value) in [
            (Tlp::new(101, 16, 1), Value::UInt8(2)),
            (Tlp::new(101, 16, 1), Value::Binary(1)),
            (Tlp::new(101, 16, 2), Value::Float(-1.)),
            (Tlp::new(101, 16, 2), Value::Float(43201.)),
            (Tlp::new(101, 16, 2), Value::Float(f32::NAN)),
            (Tlp::new(101, 16, 2), Value::Float(f32::INFINITY)),
            (Tlp::new(103, 18, 0), Value::Ascii("TOO SHORT".into())),
            (Tlp::new(103, 18, 0), Value::Ascii("\0".repeat(10))),
            (Tlp::new(103, 16, 0), Value::Ascii("TAG       ".into())),
            (Tlp::new(103, 99, 0), Value::Ascii("TAG       ".into())),
            (Tlp::new(103, 18, 19), Value::Float(1.)),
            (Tlp::new(102, 17, 14), Value::Float(1.)),
            (Tlp::new(104, 19, 7), Value::UInt8(1)),
        ] {
            assert!(
                WritePlan::build_arbitrary(
                    &catalog,
                    &catalog.context(),
                    &[(writes[0].0, writes[0].1.clone()), (tlp, value)],
                    client.profile()
                )
                .is_err()
            );
        }
        // All accepted enum codes and both finite filter bounds; no device send from planning.
        for value in [Value::UInt8(0), Value::UInt8(1)] {
            assert!(
                WritePlan::build_arbitrary(
                    &catalog,
                    &catalog.context(),
                    &[(Tlp::new(101, 16, 1), value)],
                    client.profile()
                )
                .is_ok()
            );
        }
        for value in [0.0, 43200.0] {
            assert!(
                WritePlan::build_arbitrary(
                    &catalog,
                    &catalog.context(),
                    &[(Tlp::new(101, 16, 2), Value::Float(value))],
                    client.profile()
                )
                .is_ok()
            );
        }
        let plan =
            WritePlan::build_arbitrary(&catalog, &catalog.context(), &writes, client.profile())
                .unwrap();
        assert_eq!(
            client.write_plan(plan).await.unwrap().batches,
            [WriteOutcome::Acknowledged]
        );
        assert!(peer.expected.lock().unwrap().is_empty());
        client.stop().await;
    }
}

#[cfg(feature = "simulator")]
#[tokio::test]
async fn req_write_simulator_updates_each_audited_field_with_both_services() {
    // REQ-181-SIMULATOR / REQ-WRITE-CATALOG: explicit writes/read-back, all three layouts.
    use rocsteady::simulator::{Model, Simulator};
    for (layout, firmware) in [
        (
            IoLayout::Compatibility160,
            FirmwareVersion::new(1, 0).unwrap(),
        ),
        (
            IoLayout::Compatibility160,
            FirmwareVersion::new(3, 60).unwrap(),
        ),
        (
            IoLayout::Compatibility240,
            FirmwareVersion::new(3, 60).unwrap(),
        ),
        (IoLayout::Expanded224, FirmwareVersion::new(3, 60).unwrap()),
    ] {
        for arbitrary in [false, true] {
            let mut model = Model::default();
            model.set_io_layout(layout);
            for (i, pt) in (101..=105).enumerate() {
                model.io_point_types[16 + i] = pt;
            }
            let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), model)
                .await
                .unwrap();
            let client = Client::tcp(
                server.endpoint().to_string(),
                ClientConfig {
                    profile: Arc::new(Roc800Profile { layout }),
                    heartbeat_interval: None,
                    ..Default::default()
                },
            )
            .unwrap();
            let catalog =
                Roc800Catalog::from_discovery(client.discover_io(firmware).await.unwrap()).unwrap();
            let writes = catalog_writes();
            let plan = if arbitrary {
                WritePlan::build_arbitrary(&catalog, &catalog.context(), &writes, client.profile())
            } else {
                WritePlan::build(&catalog, &catalog.context(), &writes, client.profile())
            }
            .unwrap();
            let expected_batches = plan.batches().len();
            assert_eq!(
                client.write_plan(plan).await.unwrap().batches,
                vec![WriteOutcome::Acknowledged; expected_batches]
            );
            let values = client
                .read_schema(
                    &catalog,
                    &catalog.context(),
                    &writes.iter().map(|(t, _)| *t).collect::<Vec<_>>(),
                )
                .await
                .unwrap();
            assert_eq!(
                values,
                writes.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>()
            );
            client.stop().await;
            server.stop().await.unwrap();
        }
    }
}

#[cfg(feature = "simulator")]
#[tokio::test]
async fn pol_181_lost_ack_recovery_retains_effects_without_replay_or_resume() {
    // POL-181-NO-REPLAY: effect/response loss on batch 1; later batch never attempts.
    use rocsteady::simulator::{Fault, Model, Simulator};
    let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), Model::default())
        .await
        .unwrap();
    let client = Client::tcp(
        server.endpoint().to_string(),
        ClientConfig {
            heartbeat_interval: None,
            profile: Arc::new(Limits(24)),
            ..Default::default()
        },
    )
    .unwrap();
    client.connected().await.unwrap();
    let catalog = Roc800Catalog::new(FirmwareVersion::new(3, 60).unwrap()).unwrap();
    let writes = vec![
        (
            Tlp::new(91, 0, 2),
            Value::Ascii("FIRST WRITE         ".into()),
        ),
        (
            Tlp::new(91, 0, 2),
            Value::Ascii("SECOND WRITE        ".into()),
        ),
    ];
    let plan = WritePlan::build_arbitrary(&catalog, &catalog.context(), &writes, client.profile())
        .unwrap();
    server.inject(Fault::Disconnect);
    let failure = client.write_plan(plan).await.unwrap_err();
    assert_eq!(
        failure.progress.batches,
        [WriteOutcome::Uncertain, WriteOutcome::NotAttempted]
    );
    assert_eq!(
        client
            .read_parameters(&[Parameter::new(Tlp::new(91, 0, 2), DataType::Ascii(20))])
            .await
            .unwrap(),
        [writes[0].1.clone()]
    );
    assert_eq!(server.stats().requests, 4); // Setup, write, replacement setup, explicit read; no replay.
    client.stop().await;
    assert_eq!(
        server
            .model()
            .written_parameters
            .lock()
            .unwrap()
            .get(&writes[0].0),
        Some(&writes[0].1)
    );
    server.stop().await.unwrap();
}

#[cfg(feature = "simulator")]
#[test]
fn req_181_simulator_reports_partial_effect_and_rejects_malformed_tuples() {
    // POL-181-PARTIAL-DEVICE: deliberate fixture partial effects, not device atomicity evidence.
    use rocsteady::{protocol::Frame, simulator::Model};
    let model = Model::default();
    let mut authenticated = false;
    let mut data = vec![2, 91, 0, 2];
    data.extend(b"FIRST WRITE         ");
    data.extend([91, 0, 3]);
    data.extend(b"READ ONLY FIELD     ");
    let reply = model
        .respond(
            &Frame::new(Address::new(1, 2), Address::new(1, 0), 181, data),
            &mut authenticated,
        )
        .unwrap();
    assert_eq!(reply.opcode, 255);
    assert_eq!(reply.data, [2, 2]);
    assert_eq!(
        model
            .written_parameters
            .lock()
            .unwrap()
            .get(&Tlp::new(91, 0, 2)),
        Some(&Value::Ascii("FIRST WRITE         ".into()))
    );
    for payload in [
        vec![],
        vec![0, 0, 0, 0, 0],
        vec![1, 91, 0, 2, b'A'],
        vec![1, 91, 0],
    ] {
        // trailing case below separately
        let reply = model
            .respond(
                &Frame::new(Address::new(1, 2), Address::new(1, 0), 181, payload),
                &mut authenticated,
            )
            .unwrap();
        assert_eq!(reply.opcode, 255);
    }
    let mut trailing = vec![1, 91, 0, 2];
    trailing.extend(b"FIRST WRITE         ");
    trailing.push(0);
    assert_eq!(
        model
            .respond(
                &Frame::new(Address::new(1, 2), Address::new(1, 0), 181, trailing),
                &mut authenticated
            )
            .unwrap()
            .opcode,
        255
    );
}
