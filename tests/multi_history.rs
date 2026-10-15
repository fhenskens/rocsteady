include!("../examples/support/discovery_fixture.rs");
use rocsteady::{
    Client, ClientConfig, ConfiguredHistoryPolicy, DecodeError, DefaultProfile, Error,
    HistoryContinuity, HistoryKind, MultiHistoryCodec, MultiHistoryPage, MultiHistoryRange,
    MultiHistoryRequest, PayloadLimits, ProfileMetadata, ProseMinuteHistoryPolicy, ProtocolProfile,
    ReplyCodec,
};
use std::time::Duration;
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
    request: usize,
    response: usize,
}
impl ProtocolProfile for Budget {
    fn capability(&self, op: u8) -> rocsteady::Capability {
        DefaultProfile.capability(op)
    }
    fn metadata(&self) -> ProfileMetadata {
        DefaultProfile.metadata()
    }
    fn limits(&self, op: u8) -> PayloadLimits {
        if op == 136 {
            PayloadLimits {
                request: self.request,
                response: self.response,
            }
        } else {
            PayloadLimits::default()
        }
    }
}
fn client(peer: &DiscoveryFixture, response: usize) -> Client {
    Client::tcp(
        &peer.endpoint,
        ClientConfig {
            profile: Arc::new(Budget {
                request: 7,
                response,
            }),
            ..config()
        },
    )
    .unwrap()
}
fn q(periods: u8) -> MultiHistoryRequest {
    MultiHistoryRequest::new(0, 0, HistoryKind::Periodic, 7, 2, periods).unwrap()
}
fn range(total: u32) -> MultiHistoryRange {
    MultiHistoryRange::new(0, 0, HistoryKind::Periodic, 7, 2, total).unwrap()
}
fn requests(peer: &DiscoveryFixture) -> Vec<Vec<u8>> {
    peer.peer
        .requests
        .lock()
        .unwrap()
        .iter()
        .filter(|(op, _)| *op == 136)
        .map(|(_, b)| b.clone())
        .collect()
}
// REQ-136-REQUEST / POL-136-BOUNDS: §2.16/Table 2-19 pp.2-32–33.
// Every selector; UINT16 start, segment/point wire boundaries, product includes TIME.
#[test]
fn request_fields_selectors_products_and_applicability() {
    for kind in [
        HistoryKind::Minute,
        HistoryKind::Periodic,
        HistoryKind::Daily,
    ] {
        let start = if kind == HistoryKind::Minute {
            59
        } else {
            0x1234
        };
        let request = MultiHistoryRequest::new(10, start, kind, 254, 2, 1).unwrap();
        let [lo, hi] = start.to_le_bytes();
        assert_eq!(request.payload(), [10, lo, hi, kind.as_byte(), 254, 2, 1]);
    }
    for kind in [
        HistoryKind::PeriodicTimestamps,
        HistoryKind::DailyTimestamps,
    ] {
        assert!(MultiHistoryRequest::new(0, 0, kind, 0, 1, 1).is_err());
    }
    assert!(MultiHistoryRequest::new(11, 0, HistoryKind::Daily, 0, 1, 1).is_err());
    assert!(MultiHistoryRequest::new(0, 60, HistoryKind::Minute, 0, 1, 0).is_err());
    assert!(
        MultiHistoryRequest::with_policy(
            0,
            60,
            HistoryKind::Minute,
            0,
            1,
            1,
            &ProseMinuteHistoryPolicy
        )
        .is_ok()
    );
    assert!(MultiHistoryRequest::new(0, 0, HistoryKind::Daily, 255, 2, 0).is_err());
    for points in 0..=255u8 {
        for periods in 0..=61u8 {
            assert_eq!(
                MultiHistoryRequest::new(0, 0, HistoryKind::Periodic, 0, points, periods).is_ok(),
                (usize::from(points) + 1) * usize::from(periods) <= 60
            );
        }
    }
    let policy = ConfiguredHistoryPolicy::new(13, 200, 60, 5, 3).unwrap();
    assert!(
        MultiHistoryRequest::with_policy(13, 4, HistoryKind::Periodic, 198, 2, 1, &policy).is_ok()
    );
    assert!(
        MultiHistoryRequest::with_policy(13, 4, HistoryKind::Periodic, 199, 2, 0, &policy).is_err()
    );
    assert!(
        MultiHistoryRequest::with_policy(13, 4, HistoryKind::Periodic, 198, 2, 2, &policy).is_err()
    );
    assert!(
        MultiHistoryRange::with_policy(13, 4, HistoryKind::Periodic, 198, 2, 6, &policy).is_err()
    );
    assert!(MultiHistoryRange::new(0, 65535, HistoryKind::Daily, 0, 1, 2).is_err());
}
// REQ-136-ORDER / REQ-136-RAW: Table 2-19 literal two-period/two-point response.
// TIME is not FLOAT; signed zero, NaN payload, infinity and ordinary floats survive.
#[test]
fn literal_time_major_rows_preserve_timestamp_and_float_bits() {
    let request = MultiHistoryRequest::new(10, 0x1234, HistoryKind::Periodic, 7, 2, 2).unwrap();
    let bytes = [
        10, 0x34, 0x12, 0xff, 0xff, 6, 0x78, 0x56, 0x34, 0x12, 0, 0, 0x80, 0x3f, 0, 0, 0, 0x80,
        0xff, 0xff, 0xff, 0xff, 0x34, 0x12, 0xc0, 0x7f, 0, 0, 0x80, 0x7f,
    ];
    let page = MultiHistoryCodec.decode(&request, &bytes).unwrap();
    assert_eq!(page.raw, bytes);
    assert_eq!(page.current_index, 65535);
    assert_eq!(
        page.periods.iter().map(|r| r.timestamp).collect::<Vec<_>>(),
        [0x12345678, u32::MAX]
    );
    assert_eq!(
        page.periods
            .iter()
            .flat_map(|r| r.values.iter().map(|v| v.to_bits()))
            .collect::<Vec<_>>(),
        [0x3f800000, 0x80000000, 0x7fc01234, 0x7f800000]
    );
}
// REQ-136-VALIDATE: exact segment/index echoes, whole-period counts and all byte boundaries.
#[test]
fn malformed_echoes_counts_truncations_and_trailing_bytes_are_rejected() {
    let request = q(2);
    let b = multi_history_literal(0, 0, 7, 2, 0, 2);
    for len in 0..b.len() {
        assert!(
            MultiHistoryCodec.decode(&request, &b[..len]).is_err(),
            "length {len}"
        );
    }
    let mut extra = b.clone();
    extra.push(0);
    assert!(MultiHistoryCodec.decode(&request, &extra).is_err());
    for index in [0, 1, 2, 5] {
        let mut wrong = b.clone();
        wrong[index] ^= 1;
        assert!(MultiHistoryCodec.decode(&request, &wrong).is_err());
    }
    for elements in [1, 2, 4, 5, 7, 60, 255] {
        let mut wrong = b.clone();
        wrong[5] = elements;
        wrong.resize(6 + 4 * usize::from(elements), 0);
        assert!(MultiHistoryCodec.decode(&request, &wrong).is_err());
    }
    let minute = MultiHistoryRequest::new(0, 0, HistoryKind::Minute, 7, 2, 1).unwrap();
    assert!(
        MultiHistoryCodec
            .decode(&minute, &multi_history_literal(0, 0, 7, 2, 60, 1))
            .is_err()
    );
    assert!(
        MultiHistoryCodec
            .decode(&q(0), &multi_history_literal(0, 0, 7, 2, 0, 1))
            .is_err()
    );
    assert!(
        MultiHistoryCodec
            .decode(&q(2), &[0, 0, 0, 0, 0, 0])
            .unwrap()
            .periods
            .is_empty()
    );
}
// REQ-136-TCP / REQ-136-ZERO: max 60 elements, all selectors, timestamps-only,
// index-only and zero-element invalid-request shape over independent loopback TCP.
#[tokio::test]
async fn literal_peer_covers_all_selectors_and_maximum_payload() {
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer, 246);
    for kind in [
        HistoryKind::Minute,
        HistoryKind::Periodic,
        HistoryKind::Daily,
    ] {
        for (points, periods) in [(2, 20), (59, 1), (0, 60), (255, 0)] {
            let request = MultiHistoryRequest::new(0, 0, kind, 0, points, periods).unwrap();
            let page = c.read_multi_history(request).await.unwrap();
            assert_eq!(
                page.raw.len(),
                6 + 4 * (usize::from(points) + 1) * usize::from(periods)
            );
            assert_eq!(page.periods.len(), usize::from(periods));
            if periods > 0 {
                assert_eq!(page.periods[0].values.len(), usize::from(points));
            }
        }
    }
    assert_eq!(requests(&peer).len(), 12);
    assert_eq!(requests(&peer)[0], [0, 0, 0, 0, 0, 2, 20]);
    c.stop().await;
}
// POL-136-PAGING: profile limits split whole periods, known rings wrap explicitly.
#[tokio::test]
async fn bounded_ranges_split_periods_and_known_ring_boundaries() {
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer, 30); // two 3-word periods
    let result = c.read_multi_history_range(range(5)).await.unwrap();
    assert!(result.complete);
    assert_eq!(result.remaining, 0);
    assert_eq!(result.next_index, Some(5));
    assert_eq!(
        result
            .pages
            .iter()
            .map(|p| (p.request.start(), p.periods.len()))
            .collect::<Vec<_>>(),
        [(0, 2), (2, 2), (4, 1)]
    );
    assert_eq!(
        requests(&peer),
        [
            vec![0, 0, 0, 1, 7, 2, 2],
            vec![0, 2, 0, 1, 7, 2, 2],
            vec![0, 4, 0, 1, 7, 2, 1]
        ]
    );
    let policy = ConfiguredHistoryPolicy::new(10, 200, 60, 5, 3).unwrap();
    let r = MultiHistoryRange::with_policy(0, 4, HistoryKind::Periodic, 7, 2, 5, &policy).unwrap();
    let result = c.read_multi_history_range(r).await.unwrap();
    assert_eq!(
        result
            .pages
            .iter()
            .map(|p| (p.request.start(), p.periods.len()))
            .collect::<Vec<_>>(),
        [(4, 1), (0, 2), (2, 2)]
    );
    assert_eq!(result.next_index, Some(4));
    let zero = c.read_multi_history_range(range(0)).await.unwrap();
    assert!(zero.complete);
    assert_eq!(zero.pages.len(), 1);
    let edge = c
        .read_multi_history_range(
            MultiHistoryRange::new(0, 65535, HistoryKind::Daily, 7, 2, 1).unwrap(),
        )
        .await
        .unwrap();
    assert!(edge.complete);
    assert_eq!(edge.next_index, None);
    c.stop().await;
}
// POL-136-BUDGET: admission is avoided for impossible request/reply budgets;
// OP-136 uses its 246-byte max, not OP-135's 245-byte header shape.
#[tokio::test]
async fn impossible_budgets_fail_locally_and_exact_budgets_pass() {
    for (request, response, total) in [(6, 246, 1), (7, 5, 0), (7, 17, 1)] {
        let peer = DiscoveryFixture::start(0).await;
        let c = Client::tcp(
            &peer.endpoint,
            ClientConfig {
                profile: Arc::new(Budget { request, response }),
                ..config()
            },
        )
        .unwrap();
        assert!(c.read_multi_history_range(range(total)).await.is_err());
        assert!(requests(&peer).is_empty());
        c.stop().await;
    }
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer, 245);
    assert!(c.read_multi_history(q(20)).await.is_err());
    assert!(requests(&peer).is_empty());
    let result = c.read_multi_history_range(range(20)).await.unwrap();
    assert_eq!(
        result
            .pages
            .iter()
            .map(|p| p.periods.len())
            .collect::<Vec<_>>(),
        [19, 1]
    );
    c.stop().await;
}
// POL-136-SHORT: empty/short integral-period responses stop, with no guessed padding/retry.
#[tokio::test]
async fn short_or_zero_element_replies_stop_with_explicit_incomplete_progress() {
    for count in [0, 1] {
        let peer = DiscoveryFixture::start(0).await;
        let c = client(&peer, 30);
        peer.peer.sample_short_at.store(1, Ordering::SeqCst);
        peer.peer.sample_short_count.store(count, Ordering::SeqCst);
        let result = c.read_multi_history_range(range(5)).await.unwrap();
        assert!(result.short_page && !result.complete);
        assert_eq!(result.remaining, 5 - count as u32);
        assert_eq!(result.next_index, Some(count as u16));
        assert_eq!(result.pages.len(), 1);
        assert_eq!(requests(&peer).len(), 1);
        c.stop().await;
    }
}
// POL-136-INDEX-CHANGE: changed index stops by default; optional observation retains pages.
#[tokio::test]
async fn changed_index_preserves_the_observed_page_and_healthy_connection() {
    for observe in [false, true] {
        let peer = DiscoveryFixture::start(0).await;
        let c = client(&peer, 30);
        peer.peer.sample_change_at.store(2, Ordering::SeqCst);
        peer.peer.sample_change_index.store(1, Ordering::SeqCst);
        let r = if observe {
            range(5).with_continuity(HistoryContinuity::ObserveIndexChanges)
        } else {
            range(5)
        };
        let result = c.read_multi_history_range(r).await;
        let progress = if observe {
            result.unwrap()
        } else {
            let e = result.unwrap_err();
            assert!(matches!(
                e.error,
                etherbird::Error::Operation(Error::HistoryChanged(_))
            ));
            e.progress
        };
        assert!(progress.index_changed);
        assert_eq!(progress.complete, observe);
        assert_eq!(progress.pages.len(), if observe { 3 } else { 2 });
        c.read_clock().await.unwrap();
        assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
        c.stop().await;
    }
}
// POL-136-PROGRESS / POL-136-NO-REPLAY: malformed, rejected, disconnected and stalled page 2.
#[tokio::test]
async fn interrupted_page_returns_confirmed_prefix_and_explicit_recovery_only() {
    for fault in [1, 3, 4, 5] {
        let peer = DiscoveryFixture::start(0).await;
        let c = client(&peer, 30);
        peer.peer.sample_fail_at.store(2, Ordering::SeqCst);
        peer.peer.sample_fault.store(fault, Ordering::SeqCst);
        let e = c.read_multi_history_range(range(5)).await.unwrap_err();
        assert_eq!(e.progress.pages.len(), 1);
        assert_eq!(e.progress.remaining, 3);
        assert_eq!(e.progress.next_index, Some(2));
        assert_eq!(requests(&peer).len(), 2);
        c.read_clock().await.unwrap();
        assert_eq!(
            peer.peer.connections.load(Ordering::SeqCst),
            if fault == 3 { 1 } else { 2 }
        );
        let next = c
            .read_multi_history_range(
                MultiHistoryRange::new(0, 2, HistoryKind::Periodic, 7, 2, 3).unwrap(),
            )
            .await
            .unwrap();
        assert!(next.complete);
        assert_eq!(requests(&peer).len(), 4);
        if fault != 3 {
            assert_ne!(next.generation, e.progress.generation);
        }
        c.stop().await;
    }
}
// POL-136-CANCELLATION: call deadline, caller cancellation and awaited shutdown retire pending pages.
#[tokio::test]
async fn deadline_cancellation_and_stop_preserve_no_replay_contract() {
    for mode in 0..3 {
        let peer = DiscoveryFixture::start(0).await;
        let c = Client::tcp(
            &peer.endpoint,
            ClientConfig {
                profile: Arc::new(Budget {
                    request: 7,
                    response: 30,
                }),
                call_timeout: Duration::from_millis(250),
                response_timeout: Duration::from_secs(2),
                ..config()
            },
        )
        .unwrap();
        c.connected().await.unwrap();
        peer.peer.sample_fail_at.store(2, Ordering::SeqCst);
        peer.peer.sample_fault.store(5, Ordering::SeqCst);
        let caller = c.clone();
        let task = tokio::spawn(async move { caller.read_multi_history_range(range(5)).await });
        tokio::time::timeout(Duration::from_secs(1), async {
            while peer.peer.sample_reads.load(Ordering::SeqCst) < 2 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        if mode == 1 {
            task.abort();
            assert!(task.await.unwrap_err().is_cancelled());
        } else {
            if mode == 2 {
                c.stop().await;
            }
            let e = task.await.unwrap().unwrap_err();
            assert_eq!(e.progress.pages.len(), 1);
            assert_eq!(e.progress.remaining, 3);
            if mode == 0 {
                assert!(matches!(e.error, etherbird::Error::Timeout));
            }
        }
        if mode != 2 {
            c.read_clock().await.unwrap();
            assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
        }
        assert_eq!(requests(&peer).len(), 2);
        c.stop().await;
    }
}
struct CustomCodec;
impl ReplyCodec<MultiHistoryRequest> for CustomCodec {
    type Output = usize;
    fn decode(&self, q: &MultiHistoryRequest, b: &[u8]) -> Result<usize, DecodeError> {
        Ok(MultiHistoryCodec.decode(q, b)?.periods.len())
    }
}
struct BadProfile;
impl ProtocolProfile for BadProfile {
    fn capability(&self, op: u8) -> rocsteady::Capability {
        DefaultProfile.capability(op)
    }
    fn metadata(&self) -> ProfileMetadata {
        DefaultProfile.metadata()
    }
    fn decode_multi_history(
        &self,
        q: &MultiHistoryRequest,
        b: &[u8],
    ) -> Result<MultiHistoryPage, DecodeError> {
        let mut p = MultiHistoryCodec.decode(q, b)?;
        p.periods[0].values.pop();
        Ok(p)
    }
}
// POL-136-EXTENSION: caller codecs and profile codecs run under the exchange guard;
// profile output cannot bypass period-shape checks. Standalone/workflow use the same API.
#[tokio::test]
async fn public_codecs_profiles_sessions_and_workflows_obey_the_contract() {
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer, 246);
    assert_eq!(
        c.read_multi_history_with_codec(q(1), CustomCodec)
            .await
            .unwrap(),
        1
    );
    struct Read;
    #[etherbird::async_trait]
    impl rocsteady::Workflow for Read {
        type Output = ();
        async fn run(self, c: &mut rocsteady::ManagedContext<'_>) -> Result<(), Error> {
            assert_eq!(c.read_multi_history(q(1)).await?.periods.len(), 1);
            assert_eq!(c.read_multi_history_with_codec(q(1), CustomCodec).await?, 1);
            Ok(())
        }
    }
    c.workflow(Read).await.unwrap();
    c.stop().await;
    let connector = rocsteady::transport::TcpConnector::new(peer.endpoint.clone());
    let stream = rocsteady::transport::Connector::connect(&connector)
        .await
        .unwrap();
    let session = rocsteady::Session::new(
        stream,
        rocsteady::Address::new(1, 0),
        rocsteady::Address::new(1, 2),
        Duration::from_secs(1),
        rocsteady::protocol::CrcMode::Required,
    )
    .unwrap();
    assert_eq!(
        session
            .read_multi_history(q(1))
            .await
            .unwrap()
            .periods
            .len(),
        1
    );
    assert_eq!(
        session
            .read_multi_history_with_codec(q(1), CustomCodec)
            .await
            .unwrap(),
        1
    );
    let bad = Client::tcp(
        &peer.endpoint,
        ClientConfig {
            profile: Arc::new(BadProfile),
            ..config()
        },
    )
    .unwrap();
    bad.connected().await.unwrap();
    let before = peer.peer.connections.load(Ordering::SeqCst);
    assert!(bad.read_multi_history(q(1)).await.is_err());
    bad.read_clock().await.unwrap();
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), before + 1);
    bad.stop().await;
}

// POL-136-LATE: the retired stream may deliver a delayed reply, but it cannot
// contaminate the replacement. A peer barrier controls release instead of sleeping.
#[tokio::test]
async fn late_reply_cannot_contaminate_the_replacement() {
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer, 30);
    peer.peer.sample_fail_at.store(2, Ordering::SeqCst);
    peer.peer.sample_fault.store(10, Ordering::SeqCst);
    let e = c.read_multi_history_range(range(5)).await.unwrap_err();
    assert_eq!(e.progress.pages.len(), 1);
    assert_eq!(requests(&peer).len(), 2);
    c.read_clock().await.unwrap();
    peer.peer.release_late_sample.notify_one();
    assert_eq!(c.read_multi_history(q(1)).await.unwrap().periods.len(), 1);
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
    assert_eq!(requests(&peer).len(), 3);
    c.stop().await;
}
// POL-136-POLICY: custom per-point policies cannot combine incompatible rings.
#[test]
fn custom_geometry_is_checked_for_every_selected_point() {
    struct Geometry;
    impl rocsteady::HistoryIndexPolicy for Geometry {
        fn bounds(
            &self,
            _: u8,
            point: u8,
            _: HistoryKind,
        ) -> Result<rocsteady::HistoryBounds, Error> {
            rocsteady::HistoryBounds::new(10, Some(200), Some(if point == 8 { 6 } else { 5 }))
        }
    }
    assert!(
        MultiHistoryRequest::with_policy(0, 0, HistoryKind::Periodic, 7, 2, 1, &Geometry).is_err()
    );
    let one =
        MultiHistoryRequest::with_policy(0, 4, HistoryKind::Periodic, 7, 1, 1, &Geometry).unwrap();
    assert_eq!(one.bounds().capacity(), Some(5));
    assert!(
        MultiHistoryRequest::with_policy(0, 5, HistoryKind::Periodic, 7, 1, 0, &Geometry).is_err()
    );
}
