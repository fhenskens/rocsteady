include!("../examples/support/discovery_fixture.rs");
use rocsteady::{
    ArchivalMethod, CalendarOccurrenceCodec, Capability, Client, ClientConfig, DecodeError,
    DefaultProfile, Error, HistoryTagsCodec, HistoryTagsRequest, MinMaxCodec, MinMaxRequest,
    OccurrenceTime, PayloadLimits, PrefixedEpochOccurrenceCodec, ProfileMetadata, ProtocolProfile,
    RawOccurrenceCodec, ReplyCodec,
};
use std::time::Duration;
fn config() -> ClientConfig {
    ClientConfig {
        heartbeat_interval: None,
        response_timeout: Duration::from_millis(100),
        call_timeout: Duration::from_secs(3),
        recovery: etherbird::Config {
            retry_delay: Duration::from_millis(1),
            ..Default::default()
        },
        ..Default::default()
    }
}
fn tags_literal(segment: u8, points: &[u8], index: u16) -> Vec<u8> {
    let mut b = vec![segment, points.len() as u8];
    b.extend(index.to_le_bytes());
    for p in points {
        b.push(*p);
        b.extend(*b"FLOW      ");
    }
    b
}

// REQ-105-REQUEST / REQ-108-REQUEST: Tables 2-14/2-15, pp.2-23–24.
// Baseline segment 0–10, OP-108 points 0–199, max 20 in request order.
#[test]
fn requests_validate_boundaries_and_explicit_applicability_overrides() {
    assert_eq!(MinMaxRequest::new(10, 255).unwrap().payload(), [10, 255]);
    assert!(MinMaxRequest::new(11, 0).is_err());
    assert_eq!(
        MinMaxRequest::with_segment_limit(13, 255, 13)
            .unwrap()
            .payload(),
        [13, 255]
    );
    assert!(MinMaxRequest::with_segment_limit(14, 0, 13).is_err());
    let q = HistoryTagsRequest::new(10, vec![199; 20]).unwrap();
    assert_eq!(&q.payload()[..2], &[10, 20]);
    assert_eq!(q.payload().len(), 22);
    for points in [vec![], vec![0; 21], vec![200]] {
        assert!(HistoryTagsRequest::new(0, points).is_err());
    }
    assert!(HistoryTagsRequest::new(11, vec![0]).is_err());
    assert_eq!(
        HistoryTagsRequest::new(0, vec![7, 2, 7]).unwrap().payload(),
        [0, 3, 7, 2, 7]
    );
    assert_eq!(
        HistoryTagsRequest::with_limits(13, vec![255], 13, 255)
            .unwrap()
            .payload(),
        [13, 1, 255]
    );
}

// REQ-105-FIELDS: Table 2-14 offsets, all six IEEE floats, TLP, method and four times.
#[test]
fn min_max_literals_cover_every_field_and_unknown_archival_method() {
    let q = MinMaxRequest::new(10, 199).unwrap();
    let b = min_max_literal(10, 199);
    let r = MinMaxCodec(RawOccurrenceCodec).decode(&q, &b).unwrap();
    assert_eq!(r.request, q);
    assert_eq!(r.archival_method.0, 128);
    assert_eq!(r.tlp, rocsteady::Tlp::new(103, 16, 21));
    assert_eq!(
        (
            r.current,
            r.today.minimum,
            r.today.maximum,
            r.yesterday.minimum,
            r.yesterday.maximum,
            r.last_completed_period
        ),
        (12.5, -1.0, 100.0, -10.0, 200.0, 1.5)
    );
    assert_eq!(r.today.minimum_time.raw(), &[1, 2, 3, 4, 5]);
    assert_eq!(r.today.maximum_time.raw(), &[6, 7, 8, 9, 10]);
    assert_eq!(r.yesterday.minimum_time.raw(), &[11, 12, 13, 14, 2]);
    assert_eq!(r.yesterday.maximum_time.raw(), &[15, 16, 17, 18, 12]);
    assert_eq!(r.raw, b);
    for (code, key) in [
        (0, "undefined"),
        (128, "hourly_average"),
        (129, "hourly_accumulated"),
        (130, "hourly_current"),
        (134, "hourly_totalize"),
        (67, "fst_timestamp"),
        (65, "fst_value"),
    ] {
        assert_eq!(ArchivalMethod(code).key(), Some(key));
        let mut bytes = b;
        bytes[2] = code;
        assert_eq!(
            MinMaxCodec(RawOccurrenceCodec)
                .decode(&q, &bytes)
                .unwrap()
                .current
                .to_bits(),
            12.5f32.to_bits()
        );
    }
    let mut unknown = b;
    unknown[2] = 255;
    unknown[6..10].copy_from_slice(&[0x34, 0x12, 0xc0, 0x7f]);
    let r = MinMaxCodec(RawOccurrenceCodec)
        .decode(&q, &unknown)
        .unwrap();
    assert_eq!(r.archival_method, ArchivalMethod(255));
    assert!(r.archival_method.key().is_none());
    assert_eq!(r.current.to_bits(), 0x7fc01234);
    assert_eq!(r.raw, unknown);
}

// REQ-105-VALIDATE: exact 50-byte layout and both request echoes, Table 2-14.
#[test]
fn min_max_rejects_every_truncation_trailing_data_and_wrong_echo() {
    let q = MinMaxRequest::new(0, 7).unwrap();
    let b = min_max_literal(0, 7);
    let codec = MinMaxCodec(RawOccurrenceCodec);
    for n in 0..50 {
        assert!(codec.decode(&q, &b[..n]).is_err());
    }
    let mut extra = b.to_vec();
    extra.push(0);
    assert!(codec.decode(&q, &extra).is_err());
    for i in [0, 1] {
        let mut wrong = b;
        wrong[i] ^= 1;
        assert!(codec.decode(&q, &wrong).is_err());
    }
}

// POL-105-OCCURRENCES: DEC-4.01. Identical five bytes support competing
// named provisional interpretations. Parsing success NEVER chooses a codec.
#[test]
fn raw_calendar_and_prefixed_epoch_choices_are_explicit_and_lossless() {
    let b = [1, 2, 3, 4, 5];
    assert_eq!(
        RawOccurrenceCodec.decode(&(), &b).unwrap(),
        OccurrenceTime::Raw(b)
    );
    assert!(matches!(
        CalendarOccurrenceCodec.decode(&(), &b).unwrap(),
        OccurrenceTime::Calendar {
            second: 1,
            minute: 2,
            hour: 3,
            day: 4,
            month: 5,
            ..
        }
    ));
    assert!(matches!(
        PrefixedEpochOccurrenceCodec.decode(&(), &b).unwrap(),
        OccurrenceTime::PrefixedEpoch {
            prefix: 1,
            seconds: 0x05040302,
            ..
        }
    ));
    for raw in [
        [0; 5],
        [255; 5],
        [1, 60, 0, 1, 1],
        [1, 1, 24, 1, 1],
        [1, 1, 0, 31, 4],
        [1, 1, 0, 1, 0],
    ] {
        assert_eq!(RawOccurrenceCodec.decode(&(), &raw).unwrap().raw(), &raw);
        assert_eq!(
            PrefixedEpochOccurrenceCodec
                .decode(&(), &raw)
                .unwrap()
                .raw(),
            &raw
        );
        assert!(CalendarOccurrenceCodec.decode(&(), &raw).is_err());
    }
    assert!(
        CalendarOccurrenceCodec
            .decode(&(), &[59, 59, 23, 29, 2])
            .is_ok()
    ); // no inferred year
    for n in [0, 4, 6] {
        let raw = vec![0; n];
        assert!(RawOccurrenceCodec.decode(&(), &raw).is_err());
        assert!(CalendarOccurrenceCodec.decode(&(), &raw).is_err());
        assert!(PrefixedEpochOccurrenceCodec.decode(&(), &raw).is_err());
    }
}

// REQ-108-REPLY: Table 2-15: common UINT16 index + ordered (point,AC10).
// Preserve padding/unknown bytes and validate every count/point echo and byte length.
#[test]
fn history_tags_validate_maximum_count_echoes_order_and_padding() {
    let q = HistoryTagsRequest::new(10, vec![199; 20]).unwrap();
    let b = tags_literal(10, q.points(), u16::MAX);
    assert_eq!(b.len(), 224);
    let r = HistoryTagsCodec.decode(&q, &b).unwrap();
    assert_eq!(r.periodic_index, u16::MAX);
    assert_eq!(r.tags.len(), 20);
    for tag in &r.tags {
        assert_eq!(tag.point, 199);
        assert_eq!(tag.tag.printable_ascii(), Some("FLOW      "));
    }
    for n in 0..b.len() {
        assert!(HistoryTagsCodec.decode(&q, &b[..n]).is_err());
    }
    let mut extra = b.clone();
    extra.push(0);
    assert!(HistoryTagsCodec.decode(&q, &extra).is_err());
    for i in [0, 1, 4, 15, 213] {
        let mut wrong = b.clone();
        wrong[i] ^= 1;
        assert!(HistoryTagsCodec.decode(&q, &wrong).is_err());
    }
    let q = HistoryTagsRequest::new(0, vec![7, 2, 7]).unwrap();
    let mut b = tags_literal(0, q.points(), 1);
    b[5..15].copy_from_slice(&[0, 255, 32, 32, 32, 32, 32, 32, 32, 32]);
    let r = HistoryTagsCodec.decode(&q, &b).unwrap();
    assert_eq!(&r.tags[0].tag.0[..2], &[0, 255]);
    assert!(r.tags[0].tag.printable_ascii().is_none());
    b.swap(4, 15);
    assert!(HistoryTagsCodec.decode(&q, &b).is_err());
}
struct CustomTime;
impl ReplyCodec<()> for CustomTime {
    type Output = Vec<u8>;
    fn decode(&self, _: &(), bytes: &[u8]) -> Result<Self::Output, DecodeError> {
        Ok(bytes.iter().rev().copied().collect())
    }
}
struct CalendarProfile;
impl ProtocolProfile for CalendarProfile {
    fn metadata(&self) -> ProfileMetadata {
        ProfileMetadata {
            name: "explicit-calendar-fixture",
            applicability: "synthetic fixture",
            assumptions: "provisional calendar",
            evidence: "tests/history_metadata.rs; no hardware",
        }
    }
    fn capability(&self, opcode: u8) -> Capability {
        DefaultProfile.capability(opcode)
    }
    fn decode_min_max(
        &self,
        q: &MinMaxRequest,
        b: &[u8],
    ) -> Result<rocsteady::MinMaxReply, DecodeError> {
        MinMaxCodec(CalendarOccurrenceCodec).decode(q, b)
    }
    fn decode_history_tags(
        &self,
        q: &HistoryTagsRequest,
        b: &[u8],
    ) -> Result<rocsteady::HistoryTagsReply, DecodeError> {
        HistoryTagsCodec.decode(q, b)
    }
}

// REQ-105-TCP / REQ-108-TCP / POL-105-EXTENSION: independent literals
// through default methods, profile selection, prebuilt codecs and a caller codec.
#[tokio::test]
async fn independent_tcp_defaults_and_replaceable_prebuilt_and_custom_codecs() {
    let peer = DiscoveryFixture::start(0).await;
    let client = Client::tcp(&peer.endpoint, config()).unwrap();
    let q = MinMaxRequest::new(0, 7).unwrap();
    let r = client.read_min_max(q).await.unwrap();
    assert!(matches!(r.today.minimum_time, OccurrenceTime::Raw(_)));
    assert_eq!(r.current, 12.5);
    let calendar = client
        .read_min_max_with_codec(q, MinMaxCodec(CalendarOccurrenceCodec))
        .await
        .unwrap();
    assert!(matches!(
        calendar.today.minimum_time,
        OccurrenceTime::Calendar { month: 5, .. }
    ));
    let epoch = client
        .read_min_max_with_codec(q, MinMaxCodec(PrefixedEpochOccurrenceCodec))
        .await
        .unwrap();
    assert!(matches!(
        epoch.today.minimum_time,
        OccurrenceTime::PrefixedEpoch {
            seconds: 0x05040302,
            ..
        }
    ));
    let custom = client
        .read_min_max_with_codec(q, MinMaxCodec(CustomTime))
        .await
        .unwrap();
    assert_eq!(custom.today.minimum_time, [5, 4, 3, 2, 1]);
    // Baseline maximum count, plus explicitly caller-asserted expanded applicability.
    let max_tags = client
        .read_history_tags(HistoryTagsRequest::new(10, (0..20).collect()).unwrap())
        .await
        .unwrap();
    assert_eq!(
        max_tags.tags.iter().map(|t| t.point).collect::<Vec<_>>(),
        (0..20).collect::<Vec<_>>()
    );
    assert_eq!(
        client
            .read_min_max(MinMaxRequest::with_segment_limit(13, 255, 13).unwrap())
            .await
            .unwrap()
            .request
            .segment(),
        13
    );
    assert_eq!(
        client
            .read_history_tags(HistoryTagsRequest::with_limits(13, vec![255], 13, 255).unwrap())
            .await
            .unwrap()
            .tags[0]
            .point,
        255
    );
    for index in [u16::MAX, 0, 17] {
        peer.peer
            .history_periodic_index
            .store(usize::from(index), Ordering::SeqCst);
        let tags = client
            .read_history_tags(HistoryTagsRequest::new(0, vec![7, 2, 7]).unwrap())
            .await
            .unwrap();
        assert_eq!(tags.periodic_index, index);
        assert_eq!(
            tags.tags.iter().map(|t| t.point).collect::<Vec<_>>(),
            [7, 2, 7]
        );
    }
    assert!(
        peer.peer
            .requests
            .lock()
            .unwrap()
            .contains(&(105, vec![0, 7]))
    );
    assert!(
        peer.peer
            .requests
            .lock()
            .unwrap()
            .contains(&(108, vec![0, 3, 7, 2, 7]))
    );
    client.stop().await;
    let client = Client::tcp(
        &peer.endpoint,
        ClientConfig {
            profile: Arc::new(CalendarProfile),
            ..config()
        },
    )
    .unwrap();
    assert!(matches!(
        client.read_min_max(q).await.unwrap().today.minimum_time,
        OccurrenceTime::Calendar { .. }
    ));
    assert_eq!(
        client
            .read_history_tags(HistoryTagsRequest::new(0, vec![7]).unwrap())
            .await
            .unwrap()
            .tags
            .len(),
        1
    );
    client.stop().await;
}
struct Limited {
    request: usize,
    response: usize,
    unavailable: bool,
}
impl ProtocolProfile for Limited {
    fn metadata(&self) -> ProfileMetadata {
        DefaultProfile.metadata()
    }
    fn capability(&self, op: u8) -> Capability {
        if self.unavailable && matches!(op, 105 | 108) {
            Capability::Unavailable
        } else {
            DefaultProfile.capability(op)
        }
    }
    fn limits(&self, op: u8) -> PayloadLimits {
        if matches!(op, 105 | 108) {
            PayloadLimits {
                request: self.request,
                response: self.response,
            }
        } else {
            PayloadLimits::default()
        }
    }
}
// POL-HISTORY-BUDGET: known Table 2-14/2-15 response budgets fail before admission.
#[tokio::test]
async fn impossible_request_reply_budgets_and_capabilities_fail_before_connect() {
    for (opcode, req, resp, unavailable) in [
        (105, 1, 255, false),
        (105, 255, 49, false),
        (108, 2, 255, false),
        (108, 255, 14, false),
        (105, 255, 255, true),
        (108, 255, 255, true),
    ] {
        let peer = DiscoveryFixture::start(0).await;
        let client = Client::tcp(
            &peer.endpoint,
            ClientConfig {
                profile: Arc::new(Limited {
                    request: req,
                    response: resp,
                    unavailable,
                }),
                ..config()
            },
        )
        .unwrap();
        let result = if opcode == 105 {
            client
                .read_min_max(MinMaxRequest::new(0, 7).unwrap())
                .await
                .map(|_| ())
        } else {
            client
                .read_history_tags(HistoryTagsRequest::new(0, vec![7]).unwrap())
                .await
                .map(|_| ())
        };
        assert!(matches!(
            result,
            Err(etherbird::Error::Operation(
                Error::InvalidRequest(_) | Error::CapabilityUnavailable(_)
            ))
        ));
        assert_eq!(peer.peer.history_reads.load(Ordering::SeqCst), 0);
        assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 0);
        client.stop().await;
    }
}

// POL-HISTORY-RETIREMENT: malformed echoes/lengths/counts retire the generation;
// valid device errors stay healthy; no failed metadata request is replayed.
#[tokio::test]
async fn malformed_metadata_and_valid_device_rejections_have_distinct_lifecycle_outcomes() {
    for opcode in [105, 108] {
        let peer = DiscoveryFixture::start(0).await;
        let client = Client::tcp(&peer.endpoint, config()).unwrap();
        client.connected().await.unwrap();
        let mut connections = 1;
        let faults = if opcode == 105 {
            vec![3, 1, 2, 6, 7, 4]
        } else {
            vec![3, 1, 2, 6, 7, 9, 4]
        };
        for fault in faults {
            let reads = peer.peer.history_reads.load(Ordering::SeqCst);
            peer.peer.fault_opcode.store(opcode, Ordering::SeqCst);
            peer.peer.fault.store(fault, Ordering::SeqCst);
            let result = if opcode == 105 {
                client
                    .read_min_max(MinMaxRequest::new(0, 7).unwrap())
                    .await
                    .map(|_| ())
            } else {
                client
                    .read_history_tags(HistoryTagsRequest::new(0, vec![7]).unwrap())
                    .await
                    .map(|_| ())
            };
            assert!(result.is_err());
            if fault == 3 {
                assert!(matches!(
                    result,
                    Err(etherbird::Error::Operation(Error::Device(_)))
                ));
            } else {
                connections += 1;
            }
            client.read_clock().await.unwrap();
            assert_eq!(peer.peer.connections.load(Ordering::SeqCst), connections);
            assert_eq!(peer.peer.history_reads.load(Ordering::SeqCst), reads + 1);
        }
        client.stop().await;
    }
}

// POL-HISTORY-CANCEL: deadlines/cancellation retire active exchanges and cannot
// finish late or replay an old metadata read after replacement.
#[tokio::test]
async fn interrupted_metadata_requests_return_once_and_recover_for_explicit_calls() {
    for opcode in [105, 108] {
        for cancel in [false, true] {
            let peer = DiscoveryFixture::start(0).await;
            let client = Client::tcp(&peer.endpoint, config()).unwrap();
            client.connected().await.unwrap();
            peer.peer.fault_opcode.store(opcode, Ordering::SeqCst);
            peer.peer.fault.store(5, Ordering::SeqCst);
            let caller = client.clone();
            let task = tokio::spawn(async move {
                if opcode == 105 {
                    caller
                        .read_min_max(MinMaxRequest::new(0, 7).unwrap())
                        .await
                        .map(|_| ())
                } else {
                    caller
                        .read_history_tags(HistoryTagsRequest::new(0, vec![7]).unwrap())
                        .await
                        .map(|_| ())
                }
            });
            tokio::time::timeout(Duration::from_secs(1), async {
                while peer.peer.history_reads.load(Ordering::SeqCst) == 0 {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
            if cancel {
                task.abort();
                assert!(task.await.unwrap_err().is_cancelled());
            } else {
                assert!(task.await.unwrap().is_err());
            }
            client.read_clock().await.unwrap();
            assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
            assert_eq!(peer.peer.history_reads.load(Ordering::SeqCst), 1);
            client.stop().await;
        }
    }
}

// POL-HISTORY-OWNERSHIP: the same public decoders operate in standalone sessions
// and exclusive workflows, including alternate-codec selection under the guard.
#[tokio::test]
async fn standalone_and_workflow_interfaces_use_the_same_typed_contract() {
    let peer = DiscoveryFixture::start(0).await;
    let stream = tokio::net::TcpStream::connect(&peer.endpoint)
        .await
        .unwrap();
    let session = rocsteady::Session::new(
        Box::new(stream),
        rocsteady::Address::new(1, 0),
        rocsteady::Address::new(1, 2),
        Duration::from_secs(1),
        rocsteady::protocol::CrcMode::Required,
    )
    .unwrap();
    let q = MinMaxRequest::new(0, 7).unwrap();
    assert_eq!(session.read_min_max(q).await.unwrap().current, 12.5);
    assert!(matches!(
        session
            .read_min_max_with_codec(q, MinMaxCodec(CalendarOccurrenceCodec))
            .await
            .unwrap()
            .today
            .minimum_time,
        OccurrenceTime::Calendar { .. }
    ));
    assert_eq!(
        session
            .read_history_tags(HistoryTagsRequest::new(0, vec![7]).unwrap())
            .await
            .unwrap()
            .tags
            .len(),
        1
    );
    let client = Client::tcp(&peer.endpoint, config()).unwrap();
    struct ReadBoth;
    #[etherbird::async_trait]
    impl rocsteady::Workflow for ReadBoth {
        type Output = u64;
        async fn run(self, c: &mut rocsteady::ManagedContext<'_>) -> Result<u64, Error> {
            let g = c.generation();
            let q = MinMaxRequest::new(0, 7)?;
            assert_eq!(c.read_min_max(q).await?.current, 12.5);
            assert_eq!(
                c.read_min_max_with_codec(q, MinMaxCodec(CustomTime))
                    .await?
                    .today
                    .minimum_time,
                [5, 4, 3, 2, 1]
            );
            assert_eq!(
                c.read_history_tags(HistoryTagsRequest::new(0, vec![7])?)
                    .await?
                    .tags
                    .len(),
                1
            );
            assert_eq!(g, c.generation());
            Ok(g)
        }
    }
    assert!(client.workflow(ReadBoth).await.unwrap() > 0);
    client.stop().await;
}

// POL-105-DECODE-GUARD: a consumer decoder failure is a malformed typed reply,
// even when the default raw codec could decode the same payload successfully.
#[tokio::test]
async fn custom_occurrence_decoder_failure_retires_before_the_next_call() {
    struct Reject;
    impl ReplyCodec<()> for Reject {
        type Output = ();
        fn decode(&self, _: &(), _: &[u8]) -> Result<(), DecodeError> {
            Err(DecodeError("fixture interpretation rejected"))
        }
    }
    let peer = DiscoveryFixture::start(0).await;
    let client = Client::tcp(&peer.endpoint, config()).unwrap();
    assert!(matches!(
        client
            .read_min_max_with_codec(MinMaxRequest::new(0, 7).unwrap(), MinMaxCodec(Reject))
            .await,
        Err(etherbird::Error::Operation(Error::InvalidResponse(_)))
    ));
    client.read_clock().await.unwrap();
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
    assert_eq!(peer.peer.history_reads.load(Ordering::SeqCst), 1);
    client.stop().await;
}
