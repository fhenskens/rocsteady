include!("../examples/support/discovery_fixture.rs");
use rocsteady::{
    Client, ClientConfig, ConfiguredHistoryPolicy, DecodeError, DefaultProfile, Error,
    HistoryBounds, HistoryCodec, HistoryContinuity, HistoryIndexPolicy, HistoryKind, HistoryPage,
    HistoryRange, HistoryRequest, HistorySamples, PayloadLimits, ProfileMetadata,
    ProseMinuteHistoryPolicy, ProtocolProfile, ReplyCodec,
};
use std::time::Duration;
fn kinds() -> [HistoryKind; 5] {
    [
        HistoryKind::Minute,
        HistoryKind::Periodic,
        HistoryKind::Daily,
        HistoryKind::PeriodicTimestamps,
        HistoryKind::DailyTimestamps,
    ]
}
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
struct Budget {
    request: usize,
    response: usize,
    unavailable: bool,
}
impl ProtocolProfile for Budget {
    fn metadata(&self) -> ProfileMetadata {
        DefaultProfile.metadata()
    }
    fn capability(&self, op: u8) -> rocsteady::Capability {
        if op == 135 && self.unavailable {
            rocsteady::Capability::Unavailable
        } else {
            DefaultProfile.capability(op)
        }
    }
    fn limits(&self, op: u8) -> PayloadLimits {
        if op == 135 {
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
                request: 6,
                response,
                unavailable: false,
            }),
            ..config()
        },
    )
    .unwrap()
}
fn requests(peer: &DiscoveryFixture) -> Vec<Vec<u8>> {
    peer.peer
        .requests
        .lock()
        .unwrap()
        .iter()
        .filter(|(op, _)| *op == 135)
        .map(|(_, b)| b.clone())
        .collect()
}

// REQ-135-REQUEST / POL-135-BOUNDS: §2.15/Table 2-18, pp.2-31–32.
// Six-byte request, max 60, zero-count probe, table/prose alternatives and known geometry.
#[test]
fn request_bounds_policies_and_wire_fields_are_explicit() {
    assert_eq!(
        HistoryRequest::new(10, 255, HistoryKind::Periodic, 0x1234, 60)
            .unwrap()
            .payload(),
        [10, 255, 1, 0x34, 0x12, 60]
    );
    assert_eq!(
        HistoryRequest::new(0, 0, HistoryKind::Minute, 0, 60)
            .unwrap()
            .count(),
        60
    );
    assert!(HistoryRequest::new(0, 0, HistoryKind::Minute, 60, 0).is_err());
    assert!(HistoryRequest::new(0, 0, HistoryKind::Minute, 59, 2).is_err());
    assert!(HistoryRequest::new(0, 0, HistoryKind::Periodic, 0, 61).is_err());
    assert!(HistoryRequest::new(11, 0, HistoryKind::Periodic, 0, 0).is_err());
    assert!(HistoryRequest::new(0, 0, HistoryKind::Periodic, 65535, 2).is_err());
    assert!(HistoryRequest::new(0, 0, HistoryKind::Periodic, 65535, 1).is_ok());
    let q =
        HistoryRequest::with_policy(0, 0, HistoryKind::Minute, 60, 1, &ProseMinuteHistoryPolicy)
            .unwrap();
    assert_eq!(q.bounds().capacity(), Some(61));
    assert_eq!(q.start(), 60);
    assert!(
        HistoryRequest::with_policy(0, 0, HistoryKind::Minute, 61, 0, &ProseMinuteHistoryPolicy)
            .is_err()
    );
    let policy = ConfiguredHistoryPolicy::new(13, 200, 60, 840, 35).unwrap();
    for (kind, capacity) in [
        (HistoryKind::Periodic, 840),
        (HistoryKind::PeriodicTimestamps, 840),
        (HistoryKind::Daily, 35),
        (HistoryKind::DailyTimestamps, 35),
    ] {
        assert!(HistoryRequest::with_policy(13, 199, kind, capacity - 1, 1, &policy).is_ok());
        assert!(HistoryRequest::with_policy(13, 199, kind, capacity, 0, &policy).is_err());
    }
    assert!(HistoryRequest::with_policy(0, 200, HistoryKind::Periodic, 0, 1, &policy).is_err());
    for capacity in [0, 65537] {
        assert!(HistoryBounds::new(10, None, Some(capacity)).is_err());
    }
    for count in [0, 257] {
        assert!(HistoryBounds::new(10, Some(count), None).is_err());
    }
    assert!(ConfiguredHistoryPolicy::new(10, 200, 60, 0, 35).is_err());
    assert!(HistoryRange::new(0, 0, HistoryKind::Minute, 59, 60).is_ok());
    assert!(HistoryRange::new(0, 0, HistoryKind::Minute, 59, 61).is_err());
    assert!(HistoryRange::new(0, 0, HistoryKind::Periodic, 65535, 2).is_err());
    assert!(HistoryRange::new(0, 0, HistoryKind::Periodic, 0, 65537).is_err());
}

// REQ-135-SELECTORS: all five selectors, raw TIME/u32 vs IEEE FLOAT; Table 2-18.
#[test]
fn every_selector_decodes_literals_with_index_only_and_maximum_page_shapes() {
    for (selector, kind) in kinds().into_iter().enumerate() {
        assert_eq!(kind.as_byte(), selector as u8);
        for count in [0, 1, 60] {
            let q = HistoryRequest::new(10, 199, kind, 0, count).unwrap();
            let b = single_history_literal(10, 199, kind.as_byte(), 59, count);
            assert_eq!(b.len(), 5 + 4 * usize::from(count));
            let p = HistoryCodec.decode(&q, &b).unwrap();
            assert_eq!(p.request, q);
            assert_eq!(p.current_index, 59);
            assert_eq!(p.raw, b);
            match p.samples {
                HistorySamples::Values(v) => {
                    assert!(!kind.is_timestamp());
                    assert_eq!(v, vec![12.5; usize::from(count)]);
                }
                HistorySamples::Timestamps(v) => {
                    assert!(kind.is_timestamp());
                    assert_eq!(v, vec![0x12345678; usize::from(count)]);
                }
            }
        }
    }
    let q = HistoryRequest::new(0, 7, HistoryKind::Periodic, 0, 2).unwrap();
    let b = [0, 7, 0xff, 0xff, 2, 0x34, 0x12, 0xc0, 0x7f, 0, 0, 0, 0x80];
    let p = HistoryCodec.decode(&q, &b).unwrap();
    let HistorySamples::Values(v) = p.samples else {
        panic!("FLOAT expected")
    };
    assert_eq!(v[0].to_bits(), 0x7fc01234);
    assert_eq!(v[1].to_bits(), 0x80000000);
    let tq = HistoryRequest::new(0, 7, HistoryKind::PeriodicTimestamps, 0, 2).unwrap();
    assert_eq!(
        HistoryCodec.decode(&tq, &b).unwrap().samples,
        HistorySamples::Timestamps(vec![0x7fc01234, 0x80000000])
    );
}

// REQ-135-VALIDATE: exact returned count/width, segment+point echoes and selected current bounds.
// The reply has no selector or starting-index echo; no such correlation is invented.
#[test]
fn malformed_count_echo_width_and_current_index_are_rejected() {
    let q = HistoryRequest::new(0, 7, HistoryKind::Minute, 0, 60).unwrap();
    let b = single_history_literal(0, 7, 0, 59, 60);
    for n in 0..b.len() {
        assert!(HistoryCodec.decode(&q, &b[..n]).is_err());
    }
    let mut trailing = b.clone();
    trailing.push(0);
    assert!(HistoryCodec.decode(&q, &trailing).is_err());
    for i in [0, 1] {
        let mut wrong = b.clone();
        wrong[i] ^= 1;
        assert!(HistoryCodec.decode(&q, &wrong).is_err());
    }
    assert!(
        HistoryCodec
            .decode(&q, &single_history_literal(0, 7, 0, 59, 61))
            .is_err()
    );
    assert!(
        HistoryCodec
            .decode(&q, &single_history_literal(0, 7, 0, 60, 1))
            .is_err()
    );
    let q =
        HistoryRequest::with_policy(0, 7, HistoryKind::Minute, 60, 1, &ProseMinuteHistoryPolicy)
            .unwrap();
    assert!(
        HistoryCodec
            .decode(&q, &single_history_literal(0, 7, 0, 60, 1))
            .is_ok()
    );
    let probe = HistoryRequest::new(0, 7, HistoryKind::Periodic, 0, 0).unwrap();
    assert!(
        HistoryCodec
            .decode(&probe, &single_history_literal(0, 7, 1, 65535, 0))
            .is_ok()
    );
    assert!(
        HistoryCodec
            .decode(&probe, &single_history_literal(0, 7, 1, 0, 1))
            .is_err()
    );
    let q = HistoryRequest::new(0, 7, HistoryKind::Daily, 0, 2).unwrap();
    assert!(
        HistoryCodec
            .decode(&q, &single_history_literal(0, 7, 2, 0, 1))
            .is_ok()
    ); // short valid page
    assert!(
        HistoryCodec
            .decode(&q, &single_history_literal(0, 7, 2, 0, 3))
            .is_err()
    );
}

// REQ-135-TCP: literal requests/replies, every selector and count 0/60 at exact 245-byte budget.
#[tokio::test]
async fn independent_tcp_supports_all_selectors_probes_and_named_alternative() {
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer, 245);
    for kind in kinds() {
        for count in [0, 60] {
            let p = client
                .read_history(HistoryRequest::new(0, 7, kind, 0, count).unwrap())
                .await
                .unwrap();
            assert_eq!(p.samples.len(), usize::from(count));
            assert_eq!(
                p.raw,
                single_history_literal(0, 7, kind.as_byte(), 0, count)
            );
        }
    }
    client
        .read_history(HistoryRequest::new(0, 7, HistoryKind::Periodic, 0x1234, 1).unwrap())
        .await
        .unwrap();
    assert!(requests(&peer).contains(&vec![0, 7, 1, 0x34, 0x12, 1]));
    peer.peer.history_periodic_index.store(60, Ordering::SeqCst);
    let q =
        HistoryRequest::with_policy(0, 7, HistoryKind::Minute, 60, 1, &ProseMinuteHistoryPolicy)
            .unwrap();
    assert_eq!(client.read_history(q).await.unwrap().current_index, 60);
    assert!(requests(&peer).contains(&vec![0, 7, 0, 60, 0, 1]));
    client.stop().await;
}

// POL-135-PAGING: bounded request/reply budgets, explicit known ring boundaries and generation ownership.
#[tokio::test]
async fn paging_splits_profile_limits_and_known_retention_boundary() {
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer, 13); // two items
    let policy = ConfiguredHistoryPolicy::new(10, 200, 60, 5, 3).unwrap();
    let range = HistoryRange::with_policy(0, 7, HistoryKind::Periodic, 4, 5, &policy).unwrap();
    let result = client.read_history_range(range).await.unwrap();
    assert!(result.complete);
    assert!(!result.short_page && !result.index_changed);
    assert_eq!(result.remaining, 0);
    assert_eq!(result.next_index, Some(4));
    assert!(result.generation.is_some());
    assert_eq!(
        result
            .pages
            .iter()
            .map(|p| (p.request.start(), p.samples.len()))
            .collect::<Vec<_>>(),
        [(4, 1), (0, 2), (2, 2)]
    );
    assert_eq!(
        requests(&peer),
        [
            vec![0, 7, 1, 4, 0, 1],
            vec![0, 7, 1, 0, 0, 2],
            vec![0, 7, 1, 2, 0, 2]
        ]
    );
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
    let probe = client
        .read_history_range(HistoryRange::new(0, 7, HistoryKind::Periodic, 65535, 0).unwrap())
        .await
        .unwrap();
    assert!(probe.complete);
    assert_eq!(probe.pages.len(), 1);
    assert_eq!(probe.next_index, Some(65535));
    let end = client
        .read_history_range(HistoryRange::new(0, 7, HistoryKind::Periodic, 65535, 1).unwrap())
        .await
        .unwrap();
    assert!(end.complete);
    assert_eq!(end.next_index, None);
    client.stop().await;
}

// POL-135-BUDGET: impossible single-page/range plans fail before connection/admission.
#[tokio::test]
async fn invalid_capability_and_budgets_fail_before_any_request() {
    for (request, response, unavailable) in [
        (5, 255, false),
        (6, 4, false),
        (6, 8, false),
        (6, 255, true),
    ] {
        let peer = DiscoveryFixture::start(0).await;
        let client = Client::tcp(
            &peer.endpoint,
            ClientConfig {
                profile: Arc::new(Budget {
                    request,
                    response,
                    unavailable,
                }),
                ..config()
            },
        )
        .unwrap();
        let error = client
            .read_history_range(HistoryRange::new(0, 7, HistoryKind::Periodic, 0, 1).unwrap())
            .await
            .unwrap_err();
        assert!(error.progress.pages.is_empty());
        assert_eq!(error.progress.remaining, 1);
        assert_eq!(error.progress.generation, None);
        assert_eq!(peer.peer.sample_reads.load(Ordering::SeqCst), 0);
        assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 0);
        client.stop().await;
    }
    let header_peer = DiscoveryFixture::start(0).await;
    let header_client = client(&header_peer, 5);
    let probe = header_client
        .read_history_range(HistoryRange::new(0, 7, HistoryKind::Periodic, 0, 0).unwrap())
        .await
        .unwrap();
    assert!(probe.complete);
    assert_eq!(probe.pages.len(), 1);
    assert!(probe.pages[0].samples.is_empty());
    header_client.stop().await;
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer, 244);
    assert!(
        client
            .read_history(HistoryRequest::new(0, 7, HistoryKind::Periodic, 0, 60).unwrap())
            .await
            .is_err()
    );
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 0);
    let result = client
        .read_history_range(HistoryRange::new(0, 7, HistoryKind::Periodic, 0, 60).unwrap())
        .await
        .unwrap();
    assert_eq!(
        result
            .pages
            .iter()
            .map(|p| p.samples.len())
            .collect::<Vec<_>>(),
        [59, 1]
    );
    client.stop().await;
}

// POL-135-SHORT: a short/empty valid reply stops without guessing absent history or spinning.
#[tokio::test]
async fn short_pages_preserve_remaining_window_and_do_not_request_more() {
    for (at, n, pages, remaining, next) in [(1, 0, 1, 5, 0), (1, 1, 1, 4, 1), (2, 1, 2, 2, 3)] {
        let peer = DiscoveryFixture::start(0).await;
        let client = client(&peer, 13);
        peer.peer.sample_short_at.store(at, Ordering::SeqCst);
        peer.peer.sample_short_count.store(n, Ordering::SeqCst);
        let result = client
            .read_history_range(HistoryRange::new(0, 7, HistoryKind::Periodic, 0, 5).unwrap())
            .await
            .unwrap();
        assert!(!result.complete && result.short_page);
        assert_eq!(result.pages.len(), pages);
        assert_eq!(result.remaining, remaining);
        assert_eq!(result.next_index, Some(next));
        assert_eq!(peer.peer.sample_reads.load(Ordering::SeqCst), pages);
        client.stop().await;
    }
}

// POL-135-INDEX-CHANGE: stop by default on observed ring movement, or explicitly
// return bounded observations with a flag. Equal cursors do not prove continuity.
#[tokio::test]
async fn observed_index_change_is_explicit_and_preserves_healthy_connection() {
    for observe in [false, true] {
        let peer = DiscoveryFixture::start(0).await;
        let client = client(&peer, 13);
        peer.peer.sample_change_at.store(2, Ordering::SeqCst);
        peer.peer.sample_change_index.store(1, Ordering::SeqCst);
        let mut range = HistoryRange::new(0, 7, HistoryKind::Periodic, 0, 5).unwrap();
        if observe {
            range = range.with_continuity(HistoryContinuity::ObserveIndexChanges);
        }
        let result = client.read_history_range(range).await;
        let progress = if observe {
            let p = result.unwrap();
            assert!(p.complete);
            assert_eq!(p.pages.len(), 3);
            p
        } else {
            let e = result.unwrap_err();
            assert!(matches!(
                e.error,
                etherbird::Error::Operation(Error::HistoryChanged(_))
            ));
            assert!(!e.progress.complete);
            assert_eq!(e.progress.remaining, 1);
            assert_eq!(e.progress.next_index, Some(4));
            assert_eq!(e.progress.pages.len(), 2);
            e.progress
        };
        assert!(progress.index_changed);
        client.read_clock().await.unwrap();
        assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
        client.stop().await;
    }
}

// POL-135-PROGRESS / POL-135-NO-REPLAY: failure on page two returns only confirmed
// pages and an explicit next position. Recovery never restarts or appends silently.
#[tokio::test]
async fn interrupted_second_page_returns_progress_then_allows_explicit_new_window() {
    for fault in [1, 3, 4, 5] {
        let peer = DiscoveryFixture::start(0).await;
        let client = client(&peer, 13);
        peer.peer.sample_fail_at.store(2, Ordering::SeqCst);
        peer.peer.sample_fault.store(fault, Ordering::SeqCst);
        let e = client
            .read_history_range(HistoryRange::new(0, 7, HistoryKind::Periodic, 0, 5).unwrap())
            .await
            .unwrap_err();
        assert_eq!(e.progress.pages.len(), 1);
        assert_eq!(e.progress.pages[0].samples.len(), 2);
        assert_eq!(e.progress.remaining, 3);
        assert_eq!(e.progress.next_index, Some(2));
        assert!(!e.progress.complete);
        assert_eq!(peer.peer.sample_reads.load(Ordering::SeqCst), 2);
        client.read_clock().await.unwrap();
        assert_eq!(
            peer.peer.connections.load(Ordering::SeqCst),
            if fault == 3 { 1 } else { 2 }
        );
        let next = client
            .read_history_range(HistoryRange::new(0, 7, HistoryKind::Periodic, 2, 3).unwrap())
            .await
            .unwrap();
        assert!(next.complete);
        assert_eq!(next.pages.len(), 2);
        assert_eq!(peer.peer.sample_reads.load(Ordering::SeqCst), 4);
        if fault != 3 {
            assert_ne!(e.progress.generation, next.generation);
        }
        client.stop().await;
    }
}

// POL-135-CALL-DEADLINE: call deadline includes all pages and retains the completed
// prefix; pending work retires. Cancellation also cannot finish or replay late.
#[tokio::test]
async fn call_deadline_and_cancellation_retire_pending_history_pages() {
    for cancel in [false, true] {
        let peer = DiscoveryFixture::start(0).await;
        let client = Client::tcp(
            &peer.endpoint,
            ClientConfig {
                profile: Arc::new(Budget {
                    request: 6,
                    response: 13,
                    unavailable: false,
                }),
                call_timeout: Duration::from_millis(200),
                response_timeout: Duration::from_secs(1),
                ..config()
            },
        )
        .unwrap();
        client.connected().await.unwrap();
        peer.peer.sample_fail_at.store(2, Ordering::SeqCst);
        peer.peer.sample_fault.store(5, Ordering::SeqCst);
        let caller = client.clone();
        let task = tokio::spawn(async move {
            caller
                .read_history_range(HistoryRange::new(0, 7, HistoryKind::Periodic, 0, 5).unwrap())
                .await
        });
        tokio::time::timeout(Duration::from_secs(1), async {
            while peer.peer.sample_reads.load(Ordering::SeqCst) < 2 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        if cancel {
            task.abort();
            assert!(task.await.unwrap_err().is_cancelled());
        } else {
            let e = task.await.unwrap().unwrap_err();
            assert!(matches!(e.error, etherbird::Error::Timeout));
            assert_eq!(e.progress.pages.len(), 1);
            assert_eq!(e.progress.next_index, Some(2));
            assert_eq!(e.progress.remaining, 3);
        }
        client.read_clock().await.unwrap();
        assert_eq!(peer.peer.sample_reads.load(Ordering::SeqCst), 2);
        assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
        client.stop().await;
    }
}

struct CustomCodec;
impl ReplyCodec<HistoryRequest> for CustomCodec {
    type Output = (u16, usize);
    fn decode(&self, q: &HistoryRequest, b: &[u8]) -> Result<Self::Output, DecodeError> {
        let page = HistoryCodec.decode(q, b)?;
        Ok((page.current_index, page.samples.len()))
    }
}
struct CustomPolicy;
impl HistoryIndexPolicy for CustomPolicy {
    fn bounds(&self, _: u8, _: u8, _: HistoryKind) -> Result<HistoryBounds, Error> {
        HistoryBounds::new(13, Some(8), Some(4))
    }
}
struct FixtureProfile {
    invalid_shape: bool,
}
impl ProtocolProfile for FixtureProfile {
    fn metadata(&self) -> ProfileMetadata {
        ProfileMetadata {
            name: "fixture-history-codec",
            applicability: "synthetic fixture",
            assumptions: "consumer interpretation",
            evidence: "tests/history.rs",
        }
    }
    fn capability(&self, op: u8) -> rocsteady::Capability {
        DefaultProfile.capability(op)
    }
    fn decode_history(&self, q: &HistoryRequest, b: &[u8]) -> Result<HistoryPage, DecodeError> {
        let mut p = HistoryCodec.decode(q, b)?;
        if self.invalid_shape {
            p.samples = HistorySamples::Timestamps(vec![0; 100]);
        } else if let HistorySamples::Values(v) = &mut p.samples {
            for x in v {
                *x *= 2.0;
            }
        }
        Ok(p)
    }
}
// POL-135-EXTENSION: built-in and caller policies/codecs share public interfaces.
// A profile cannot corrupt range shape under the retirement guard.
#[tokio::test]
async fn public_custom_policy_codec_and_profile_are_replaceable_with_guarded_shape() {
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer, 245);
    let q = HistoryRequest::with_policy(13, 7, HistoryKind::Minute, 3, 1, &CustomPolicy).unwrap();
    assert_eq!(
        client
            .read_history_with_codec(q, CustomCodec)
            .await
            .unwrap(),
        (0, 1)
    );
    client.stop().await;
    struct Reject;
    impl ReplyCodec<HistoryRequest> for Reject {
        type Output = ();
        fn decode(&self, _: &HistoryRequest, _: &[u8]) -> Result<(), DecodeError> {
            Err(DecodeError("consumer interpretation rejected"))
        }
    }
    let rejected = Client::tcp(&peer.endpoint, config()).unwrap();
    rejected.connected().await.unwrap();
    let before = peer.peer.connections.load(Ordering::SeqCst);
    assert!(matches!(
        rejected.read_history_with_codec(q, Reject).await,
        Err(etherbird::Error::Operation(Error::InvalidResponse(_)))
    ));
    rejected.read_clock().await.unwrap();
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), before + 1);
    rejected.stop().await;
    for invalid in [false, true] {
        let client = Client::tcp(
            &peer.endpoint,
            ClientConfig {
                profile: Arc::new(FixtureProfile {
                    invalid_shape: invalid,
                }),
                ..config()
            },
        )
        .unwrap();
        client.connected().await.unwrap();
        let before = peer.peer.connections.load(Ordering::SeqCst);
        let result = client
            .read_history(HistoryRequest::new(0, 7, HistoryKind::Periodic, 0, 1).unwrap())
            .await;
        if invalid {
            assert!(matches!(
                result,
                Err(etherbird::Error::Operation(Error::InvalidResponse(_)))
            ));
        } else {
            assert_eq!(result.unwrap().samples, HistorySamples::Values(vec![25.0]));
        }
        client.read_clock().await.unwrap();
        if invalid {
            assert_eq!(peer.peer.connections.load(Ordering::SeqCst), before + 1);
        }
        client.stop().await;
    }
}

// POL-135-STOP: awaited shutdown cancels the pending page and returns the
// confirmed prefix to the waiting caller without starting a replacement.
#[tokio::test]
async fn awaited_shutdown_returns_confirmed_history_prefix() {
    let peer = DiscoveryFixture::start(0).await;
    let client = client(&peer, 13);
    peer.peer.sample_fail_at.store(2, Ordering::SeqCst);
    peer.peer.sample_fault.store(5, Ordering::SeqCst);
    let caller = client.clone();
    let task = tokio::spawn(async move {
        caller
            .read_history_range(HistoryRange::new(0, 7, HistoryKind::Periodic, 0, 5).unwrap())
            .await
    });
    tokio::time::timeout(Duration::from_secs(1), async {
        while peer.peer.sample_reads.load(Ordering::SeqCst) < 2 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    client.stop().await;
    let error = task.await.unwrap().unwrap_err();
    assert_eq!(error.progress.pages.len(), 1);
    assert_eq!(error.progress.next_index, Some(2));
    assert_eq!(error.progress.remaining, 3);
    assert!(!error.progress.complete);
    assert_eq!(peer.peer.sample_reads.load(Ordering::SeqCst), 2);
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
    assert!(client.read_clock().await.is_err());
}

// POL-135-SESSION: standalone/managed workflow methods use identical codecs and
// keep several explicitly requested pages within one exclusive generation.
#[tokio::test]
async fn standalone_and_workflow_history_methods_share_the_contract() {
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
    let q = HistoryRequest::new(0, 7, HistoryKind::DailyTimestamps, 0, 1).unwrap();
    assert_eq!(
        session.read_history(q).await.unwrap().samples,
        HistorySamples::Timestamps(vec![0x12345678])
    );
    assert_eq!(
        session
            .read_history_with_codec(q, CustomCodec)
            .await
            .unwrap(),
        (0, 1)
    );
    struct ReadPages;
    #[etherbird::async_trait]
    impl rocsteady::Workflow for ReadPages {
        type Output = ();
        async fn run(self, c: &mut rocsteady::ManagedContext<'_>) -> Result<(), Error> {
            let g = c.generation();
            let q = HistoryRequest::new(0, 7, HistoryKind::Periodic, 0, 1)?;
            assert_eq!(c.read_history(q).await?.samples.len(), 1);
            assert_eq!(c.read_history_with_codec(q, CustomCodec).await?, (0, 1));
            assert_eq!(c.generation(), g);
            Ok(())
        }
    }
    let client = client(&peer, 245);
    client.workflow(ReadPages).await.unwrap();
    client.stop().await;
}
