//! Fixture credentials and procedure; no enhanced-security algorithm is implied.
use rocsteady::{
    Client, ClientConfig, Error, LegacySetup, Login, ManagedContext, SessionState, SetupStrategy,
    simulator::{Fault, Model, Simulator},
    transport::TcpConnector,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

#[etherbird::async_trait]
trait CredentialProvider: Send + Sync {
    async fn login(&self) -> Result<Login, Error>;
}
struct FixtureCredentials(AtomicUsize);
#[etherbird::async_trait]
impl CredentialProvider for FixtureCredentials {
    async fn login(&self) -> Result<Login, Error> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Login::new(*b"TST", 0x1234, None)
    }
}
struct ProviderSetup(Arc<dyn CredentialProvider>);
#[etherbird::async_trait]
impl SetupStrategy for ProviderSetup {
    async fn setup(&self, context: &mut ManagedContext<'_>) -> Result<SessionState, Error> {
        let login = self.0.login().await?;
        LegacySetup { login: Some(login) }.setup(context).await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = Simulator::bind(
        "127.0.0.1:0".parse()?,
        Model {
            login: Some(Login::new(*b"TST", 0x1234, None)?),
            ..Default::default()
        },
    )
    .await?;
    let provider = Arc::new(FixtureCredentials(AtomicUsize::new(0)));
    let client = Client::with_setup(
        Arc::new(TcpConnector::new(server.endpoint().to_string())),
        ClientConfig {
            heartbeat_interval: None,
            recovery: etherbird::Config {
                retry_delay: Duration::from_millis(5),
                ..Default::default()
            },
            ..Default::default()
        },
        Arc::new(ProviderSetup(provider.clone())),
    )?;
    let result = async {
        client.connected().await?;
        let mut changes = client.subscribe_state();
        let mut events = client.subscribe_events();
        server.inject(Fault::SrbxBeforeReply);
        client.read_clock().await?;
        assert_eq!(events.recv().await?.opcode, 224);
        changes.borrow_and_update();
        server.inject(Fault::Disconnect);
        assert!(client.read_clock().await.is_err()); // This attempt is never replayed.
        client.connected().await?;
        println!(
            "restored state: {:?}, clock: {:?}",
            client.state(),
            client.read_clock().await?
        );
        assert!(provider.0.load(Ordering::SeqCst) >= 2);
        Ok::<_, Box<dyn std::error::Error>>(())
    }
    .await;
    client.stop().await;
    server.stop().await?;
    result
}
