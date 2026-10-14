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
        .discover_io(rocsteady::FirmwareVersion::new(3, 0).unwrap())
        .await
        .unwrap();
    let ai = io.by_logical(103, 0).unwrap().tlp(21).unwrap();
    assert_eq!(ai, rocsteady::Tlp::new(103, 16, 21));
    let catalog = rocsteady::Roc800Catalog::from_discovery(io).unwrap();
    assert_eq!(
        discovery_client
            .read_schema(&catalog, &catalog.context(), &[
                ai, rocsteady::Tlp::new(136, 0, 5), rocsteady::Tlp::new(136, 0, 7)
            ])
            .await
            .unwrap(),
        [rocsteady::Value::Float(12.5), rocsteady::Value::UInt16(2026),
         rocsteady::Value::Time(0x12345678)]
    );
    discovery_client.stop().await;
    server.stop().await.unwrap();
    println!("packaged external consumer passed");
}
