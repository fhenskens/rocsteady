use rocsteady::{
    Client, ClientConfig, DecodeError, Error, LegacySetup, ManagedContext, Operation, SessionState,
    SetupStrategy, Workflow,
    simulator::{Model, Simulator},
    transport::TcpConnector,
};
use std::sync::Arc;

struct Year;
impl Operation for Year {
    type Output = u16;
    fn encode(&self) -> Result<(u8, Vec<u8>), Error> {
        Ok((7, vec![]))
    }
    fn decode(&self, bytes: &[u8]) -> Result<u16, DecodeError> {
        rocsteady::Clock::decode(bytes)
            .map(|clock| clock.year)
            .map_err(|_| DecodeError("clock fixture"))
    }
}
struct Setup;
#[etherbird::async_trait]
impl SetupStrategy for Setup {
    async fn setup(&self, context: &mut ManagedContext<'_>) -> Result<SessionState, Error> {
        LegacySetup { login: None }.setup(context).await?;
        Ok(Box::new(context.generation()))
    }
}
struct Pair;
struct Catalog;
impl rocsteady::SchemaProvider for Catalog {
    fn lookup(
        &self,
        context: &rocsteady::SchemaContext,
        tlp: rocsteady::Tlp,
    ) -> Result<Option<rocsteady::ParameterSchema>, Error> {
        if context.device_family.as_deref() != Some("consumer-fixture") {
            return Err(Error::CapabilityUnavailable("consumer identity"));
        }
        Ok(
            (tlp == rocsteady::Tlp::new(200, 0, 1)).then_some(rocsteady::ParameterSchema {
                data_type: rocsteady::DataType::UInt16,
                access: rocsteady::Access::ReadOnly,
                logical_range: Some((0, 0)),
                numeric_range: None,
            }),
        )
    }
}
#[etherbird::async_trait]
impl Workflow for Pair {
    type Output = u16;
    async fn run(self, context: &mut ManagedContext<'_>) -> Result<u16, Error> {
        assert_eq!(*context.require_state::<u64>()?, context.generation());
        let handle = context.bind(1u8);
        assert_eq!(*handle.get(context)?, 1);
        context.operation(Year).await?;
        context.operation(Year).await
    }
}
#[tokio::main]
async fn main() {
    let mut model = Model::default();
    model.io_point_types[16] = 103;
    model.parameters.insert(
        rocsteady::Tlp::new(103, 16, 21),
        rocsteady::Value::Float(12.5),
    );
    model.parameters.insert(
        rocsteady::Tlp::new(136, 0, 5),
        rocsteady::Value::UInt16(2026),
    );
    model.parameters.insert(
        rocsteady::Tlp::new(136, 0, 7),
        rocsteady::Value::Time(0x12345678),
    );
    model.parameters.insert(
        rocsteady::Tlp::new(200, 0, 1),
        rocsteady::Value::UInt16(123),
    );
    model.parameters.insert(rocsteady::Tlp::new(95, 1, 36), rocsteady::Value::UInt16(42));
    model.parameters.insert(rocsteady::Tlp::new(139, 26, 3), rocsteady::Value::Ascii("2.02      ".into()));
    model.parameters.insert(rocsteady::Tlp::new(103, 16, 32), rocsteady::Value::Binary(0xff));
    model.parameters.insert(rocsteady::Tlp::new(103, 16, 39), rocsteady::Value::Float(20.0));
    // Literal Table 2-16 parameter alarm and Table 2-17 clock-set event.
    model.alarm_log.insert(0, [0xc1,1,2,3,4,0,103,16,21,
        b'A',b'L',b'A',b'R',b'M',b' ',b' ',b' ',b' ',b' ',0,0,0x48,0x41]);
    model.alarm_index = 0;
    model.event_log.insert(0, [6,1,2,3,4,0x78,0x56,0x34,0x12,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    let server = Simulator::bind("127.0.0.1:0".parse().unwrap(), model)
        .await
        .unwrap();
    let client = Client::with_setup(
        Arc::new(TcpConnector::new(server.endpoint().to_string())),
        ClientConfig {
            heartbeat_interval: None,
            ..Default::default()
        },
        Arc::new(Setup),
    )
    .unwrap();
    assert_eq!(client.operation(Year).await.unwrap(), 2026);
    assert_eq!(client.workflow(Pair).await.unwrap(), 2026);
    assert_eq!(
        client
            .operation(rocsteady::DecodedOperation {
                request: (),
                opcode: 7,
                payload: vec![],
                codec: rocsteady::ClockCodec
            })
            .await
            .unwrap()
            .year,
        2026
    );
    assert_eq!(
        client.profile().capability(180),
        rocsteady::Capability::Verified
    );
    assert_eq!(
        client
            .read_schema(
                &Catalog,
                &rocsteady::SchemaContext {
                    device_family: Some("consumer-fixture".into()),
                    firmware: None
                },
                &[rocsteady::Tlp::new(200, 0, 1)]
            )
            .await
            .unwrap(),
        [rocsteady::Value::UInt16(123)]
    );
    client.stop().await;
    // Public discovery/catalog APIs against the extracted publication payload.
    let discovery_client = Client::tcp(
        server.endpoint().to_string(),
        ClientConfig {
            profile: Arc::new(rocsteady::Roc800Profile {
                layout: rocsteady::IoLayout::Compatibility160,
            }),
            heartbeat_interval: None,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        discovery_client
            .scan_point_types(0, 256)
            .await
            .unwrap()
            .len(),
        2
    );
    let io = discovery_client
        .discover_io(rocsteady::FirmwareVersion::new(3, 10).unwrap())
        .await
        .unwrap();
    let ai = io.by_logical(103, 0).unwrap().tlp(21).unwrap();
    assert_eq!(ai, rocsteady::Tlp::new(103, 16, 21));
    let catalog = rocsteady::Roc800Catalog::from_discovery(io).unwrap();
    assert_eq!(
        discovery_client
            .read_schema(&catalog, &catalog.context(), &[
                ai, rocsteady::Tlp::new(136, 0, 5), rocsteady::Tlp::new(136, 0, 7),
                rocsteady::Tlp::new(95, 1, 36), rocsteady::Tlp::new(139, 26, 3),
                rocsteady::Tlp::new(103, 16, 32), rocsteady::Tlp::new(103, 16, 39)
            ])
            .await
            .unwrap(),
        [rocsteady::Value::Float(12.5), rocsteady::Value::UInt16(2026),
         rocsteady::Value::Time(0x12345678), rocsteady::Value::UInt16(42),
         rocsteady::Value::Ascii("2.02      ".into()), rocsteady::Value::Binary(0xff),
         rocsteady::Value::Float(20.0)]
    );
    let alarms = rocsteady::Roc800Catalog::entry(103, 32).unwrap().metadata;
    assert_eq!(alarms.active_bits(255).count(), 7);
    assert_eq!(alarms.uninterpreted_bits(255), 0x20);
    let cal = rocsteady::Roc800Catalog::entry(103, 34).unwrap().metadata;
    assert_eq!(cal.enum_meaning(4).unwrap().key, "stop");
    assert!(cal.enum_meaning(255).is_none());
    assert!(cal.write_caveat.is_some());
    let alarms = discovery_client.read_alarm_range(0, 1).await.unwrap();
    assert_eq!(alarms.records().count(), 1);
    let checkpoint = alarms.checkpoint(1).unwrap();
    assert!(discovery_client.resume_alarms(checkpoint.clone(), 0).await.unwrap().anchor_verified);
    server.raise_srbx(0).unwrap();
    let mut poller = discovery_client.srbx_poller(std::time::Duration::from_millis(1)).unwrap();
    assert_eq!(poller.poll().await.unwrap().current_alarm_index, 0);
    assert_eq!(discovery_client.acknowledge_alarms(&checkpoint).await.unwrap().index, 0);
    assert!(!server.srbx_status().pending);
    let events = discovery_client.read_events(rocsteady::LogRequest::new(0, 1).unwrap()).await.unwrap();
    assert!(matches!(events.records()[0].data(), rocsteady::EventData::ClockSet { time: 0x12345678 }));
    discovery_client.stop().await;
    server.stop().await.unwrap();
    println!("packaged external consumer passed");
}
