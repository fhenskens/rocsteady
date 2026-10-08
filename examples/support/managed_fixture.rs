use rocsteady::transport::{Connector, Stream};
use rocsteady::{Client, ClientConfig, Error, ManagedContext, Operation, SetupStrategy, Workflow};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct FixtureClock;
impl Operation for FixtureClock {
    type Output = u16;
    fn encode(&self) -> Result<(u8, Vec<u8>), Error> {
        Ok((7, vec![]))
    }
    fn decode(&self, bytes: &[u8]) -> Result<u16, rocsteady::DecodeError> {
        if bytes.len() != 8 {
            return Err(rocsteady::DecodeError("fixture clock width"));
        }
        Ok(u16::from_le_bytes([bytes[5], bytes[6]]))
    }
}
struct FixtureSetup;
#[etherbird::async_trait]
impl SetupStrategy for FixtureSetup {
    async fn setup(
        &self,
        context: &mut ManagedContext<'_>,
    ) -> Result<rocsteady::SessionState, Error> {
        assert_eq!(context.operation(FixtureClock).await?, 2026);
        Ok(Box::new(context.generation()))
    }
}
struct Pair;
#[etherbird::async_trait]
impl Workflow for Pair {
    type Output = u64;
    async fn run(self, context: &mut ManagedContext<'_>) -> Result<u64, Error> {
        assert_eq!(context.state::<u64>(), Some(&context.generation()));
        context.operation(FixtureClock).await?;
        tokio::time::sleep(Duration::from_millis(15)).await;
        context.operation(FixtureClock).await?;
        Ok(context.generation())
    }
}
// Independent framing oracle. No production frame/CRC/typed codecs are used.
fn reply(opcode: u8, payload: &[u8]) -> Vec<u8> {
    let mut bytes = vec![1, 0, 1, 2, opcode, payload.len() as u8];
    bytes.extend(payload);
    let mut crc = 0u16;
    for byte in &bytes {
        crc ^= u16::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xa001
            } else {
                crc >> 1
            };
        }
    }
    bytes.extend(crc.to_le_bytes());
    bytes
}
struct Peer {
    connections: AtomicUsize,
    requests: AtomicUsize,
    fault: AtomicUsize,
}
struct FixtureConnector(Arc<Peer>);
#[etherbird::async_trait]
impl Connector for FixtureConnector {
    async fn connect(&self) -> Result<Stream, Error> {
        self.0.connections.fetch_add(1, Ordering::SeqCst);
        let (stream, mut peer) = tokio::io::duplex(512);
        let stats = self.0.clone();
        tokio::spawn(async move {
            loop {
                let mut request = [0; 8];
                if peer.read_exact(&mut request).await.is_err() {
                    break;
                }
                assert_eq!(request, [1, 2, 1, 0, 7, 0, 0x7b, 0xdd]);
                stats.requests.fetch_add(1, Ordering::SeqCst);
                let fault = stats.fault.swap(0, Ordering::SeqCst);
                let bytes = match fault {
                    1 => reply(255, &[2, 0]),
                    2 => reply(7, &[1]),
                    3 => break, // Request observed (possible side effect), then connection lost.
                    4 => {
                        tokio::time::sleep(Duration::from_millis(200)).await;
                        reply(7, &[1, 2, 3, 8, 10, 0xea, 7, 5])
                    }
                    _ => reply(7, &[1, 2, 3, 8, 10, 0xea, 7, 5]),
                };
                if peer.write_all(&bytes).await.is_err() {
                    break;
                }
            }
        });
        Ok(Box::new(stream))
    }
}
fn fixture(call_timeout: Duration) -> (Arc<Peer>, Client) {
    fixture_with_heartbeat(call_timeout, None)
}
fn fixture_with_heartbeat(call_timeout: Duration, heartbeat_interval: Option<Duration>) -> (Arc<Peer>, Client) {
    let peer = Arc::new(Peer {
        connections: AtomicUsize::new(0),
        requests: AtomicUsize::new(0),
        fault: AtomicUsize::new(0),
    });
    let client = Client::with_setup(
        Arc::new(FixtureConnector(peer.clone())),
        ClientConfig {
            heartbeat_interval,
            response_timeout: Duration::from_millis(100),
            call_timeout,
            recovery: etherbird::Config {
                retry_delay: Duration::from_millis(1),
                ..Default::default()
            },
            ..Default::default()
        },
        Arc::new(FixtureSetup),
    )
    .unwrap();
    (peer, client)
}
