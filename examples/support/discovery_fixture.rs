// Independent wire oracle for Tables 2-12/2-13 and selected Chapter 3 rows.
// Uses no production frame, CRC, discovery or parameter encoders.
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Default)]
pub struct DiscoveryPeer {
    pub connections: AtomicUsize,
    pub parameter_reads: AtomicUsize,
    pub type_queries: AtomicUsize,
    pub logical_queries: AtomicUsize,
    pub config_queries: AtomicUsize,
    pub fault_opcode: AtomicUsize,
    pub fault: AtomicUsize,
    pub change_type_at: AtomicUsize,
    pub change_logical_at: AtomicUsize,
    pub change_config_at: AtomicUsize,
    pub requests: Mutex<Vec<(u8, Vec<u8>)>>,
}
pub struct DiscoveryFixture {
    pub endpoint: String,
    pub peer: Arc<DiscoveryPeer>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for DiscoveryFixture {
    fn drop(&mut self) {
        self.task.abort();
    }
}
fn discovery_crc(bytes: &[u8]) -> u16 {
    let mut crc = 0u16;
    for &byte in bytes {
        crc ^= u16::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xa001
            } else {
                crc >> 1
            };
        }
    }
    crc
}
/// Independently transcribed type/width fixtures, not derived from CatalogEntry.
pub fn catalog_literal(point_type: u8, parameter: u8) -> Option<Vec<u8>> {
    Some(match (point_type, parameter) {
        (91, 0) => vec![1],
        (91, 1) => vec![2],
        (91, 2) => b"Example ROC Station ".to_vec(),
        (91, 3) => b"W68000 Ver 3.00     ".to_vec(),
        (91, 4) => b"Oct 08,2022 00:00   ".to_vec(),
        (91, 5) => b"Example Vendor      ".to_vec(),
        (91, 6) => b"ROC809              ".to_vec(),
        (91, 8 | 9) => vec![0xc2, 0x01], // UINT16: 450.
        (91, 41) => vec![0x78, 0x56, 0x34, 0x12],
        (91, 42) => vec![0x00, 0x00, 0x01, 0x00],
        (91, 50) => vec![0],
        (101, 0) => b"DI SAMPLE ".to_vec(),
        (101, 1 | 3) => vec![1],
        (101, 6) => vec![0x04, 0x03, 0x02, 0x01],
        (103, 0) => b"AI SAMPLE ".to_vec(),
        (103, 1) => b"kPa       ".to_vec(),
        (103, 2) => vec![1],
        (103, 3 | 4) => vec![0, 0, 0x80, 0x3f], // FLOAT: 1.0.
        (103, 7) => vec![0x34, 0x12],
        (103, 21) => vec![0, 0, 0x48, 0x41], // FLOAT: 12.5.
        (103, 32) => vec![0xa5],
        (109, 0) => b"Battery   ".to_vec(),
        (109, 1) => b"Volts     ".to_vec(),
        (109, 4) => vec![0, 0, 0x80, 0x3f],
        (109, 7) => vec![0x78, 0x56],
        (109, 12) => vec![0, 0, 0x40, 0x41], // FLOAT: 12.0; different parameter from PT-103.
        (109, 23) => vec![0x81],
        // Table 3-48: independent widths and boundary/raw values.
        (136, 0 | 1) => vec![59],
        (136, 2 | 10 | 15) => vec![23],
        (136, 3) => vec![31],
        (136, 4 | 13 | 18) => vec![12],
        (136, 5) => vec![0x38, 0x08], // UINT16: 2104.
        (136, 6 | 11 | 16) => vec![7],
        (136, 7) => vec![0xff, 0xff, 0xff, 0xff], // Raw TIME, no timezone inference.
        (136, 8) => vec![1],
        (136, 9) => vec![0x3f, 0x42, 0x0f, 0x00], // UINT32: 999999.
        (136, 12) => vec![6],
        (136, 14) => vec![0x78, 0x56, 0x34, 0x12],
        (136, 17) => vec![56], // Preserve the disputed source value.
        (136, 19) => vec![0, 0, 0, 0],
        _ => return None,
    })
}
impl DiscoveryFixture {
    pub async fn start(layout_status: u8) -> Self {
        // Source prose/Table 2-12 lengths, independent of IoLayout methods.
        let (length, stride) = match layout_status {
            0 => (160, 16),
            1 => (240, 16),
            2 => (224, 8),
            _ => panic!("fixture layout"),
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = listener.local_addr().unwrap().to_string();
        let peer = Arc::new(DiscoveryPeer::default());
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
                        if stream.read_exact(&mut header).await.is_err() {
                            break;
                        }
                        assert_eq!(&header[..4], &[1, 2, 1, 0]);
                        let mut body = vec![0; usize::from(header[5]) + 2];
                        stream.read_exact(&mut body).await.unwrap();
                        let mut wire = header.to_vec();
                        wire.extend(&body[..body.len() - 2]);
                        assert_eq!(discovery_crc(&wire).to_le_bytes(), body[body.len() - 2..]);
                        let payload = body[..body.len() - 2].to_vec();
                        let mut opcode = header[4];
                        stats
                            .requests
                            .lock()
                            .unwrap()
                            .push((opcode, payload.clone()));
                        let mut response = match opcode {
                            6 => {
                                assert!(payload.is_empty());
                                let query = stats.config_queries.fetch_add(1, Ordering::SeqCst) + 1;
                                let mut bytes = vec![0; 98];
                                bytes[0] = 1;
                                bytes[4] = layout_status;
                                bytes[18] = 6;
                                bytes[50] = 1; // Table 2-2 count for PT-91 (offset 19 + 91 - 60).
                                bytes[60] = 1;
                                bytes[62] = 1;
                                bytes[68] = 5; // PT-101/103/109 counts.
                                bytes[95] = 1; // PT-136 is a singleton, outside the I/O map.
                                if stats.change_config_at.load(Ordering::SeqCst) == query {
                                    bytes[6] = 1;
                                }
                                bytes
                            }
                            7 => {
                                assert!(payload.is_empty());
                                vec![1, 2, 3, 8, 10, 0xea, 7, 5]
                            }
                            50 => {
                                assert!(matches!(payload.as_slice(), [0] | [1]));
                                let mut bytes = vec![0; length];
                                if payload == [0] {
                                    let query =
                                        stats.type_queries.fetch_add(1, Ordering::SeqCst) + 1;
                                    bytes[..5].fill(109);
                                    bytes[stride] = 103;
                                    bytes[2 * stride] = 101;
                                    bytes[length - 1] = 200; // Unknown type retained.
                                    if stats.change_type_at.load(Ordering::SeqCst) == query {
                                        bytes[stride] = 104;
                                    }
                                } else {
                                    let query =
                                        stats.logical_queries.fetch_add(1, Ordering::SeqCst) + 1;
                                    bytes[..5].copy_from_slice(&[0, 1, 2, 3, 4]);
                                    bytes[length - 1] = 254;
                                    if stats.change_logical_at.load(Ordering::SeqCst) == query {
                                        bytes[stride] = 1;
                                    }
                                }
                                bytes
                            }
                            100 => {
                                assert_eq!(payload.len(), 3);
                                assert_eq!(payload[0], 11);
                                assert!(payload[2] <= 245);
                                assert!(u16::from(payload[1]) + u16::from(payload[2]) <= 256);
                                let mut bytes = payload.clone();
                                for point_type in u16::from(payload[1])
                                    ..u16::from(payload[1]) + u16::from(payload[2])
                                {
                                    bytes.push(match point_type {
                                        0..=7 => point_type as u8,
                                        60 => 253,
                                        61 => 42,
                                        91 | 101 | 103 | 109 | 136 => 254,
                                        _ => 255,
                                    });
                                }
                                bytes
                            }
                            180 => {
                                stats.parameter_reads.fetch_add(1, Ordering::SeqCst);
                                assert_eq!(payload.len(), 1 + usize::from(payload[0]) * 3);
                                let mut bytes = vec![payload[0]];
                                for tlp in payload[1..].as_chunks::<3>().0 {
                                    let valid_location = match tlp[0] {
                                        91 | 136 => tlp[1] == 0,
                                        101 => usize::from(tlp[1]) == 2 * stride,
                                        103 => usize::from(tlp[1]) == stride,
                                        109 => tlp[1] < 5,
                                        _ => false,
                                    };
                                    assert!(
                                        valid_location,
                                        "fixture TLP must use physical position: {tlp:?}"
                                    );
                                    bytes.extend(tlp);
                                    bytes.extend(
                                        catalog_literal(tlp[0], tlp[2])
                                            .expect("known literal catalog row"),
                                    );
                                }
                                assert!(bytes.len() <= 240);
                                bytes
                            }
                            _ => panic!("unexpected discovery fixture opcode"),
                        };
                        if stats.fault_opcode.load(Ordering::SeqCst) == usize::from(opcode) {
                            match stats.fault.swap(0, Ordering::SeqCst) {
                                1 => {
                                    response.pop();
                                } // Truncated but valid frame.
                                2 => {
                                    response[1] ^= 1;
                                } // Wrong opcode-100 start echo.
                                3 => {
                                    opcode = 255;
                                    response = vec![20, 0];
                                }
                                4 => break,
                                5 => std::future::pending::<()>().await,
                                6 => response.push(0),
                                7 => response[0] ^= 1,
                                8 => response[2] ^= 1,
                                _ => (),
                            }
                        }
                        let mut bytes = vec![1, 0, 1, 2, opcode, response.len() as u8];
                        bytes.extend(response);
                        let crc = discovery_crc(&bytes);
                        bytes.extend(crc.to_le_bytes());
                        if stream.write_all(&bytes).await.is_err() {
                            break;
                        }
                    }
                });
            }
        });
        Self {
            endpoint,
            peer,
            task,
        }
    }
}
