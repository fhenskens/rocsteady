include!("../examples/support/profile_fixture.rs");
use rocsteady::{
    Access, Capability, Client, ClientConfig, Clock, DataType, DecodeError, DecodedOperation,
    Error, ManagedContext, ParameterSchema, PayloadLimits, ProtocolProfile, ReadPlan, ReplyCodec,
    SchemaChain, SchemaContext, SchemaProvider, Tlp, Value, Workflow,
};
use std::time::Duration;

fn client(fixture: &ProfileFixture, profile: Arc<dyn ProtocolProfile>) -> Client {
    Client::tcp(
        &fixture.endpoint,
        ClientConfig {
            profile,
            heartbeat_interval: None,
            response_timeout: Duration::from_secs(1),
            call_timeout: Duration::from_secs(3),
            recovery: etherbird::Config {
                retry_delay: Duration::from_millis(1),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap()
}
fn tlps() -> Vec<Tlp> {
    vec![
        Tlp::new(200, 0, 1),
        Tlp::new(200, 0, 2),
        Tlp::new(200, 0, 1),
    ]
}
#[tokio::test]
async fn mixed_width_reads_preserve_duplicates_and_split_over_tcp() {
    let fixture = ProfileFixture::start().await;
    let client = client(&fixture, Arc::new(FixtureProfile));
    let plan = ReadPlan::build(
        &FixtureSchema,
        &fixture_identity(),
        &tlps(),
        client.profile(),
    )
    .unwrap();
    assert_eq!(
        plan.batches().iter().map(Vec::len).collect::<Vec<_>>(),
        [2, 1]
    );
    assert_eq!(
        client.read_plan(plan).await.unwrap(),
        [
            Value::UInt16(0x1234),
            Value::Ascii("TEST".into()),
            Value::UInt16(0x1234)
        ]
    );
    assert_eq!(fixture.peer.reads.load(Ordering::SeqCst), 2);
    assert_eq!(fixture.peer.connections.load(Ordering::SeqCst), 1);
    client.stop().await;
}
#[tokio::test]
async fn failed_second_batch_reports_confirmed_progress_without_resume() {
    let fixture = ProfileFixture::start().await;
    let client = client(&fixture, Arc::new(FixtureProfile));
    fixture.peer.fail_batch.store(2, Ordering::SeqCst);
    let error = client
        .read_schema(&FixtureSchema, &fixture_identity(), &tlps())
        .await
        .unwrap_err();
    assert_eq!(error.progress.completed_batches, 1);
    assert_eq!(
        error.progress.values,
        [Value::UInt16(0x1234), Value::Ascii("TEST".into())]
    );
    assert!(matches!(
        error.error,
        etherbird::Error::Operation(Error::Io(_))
    ));
    client.read_clock().await.unwrap();
    assert_eq!(fixture.peer.reads.load(Ordering::SeqCst), 2);
    assert_eq!(fixture.peer.connections.load(Ordering::SeqCst), 2);
    client.stop().await;
}
#[tokio::test]
async fn call_timeout_preserves_completed_batches_and_retires_active_exchange() {
    let fixture = ProfileFixture::start().await;
    let client = Client::tcp(
        &fixture.endpoint,
        ClientConfig {
            profile: Arc::new(FixtureProfile),
            heartbeat_interval: None,
            call_timeout: Duration::from_millis(250),
            response_timeout: Duration::from_secs(2),
            recovery: etherbird::Config {
                retry_delay: Duration::from_millis(1),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap();
    client.connected().await.unwrap();
    fixture.peer.stall_batch.store(2, Ordering::SeqCst);
    let error = client
        .read_schema(&FixtureSchema, &fixture_identity(), &tlps())
        .await
        .unwrap_err();
    assert!(matches!(error.error, etherbird::Error::Timeout));
    assert_eq!(error.progress.completed_batches, 1);
    assert_eq!(error.progress.values.len(), 2);
    // Etherbird signals active-job cancellation asynchronously on call expiry.
    // Observe replacement before making a new explicit attempt.
    tokio::time::timeout(Duration::from_secs(1), async {
        while fixture.peer.connections.load(Ordering::SeqCst) < 2 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    client.connected().await.unwrap();
    client.read_clock().await.unwrap();
    assert_eq!(fixture.peer.reads.load(Ordering::SeqCst), 2);
    assert_eq!(fixture.peer.connections.load(Ordering::SeqCst), 2);
    client.stop().await;
}
struct AckCodec;
impl ReplyCodec<()> for AckCodec {
    type Output = ();
    fn decode(&self, _: &(), payload: &[u8]) -> Result<(), DecodeError> {
        if payload.is_empty() {
            Ok(())
        } else {
            Err(DecodeError("nonempty fixture ack"))
        }
    }
}
struct CommitThenRead;
#[etherbird::async_trait]
impl Workflow for CommitThenRead {
    type Output = ();
    async fn run(self, context: &mut ManagedContext<'_>) -> Result<(), Error> {
        context
            .operation(DecodedOperation {
                request: (),
                opcode: 200,
                payload: vec![0x55],
                codec: AckCodec,
            })
            .await?;
        context.read_clock().await?;
        Ok(())
    }
}
#[tokio::test]
async fn committed_side_effect_with_lost_ack_is_never_replayed() {
    let fixture = ProfileFixture::start().await;
    let client = client(&fixture, Arc::new(FixtureProfile));
    assert!(client.workflow(CommitThenRead).await.is_err());
    client.read_clock().await.unwrap();
    assert_eq!(fixture.peer.effects.load(Ordering::SeqCst), 1);
    assert_eq!(fixture.peer.connections.load(Ordering::SeqCst), 2);
    // A caller's explicit new attempt is a second effect, not an automatic retry.
    assert!(
        client
            .operation(DecodedOperation {
                request: (),
                opcode: 200,
                payload: vec![0x55],
                codec: AckCodec
            })
            .await
            .is_err()
    );
    client.read_clock().await.unwrap();
    assert_eq!(fixture.peer.effects.load(Ordering::SeqCst), 2);
    client.stop().await;
}
#[tokio::test]
async fn malformed_replies_retire_but_device_rejection_preserves_connection() {
    let fixture = ProfileFixture::start().await;
    let client = client(&fixture, Arc::new(FixtureProfile));
    client.connected().await.unwrap();
    fixture.peer.fault.store(2, Ordering::SeqCst);
    let error = client
        .read_schema(&FixtureSchema, &fixture_identity(), &tlps())
        .await
        .unwrap_err();
    assert!(matches!(
        error.error,
        etherbird::Error::Operation(Error::Device(_))
    ));
    assert!(error.progress.values.is_empty());
    client.read_clock().await.unwrap();
    assert_eq!(fixture.peer.connections.load(Ordering::SeqCst), 1);
    for fault in [1, 3, 4, 5, 6, 7, 8, 9] {
        let before = fixture.peer.connections.load(Ordering::SeqCst);
        fixture.peer.fault.store(fault, Ordering::SeqCst);
        let error = client
            .read_schema(&FixtureSchema, &fixture_identity(), &tlps())
            .await
            .unwrap_err();
        assert!(error.progress.values.is_empty());
        client.read_clock().await.unwrap();
        assert_eq!(fixture.peer.connections.load(Ordering::SeqCst), before + 1);
    }
    client.stop().await;
}

struct AlternativeClock;
impl ProtocolProfile for AlternativeClock {
    fn metadata(&self) -> rocsteady::ProfileMetadata {
        FixtureProfile.metadata()
    }
    fn capability(&self, opcode: u8) -> Capability {
        FixtureProfile.capability(opcode)
    }
    fn decode_clock(&self, payload: &[u8]) -> Result<Clock, DecodeError> {
        // Identical-length synthetic alternative, selected explicitly. Never autodetected.
        let mut clock = rocsteady::ClockCodec.decode(&(), payload)?;
        clock.year += 1;
        Ok(clock)
    }
    fn accepts_source(&self, expected: rocsteady::Address, actual: rocsteady::Address) -> bool {
        expected == actual || actual == rocsteady::Address::new(3, 2)
    }
}
#[tokio::test]
async fn explicit_profile_controls_readiness_and_clock_decoding() {
    let fixture = ProfileFixture::start().await;
    let client = client(&fixture, Arc::new(AlternativeClock));
    client.connected().await.unwrap();
    assert_eq!(client.read_clock().await.unwrap().year, 2027);
    fixture.peer.fault.store(3, Ordering::SeqCst);
    assert_eq!(client.read_clock().await.unwrap().year, 2027);
    fixture.peer.fault.store(4, Ordering::SeqCst);
    assert!(client.read_clock().await.is_err());
    assert_eq!(client.read_clock().await.unwrap().year, 2027);
    assert_eq!(fixture.peer.connections.load(Ordering::SeqCst), 2);
    client.stop().await;
}
struct ContextCodec;
impl ReplyCodec<u16> for ContextCodec {
    type Output = u16;
    fn decode(&self, request: &u16, payload: &[u8]) -> Result<u16, DecodeError> {
        let year = rocsteady::ClockCodec.decode(&(), payload)?.year;
        if year == *request {
            Ok(year)
        } else {
            Err(DecodeError("request context mismatch"))
        }
    }
}
#[tokio::test]
async fn contextual_codec_is_replaceable_and_decode_failure_retires() {
    let fixture = ProfileFixture::start().await;
    let client = client(&fixture, Arc::new(FixtureProfile));
    assert_eq!(
        client
            .operation(DecodedOperation {
                request: 2026,
                opcode: 7,
                payload: vec![],
                codec: ContextCodec
            })
            .await
            .unwrap(),
        2026
    );
    assert!(
        client
            .operation(DecodedOperation {
                request: 2025,
                opcode: 7,
                payload: vec![],
                codec: ContextCodec
            })
            .await
            .is_err()
    );
    client.read_clock().await.unwrap();
    assert_eq!(fixture.peer.connections.load(Ordering::SeqCst), 2);
    client.stop().await;
}

struct Override(ParameterSchema);
impl SchemaProvider for Override {
    fn lookup(&self, _: &SchemaContext, _: Tlp) -> Result<Option<ParameterSchema>, Error> {
        Ok(Some(self.0))
    }
}
fn schema(data_type: DataType, access: Access) -> ParameterSchema {
    ParameterSchema {
        data_type,
        access,
        logical_range: None,
        numeric_range: None,
    }
}
#[test]
fn providers_validate_access_applicability_widths_and_override_precedence() {
    let profile = FixtureProfile;
    let identity = fixture_identity();
    let chain = SchemaChain(vec![
        Arc::new(Override(schema(DataType::UInt8, Access::ReadOnly))),
        Arc::new(FixtureSchema),
    ]);
    let plan = ReadPlan::build(&chain, &identity, &tlps(), &profile).unwrap();
    assert_eq!(plan.batches()[0][0].data_type, DataType::UInt8);
    let fallback = SchemaChain(vec![
        Arc::new(FixtureSchema),
        Arc::new(Override(schema(DataType::UInt8, Access::ReadOnly))),
    ]);
    assert!(ReadPlan::build(&fallback, &SchemaContext::default(), &tlps(), &profile).is_err());
    assert_eq!(
        ReadPlan::build(&fallback, &identity, &[Tlp::new(201, 0, 1)], &profile)
            .unwrap()
            .batches()[0][0]
            .data_type,
        DataType::UInt8
    );
    assert!(ReadPlan::build(&FixtureSchema, &SchemaContext::default(), &tlps(), &profile).is_err());
    assert!(ReadPlan::build(&FixtureSchema, &identity, &[Tlp::new(200, 16, 1)], &profile).is_err());
    assert!(ReadPlan::build(&FixtureSchema, &identity, &[Tlp::new(200, 0, 9)], &profile).is_err());
    assert!(ReadPlan::build(&FixtureSchema, &identity, &[], &profile).is_err());
    for access in [Access::WriteOnly, Access::Conditional] {
        assert!(
            ReadPlan::build(
                &Override(schema(DataType::UInt16, access)),
                &identity,
                &tlps(),
                &profile
            )
            .is_err()
        );
    }
    for width in [0, 10, usize::MAX] {
        assert!(
            ReadPlan::build(
                &Override(schema(DataType::Ascii(width), Access::ReadOnly)),
                &identity,
                &tlps(),
                &profile
            )
            .is_err()
        );
    }
}
struct Limits(PayloadLimits);
impl ProtocolProfile for Limits {
    fn metadata(&self) -> rocsteady::ProfileMetadata {
        FixtureProfile.metadata()
    }
    fn capability(&self, opcode: u8) -> Capability {
        FixtureProfile.capability(opcode)
    }
    fn limits(&self, _: u8) -> PayloadLimits {
        self.0
    }
}
#[test]
fn planning_obeys_each_budget_and_framework_ceiling() {
    for limits in [
        PayloadLimits {
            request: 4,
            response: 255,
        },
        PayloadLimits {
            request: 255,
            response: 8,
        },
    ] {
        let plan = ReadPlan::build(
            &FixtureSchema,
            &fixture_identity(),
            &tlps(),
            &Limits(limits),
        )
        .unwrap();
        assert_eq!(plan.batches().len(), 3);
    }
    let many = vec![Tlp::new(200, 0, 1); 100];
    let plan = ReadPlan::build(
        &FixtureSchema,
        &fixture_identity(),
        &many,
        &Limits(PayloadLimits {
            request: usize::MAX,
            response: usize::MAX,
        }),
    )
    .unwrap();
    assert_eq!(
        plan.batches().iter().map(Vec::len).collect::<Vec<_>>(),
        [47, 47, 6]
    );
}
#[tokio::test]
async fn local_failures_and_foreign_plan_budgets_do_not_use_channel() {
    let fixture = ProfileFixture::start().await;
    let client = client(&fixture, Arc::new(FixtureProfile));
    client.connected().await.unwrap();
    assert!(matches!(
        client.read_configuration().await,
        Err(etherbird::Error::Operation(Error::CapabilityUnavailable(_)))
    ));
    assert!(client.request(7, vec![0; 256]).await.is_err());
    assert!(client.request(24, vec![]).await.is_err());
    assert!(
        client
            .read_parameters(&[
                rocsteady::Parameter::new(Tlp::new(200, 0, 2), DataType::Ascii(4)),
                rocsteady::Parameter::new(Tlp::new(200, 1, 2), DataType::Ascii(4)),
            ])
            .await
            .is_err()
    );
    let plan = ReadPlan::build(
        &FixtureSchema,
        &fixture_identity(),
        &tlps(),
        &rocsteady::DefaultProfile,
    )
    .unwrap();
    assert!(client.read_plan(plan).await.is_err());
    assert!(
        client
            .read_schema(&FixtureSchema, &SchemaContext::default(), &tlps())
            .await
            .is_err()
    );
    assert_eq!(fixture.peer.reads.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.peer.connections.load(Ordering::SeqCst), 1);
    client.stop().await;
}

struct CountingProfile(AtomicUsize);
impl ProtocolProfile for CountingProfile {
    fn metadata(&self) -> rocsteady::ProfileMetadata {
        FixtureProfile.metadata()
    }
    fn capability(&self, opcode: u8) -> Capability {
        rocsteady::DefaultProfile.capability(opcode)
    }
    fn decode_clock(&self, payload: &[u8]) -> Result<Clock, DecodeError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        rocsteady::ClockCodec.decode(&(), payload)
    }
    fn decode_configuration(
        &self,
        payload: &[u8],
    ) -> Result<rocsteady::SystemConfiguration, DecodeError> {
        let mut bytes = payload.to_vec();
        if bytes.len() != 98 {
            return Err(DecodeError("fixture configuration length"));
        }
        bytes[18] = 42;
        rocsteady::ConfigurationCodec.decode(&(), &bytes)
    }
}
#[tokio::test]
async fn configuration_override_and_heartbeat_use_selected_profile() {
    let fixture = ProfileFixture::start().await;
    let profile = Arc::new(CountingProfile(AtomicUsize::new(0)));
    let client = Client::tcp(
        &fixture.endpoint,
        ClientConfig {
            profile: profile.clone(),
            heartbeat_interval: Some(Duration::from_millis(5)),
            ..Default::default()
        },
    )
    .unwrap();
    client.connected().await.unwrap();
    assert_eq!(client.read_configuration().await.unwrap().device_type(), 42);
    tokio::time::timeout(Duration::from_secs(1), async {
        while profile.0.load(Ordering::SeqCst) < 2 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(fixture.peer.connections.load(Ordering::SeqCst), 1);
    client.stop().await;
}

struct NoClock;
impl ProtocolProfile for NoClock {
    fn metadata(&self) -> rocsteady::ProfileMetadata {
        FixtureProfile.metadata()
    }
    fn capability(&self, _: u8) -> Capability {
        Capability::Unavailable
    }
}
struct Probe;
#[etherbird::async_trait]
impl rocsteady::SetupStrategy for Probe {
    async fn setup(
        &self,
        context: &mut ManagedContext<'_>,
    ) -> Result<rocsteady::SessionState, Error> {
        context.read_clock().await?;
        Ok(Box::new(()))
    }
}
#[tokio::test]
async fn missing_setup_capability_is_actionable_and_does_not_retry() {
    let fixture = ProfileFixture::start().await;
    let config = ClientConfig {
        profile: Arc::new(NoClock),
        heartbeat_interval: None,
        ..Default::default()
    };
    assert!(matches!(
        Client::tcp(&fixture.endpoint, config.clone()),
        Err(Error::CapabilityUnavailable(_))
    ));
    let client = Client::with_setup(
        Arc::new(rocsteady::transport::TcpConnector::new(&fixture.endpoint)),
        config,
        Arc::new(Probe),
    )
    .unwrap();
    assert!(
        matches!(client.connected().await, Err(etherbird::Error::Lifecycle(cause)) if matches!(cause.as_ref(), Error::CapabilityUnavailable(_)))
    );
    assert!(client.connected().await.is_err());
    assert_eq!(fixture.peer.connections.load(Ordering::SeqCst), 1);
    client.stop().await;
}
