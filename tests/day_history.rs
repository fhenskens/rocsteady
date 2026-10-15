include!("../examples/support/discovery_fixture.rs");
use rocsteady::{
    Client, ClientConfig, ConfiguredHistoryPolicy, DayHistoryCodec, DayHistoryReply,
    DayHistoryRequest, DayHistoryTarget, DayIndicesCodec, DayIndicesRequest, DecodeError,
    DefaultProfile, Error, HistoryDay, HistorySamples, IndexedDayHistoryRequest, PayloadLimits,
    ProfileMetadata, ProtocolProfile, ReplyCodec,
};
use std::time::Duration;
fn date() -> HistoryDay {
    HistoryDay::new(29, 2).unwrap()
}
fn iq() -> DayIndicesRequest {
    DayIndicesRequest::new(0, date()).unwrap()
}
fn q(target: DayHistoryTarget) -> DayHistoryRequest {
    DayHistoryRequest::new(0, date(), target).unwrap()
}
fn config() -> ClientConfig {
    ClientConfig {
        heartbeat_interval: None,
        response_timeout: Duration::from_millis(150),
        call_timeout: Duration::from_secs(3),
        recovery: etherbird::Config {
            retry_delay: Duration::from_millis(1),
            ..Default::default()
        },
        ..Default::default()
    }
}
struct Budget {
    opcode: u8,
    request: usize,
    response: usize,
}
impl ProtocolProfile for Budget {
    fn metadata(&self) -> ProfileMetadata {
        DefaultProfile.metadata()
    }
    fn capability(&self, op: u8) -> rocsteady::Capability {
        DefaultProfile.capability(op)
    }
    fn limits(&self, op: u8) -> PayloadLimits {
        if op == self.opcode {
            PayloadLimits {
                request: self.request,
                response: self.response,
            }
        } else {
            PayloadLimits::default()
        }
    }
}
fn client(peer: &DiscoveryFixture) -> Client {
    Client::tcp(
        &peer.endpoint,
        ClientConfig {
            profile: Arc::new(Budget {
                opcode: 135,
                request: 6,
                response: 13,
            }),
            ..config()
        },
    )
    .unwrap()
}
fn indexed(target: DayHistoryTarget) -> IndexedDayHistoryRequest {
    IndexedDayHistoryRequest::new(
        q(target),
        Arc::new(ConfiguredHistoryPolicy::new(10, 200, 60, 5, 3).unwrap()),
    )
}
fn calls(peer: &DiscoveryFixture, opcode: u8) -> Vec<Vec<u8>> {
    peer.peer
        .requests
        .lock()
        .unwrap()
        .iter()
        .filter(|(op, _)| *op == opcode)
        .map(|(_, b)| b.clone())
        .collect()
}
// REQ-137-REQUEST / REQ-138-REQUEST / POL-DAY-CALENDAR: Tables 2-20/21, pp.2-34–35.
// Exact field order, reserved point 255 and explicit applicability; no invented year/OP-137 point.
#[test]
fn request_layouts_and_yearless_calendar_boundaries() {
    assert_eq!(iq().payload(), [0, 29, 2]);
    assert_eq!(q(DayHistoryTarget::Values(254)).payload(), [0, 254, 29, 2]);
    assert_eq!(q(DayHistoryTarget::Timestamps).payload(), [0, 255, 29, 2]);
    assert!(DayHistoryRequest::new(0, date(), DayHistoryTarget::Values(255)).is_err());
    assert!(DayIndicesRequest::new(11, date()).is_err());
    assert!(DayHistoryRequest::new(11, date(), DayHistoryTarget::Values(0)).is_err());
    assert!(DayIndicesRequest::with_segment_limit(13, date(), 13).is_ok());
    assert!(
        DayHistoryRequest::with_segment_limit(13, date(), DayHistoryTarget::Timestamps, 13).is_ok()
    );
    assert!(DayIndicesRequest::with_segment_limit(14, date(), 13).is_err());
    for month in 0..=255u8 {
        for day in 0..=255u8 {
            let max = match month {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                4 | 6 | 9 | 11 => 30,
                2 => 29,
                _ => 0,
            };
            assert_eq!(HistoryDay::new(day, month).is_ok(), day > 0 && day <= max);
        }
    }
}
// REQ-137-FIELDS / REQ-137-DAILY: Table 2-20 exact little-endian fields, zero daily validity.
#[test]
fn indices_keep_raw_invalid_daily_index_and_every_uint16_field() {
    let b = [0, 0x34, 0x12, 0xff, 0xff, 0xcd, 0xab, 2, 0];
    let r = DayIndicesCodec.decode(&iq(), &b).unwrap();
    assert_eq!(
        (
            r.periodic_start,
            r.periodic_count,
            r.daily_start,
            r.daily_count
        ),
        (0x1234, 65535, Some(0xabcd), 2)
    );
    assert_eq!(r.raw, b);
    let r = DayIndicesCodec
        .decode(&iq(), &[0, 0xff, 0xff, 0, 0, 0xff, 0xff, 0, 0])
        .unwrap();
    assert_eq!(r.daily_start, None);
    assert_eq!(r.raw_daily_start, 65535);
    assert_eq!(r.periodic_count, 0);
    let r = DayIndicesCodec
        .decode(&iq(), &[0, 0, 0, 0, 0, 0, 0, 0xff, 0xff])
        .unwrap();
    assert_eq!(r.daily_start, Some(0));
    assert_eq!(r.daily_count, 65535);
}
// REQ-137-VALIDATE / REQ-138-VALIDATE: all truncations, trailing bytes, echoes and both count widths.
#[test]
fn codecs_reject_truncation_wrong_echoes_counts_and_oversized_frames() {
    let b = day_indices_literal(0);
    for len in 0..b.len() {
        assert!(DayIndicesCodec.decode(&iq(), &b[..len]).is_err());
    }
    let mut bad = b.clone();
    bad.push(0);
    assert!(DayIndicesCodec.decode(&iq(), &bad).is_err());
    bad = b;
    bad[0] = 1;
    assert!(DayIndicesCodec.decode(&iq(), &bad).is_err());
    for target in [DayHistoryTarget::Values(7), DayHistoryTarget::Timestamps] {
        let q = q(target);
        let b = day_history_literal(0, target.as_byte(), 29, 2);
        for len in 0..b.len() {
            assert!(DayHistoryCodec.decode(&q, &b[..len]).is_err());
        }
        let mut bad = b.clone();
        bad.push(0);
        assert!(DayHistoryCodec.decode(&q, &bad).is_err());
        for i in 0..8 {
            let mut bad = b.clone();
            bad[i] ^= 1;
            assert!(DayHistoryCodec.decode(&q, &bad).is_err(), "byte {i}");
        }
        let mut full = q.payload();
        full.extend([61, 0, 0, 0]);
        full.resize(252, 0);
        assert_eq!(
            DayHistoryCodec.decode(&q, &full).unwrap().periodic.len(),
            61
        );
        full[4] = 62;
        full.resize(256, 0);
        assert!(DayHistoryCodec.decode(&q, &full).is_err());
    }
}
// REQ-138-VALUES / REQ-138-TIMESTAMPS: Table 2-21 periodic words precede all daily words.
#[test]
fn float_and_time_words_are_lossless_and_never_coerced() {
    let r = DayHistoryCodec
        .decode(
            &q(DayHistoryTarget::Values(7)),
            &day_history_literal(0, 7, 29, 2),
        )
        .unwrap();
    let HistorySamples::Values(p) = r.periodic else {
        panic!()
    };
    let HistorySamples::Values(d) = r.daily else {
        panic!()
    };
    assert_eq!(
        p.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        [0x3f800000, 0xc0000000, 0x7fc01234]
    );
    assert_eq!(
        d.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        [0x80000000, 0x7f800000]
    );
    let r = DayHistoryCodec
        .decode(
            &q(DayHistoryTarget::Timestamps),
            &day_history_literal(0, 255, 29, 2),
        )
        .unwrap();
    assert_eq!(
        r.periodic,
        HistorySamples::Timestamps(vec![0x12345678, u32::MAX, 0])
    );
    assert_eq!(r.daily, HistorySamples::Timestamps(vec![1, 0x80000000]));
    let r = DayHistoryCodec
        .decode(&q(DayHistoryTarget::Values(7)), &[0, 7, 29, 2, 0, 0, 0, 0])
        .unwrap();
    assert!(r.daily.is_empty() && r.periodic.is_empty());
}
// REQ-DAY-TCP: literal fields on independent TCP peer, including multiple/zero daily counts.
#[tokio::test]
async fn literal_peer_reads_indices_values_timestamps_and_empty_days() {
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer);
    assert_eq!(c.read_day_indices(iq()).await.unwrap().daily_count, 2);
    for target in [
        DayHistoryTarget::Values(0),
        DayHistoryTarget::Values(254),
        DayHistoryTarget::Timestamps,
    ] {
        assert_eq!(
            c.read_history_day(q(target)).await.unwrap().raw,
            day_history_literal(0, target.as_byte(), 29, 2)
        );
    }
    *peer.peer.day_reply.lock().unwrap() = Some(vec![0, 7, 29, 2, 0, 0, 0, 0]);
    assert!(
        c.read_history_day(q(DayHistoryTarget::Values(7)))
            .await
            .unwrap()
            .periodic
            .is_empty()
    );
    assert_eq!(calls(&peer, 137), [vec![0, 29, 2]]);
    assert_eq!(calls(&peer, 138)[2], [0, 255, 29, 2]);
    c.stop().await;
}
// POL-DAY-BUDGET: request and header minimums checked locally; actual oversized replies retire.
#[tokio::test]
async fn exact_profile_limits_and_impossible_budgets_are_enforced() {
    for (op, request, response) in [(137, 2, 9), (137, 3, 8), (138, 3, 252), (138, 4, 7)] {
        let peer = DiscoveryFixture::start(0).await;
        let c = Client::tcp(
            &peer.endpoint,
            ClientConfig {
                profile: Arc::new(Budget {
                    opcode: op,
                    request,
                    response,
                }),
                ..config()
            },
        )
        .unwrap();
        let r = if op == 137 {
            c.read_day_indices(iq()).await.map(|_| ())
        } else {
            c.read_history_day(q(DayHistoryTarget::Values(7)))
                .await
                .map(|_| ())
        };
        assert!(matches!(
            r,
            Err(etherbird::Error::Operation(Error::InvalidRequest(_)))
        ));
        assert!(calls(&peer, op).is_empty());
        c.stop().await;
    }
    let peer = DiscoveryFixture::start(0).await;
    let c = Client::tcp(
        &peer.endpoint,
        ClientConfig {
            profile: Arc::new(Budget {
                opcode: 138,
                request: 4,
                response: 252,
            }),
            ..config()
        },
    )
    .unwrap();
    let mut b = vec![0, 7, 29, 2, 60, 0, 1, 0];
    b.resize(252, 0);
    *peer.peer.day_reply.lock().unwrap() = Some(b);
    let r = c
        .read_history_day(q(DayHistoryTarget::Values(7)))
        .await
        .unwrap();
    assert_eq!((r.periodic.len(), r.daily.len()), (60, 1));
    c.stop().await;
    let c = Client::tcp(
        &peer.endpoint,
        ClientConfig {
            profile: Arc::new(Budget {
                opcode: 138,
                request: 4,
                response: 251,
            }),
            ..config()
        },
    )
    .unwrap();
    assert!(
        c.read_history_day(q(DayHistoryTarget::Values(7)))
            .await
            .is_err()
    );
    c.read_clock().await.unwrap();
    assert_eq!(calls(&peer, 138).len(), 2);
    c.stop().await;
}
// POL-DAY-LIFECYCLE: both services preserve valid missing-day errors, retire malformed/late exchanges.
#[tokio::test]
async fn missing_day_rejections_malformed_replies_and_disconnects_have_distinct_outcomes() {
    for op in [137, 138] {
        for fault in [1, 3, 4, 5, 10] {
            let peer = DiscoveryFixture::start(0).await;
            let c = client(&peer);
            peer.peer
                .day_fail_opcode
                .store(usize::from(op), Ordering::SeqCst);
            peer.peer.day_fail_at.store(1, Ordering::SeqCst);
            peer.peer.day_fault.store(fault, Ordering::SeqCst);
            let r = if op == 137 {
                c.read_day_indices(iq()).await.map(|_| ())
            } else {
                c.read_history_day(q(DayHistoryTarget::Values(7)))
                    .await
                    .map(|_| ())
            };
            assert!(r.is_err());
            c.read_clock().await.unwrap();
            peer.peer.release_late_sample.notify_one();
            assert_eq!(
                peer.peer.connections.load(Ordering::SeqCst),
                if fault == 3 { 1 } else { 2 }
            );
            assert_eq!(calls(&peer, op).len(), 1);
            c.stop().await;
        }
    }
}
// POL-DAY-INDEXED: explicit bounded fallback owns one generation, splits/wraps both columns,
// rechecks mapping and never issues OP-138. Multiple daily entries are all retained.
#[tokio::test]
async fn indexed_alternative_pages_both_columns_and_rechecks_mapping() {
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer);
    let p = c
        .read_history_day_indexed(indexed(DayHistoryTarget::Values(7)))
        .await
        .unwrap();
    assert!(p.complete);
    assert!(!p.mapping_changed);
    assert_eq!(p.periodic_remaining, Some(0));
    assert_eq!(p.daily_remaining, Some(0));
    assert_eq!(p.periodic_next_index, Some(2));
    assert_eq!(p.daily_next_index, Some(1));
    assert_eq!(
        p.periodic_pages
            .iter()
            .map(|p| (p.request.start(), p.samples.len()))
            .collect::<Vec<_>>(),
        [(4, 1), (0, 2)]
    );
    assert_eq!(
        p.daily_pages
            .iter()
            .map(|p| (p.request.start(), p.samples.len()))
            .collect::<Vec<_>>(),
        [(2, 1), (0, 1)]
    );
    assert!(p.indices_after.is_some());
    assert!(calls(&peer, 138).is_empty());
    assert_eq!(calls(&peer, 137).len(), 2);
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
    c.stop().await;
}
// POL-DAY-TIMESTAMP-POINT: sentinel 255 is never silently reused by OP-135.
#[tokio::test]
async fn indexed_timestamps_require_explicit_point_and_use_selectors_three_and_four() {
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer);
    assert!(
        c.read_history_day_indexed(indexed(DayHistoryTarget::Timestamps))
            .await
            .is_err()
    );
    assert!(calls(&peer, 137).is_empty());
    let p = c
        .read_history_day_indexed(indexed(DayHistoryTarget::Timestamps).with_timestamp_point(7))
        .await
        .unwrap();
    assert!(p.complete);
    assert_eq!(
        calls(&peer, 135)
            .iter()
            .map(|b| (b[1], b[2]))
            .collect::<Vec<_>>(),
        [(7, 3), (7, 3), (7, 4), (7, 4)]
    );
    assert!(
        p.periodic_pages
            .iter()
            .chain(&p.daily_pages)
            .all(|p| matches!(p.samples, HistorySamples::Timestamps(_)))
    );
    c.stop().await;
}
// POL-DAY-ZERO: no daily request for invalid/zero daily index; no data calls for an empty mapping.
#[tokio::test]
async fn empty_columns_skip_invalid_indices_and_mapping_changes_are_reported() {
    for empty in [false, true] {
        let peer = DiscoveryFixture::start(0).await;
        let c = client(&peer);
        let bytes = vec![0, 4, 0, if empty { 0 } else { 3 }, 0, 255, 255, 0, 0];
        peer.peer
            .day_index_replies
            .lock()
            .unwrap()
            .extend([(1, bytes.clone()), (2, bytes)]);
        let p = c
            .read_history_day_indexed(indexed(DayHistoryTarget::Values(7)))
            .await
            .unwrap();
        assert!(p.complete);
        assert!(p.daily_pages.is_empty());
        assert_eq!(p.daily_next_index, None);
        assert_eq!(calls(&peer, 135).len(), if empty { 0 } else { 2 });
        c.stop().await;
    }
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer);
    peer.peer
        .day_index_replies
        .lock()
        .unwrap()
        .insert(2, vec![0, 0, 0, 3, 0, 2, 0, 2, 0]);
    let e = c
        .read_history_day_indexed(indexed(DayHistoryTarget::Values(7)))
        .await
        .unwrap_err();
    assert!(matches!(
        e.error,
        etherbird::Error::Operation(Error::HistoryChanged(_))
    ));
    assert!(e.progress.mapping_changed && !e.progress.complete);
    assert_eq!(e.progress.periodic_remaining, Some(0));
    c.read_clock().await.unwrap();
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
    c.stop().await;
}
// POL-DAY-PROGRESS / POL-DAY-NO-REPLAY: preserve confirmed prefix and explicit remaining positions.
#[tokio::test]
async fn indexed_failure_retains_prefix_without_replaying_or_resuming() {
    for fault in [1, 3, 4, 5] {
        let peer = DiscoveryFixture::start(0).await;
        let c = client(&peer);
        peer.peer.sample_fail_at.store(3, Ordering::SeqCst);
        peer.peer.sample_fault.store(fault, Ordering::SeqCst);
        let e = c
            .read_history_day_indexed(indexed(DayHistoryTarget::Values(7)))
            .await
            .unwrap_err();
        assert_eq!(e.progress.periodic_pages.len(), 2);
        assert!(e.progress.daily_pages.is_empty());
        assert_eq!(e.progress.periodic_remaining, Some(0));
        assert_eq!(e.progress.daily_remaining, Some(2));
        assert_eq!(e.progress.daily_next_index, Some(2));
        assert!(e.progress.indices_after.is_none());
        assert_eq!(calls(&peer, 135).len(), 3);
        c.read_clock().await.unwrap();
        assert_eq!(calls(&peer, 135).len(), 3);
        let next = c
            .read_history_day_indexed(indexed(DayHistoryTarget::Values(7)))
            .await
            .unwrap();
        assert!(next.complete);
        if fault != 3 {
            assert_ne!(next.generation, e.progress.generation);
        }
        c.stop().await;
    }
}
// POL-DAY-PLAN: both geometries and budgets must validate before any sample request.
#[tokio::test]
async fn indexed_invalid_daily_geometry_is_local_and_short_pages_stop() {
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer);
    peer.peer
        .day_index_replies
        .lock()
        .unwrap()
        .insert(1, vec![0, 4, 0, 3, 0, 2, 0, 4, 0]);
    assert!(
        c.read_history_day_indexed(indexed(DayHistoryTarget::Values(7)))
            .await
            .is_err()
    );
    assert!(calls(&peer, 135).is_empty());
    peer.peer.day_index_replies.lock().unwrap().clear();
    peer.peer.sample_short_at.store(1, Ordering::SeqCst);
    peer.peer.sample_short_count.store(0, Ordering::SeqCst);
    let p = c
        .read_history_day_indexed(indexed(DayHistoryTarget::Values(7)))
        .await
        .unwrap();
    assert!(p.short_page && !p.complete);
    assert_eq!(p.periodic_remaining, Some(3));
    assert_eq!(p.periodic_next_index, Some(4));
    assert!(p.daily_pages.is_empty());
    assert!(p.indices_after.is_none());
    assert_eq!(calls(&peer, 135).len(), 1);
    c.stop().await;
}
// POL-DAY-CANCELLATION: deadlines/abort/awaited stop never replay the pending indexed page.
#[tokio::test]
async fn indexed_deadline_cancellation_and_stop_retire_pending_work() {
    for mode in 0..3 {
        let peer = DiscoveryFixture::start(0).await;
        let c = Client::tcp(
            &peer.endpoint,
            ClientConfig {
                profile: Arc::new(Budget {
                    opcode: 135,
                    request: 6,
                    response: 13,
                }),
                call_timeout: Duration::from_millis(250),
                response_timeout: Duration::from_secs(2),
                ..config()
            },
        )
        .unwrap();
        c.connected().await.unwrap();
        peer.peer.sample_fail_at.store(3, Ordering::SeqCst);
        peer.peer.sample_fault.store(5, Ordering::SeqCst);
        let caller = c.clone();
        let t = tokio::spawn(async move {
            caller
                .read_history_day_indexed(indexed(DayHistoryTarget::Values(7)))
                .await
        });
        tokio::time::timeout(Duration::from_secs(1), async {
            while peer.peer.sample_reads.load(Ordering::SeqCst) < 3 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        if mode == 1 {
            t.abort();
            assert!(t.await.unwrap_err().is_cancelled());
        } else {
            if mode == 2 {
                c.stop().await;
            }
            let e = t.await.unwrap().unwrap_err();
            assert_eq!(e.progress.periodic_pages.len(), 2);
            assert_eq!(e.progress.daily_remaining, Some(2));
            if mode == 0 {
                assert!(matches!(e.error, etherbird::Error::Timeout));
            }
        }
        if mode != 2 {
            c.read_clock().await.unwrap();
            assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
        }
        assert_eq!(calls(&peer, 135).len(), 3);
        c.stop().await;
    }
}
// POL-DAY-EXTENSION: inferred defaults use public ReplyCodec; caller interpretation is explicit.
struct LabelledDay {
    year: u16,
}
impl ReplyCodec<DayHistoryRequest> for LabelledDay {
    type Output = (u16, DayHistoryReply);
    fn decode(&self, q: &DayHistoryRequest, b: &[u8]) -> Result<Self::Output, DecodeError> {
        Ok((self.year, DayHistoryCodec.decode(q, b)?))
    }
}
struct InvalidProfile;
impl ProtocolProfile for InvalidProfile {
    fn metadata(&self) -> ProfileMetadata {
        DefaultProfile.metadata()
    }
    fn capability(&self, op: u8) -> rocsteady::Capability {
        DefaultProfile.capability(op)
    }
    fn decode_day_indices(
        &self,
        q: &DayIndicesRequest,
        b: &[u8],
    ) -> Result<rocsteady::DayIndices, DecodeError> {
        let mut r = DayIndicesCodec.decode(q, b)?;
        r.daily_start = None;
        Ok(r)
    }
    fn decode_history_day(
        &self,
        q: &DayHistoryRequest,
        b: &[u8],
    ) -> Result<DayHistoryReply, DecodeError> {
        let mut r = DayHistoryCodec.decode(q, b)?;
        r.daily = HistorySamples::Timestamps(vec![]);
        Ok(r)
    }
}
#[tokio::test]
async fn external_codec_and_profile_validation_share_session_and_workflow_boundaries() {
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer);
    let (year, reply) = c
        .read_history_day_with_codec(q(DayHistoryTarget::Values(7)), LabelledDay { year: 2024 })
        .await
        .unwrap();
    assert_eq!(year, 2024);
    assert_eq!(reply.periodic.len(), 3);
    struct Read;
    #[etherbird::async_trait]
    impl rocsteady::Workflow for Read {
        type Output = ();
        async fn run(self, c: &mut rocsteady::ManagedContext<'_>) -> Result<(), Error> {
            assert_eq!(c.read_day_indices(iq()).await?.daily_count, 2);
            assert_eq!(
                c.read_history_day(q(DayHistoryTarget::Values(7)))
                    .await?
                    .daily
                    .len(),
                2
            );
            assert_eq!(
                c.read_day_indices_with_codec(iq(), DayIndicesCodec)
                    .await?
                    .periodic_count,
                3
            );
            assert_eq!(
                c.read_history_day_with_codec(
                    q(DayHistoryTarget::Timestamps),
                    LabelledDay { year: 2024 }
                )
                .await?
                .0,
                2024
            );
            Ok(())
        }
    }
    c.workflow(Read).await.unwrap();
    c.stop().await;
    let stream = tokio::net::TcpStream::connect(&peer.endpoint)
        .await
        .unwrap();
    let s = rocsteady::Session::new(
        Box::new(stream),
        rocsteady::Address::new(1, 0),
        rocsteady::Address::new(1, 2),
        Duration::from_secs(1),
        rocsteady::protocol::CrcMode::Required,
    )
    .unwrap();
    assert_eq!(s.read_day_indices(iq()).await.unwrap().daily_count, 2);
    assert_eq!(
        s.read_day_indices_with_codec(iq(), DayIndicesCodec)
            .await
            .unwrap()
            .periodic_count,
        3
    );
    assert_eq!(
        s.read_history_day(q(DayHistoryTarget::Values(7)))
            .await
            .unwrap()
            .daily
            .len(),
        2
    );
    assert_eq!(
        s.read_history_day_with_codec(q(DayHistoryTarget::Values(7)), LabelledDay { year: 2024 })
            .await
            .unwrap()
            .0,
        2024
    );
    for op in [137, 138] {
        let c = Client::tcp(
            &peer.endpoint,
            ClientConfig {
                profile: Arc::new(InvalidProfile),
                ..config()
            },
        )
        .unwrap();
        c.connected().await.unwrap();
        let before = peer.peer.connections.load(Ordering::SeqCst);
        let result = if op == 137 {
            c.read_day_indices(iq()).await.map(|_| ())
        } else {
            c.read_history_day(q(DayHistoryTarget::Values(7)))
                .await
                .map(|_| ())
        };
        assert!(result.is_err());
        c.read_clock().await.unwrap();
        assert_eq!(peer.peer.connections.load(Ordering::SeqCst), before + 1);
        c.stop().await;
    }
}
// POL-DAY-INDEX-MOVEMENT: each column has its own cursor; strict versus observe is explicit.
#[tokio::test]
async fn indexed_cursor_movement_is_explicit_and_postcheck_failures_keep_all_samples() {
    for observe in [false, true] {
        let peer = DiscoveryFixture::start(0).await;
        let c = client(&peer);
        peer.peer.sample_change_at.store(2, Ordering::SeqCst);
        peer.peer.sample_change_index.store(1, Ordering::SeqCst);
        let mut plan = indexed(DayHistoryTarget::Values(7));
        if observe {
            plan = plan.with_continuity(rocsteady::HistoryContinuity::ObserveIndexChanges);
        }
        let r = c.read_history_day_indexed(plan).await;
        let p = if observe {
            r.unwrap()
        } else {
            let e = r.unwrap_err();
            assert!(matches!(
                e.error,
                etherbird::Error::Operation(Error::HistoryChanged(_))
            ));
            *e.progress
        };
        assert!(p.index_changed);
        assert_eq!(p.complete, observe);
        assert_eq!(p.periodic_pages.len(), 2);
        assert_eq!(p.daily_pages.len(), if observe { 2 } else { 0 });
        c.read_clock().await.unwrap();
        assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
        c.stop().await;
    }
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer);
    peer.peer.day_fail_opcode.store(137, Ordering::SeqCst);
    peer.peer.day_fail_at.store(2, Ordering::SeqCst);
    peer.peer.day_fault.store(4, Ordering::SeqCst);
    let e = c
        .read_history_day_indexed(indexed(DayHistoryTarget::Values(7)))
        .await
        .unwrap_err();
    assert_eq!(e.progress.periodic_remaining, Some(0));
    assert_eq!(e.progress.daily_remaining, Some(0));
    assert_eq!(e.progress.daily_pages.len(), 2);
    assert!(!e.progress.complete);
    assert!(e.progress.indices_after.is_none());
    assert_eq!(calls(&peer, 135).len(), 4);
    c.read_clock().await.unwrap();
    assert_eq!(calls(&peer, 135).len(), 4);
    c.stop().await;
}
// POL-DAY-UNWRAPPED: unknown capacity never silently wraps, and all-zero columns
// tolerate changes in an explicitly invalid raw daily index.
#[tokio::test]
async fn unknown_geometry_stays_unwrapped_and_invalid_daily_index_is_not_a_mapping() {
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer);
    peer.peer.day_index_replies.lock().unwrap().extend([
        (1, vec![0, 255, 255, 1, 0, 255, 255, 0, 0]),
        (2, vec![0, 255, 255, 1, 0, 0, 0, 0, 0]),
    ]);
    let p = c
        .read_history_day_indexed(IndexedDayHistoryRequest::new(
            q(DayHistoryTarget::Values(7)),
            Arc::new(rocsteady::TableHistoryPolicy),
        ))
        .await
        .unwrap();
    assert!(p.complete);
    assert!(!p.mapping_changed);
    assert_eq!(p.periodic_next_index, None);
    assert_eq!(calls(&peer, 135), [vec![0, 7, 1, 255, 255, 1]]);
    peer.peer
        .day_index_replies
        .lock()
        .unwrap()
        .insert(3, vec![0, 255, 255, 2, 0, 0, 0, 0, 0]);
    assert!(
        c.read_history_day_indexed(IndexedDayHistoryRequest::new(
            q(DayHistoryTarget::Values(7)),
            Arc::new(rocsteady::TableHistoryPolicy)
        ))
        .await
        .is_err()
    );
    assert_eq!(calls(&peer, 135).len(), 1);
    c.stop().await;
}
