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
    pub parameter_values: Mutex<std::collections::BTreeMap<(u8, u8, u8), Vec<u8>>>,
    pub alarm_records: Mutex<std::collections::BTreeMap<u16, [u8; 23]>>,
    pub event_records: Mutex<std::collections::BTreeMap<u16, [u8; 22]>>,
    pub current_log_index: AtomicUsize,
    pub log_reads: AtomicUsize,
    pub log_fault_at: AtomicUsize,
    pub log_fault: AtomicUsize,
    pub change_log_at: AtomicUsize,
    pub change_log_index: AtomicUsize,
    pub grow_log_at: AtomicUsize,
    pub short_log_at: AtomicUsize,
    pub idle_signals: AtomicUsize,
    pub signals_sent: AtomicUsize,
    pub storm_on_stall: std::sync::atomic::AtomicBool,
    pub signal_wake: tokio::sync::Notify,
    pub signal_frame: Mutex<Option<Vec<u8>>>,
    pub srbx_pending: std::sync::atomic::AtomicBool,
    pub acknowledgements: AtomicUsize,
    pub last_ack_index: AtomicUsize,
    pub ack_fault: AtomicUsize,
    pub grow_before_ack: AtomicUsize,
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
        (91, 3) => b"W68000 Ver 3.40     ".to_vec(),
        (91, 4) => b"Oct 08,2022 00:00   ".to_vec(),
        (91, 5) => b"Example Vendor      ".to_vec(),
        (91, 6) => b"ROC809              ".to_vec(),
        (91, 8 | 9) => vec![0xc2, 0x01], // UINT16: 450.
        (91, 41) => vec![0x78, 0x56, 0x34, 0x12],
        (91, 42) => vec![0x00, 0x00, 0x01, 0x00],
        (91, 50) => vec![0],
        (91, 10) => vec![16],
        (91, 11) => vec![12],
        (91, 12) => vec![6],
        (91, 13 | 14) => vec![0xc1, 1], // UINT16: 449.
        (91, 22) => vec![1],
        (91, 23) => vec![0, 0, 0xc8, 0x42], // FLOAT: 100.
        (91, 37) => b"BOOT Ver 1.00       ".to_vec(),
        (91, 38) => b"Oct 08,2022 00:00   ".to_vec(),
        (91, 43..=46) => vec![0xff, 0xff],
        (91, 47 | 48) => vec![0x78, 0x56, 0x34, 0x12],
        (95, 0) => b"COMM2     ".to_vec(),
        (95, 1) => vec![3, 0], // Generator selection is UINT16, not UINT8.
        (95, 2 | 4) => vec![2],
        (95, 3) => vec![8],
        (95, 5) => vec![19], // Post-2.00 Ethernet code exceeds the printed range.
        (95, 6) => vec![1],
        (95, 7 | 8) => vec![0, 0, 0x80, 0x3f],
        (95, 9) => vec![255],
        (95, 36 | 42) => vec![0xff, 0xff],
        (95, 37) => vec![0x78, 0x56, 0x34, 0x12],
        (95, 44) => vec![2],
        (139, 0) => vec![36], // IEC module code absent from printed range.
        (139, 1) => vec![2], // Module failure is documented outside printed 0-1.
        (139, 2) => vec![2],
        (139, 3) => b"2.02      ".to_vec(),
        (139, 4) => b"W68000              ".to_vec(),
        (139, 8) => b"Oct 08,2022 00:00   ".to_vec(),
        (139, 9) => b"MODULE DATA         ".to_vec(),
        (101, 0) => b"DI SAMPLE ".to_vec(),
        (101, 1 | 3) => vec![1],
        (101, 2 | 13 | 14) => vec![0, 0, 0x80, 0x3f],
        (101, 4 | 5 | 9..=11 | 15) => vec![1],
        (101, 7 | 8) => vec![0, 0x24, 0x74, 0x49], // FLOAT: 1000000.
        (101, 12) => vec![0xff], // Known and unused alarm bits survive.
        (101, 6) => vec![0x04, 0x03, 0x02, 0x01],
        (103, 0) => b"AI SAMPLE ".to_vec(),
        (103, 1) => b"kPa       ".to_vec(),
        (103, 2) => vec![1],
        (103, 3 | 4) => vec![0, 0, 0x80, 0x3f], // FLOAT: 1.0.
        (103, 5) => vec![99],
        (103, 6 | 22 | 29..=31 | 36 | 38) => vec![1],
        (103, 8..=12) => vec![0xff, 0xff],
        (103, 13..=19 | 23..=28 | 37) => vec![0, 0, 0x20, 0xc1], // FLOAT: -10.
        (103, 20) => vec![0, 0, 0x48, 0x41],
        (103, 33) => vec![0, 0, 0x61, 0x45], // FLOAT: 3600.
        (103, 34) => vec![4],
        (103, 35) => vec![6],
        (103, 39) => vec![0, 0, 0xa0, 0x41], // FLOAT: 20.
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
                let (stream, _) = listener.accept().await.unwrap();
                stats.connections.fetch_add(1, Ordering::SeqCst);
                let stats = stats.clone();
                connections.spawn(async move {
                    let (mut reader, mut writer) = stream.into_split();
                    loop {
                        if emit_idle_signals(&mut writer, &stats).await.is_err() { break; }
                        let mut header = [0; 6];
                        let failed = {
                            let read = reader.read_exact(&mut header);
                            tokio::pin!(read);
                            loop {
                                tokio::select! {
                                    biased;
                                    result = &mut read => break result.is_err(),
                                    _ = stats.signal_wake.notified() => {
                                        if emit_idle_signals(&mut writer, &stats).await.is_err() { return; }
                                    }
                                }
                            }
                        };
                        if failed { break; }
                        assert_eq!(&header[..4], &[1, 2, 1, 0]);
                        let mut body = vec![0; usize::from(header[5]) + 2];
                        reader.read_exact(&mut body).await.unwrap();
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
                                        91 | 95 | 101 | 103 | 109 | 136 | 139 => 254,
                                        _ => 255,
                                    });
                                }
                                bytes
                            }
                            225 => {
                                assert_eq!(payload.len(), 2);
                                let index = u16::from_le_bytes([payload[0], payload[1]]);
                                assert!(index < 450);
                                let fault = stats.ack_fault.swap(0, Ordering::SeqCst);
                                if fault == 2 { opcode = 255; vec![20, 0] }
                                else {
                                    if stats.grow_before_ack.swap(0, Ordering::SeqCst) != 0 {
                                        let current = stats.current_log_index.load(Ordering::SeqCst);
                                        stats.current_log_index.store((current + 1) % 450, Ordering::SeqCst);
                                    }
                                    stats.acknowledgements.fetch_add(1, Ordering::SeqCst);
                                    stats.last_ack_index.store(usize::from(index), Ordering::SeqCst);
                                    if usize::from(index) == stats.current_log_index.load(Ordering::SeqCst) {
                                        stats.srbx_pending.store(false, Ordering::SeqCst);
                                    }
                                    match fault {
                                        1 => vec![1], // Remote acceptance but malformed reply.
                                        3 => break, // Effect committed, reply lost.
                                        4 => { std::future::pending::<()>().await; vec![] },
                                        _ => vec![],
                                    }
                                }
                            }
                            118 | 119 => {
                                let query = stats.log_reads.fetch_add(1, Ordering::SeqCst) + 1;
                                assert_eq!(payload.len(), 3);
                                let start = u16::from_le_bytes([payload[1], payload[2]]);
                                let count = payload[0];
                                assert!(count <= 10 && start < 450 && start + u16::from(count) <= 450);
                                if stats.grow_log_at.load(Ordering::SeqCst) == query {
                                    let current = stats.current_log_index.load(Ordering::SeqCst);
                                    stats.current_log_index.store((current + 1) % 450, Ordering::SeqCst);
                                }
                                let count = if stats.short_log_at.load(Ordering::SeqCst) == query { count.saturating_sub(1) } else { count };
                                let current = stats.current_log_index.load(Ordering::SeqCst) as u16;
                                let mut bytes = vec![count];
                                bytes.extend(start.to_le_bytes()); bytes.extend(current.to_le_bytes());
                                for index in start..start+u16::from(count) {
                                    if opcode == 118 {
                                        bytes.extend(stats.alarm_records.lock().unwrap().get(&index).copied().unwrap_or([0;23]));
                                    } else {
                                        bytes.extend(stats.event_records.lock().unwrap().get(&index).copied().unwrap_or([0;22]));
                                    }
                                }
                                if stats.change_log_at.load(Ordering::SeqCst) == query {
                                    let index = stats.change_log_index.load(Ordering::SeqCst) as u16;
                                    if opcode == 118 { stats.alarm_records.lock().unwrap().get_mut(&index).unwrap()[1] ^= 1; }
                                    else { stats.event_records.lock().unwrap().get_mut(&index).unwrap()[1] ^= 1; }
                                }
                                if stats.log_fault_at.load(Ordering::SeqCst) == query {
                                    match stats.log_fault.load(Ordering::SeqCst) {
                                        1 => { bytes.pop(); },
                                        2 => bytes[1] ^= 1,
                                        3 => { opcode = 255; bytes = vec![20,0]; },
                                        4 => break,
                                        5 => {
                                            if stats.storm_on_stall.load(Ordering::SeqCst) {
                                                loop {
                                                    if writer.write_all(&[1,0,1,2,224,0,232,45]).await.is_err() { return; }
                                                    stats.signals_sent.fetch_add(1, Ordering::SeqCst);
                                                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                                                }
                                            }
                                            std::future::pending::<()>().await;
                                        },
                                        _ => (),
                                    }
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
                                        95 => tlp[1] <= 5 && (tlp[1] != 1 || matches!(tlp[2], 36 | 37 | 42 | 44)),
                                        139 => tlp[1] <= 26,
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
                                    let override_value = stats.parameter_values.lock().unwrap()
                                        .get(&(tlp[0], tlp[1], tlp[2])).cloned();
                                    bytes.extend(override_value.unwrap_or_else(||
                                        catalog_literal(tlp[0], tlp[2]).expect("known literal catalog row")));

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
                        if writer.write_all(&bytes).await.is_err() {
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

async fn emit_idle_signals(writer: &mut tokio::net::tcp::OwnedWriteHalf, stats: &DiscoveryPeer) -> std::io::Result<()> {
    let count = stats.idle_signals.swap(0, Ordering::SeqCst);
    // Chapter 6 published CRC vector, independent of production frame/CRC encoders.
    let bytes = stats.signal_frame.lock().unwrap().clone().unwrap_or(vec![1,0,1,2,224,0,232,45]);
    for _ in 0..count {
        writer.write_all(&bytes).await?;
        stats.signals_sent.fetch_add(1, Ordering::SeqCst);
    }
    Ok(())
}
