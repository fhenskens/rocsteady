// Independent loopback peer for extension contracts, not a vendor/device codec.
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Default)]
pub struct ProfilePeer {
    pub connections: AtomicUsize,
    pub reads: AtomicUsize,
    pub effects: AtomicUsize,
    pub fail_batch: AtomicUsize,
    pub stall_batch: AtomicUsize,
    pub fault: AtomicUsize,
}
pub struct ProfileFixture {
    pub endpoint: String,
    pub peer: Arc<ProfilePeer>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for ProfileFixture {
    fn drop(&mut self) { self.task.abort(); }
}
fn fixture_crc(bytes: &[u8]) -> u16 {
    let mut crc = 0u16;
    for &byte in bytes {
        crc ^= u16::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xa001 } else { crc >> 1 };
        }
    }
    crc
}
impl ProfileFixture {
    pub async fn start() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = listener.local_addr().unwrap().to_string();
        let peer = Arc::new(ProfilePeer::default());
        let stats = peer.clone();
        let task = tokio::spawn(async move {
            let mut connections = tokio::task::JoinSet::new();
            loop {
                let (mut stream, _) = listener.accept().await.unwrap();
                stats.connections.fetch_add(1, Ordering::SeqCst);
                let stats = stats.clone();
                connections.spawn(async move {
                    loop {
                        let mut header = [0; 6];
                        if stream.read_exact(&mut header).await.is_err() { break; }
                        assert_eq!(&header[..4], &[1, 2, 1, 0]);
                        let mut body = vec![0; usize::from(header[5]) + 2];
                        stream.read_exact(&mut body).await.unwrap();
                        let mut wire = header.to_vec();
                        wire.extend(&body[..body.len()-2]);
                        assert_eq!(fixture_crc(&wire).to_le_bytes(), body[body.len()-2..]);
                        let payload = &body[..body.len()-2];
                        let mut opcode = header[4];
                        let mut response = match opcode {
                            6 => { assert!(payload.is_empty()); let mut bytes = vec![0; 98]; bytes[0] = 1; bytes[18] = 6; bytes }
                            7 => { assert!(payload.is_empty()); vec![1, 2, 3, 8, 10, 0xea, 7, 5] }
                            180 => {
                                assert_eq!(payload.len(), 1 + 3 * usize::from(payload[0]));
                                let batch = stats.reads.fetch_add(1, Ordering::SeqCst) + 1;
                                if stats.fail_batch.load(Ordering::SeqCst) == batch { break; }
                                if stats.stall_batch.load(Ordering::SeqCst) == batch {
                                    std::future::pending::<()>().await;
                                }
                                let mut response = vec![payload[0]];
                                for tlp in payload[1..].as_chunks::<3>().0 {
                                    assert_eq!(tlp[0], 200);
                                    response.extend(tlp);
                                    match tlp[2] {
                                        1 => response.extend([0x34, 0x12]),
                                        2 => response.extend(b"TEST"),
                                        _ => panic!("unexpected fixture parameter"),
                                    }
                                }
                                response
                            }
                            // Fixture-only side effect: commit, then drop before acknowledgement.
                            200 => { assert_eq!(payload, &[0x55]); stats.effects.fetch_add(1, Ordering::SeqCst); break; }
                            _ => panic!("unexpected fixture opcode"),
                        };
                        let fault = stats.fault.swap(0, Ordering::SeqCst);
                        let mut addresses = vec![1, 0, 1, 2];
                        match fault {
                            1 => response[1] ^= 1, // Wrong TLP echo (parameter responses only).
                            2 => { opcode = 255; response = vec![2, 0]; }
                            3 => addresses[2] = 3,
                            4 => addresses[0] = 3,
                            5 => opcode = 8,
                            7 => response.pop().map(|_| ()).unwrap(),
                            8 => response.push(0),
                            9 => response[0] += 1,
                            _ => (),
                        }
                        addresses.extend([opcode, response.len() as u8]);
                        addresses.extend(response);
                        let mut crc = fixture_crc(&addresses);
                        if fault == 6 { crc ^= 1; }
                        addresses.extend(crc.to_le_bytes());
                        if stream.write_all(&addresses).await.is_err() { break; }
                    }
                });
            }
        });
        Self { endpoint, peer, task }
    }
}

/// A user-defined catalog whose applicability must be supplied explicitly.
pub struct FixtureSchema;
impl rocsteady::SchemaProvider for FixtureSchema {
    fn lookup(&self, context: &rocsteady::SchemaContext, tlp: rocsteady::Tlp) -> Result<Option<rocsteady::ParameterSchema>, rocsteady::Error> {
        if context.device_family.as_deref() != Some("fixture") || context.firmware.as_deref() != Some("1") {
            return Err(rocsteady::Error::CapabilityUnavailable("fixture schema identity"));
        }
        if tlp.point_type != 200 { return Ok(None); }
        let data_type = match tlp.parameter {
            1 => rocsteady::DataType::UInt16,
            2 => rocsteady::DataType::Ascii(4),
            _ => return Ok(None),
        };
        Ok(Some(rocsteady::ParameterSchema { data_type, access: rocsteady::Access::ReadOnly, logical_range: Some((0, 15)), numeric_range: None }))
    }
}
pub fn fixture_identity() -> rocsteady::SchemaContext {
    rocsteady::SchemaContext { device_family: Some("fixture".into()), firmware: Some("1".into()) }
}

/// Tight limits demonstrate both request and response batch boundaries.
pub struct FixtureProfile;
impl rocsteady::ProtocolProfile for FixtureProfile {
    fn metadata(&self) -> rocsteady::ProfileMetadata {
        rocsteady::ProfileMetadata { name: "extension-fixture", applicability: "loopback fixture only", assumptions: "fixture schema and ordinary unicast", evidence: "tests/profiles_schema.rs; no hardware evidence" }
    }
    fn capability(&self, opcode: u8) -> rocsteady::Capability {
        match opcode { 7 | 180 | 200 => rocsteady::Capability::UserSupplied, _ => rocsteady::Capability::Unavailable }
    }
    fn limits(&self, opcode: u8) -> rocsteady::PayloadLimits {
        if opcode == 180 { rocsteady::PayloadLimits { request: 7, response: 13 } }
        else { rocsteady::PayloadLimits::default() }
    }
}
