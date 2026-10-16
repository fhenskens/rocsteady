//! D301180X012 October 2022 §2.26/Table 2-29, printed pp.2-42–43.
use rocsteady::*;
use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
fn segment() -> TransactionSegment {
    TransactionSegment::new(13, FirmwareVersion::new(3, 50).unwrap()).unwrap()
}
fn list(count: u8, more: u8, entries: &[u8]) -> Vec<u8> {
    let mut b = vec![1, count, more];
    b.extend([b'T', b'X', 0, 0xff, b' ', b' ', b' ', b' ', b' ', b' ']);
    b.extend([7, 0]);
    b.extend(entries);
    b
}
fn fields() -> Vec<u8> {
    vec![3, 0x34, 0x12, 9, b'A', 0, b'Z']
}
#[test]
fn transaction_wire_headers_lengths_flags_and_firmware_are_explicit() {
    // REQ-206-LIST / REQ-206-DATA: Table 2-29 offsets are full-frame offsets;
    // data size counts continuation plus data. Description/TIME remain raw.
    assert_eq!(
        TransactionListRequest::new(segment(), 0x1234).payload(),
        [1, 13, 0x34, 0x12]
    );
    assert_eq!(
        TransactionDataRequest::new(segment(), 0x5678, 0x1234).payload(),
        [2, 13, 0x78, 0x56, 0x34, 0x12]
    );
    let q = TransactionListRequest::new(segment(), 0);
    let bytes = list(
        2,
        0,
        &[0xff, 0xff, 0, 0, 0, 0, 0, 0, 0xff, 0xff, 0xff, 0xff],
    );
    let page = q.decode(&bytes).unwrap();
    assert_eq!(
        page.entries,
        [
            TransactionEntry {
                number: 65535,
                created: 0
            },
            TransactionEntry {
                number: 0,
                created: u32::MAX
            }
        ]
    );
    assert_eq!(
        page.description,
        [b'T', b'X', 0, 0xff, b' ', b' ', b' ', b' ', b' ', b' ']
    );
    assert_eq!(page.payload_size, 7);
    assert_eq!(page.raw, bytes);
    for n in 0..bytes.len() {
        assert!(q.decode(&bytes[..n]).is_err());
    }
    let mut extra = bytes.clone();
    extra.push(0);
    assert!(q.decode(&extra).is_err());
    for index in [0, 1, 2] {
        let mut bad = bytes.clone();
        bad[index] = 255;
        assert!(q.decode(&bad).is_err());
    }
    assert!(q.decode(&list(0, 0, &[])).unwrap().entries.is_empty());
    // POL-206-NO-PROGRESS: an empty continued page cannot advance the caller's cursor.
    assert!(q.decode(&list(0, 1, &[])).is_err());
    let maximum = list(40, 0, &[0; 240]);
    assert_eq!(q.decode(&maximum).unwrap().entries.len(), 40);
    assert!(q.decode(&list(41, 0, &[0; 246])).is_err());
    let q = TransactionDataRequest::new(segment(), 65535, 65535);
    let data = [2, 4, 0, 1, 255, 0];
    assert_eq!(q.decode(&data).unwrap().bytes, [1, 255, 0]);
    for n in 0..data.len() {
        assert!(q.decode(&data[..n]).is_err());
    }
    for index in [0, 1, 2] {
        let mut bad = data;
        bad[index] = 255;
        assert!(q.decode(&bad).is_err());
    }
    assert!(q.decode(&[2, 1, 0]).unwrap().bytes.is_empty());
    assert!(q.decode(&[2, 1, 1]).is_err());
    let mut maximum = vec![2, 253, 0];
    maximum.extend([0; 252]);
    assert_eq!(q.decode(&maximum).unwrap().bytes.len(), 252);
    maximum.push(0);
    assert!(q.decode(&maximum).is_err());
    // POL-206-APPLICABILITY: explicit reviewed firmware, no parse-based inference.
    for (major, minor, valid) in [
        (1, 0, false),
        (3, 49, false),
        (3, 50, true),
        (3, 60, true),
        (3, 61, false),
        (4, 0, false),
    ] {
        assert_eq!(
            TransactionSegment::new(255, FirmwareVersion::new(major, minor).unwrap()).is_ok(),
            valid
        );
    }
}
#[test]
fn transaction_tags_are_distinct_lossless_and_never_guess_unknown_widths() {
    // REQ-206-TAGS: own Table 2-29 vocabulary, including tag gaps 13/16/19.
    // POL-206-FRAMING: both named interpretations require explicit selection;
    // STRING/T_STRING payloads stay raw, including nonprintable bytes.
    let cases = vec![
        (1, vec![255], Some(Value::UInt8(255))),
        (2, vec![0x80], Some(Value::Int8(-128))),
        (3, vec![0x34, 0x12], Some(Value::UInt16(0x1234))),
        (4, vec![0, 0x80], Some(Value::Int16(-32768))),
        (5, vec![0xff; 4], Some(Value::UInt32(u32::MAX))),
        (6, vec![0, 0, 0, 0x80], Some(Value::Int32(i32::MIN))),
        (7, vec![0, 0, 0xc0, 0x3f], Some(Value::Float(1.5))),
        (
            8,
            vec![0, 0, 0, 0, 0, 0, 0xf0, 0x3f],
            Some(Value::Double(1.0)),
        ),
        (9, vec![0xff; 3], None),
        (10, vec![0; 7], None),
        (11, vec![b'A'; 10], None),
        (12, vec![b'B'; 20], None),
        (14, vec![b'C'; 30], None),
        (15, vec![b'D'; 40], None),
        (17, vec![0xff], Some(Value::Binary(0xff))),
        (18, vec![91, 0, 2], Some(Value::Tlp(Tlp::new(91, 0, 2)))),
        (20, vec![0xff; 4], Some(Value::Time(u32::MAX))),
    ];
    for layout in [
        TransactionFieldLayout::TaggedValues,
        TransactionFieldLayout::TlpTaggedValues,
    ] {
        let mut all = Vec::new();
        for (tag, raw, scalar) in &cases {
            let mut b = if layout == TransactionFieldLayout::TlpTaggedValues {
                vec![101, 16, 3]
            } else {
                vec![]
            };
            b.push(*tag);
            b.extend(raw);
            let values = decode_transaction_fields(&b, layout).unwrap();
            assert_eq!(values.len(), 1);
            assert_eq!(values[0].raw, *raw);
            assert_eq!(values[0].tag, *tag);
            assert_eq!(
                values[0].tlp,
                if layout == TransactionFieldLayout::TlpTaggedValues {
                    Some(Tlp::new(101, 16, 3))
                } else {
                    None
                }
            );
            assert_eq!(
                values[0].value,
                match scalar {
                    Some(v) => TransactionValue::Scalar(v.clone()),
                    None => TransactionValue::String(raw.clone()),
                }
            );
            for n in 1..b.len() {
                assert!(
                    decode_transaction_fields(&b[..n], layout).is_err(),
                    "tag {tag}, length {n}"
                );
            }
            all.extend(b);
        }
        assert_eq!(decode_transaction_fields(&all, layout).unwrap().len(), 17);
    }
    for tag in 0..=255 {
        if !cases.iter().any(|(t, _, _)| *t == tag) {
            for layout in [
                TransactionFieldLayout::TaggedValues,
                TransactionFieldLayout::TlpTaggedValues,
            ] {
                let mut bytes = if layout == TransactionFieldLayout::TlpTaggedValues {
                    vec![101, 16, 3]
                } else {
                    vec![]
                };
                bytes.extend([tag, 0, 0, 0, 0]);
                assert!(decode_transaction_fields(&bytes, layout).is_err());
            }
        }
    }
    for (tag, raw) in [
        (7, vec![1, 0, 0xc0, 0x7f]),
        (8, vec![0, 0, 0, 0, 0, 0, 0xf0, 0x7f]),
    ] {
        let mut b = vec![tag];
        b.extend(&raw);
        let decoded = decode_transaction_fields(&b, TransactionFieldLayout::TaggedValues).unwrap();
        assert_eq!(decoded[0].raw, raw);
        let TransactionValue::Scalar(value) = &decoded[0].value else {
            panic!()
        };
        assert_eq!(value.encode().unwrap(), raw);
    }
}
fn crc(b: &[u8]) -> u16 {
    let mut c = 0u16;
    for b in b {
        c ^= u16::from(*b);
        for _ in 0..8 {
            c = if c & 1 == 0 {
                c >> 1
            } else {
                (c >> 1) ^ 0xa001
            };
        }
    }
    c
}
#[derive(Clone)]
enum Reply {
    Data(Vec<u8>),
    Device,
    Disconnect,
    Stall,
    Wait(Arc<tokio::sync::Notify>, Vec<u8>),
}
type Exchanges = Arc<Mutex<VecDeque<(Vec<u8>, Reply)>>>;
struct Peer {
    endpoint: String,
    queue: Exchanges,
    calls: Arc<AtomicUsize>,
    connections: Arc<AtomicUsize>,
    seen: Arc<tokio::sync::Notify>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Peer {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl Peer {
    async fn start(exchanges: Vec<(Vec<u8>, Reply)>) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = listener.local_addr().unwrap().to_string();
        let queue = Arc::new(Mutex::new(VecDeque::from(exchanges)));
        let calls = Arc::new(AtomicUsize::new(0));
        let connections = Arc::new(AtomicUsize::new(0));
        let accepted_count = connections.clone();
        let seen = Arc::new(tokio::sync::Notify::new());
        let (q, n, notice) = (queue.clone(), calls.clone(), seen.clone());
        let task = tokio::spawn(async move {
            let mut tasks = tokio::task::JoinSet::new();
            loop {
                tokio::select! {
                    Some(result)=tasks.join_next(),if !tasks.is_empty()=>{result.unwrap();},
                    accepted=listener.accept()=>{
                        let (mut socket,_)=accepted.unwrap(); accepted_count.fetch_add(1, Ordering::SeqCst);let(q,n,notice)=(q.clone(),n.clone(),notice.clone());
                        tasks.spawn(async move{loop{
                            let mut h=[0;6];if socket.read_exact(&mut h).await.is_err(){break;}
                            assert_eq!(&h[..4],&[1,2,1,0]);let mut b=vec![0;usize::from(h[5])+2];socket.read_exact(&mut b).await.unwrap();
                            let mut wire=h.to_vec();wire.extend(&b[..b.len()-2]);assert_eq!(crc(&wire).to_le_bytes(),b[b.len()-2..]);
                            let (opcode,data)=if h[4]==7{assert_eq!(h[5],0); if n.load(Ordering::SeqCst) > 0 { assert!(q.lock().unwrap().is_empty(), "competing clock entered between transaction pages"); }(7,vec![1,2,3,8,10,0xea,7,5])}else{
                                assert_eq!(h[4],206);let(expected,response)=q.lock().unwrap().pop_front().expect("unexpected transaction replay/request");assert_eq!(&b[..b.len()-2],expected);
                                n.fetch_add(1,Ordering::SeqCst);notice.notify_one();
                                match response{Reply::Data(data)=>(206,data),Reply::Device=>(255,vec![14,8]),Reply::Disconnect=>return,Reply::Stall=>std::future::pending().await,Reply::Wait(release,data)=>{release.notified().await;(206,data)}}
                            };
                            let mut reply=vec![1,0,1,2,opcode,data.len() as u8];reply.extend(data);reply.extend(crc(&reply).to_le_bytes());socket.write_all(&reply).await.unwrap();
                        }});
                    }
                }
            }
        });
        Self {
            endpoint,
            queue,
            calls,
            connections,
            seen,
            task,
        }
    }
    fn client(&self) -> Client {
        Client::tcp(
            &self.endpoint,
            ClientConfig {
                heartbeat_interval: None,
                call_timeout: Duration::from_secs(2),
                ..Default::default()
            },
        )
        .unwrap()
    }
}
#[tokio::test]
async fn transaction_inventory_and_split_record_use_independent_tcp_literals() {
    // REQ-206-LIST / REQ-206-DATA / POL-206-ORDER / POL-206-SPLIT:
    // stored indices advance by returned count; raw order spans number rollover.
    let peer = Peer::start(vec![
        (
            vec![1, 13, 0, 0],
            Reply::Data(list(2, 1, &[0xff, 0xff, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0])),
        ),
        (
            vec![1, 13, 2, 0],
            Reply::Data(list(1, 0, &[42, 0, 2, 0, 0, 0])),
        ),
        (
            vec![2, 13, 42, 0, 0, 0],
            Reply::Data(vec![2, 3, 1, 3, 0x34]),
        ),
        (
            vec![2, 13, 42, 0, 2, 0],
            Reply::Data(vec![2, 6, 0, 0x12, 9, b'A', 0, b'Z']),
        ),
    ])
    .await;
    let client = peer.client();
    let inventory = client
        .transaction_inventory(TransactionListRequest::new(segment(), 0), 3)
        .await
        .unwrap();
    assert!(inventory.complete);
    assert_eq!(inventory.next_offset, None);
    assert_eq!(
        inventory
            .pages
            .iter()
            .flat_map(|p| p.entries.iter().map(|e| e.number))
            .collect::<Vec<_>>(),
        [65535, 0, 42]
    );
    let record = client
        .read_transaction(segment(), 42, inventory.pages[0].payload_size, 3)
        .await
        .unwrap();
    assert_eq!(record.bytes, fields());
    assert!(record.complete);
    assert_eq!(record.pages.len(), 2);
    let parsed =
        decode_transaction_fields(&record.bytes, TransactionFieldLayout::TaggedValues).unwrap();
    assert_eq!(
        parsed[0].value,
        TransactionValue::Scalar(Value::UInt16(0x1234))
    );
    assert_eq!(
        parsed[1].value,
        TransactionValue::String(vec![b'A', 0, b'Z'])
    );
    assert!(peer.queue.lock().unwrap().is_empty());
    client.stop().await;
}
#[tokio::test]
async fn transaction_failed_second_pages_preserve_confirmed_prefix_without_replay() {
    // POL-206-PROGRESS / POL-206-NO-REPLAY: malformed, lost and rejected second
    // pages retain only the acknowledged prefix. A new clock read cannot resume OP-206.
    for inventory in [false, true] {
        for reply in [Reply::Data(vec![99]), Reply::Device, Reply::Disconnect] {
            let exchanges = if inventory {
                vec![
                    (
                        vec![1, 13, 0, 0],
                        Reply::Data(list(1, 1, &[42, 0, 0, 0, 0, 0])),
                    ),
                    (vec![1, 13, 1, 0], reply),
                ]
            } else {
                vec![
                    (
                        vec![2, 13, 42, 0, 0, 0],
                        Reply::Data(vec![2, 3, 1, 3, 0x34]),
                    ),
                    (vec![2, 13, 42, 0, 2, 0], reply),
                ]
            };
            let peer = Peer::start(exchanges).await;
            let client = peer.client();
            if inventory {
                let failure = client
                    .transaction_inventory(TransactionListRequest::new(segment(), 0), 5)
                    .await
                    .unwrap_err();
                assert_eq!(failure.progress.pages.len(), 1);
                assert_eq!(failure.progress.next_offset, Some(1));
                assert!(!failure.progress.complete);
            } else {
                let failure = client
                    .read_transaction(segment(), 42, 7, 5)
                    .await
                    .unwrap_err();
                assert_eq!(failure.progress.bytes, [3, 0x34]);
                assert_eq!(failure.progress.pages.len(), 1);
                assert_eq!(failure.progress.next_offset, Some(2));
                assert!(!failure.progress.complete);
            }
            client.read_clock().await.unwrap();
            assert_eq!(peer.calls.load(Ordering::SeqCst), 2);
            assert!(peer.queue.lock().unwrap().is_empty());
            client.stop().await;
        }
    }
}
#[tokio::test]
async fn transaction_budgets_metadata_changes_and_offsets_are_bounded() {
    // POL-206-BOUNDS / POL-206-METADATA: max pages, declared payload geometry,
    // offset overflow and changed segment metadata fail explicitly, never wrap.
    let mut changed = list(1, 0, &[43, 0, 0, 0, 0, 0]);
    changed[13] = 8;
    for inventory in [false, true] {
        let first = if inventory {
            (
                vec![1, 13, 0, 0],
                Reply::Data(list(1, 1, &[42, 0, 0, 0, 0, 0])),
            )
        } else {
            (
                vec![2, 13, 42, 0, 0, 0],
                Reply::Data(vec![2, 3, 1, 3, 0x34]),
            )
        };
        let peer = Peer::start(vec![first]).await;
        let client = peer.client();
        if inventory {
            let e = client
                .transaction_inventory(TransactionListRequest::new(segment(), 0), 1)
                .await
                .unwrap_err();
            assert_eq!(e.progress.pages.len(), 1);
            assert_eq!(e.progress.next_offset, Some(1));
        } else {
            let e = client
                .read_transaction(segment(), 42, 7, 1)
                .await
                .unwrap_err();
            assert_eq!(e.progress.bytes, [3, 0x34]);
            assert_eq!(e.progress.next_offset, Some(2));
        }
        client.read_clock().await.unwrap();
        assert_eq!(peer.calls.load(Ordering::SeqCst), 1);
        client.stop().await;
    }
    let peer = Peer::start(vec![
        (
            vec![1, 13, 0, 0],
            Reply::Data(list(1, 1, &[42, 0, 0, 0, 0, 0])),
        ),
        (vec![1, 13, 1, 0], Reply::Data(changed)),
    ])
    .await;
    let client = peer.client();
    let error = client
        .transaction_inventory(TransactionListRequest::new(segment(), 0), 5)
        .await
        .unwrap_err();
    assert_eq!(error.progress.pages.len(), 1);
    assert!(matches!(
        error.error,
        etherbird::Error::Operation(Error::HistoryChanged(_))
    ));
    client.stop().await;
    let peer = Peer::start(vec![(
        vec![1, 13, 255, 255],
        Reply::Data(list(1, 1, &[42, 0, 0, 0, 0, 0])),
    )])
    .await;
    let client = peer.client();
    assert!(
        client
            .transaction_inventory(TransactionListRequest::new(segment(), 65535), 2)
            .await
            .unwrap_err()
            .progress
            .pages
            .is_empty()
    );
    client.stop().await;
    for (expected, reply) in [
        (1, vec![2, 3, 0, 1, 2]),
        (3, vec![2, 3, 0, 1, 2]),
        (2, vec![2, 3, 1, 1, 2]),
        (7, vec![2, 1, 1]),
    ] {
        let peer = Peer::start(vec![(vec![2, 13, 42, 0, 0, 0], Reply::Data(reply))]).await;
        let client = peer.client();
        let e = client
            .read_transaction(segment(), 42, expected, 5)
            .await
            .unwrap_err();
        assert!(e.progress.bytes.is_empty());
        assert!(!e.progress.complete);
        client.stop().await;
    }
}
#[tokio::test]
async fn transaction_preflight_limits_fail_without_io() {
    // POL-206-PREFLIGHT: invalid page limits and profile budgets/availability fail
    // before connection/admission; source segment/firmware is an explicit assertion.
    struct Blocked;
    impl ProtocolProfile for Blocked {
        fn metadata(&self) -> ProfileMetadata {
            DefaultProfile.metadata()
        }
        fn capability(&self, op: u8) -> Capability {
            if op == 206 {
                Capability::Unavailable
            } else {
                DefaultProfile.capability(op)
            }
        }
    }
    let peer = Peer::start(vec![]).await;
    let client = Client::tcp(
        &peer.endpoint,
        ClientConfig {
            profile: Arc::new(Blocked),
            heartbeat_interval: None,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        client
            .list_transactions(TransactionListRequest::new(segment(), 0))
            .await
            .is_err()
    );
    assert!(
        client
            .read_transaction_page(TransactionDataRequest::new(segment(), 0, 0))
            .await
            .is_err()
    );
    for pages in [0, 1, 65537] {
        assert!(
            client
                .transaction_inventory(TransactionListRequest::new(segment(), 0), pages)
                .await
                .is_err()
        );
        assert!(
            client
                .read_transaction(segment(), 0, 0, pages)
                .await
                .is_err()
        );
    }
    assert_eq!(peer.calls.load(Ordering::SeqCst), 0);
    assert_eq!(peer.connections.load(Ordering::SeqCst), 0);
    client.stop().await;
}
#[tokio::test]
async fn transaction_deadline_and_cancellation_keep_prefix_and_never_resume() {
    // POL-206-CANCEL: second exchange stalls after one confirmed page. Deadline,
    // caller abort and awaited stop must not admit continuation on a new generation.
    for mode in [0, 1, 2] {
        let peer = Peer::start(vec![
            (
                vec![2, 13, 42, 0, 0, 0],
                Reply::Data(vec![2, 3, 1, 3, 0x34]),
            ),
            (vec![2, 13, 42, 0, 2, 0], Reply::Stall),
        ])
        .await;
        let client = Client::tcp(
            &peer.endpoint,
            ClientConfig {
                heartbeat_interval: None,
                call_timeout: Duration::from_millis(80),
                ..Default::default()
            },
        )
        .unwrap();
        let c = client.clone();
        let task = tokio::spawn(async move { c.read_transaction(segment(), 42, 7, 5).await });
        tokio::time::timeout(Duration::from_secs(2), async {
            while peer.calls.load(Ordering::SeqCst) < 2 {
                peer.seen.notified().await;
            }
        })
        .await
        .unwrap();
        if mode == 1 {
            task.abort();
            assert!(task.await.unwrap_err().is_cancelled());
        } else {
            if mode == 2 {
                client.stop().await;
            }
            let e = task.await.unwrap().unwrap_err();
            assert_eq!(e.progress.bytes, [3, 0x34]);
            assert_eq!(e.progress.next_offset, Some(2));
            assert!(!e.progress.complete);
        }
        if mode != 2 {
            let mut states = client.subscribe_state();
            tokio::time::timeout(Duration::from_secs(5), async {
                loop {
                    states.borrow_and_update();
                    if peer.connections.load(Ordering::SeqCst) >= 2
                        && client.state() == ResourceState::Connected
                    {
                        break;
                    }
                    states.changed().await.unwrap();
                }
            })
            .await
            .unwrap();
            client.read_clock().await.unwrap();
        }
        assert_eq!(peer.calls.load(Ordering::SeqCst), 2);
        client.stop().await;
    }
}

#[tokio::test]
async fn transaction_workflow_keeps_exclusive_ownership_between_pages() {
    // POL-206-OWNERSHIP: a queued clone cannot interleave between inventory pages.
    // The peer rejects any clock request while its expected transaction queue is nonempty.
    use std::future::Future;
    let release = Arc::new(tokio::sync::Notify::new());
    let peer = Peer::start(vec![
        (
            vec![1, 13, 0, 0],
            Reply::Wait(release.clone(), list(1, 1, &[42, 0, 0, 0, 0, 0])),
        ),
        (
            vec![1, 13, 1, 0],
            Reply::Data(list(1, 0, &[43, 0, 1, 0, 0, 0])),
        ),
    ])
    .await;
    let client = peer.client();
    let c = client.clone();
    let task = tokio::spawn(async move {
        c.transaction_inventory(TransactionListRequest::new(segment(), 0), 3)
            .await
    });
    peer.seen.notified().await;
    let mut clock = Box::pin(client.read_clock());
    std::future::poll_fn(|cx| {
        assert!(clock.as_mut().poll(cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    release.notify_one();
    assert!(task.await.unwrap().unwrap().complete);
    clock.await.unwrap();
    assert!(peer.queue.lock().unwrap().is_empty());
    client.stop().await;
}

#[tokio::test]
async fn transaction_standalone_malformed_reply_retires_and_profile_budgets_preflight() {
    // REQ-206-DATA / POL-206-RETIREMENT: typed decode remains under Session guard.
    for reply in [Reply::Data(vec![2, 2, 0]), Reply::Device] {
        let healthy = matches!(reply, Reply::Device);
        let peer = Peer::start(vec![(vec![2, 13, 42, 0, 0, 0], reply)]).await;
        let stream = tokio::net::TcpStream::connect(&peer.endpoint)
            .await
            .unwrap();
        let session = Session::new(
            Box::new(stream),
            Address::new(1, 0),
            Address::new(1, 2),
            Duration::from_secs(1),
            protocol::CrcMode::Required,
        )
        .unwrap();
        assert!(
            session
                .read_transaction_page(TransactionDataRequest::new(segment(), 42, 0))
                .await
                .is_err()
        );
        assert_eq!(session.is_connected(), healthy);
        session.disconnect().await;
    }
    // POL-206-PREFLIGHT: known minimum response and exact request sizes are
    // validated before admission, even if a profile advertises provisional support.
    struct Tight(usize, usize);
    impl ProtocolProfile for Tight {
        fn metadata(&self) -> ProfileMetadata {
            DefaultProfile.metadata()
        }
        fn capability(&self, op: u8) -> Capability {
            DefaultProfile.capability(op)
        }
        fn limits(&self, op: u8) -> PayloadLimits {
            if op == 206 {
                PayloadLimits {
                    request: self.0,
                    response: self.1,
                }
            } else {
                PayloadLimits::default()
            }
        }
    }
    for (request, response) in [(3, 255), (255, 2)] {
        let peer = Peer::start(vec![]).await;
        let client = Client::tcp(
            &peer.endpoint,
            ClientConfig {
                profile: Arc::new(Tight(request, response)),
                heartbeat_interval: None,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(
            client
                .list_transactions(TransactionListRequest::new(segment(), 0))
                .await
                .is_err()
        );
        assert!(
            client
                .read_transaction_page(TransactionDataRequest::new(segment(), 42, 0))
                .await
                .is_err()
        );
        assert!(
            client
                .transaction_inventory(TransactionListRequest::new(segment(), 0), 3)
                .await
                .is_err()
        );
        assert!(client.read_transaction(segment(), 42, 7, 3).await.is_err());
        assert_eq!(peer.connections.load(Ordering::SeqCst), 0);
        client.stop().await;
    }
}

#[cfg(feature = "simulator")]
#[tokio::test]
async fn transaction_simulator_reassembles_split_fields_and_retains_storage_order() {
    // REQ-206-SIMULATOR / POL-206-SPLIT: explicit byte dataset, not encoder-generated.
    // Every fixed-width tag/string can cross page boundaries; list order isn't time order.
    use simulator::{HistoryMetadataModel, Model, Simulator, TransactionSegmentModel};
    let mut raw = vec![
        1, 255, 2, 128, 3, 0x34, 0x12, 4, 0, 0x80, 5, 255, 255, 255, 255, 6, 0, 0, 0, 128, 7, 0, 0,
        0xc0, 0x3f, 8, 0, 0, 0, 0, 0, 0, 0xf0, 0x3f,
    ];
    for (tag, count) in [(9, 3), (10, 7), (11, 10), (12, 20), (14, 30), (15, 40)] {
        raw.push(tag);
        raw.extend(vec![0xff; count]);
    }
    raw.extend([17, 0xff, 18, 91, 0, 2, 20, 0xff, 0xff, 0xff, 0xff]);
    let mut history = HistoryMetadataModel::default();
    history.transactions.insert(
        13,
        TransactionSegmentModel {
            description: *b"TEST TX   ",
            payload_size: raw.len() as u16,
            records: vec![
                (65535, 100, raw.clone()),
                (0, 1, raw.clone()),
                (42, 10, raw.clone()),
            ],
            list_page_size: 1,
            data_page_size: 5,
        },
    );
    let server =
        Simulator::bind_with_history("127.0.0.1:0".parse().unwrap(), Model::default(), history)
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
    let inventory = client
        .transaction_inventory(TransactionListRequest::new(segment(), 0), 3)
        .await
        .unwrap();
    assert_eq!(
        inventory
            .pages
            .iter()
            .map(|p| p.entries[0].number)
            .collect::<Vec<_>>(),
        [65535, 0, 42]
    );
    let data = client
        .read_transaction(segment(), 42, inventory.pages[0].payload_size, 100)
        .await
        .unwrap();
    assert!(data.complete);
    assert_eq!(data.bytes, raw);
    assert_eq!(
        decode_transaction_fields(&data.bytes, TransactionFieldLayout::TaggedValues)
            .unwrap()
            .len(),
        17
    );
    let before = server.stats().requests;
    // Vanished/unknown transaction is a healthy raw device rejection, not an empty success.
    assert!(matches!(
        client
            .read_transaction(segment(), 123, 7, 5)
            .await
            .unwrap_err()
            .error,
        etherbird::Error::Operation(Error::Device(_))
    ));
    client.read_clock().await.unwrap();
    assert_eq!(server.stats().requests - before, 2);
    assert_eq!(server.stats().connections, 1);
    client.stop().await;
    server.stop().await.unwrap();
}

#[cfg(feature = "simulator")]
#[tokio::test]
async fn transaction_simulator_handles_empty_and_maximum_sized_records() {
    // REQ-206-SIMULATOR / POL-206-BOUNDS: zero-length record and UINT16 maximum
    // payload; byte offsets never wrap, and 252 data bytes fit the unicast response.
    use simulator::{HistoryMetadataModel, Model, Simulator, TransactionSegmentModel};
    let raw = vec![0xa5; 65535];
    let mut history = HistoryMetadataModel::default();
    history.transactions.insert(
        13,
        TransactionSegmentModel {
            description: *b"MAX TX    ",
            payload_size: 65535,
            records: vec![(42, 0, raw.clone())],
            ..Default::default()
        },
    );
    history.transactions.insert(
        14,
        TransactionSegmentModel {
            records: vec![(0, 0, vec![])],
            ..Default::default()
        },
    );
    let server =
        Simulator::bind_with_history("127.0.0.1:0".parse().unwrap(), Model::default(), history)
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
    let data = client
        .read_transaction(segment(), 42, 65535, 261)
        .await
        .unwrap();
    assert_eq!(data.bytes, raw);
    assert_eq!(data.pages.len(), 261);
    assert!(data.complete);
    let empty = TransactionSegment::new(14, FirmwareVersion::new(3, 60).unwrap()).unwrap();
    let data = client.read_transaction(empty, 0, 0, 1).await.unwrap();
    assert!(data.complete);
    assert!(data.bytes.is_empty());
    let model = Model::default();
    let mut auth = false;
    for payload in [
        vec![],
        vec![3, 13],
        vec![1, 13, 0],
        vec![2, 13, 0, 0, 0],
        vec![2, 13, 123, 0, 0, 0],
    ] {
        assert_eq!(
            model
                .respond(
                    &protocol::Frame::new(Address::new(1, 2), Address::new(1, 0), 206, payload),
                    &mut auth
                )
                .unwrap()
                .opcode,
            255
        );
    }
    client.stop().await;
    server.stop().await.unwrap();
}

#[tokio::test]
async fn transaction_semantic_second_page_failures_keep_prefix_and_classify_retirement() {
    // POL-206-METADATA / POL-206-BOUNDS / POL-206-PROGRESS / POL-206-NO-REPLAY:
    // a changed raw description or declared size, and contradictory final/more
    // geometry after a valid data page, cannot enter the confirmed prefix.
    // Metadata changes keep the channel; inconsistent payload geometry is an
    // InvalidResponse and retires it under the existing error classification.
    for field in [3, 13] {
        let mut changed = list(1, 0, &[43, 0, 0, 0, 0, 0]);
        changed[field] ^= 1;
        let peer = Peer::start(vec![
            (
                vec![1, 13, 0, 0],
                Reply::Data(list(1, 1, &[42, 0, 0, 0, 0, 0])),
            ),
            (vec![1, 13, 1, 0], Reply::Data(changed)),
        ])
        .await;
        let client = peer.client();
        let error = client
            .transaction_inventory(TransactionListRequest::new(segment(), 0), 5)
            .await
            .unwrap_err();
        assert!(matches!(
            error.error,
            etherbird::Error::Operation(Error::HistoryChanged(_))
        ));
        assert_eq!(error.progress.pages.len(), 1);
        assert_eq!(error.progress.pages[0].entries[0].number, 42);
        assert_eq!(error.progress.next_offset, Some(1));
        assert!(!error.progress.complete);
        client.read_clock().await.unwrap();
        assert_eq!(peer.calls.load(Ordering::SeqCst), 2);
        assert_eq!(peer.connections.load(Ordering::SeqCst), 1);
        client.stop().await;
    }
    for reply in [
        vec![2, 2, 0, 9],
        vec![2, 3, 1, 9, 8],
        vec![2, 4, 0, 9, 8, 7],
    ] {
        let peer = Peer::start(vec![
            (
                vec![2, 13, 42, 0, 0, 0],
                Reply::Data(vec![2, 3, 1, 3, 0x34]),
            ),
            (vec![2, 13, 42, 0, 2, 0], Reply::Data(reply)),
        ])
        .await;
        let client = peer.client();
        let error = client
            .read_transaction(segment(), 42, 4, 5)
            .await
            .unwrap_err();
        assert!(matches!(
            error.error,
            etherbird::Error::Operation(Error::InvalidResponse(_))
        ));
        assert_eq!(error.progress.bytes, [3, 0x34]);
        assert_eq!(error.progress.pages.len(), 1);
        assert_eq!(error.progress.next_offset, Some(2));
        assert!(!error.progress.complete);
        client.read_clock().await.unwrap();
        assert_eq!(peer.calls.load(Ordering::SeqCst), 2);
        assert_eq!(peer.connections.load(Ordering::SeqCst), 2);
        client.stop().await;
    }
}
