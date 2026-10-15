use rocsteady::*;
struct Bytes;
impl SchemaProvider for Bytes {
    fn lookup(&self, _: &SchemaContext, _: Tlp) -> Result<Option<ParameterSchema>, Error> {
        Ok(None)
    }
    fn lookup_write(&self, _: &SchemaContext, _: Tlp) -> Result<Option<ParameterSchema>, Error> {
        Ok(Some(ParameterSchema {
            data_type: DataType::UInt8,
            access: Access::ReadWrite,
            logical_range: Some((0, 0)),
            numeric_range: Some((1., 255.)),
        }))
    }
}
#[test]
fn req_166_literal_budgets_types_and_ranges() {
    // REQ-166-REQUEST / POL-WRITE-VALIDATION: Table 2-23; schema contract.
    let writes: Vec<_> = (0..=255)
        .map(|p| (Tlp::new(200, 0, p), Value::UInt8(42)))
        .collect();
    let plan =
        WritePlan::build(&Bytes, &SchemaContext::default(), &writes, &DefaultProfile).unwrap();
    assert_eq!(plan.batches().len(), 2);
    assert_eq!(&plan.batches()[0].payload()[..4], &[200, 0, 230, 0]);
    assert_eq!(plan.batches()[0].payload().len(), 234);
    assert_eq!(&plan.batches()[1].payload()[..4], &[200, 0, 26, 230]);
    assert!(plan.batches()[0].payload()[4..].iter().all(|b| *b == 42));
    for value in [Value::UInt8(0), Value::UInt16(1)] {
        assert!(
            WritePlan::build(
                &Bytes,
                &SchemaContext::default(),
                &[(Tlp::new(200, 0, 0), value)],
                &DefaultProfile
            )
            .is_err()
        );
    }
    let catalog = Roc800Catalog::new(FirmwareVersion::new(3, 60).unwrap()).unwrap();
    for (parameter, value) in [
        (2, Value::Ascii("short".into())),
        (0, Value::UInt8(1)),
        (3, Value::Ascii("X".repeat(20))),
    ] {
        assert!(
            WritePlan::build(
                &catalog,
                &catalog.context(),
                &[(Tlp::new(91, 0, parameter), value)],
                &DefaultProfile
            )
            .is_err()
        );
    }
}

include!("../examples/support/profile_fixture.rs");
#[tokio::test]
async fn pol_write_progress_lost_ack_and_device_errors_never_replay() {
    // REQ-166-ACK / REQ-181-ACK / POL-181-PROGRESS / POL-WRITE-PROGRESS / POL-WRITE-NO-REPLAY; Tables 2-23/26, DEC-6.
    for (opcode, fault) in [166, 181]
        .into_iter()
        .flat_map(|op| (0..4).map(move |fault| (op, fault)))
    {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let effects = Arc::new(AtomicUsize::new(0));
        let count = effects.clone();
        let peer = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            for i in 0..2 {
                let mut header = [0; 6];
                stream.read_exact(&mut header).await.unwrap();
                let mut body = vec![0; usize::from(header[5]) + 2];
                stream.read_exact(&mut body).await.unwrap();
                if header[4] == 7 {
                    let mut reply = vec![1, 0, 1, 2, 7, 8, 1, 2, 3, 8, 10, 0xea, 7, 5];
                    reply.extend(fixture_crc(&reply).to_le_bytes());
                    stream.write_all(&reply).await.unwrap();
                    stream.read_exact(&mut header).await.unwrap();
                    body = vec![0; usize::from(header[5]) + 2];
                    stream.read_exact(&mut body).await.unwrap();
                }
                assert_eq!(header[4], opcode);
                assert_eq!(
                    &body[..5],
                    if opcode == 166 {
                        &[200, 0, 1, 0, 42]
                    } else {
                        &[1, 200, 0, 0, 42]
                    }
                );
                count.fetch_add(1, Ordering::SeqCst);
                if i == 1 && fault == 0 {
                    return;
                }
                if i == 1 && fault == 3 {
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                    return;
                }
                let (op, data) = if i == 1 && fault == 1 {
                    (255, vec![4, 0, 2, 0])
                } else if i == 1 {
                    (opcode, vec![1])
                } else {
                    (opcode, vec![])
                };
                let mut reply = vec![1, 0, 1, 2, op, data.len() as u8];
                reply.extend(data);
                reply.extend(fixture_crc(&reply).to_le_bytes());
                stream.write_all(&reply).await.unwrap();
            }
        });
        let client = Client::tcp(
            address.to_string(),
            ClientConfig {
                heartbeat_interval: None,
                call_timeout: std::time::Duration::from_millis(100),
                profile: Arc::new(WriteFixtureProfile),
                ..Default::default()
            },
        )
        .unwrap();
        let writes = vec![(Tlp::new(200, 0, 0), Value::UInt8(42)); 3];
        let plan = build_for(opcode, &Bytes, &writes, client.profile());
        let failure = client.write_plan(plan).await.unwrap_err();
        assert_eq!(
            failure.progress.batches,
            [
                WriteOutcome::Acknowledged,
                if fault == 1 {
                    WriteOutcome::DeviceError
                } else {
                    WriteOutcome::Uncertain
                },
                WriteOutcome::NotAttempted
            ]
        );
        if fault == 1 {
            let etherbird::Error::Operation(Error::Device(errors)) = &failure.error else {
                panic!("expected raw device errors")
            };
            assert_eq!(
                errors
                    .iter()
                    .map(|e| (e.code, e.offset))
                    .collect::<Vec<_>>(),
                [(4, 0), (2, 0)]
            );
        }
        assert_eq!(effects.load(Ordering::SeqCst), 2);
        client.stop().await;
        peer.await.unwrap();
    }
}

#[cfg(feature = "simulator")]
#[tokio::test]
async fn req_166_simulator_effect_survives_lost_ack() {
    // REQ-166-SIMULATOR / POL-WRITE-NO-REPLAY: effect precedes injected disconnect.
    use rocsteady::simulator::{Fault, Model, Simulator};
    let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), Model::default())
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
    let catalog = Roc800Catalog::new(FirmwareVersion::new(3, 60).unwrap()).unwrap();
    let value = Value::Ascii("ABCDEFGHIJKLMNOPQRST".into());
    let plan = WritePlan::build(
        &catalog,
        &catalog.context(),
        &[(Tlp::new(91, 0, 2), value.clone())],
        client.profile(),
    )
    .unwrap();
    client.connected().await.unwrap();
    server.inject(Fault::Disconnect);
    let failure = client.write_plan(plan).await.unwrap_err();
    assert_eq!(failure.progress.batches, [WriteOutcome::Uncertain]);
    assert_eq!(
        server
            .model()
            .written_parameters
            .lock()
            .unwrap()
            .get(&Tlp::new(91, 0, 2)),
        Some(&value)
    );
    assert_eq!(
        client
            .read_parameters(&[Parameter::new(Tlp::new(91, 0, 2), DataType::Ascii(20))])
            .await
            .unwrap(),
        [value]
    );
    client.stop().await;
}

struct Blocked;
impl ProtocolProfile for Blocked {
    fn metadata(&self) -> ProfileMetadata {
        DefaultProfile.metadata()
    }
    fn capability(&self, opcode: u8) -> Capability {
        if matches!(opcode, 166 | 181) {
            Capability::Unavailable
        } else {
            DefaultProfile.capability(opcode)
        }
    }
}
#[tokio::test]
async fn pol_write_before_send_and_stale_generation_are_not_attempted() {
    for opcode in [166, 181] {
        // POL-WRITE-BEFORE-SEND / POL-WRITE-GENERATION: no send on preflight failure.
        let fixture = ProfileFixture::start().await;
        let client = Client::tcp(
            &fixture.endpoint,
            ClientConfig {
                profile: Arc::new(Blocked),
                heartbeat_interval: None,
                ..Default::default()
            },
        )
        .unwrap();
        let writes = [(Tlp::new(200, 0, 0), Value::UInt8(42))];
        let plan = build_for(opcode, &Bytes, &writes, &DefaultProfile);
        let failure = client.write_plan(plan).await.unwrap_err();
        assert_eq!(failure.progress.batches, [WriteOutcome::NotAttempted]);
        assert_eq!(fixture.peer.effects.load(Ordering::SeqCst), 0);
        client.stop().await;

        struct Stale;
        impl SchemaProvider for Stale {
            fn generation(&self) -> Result<Option<u64>, Error> {
                Ok(Some(u64::MAX))
            }
            fn lookup(&self, _: &SchemaContext, _: Tlp) -> Result<Option<ParameterSchema>, Error> {
                Ok(None)
            }
            fn lookup_write(
                &self,
                c: &SchemaContext,
                t: Tlp,
            ) -> Result<Option<ParameterSchema>, Error> {
                Bytes.lookup_write(c, t)
            }
        }
        let client = Client::tcp(
            &fixture.endpoint,
            ClientConfig {
                heartbeat_interval: None,
                ..Default::default()
            },
        )
        .unwrap();
        let plan = build_for(opcode, &Stale, &writes, &DefaultProfile);
        let failure = client.write_plan(plan).await.unwrap_err();
        assert_eq!(failure.progress.batches, [WriteOutcome::NotAttempted]);
        assert!(matches!(
            failure.error,
            etherbird::Error::Operation(Error::StaleGeneration { .. })
        ));
        assert_eq!(fixture.peer.effects.load(Ordering::SeqCst), 0);
        client.stop().await;
    }
}

#[tokio::test]
async fn pol_write_dropped_queued_future_closes_send_gate() {
    for opcode in [166, 181] {
        // POL-WRITE-CANCEL: drop after admission while another workflow owns the session.
        struct Hold {
            entered: Arc<tokio::sync::Notify>,
            release: Arc<tokio::sync::Notify>,
        }
        #[etherbird::async_trait]
        impl Workflow for Hold {
            type Output = ();
            async fn run(self, _: &mut ManagedContext<'_>) -> Result<(), Error> {
                self.entered.notify_one();
                self.release.notified().await;
                Ok(())
            }
        }
        let fixture = ProfileFixture::start().await;
        let client = Client::tcp(
            &fixture.endpoint,
            ClientConfig {
                heartbeat_interval: None,
                ..Default::default()
            },
        )
        .unwrap();
        let entered = Arc::new(tokio::sync::Notify::new());
        let release = Arc::new(tokio::sync::Notify::new());
        let holder = {
            let client = client.clone();
            let entered = entered.clone();
            let release = release.clone();
            tokio::spawn(async move { client.workflow(Hold { entered, release }).await })
        };
        entered.notified().await;
        let plan = build_for(
            opcode,
            &Bytes,
            &[(Tlp::new(200, 0, 0), Value::UInt8(42))],
            client.profile(),
        );
        let mut write = Box::pin(client.write_plan(plan));
        std::future::poll_fn(|cx| {
            assert!(write.as_mut().poll(cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        drop(write);
        release.notify_one();
        holder.await.unwrap().unwrap();
        // A subsequent clock exchange must remain usable; the fixture panics on either write opcode.
        client.read_clock().await.unwrap();
        assert_eq!(fixture.peer.effects.load(Ordering::SeqCst), 0);
        client.stop().await;
    }
}

// Both protocols share lifecycle contracts, but each fixture asserts its own wire literals.
fn build_for(
    opcode: u8,
    schema: &dyn SchemaProvider,
    writes: &[(Tlp, Value)],
    profile: &dyn ProtocolProfile,
) -> WritePlan {
    if opcode == 166 {
        WritePlan::build(schema, &SchemaContext::default(), writes, profile)
    } else {
        WritePlan::build_arbitrary(schema, &SchemaContext::default(), writes, profile)
    }
    .unwrap()
}
struct WriteFixtureProfile;
impl ProtocolProfile for WriteFixtureProfile {
    fn metadata(&self) -> ProfileMetadata {
        DefaultProfile.metadata()
    }
    fn capability(&self, op: u8) -> Capability {
        DefaultProfile.capability(op)
    }
    fn limits(&self, op: u8) -> PayloadLimits {
        if op == 181 {
            PayloadLimits {
                request: 5,
                response: 255,
            }
        } else {
            PayloadLimits::default()
        }
    }
}
