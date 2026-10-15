include!("../examples/support/discovery_fixture.rs");
use rocsteady::{
    Client, ClientConfig, ConfiguredHistoryCodec, ConfiguredHistoryPolicy,
    ConfiguredHistoryRequest, DecodeError, DefaultProfile, Error, HistoryContinuity, HistoryKind,
    PayloadLimits, ProfileMetadata, ProtocolProfile, ReplyCodec, SelectedHistoryCodec,
    SelectedHistoryPage, SelectedHistoryRange, SelectedHistoryRequest,
};
use std::time::Duration;
fn iq() -> ConfiguredHistoryRequest {
    ConfiguredHistoryRequest::new(0).unwrap()
}
fn q(periods: u8, timestamps: bool) -> SelectedHistoryRequest {
    SelectedHistoryRequest::new(
        0,
        0,
        HistoryKind::Periodic,
        vec![7, 2, 7],
        periods,
        timestamps,
    )
    .unwrap()
}
fn range(total: u32, timestamps: bool) -> SelectedHistoryRange {
    SelectedHistoryRange::new(
        0,
        0,
        HistoryKind::Periodic,
        vec![7, 2, 7],
        total,
        timestamps,
    )
    .unwrap()
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
    request: usize,
    response: usize,
    unavailable: bool,
}
impl ProtocolProfile for Budget {
    fn metadata(&self) -> ProfileMetadata {
        DefaultProfile.metadata()
    }
    fn capability(&self, op: u8) -> rocsteady::Capability {
        if op == 139 && self.unavailable {
            rocsteady::Capability::Unavailable
        } else {
            DefaultProfile.capability(op)
        }
    }
    fn limits(&self, op: u8) -> PayloadLimits {
        if op == 139 {
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
                request: 255,
                response,
                unavailable: false,
            }),
            ..config()
        },
    )
    .unwrap()
}
fn calls(peer: &DiscoveryFixture, command: u8) -> Vec<Vec<u8>> {
    peer.peer
        .requests
        .lock()
        .unwrap()
        .iter()
        .filter(|(op, b)| *op == 139 && b[0] == command)
        .map(|(_, b)| b.clone())
        .collect()
}
// REQ-139-REQUEST / POL-139-SELECTION: §2.19/Table 2-22, p.2-36.
// Two command shapes; 60 includes optional TIME; nonempty list is a local API policy.
#[test]
fn request_fields_products_selectors_and_explicit_geometry() {
    assert_eq!(iq().payload(), [0, 0]);
    assert!(ConfiguredHistoryRequest::new(11).is_err());
    assert_eq!(
        ConfiguredHistoryRequest::with_segment_limit(13, 13)
            .unwrap()
            .payload(),
        [0, 13]
    );
    for kind in [
        HistoryKind::Minute,
        HistoryKind::Periodic,
        HistoryKind::Daily,
    ] {
        for timestamps in [false, true] {
            let q = SelectedHistoryRequest::new(10, 0x12, kind, vec![255, 2, 255], 2, timestamps)
                .unwrap();
            assert_eq!(
                q.payload(),
                [
                    1,
                    10,
                    0x12,
                    0,
                    kind.as_byte(),
                    2,
                    u8::from(timestamps),
                    3,
                    255,
                    2,
                    255
                ]
            );
        }
    }
    for points in 1..=247 {
        for periods in 0..=61 {
            for timestamps in [false, true] {
                assert_eq!(
                    SelectedHistoryRequest::new(
                        0,
                        0,
                        HistoryKind::Periodic,
                        vec![7; points],
                        periods,
                        timestamps
                    )
                    .is_ok(),
                    (points + usize::from(timestamps)) * usize::from(periods) <= 60
                );
            }
        }
    }
    assert!(SelectedHistoryRequest::new(0, 0, HistoryKind::Periodic, vec![], 1, true).is_err());
    assert!(
        SelectedHistoryRequest::new(0, 0, HistoryKind::Periodic, vec![7; 248], 0, false).is_err()
    );
    for kind in [
        HistoryKind::PeriodicTimestamps,
        HistoryKind::DailyTimestamps,
    ] {
        assert!(SelectedHistoryRequest::new(0, 0, kind, vec![7], 1, false).is_err());
    }
    assert!(SelectedHistoryRequest::new(11, 0, HistoryKind::Daily, vec![7], 1, false).is_err());
    assert!(SelectedHistoryRequest::new(0, 60, HistoryKind::Minute, vec![7], 0, false).is_err());
    assert!(
        SelectedHistoryRequest::with_policy(
            0,
            60,
            HistoryKind::Minute,
            vec![7],
            1,
            false,
            &rocsteady::ProseMinuteHistoryPolicy
        )
        .is_ok()
    );
    let policy = ConfiguredHistoryPolicy::new(13, 200, 60, 5, 3).unwrap();
    assert!(
        SelectedHistoryRequest::with_policy(
            13,
            4,
            HistoryKind::Periodic,
            vec![199, 2],
            1,
            true,
            &policy
        )
        .is_ok()
    );
    assert!(
        SelectedHistoryRequest::with_policy(
            13,
            4,
            HistoryKind::Periodic,
            vec![199, 200],
            0,
            true,
            &policy
        )
        .is_err()
    );
    assert!(
        SelectedHistoryRequest::with_policy(
            13,
            4,
            HistoryKind::Periodic,
            vec![199, 2],
            2,
            true,
            &policy
        )
        .is_err()
    );
    assert!(
        SelectedHistoryRange::with_policy(
            13,
            4,
            HistoryKind::Periodic,
            vec![199, 2],
            6,
            true,
            &policy
        )
        .is_err()
    );
    assert!(SelectedHistoryRange::new(0, 65535, HistoryKind::Daily, vec![7], 2, false).is_err());
}
// REQ-139-INVENTORY: command/segment/count, empty/max inventory and raw order/duplicates.
#[test]
fn inventory_count_echo_and_wire_budget_preserve_all_points() {
    let b = [0, 0, 4, 255, 7, 2, 7];
    let r = ConfiguredHistoryCodec.decode(&iq(), &b).unwrap();
    assert_eq!(r.points, [255, 7, 2, 7]);
    assert_eq!(r.raw, b);
    assert!(
        ConfiguredHistoryCodec
            .decode(&iq(), &[0, 0, 0])
            .unwrap()
            .points
            .is_empty()
    );
    let mut max = vec![0, 0, 252];
    max.extend(0..252u8);
    assert_eq!(
        ConfiguredHistoryCodec
            .decode(&iq(), &max)
            .unwrap()
            .points
            .len(),
        252
    );
    for len in 0..max.len() {
        assert!(ConfiguredHistoryCodec.decode(&iq(), &max[..len]).is_err());
    }
    for i in 0..3 {
        let mut bad = b;
        bad[i] ^= 1;
        assert!(ConfiguredHistoryCodec.decode(&iq(), &bad).is_err());
    }
    let mut trailing = b.to_vec();
    trailing.push(0);
    assert!(ConfiguredHistoryCodec.decode(&iq(), &trailing).is_err());
    let mut too_many = vec![0, 0, 253];
    too_many.resize(256, 0);
    assert!(ConfiguredHistoryCodec.decode(&iq(), &too_many).is_err());
}
// REQ-139-ORDER / REQ-139-RAW: time-major rows, request order/duplicates, optional raw TIME.
#[test]
fn literal_rows_preserve_float_bits_and_optional_timestamp_word() {
    for timestamps in [false, true] {
        let request =
            SelectedHistoryRequest::new(0, 0x1234, HistoryKind::Daily, vec![7, 2], 2, timestamps)
                .unwrap();
        let mut b = vec![1, 0, 255, 255, 2, u8::from(timestamps), 2];
        if timestamps {
            b.extend([0x78, 0x56, 0x34, 0x12]);
        }
        b.extend([0, 0, 0x80, 0x3f, 0, 0, 0, 0x80]);
        if timestamps {
            b.extend([255; 4]);
        }
        b.extend([0x34, 0x12, 0xc0, 0x7f, 0, 0, 0x80, 0x7f]);
        let page = SelectedHistoryCodec.decode(&request, &b).unwrap();
        assert_eq!(page.current_index, 65535);
        assert_eq!(page.raw, b);
        assert_eq!(
            page.periods
                .iter()
                .flat_map(|r| r.values.iter().map(|v| v.to_bits()))
                .collect::<Vec<_>>(),
            [0x3f800000, 0x80000000, 0x7fc01234, 0x7f800000]
        );
        assert_eq!(
            page.periods.iter().map(|r| r.timestamp).collect::<Vec<_>>(),
            if timestamps {
                vec![Some(0x12345678), Some(u32::MAX)]
            } else {
                vec![None, None]
            }
        );
    }
    let r = SelectedHistoryCodec
        .decode(
            &q(2, true),
            &selected_history_literal(0, 0, &[7, 2, 7], true, 0, 2),
        )
        .unwrap();
    assert_eq!(r.periods[0].values, [7.0, 2.0, 7.0]);
    assert_eq!(r.periods[1].values, [1007.0, 1002.0, 1007.0]);
}
// REQ-139-VALIDATE: all truncations, trailing bytes, echoes, flags and count fields.
#[test]
fn malformed_reply_headers_counts_flags_and_current_indices_are_rejected() {
    for timestamps in [false, true] {
        let request = q(2, timestamps);
        let b = selected_history_literal(0, 0, &[7, 2, 7], timestamps, 0, 2);
        for len in 0..b.len() {
            assert!(SelectedHistoryCodec.decode(&request, &b[..len]).is_err());
        }
        for i in [0, 1, 4, 5, 6] {
            let mut bad = b.clone();
            bad[i] ^= 1;
            assert!(SelectedHistoryCodec.decode(&request, &bad).is_err());
        }
        for flag in [2, 255] {
            let mut bad = b.clone();
            bad[5] = flag;
            assert!(SelectedHistoryCodec.decode(&request, &bad).is_err());
        }
        let mut bad = b.clone();
        bad.push(0);
        assert!(SelectedHistoryCodec.decode(&request, &bad).is_err());
        let mut bad = b.clone();
        bad[4] = 3;
        bad.resize(7 + 4 * (3 + usize::from(timestamps)) * 3, 0);
        assert!(SelectedHistoryCodec.decode(&request, &bad).is_err());
        let probe = q(0, timestamps);
        assert!(SelectedHistoryCodec.decode(&probe, &b).is_err());
    }
    let minute = SelectedHistoryRequest::new(0, 0, HistoryKind::Minute, vec![7], 1, true).unwrap();
    assert!(
        SelectedHistoryCodec
            .decode(&minute, &selected_history_literal(0, 0, &[7], true, 60, 1))
            .is_err()
    );
}
// REQ-139-TCP: all selectors and flag shapes, max 60 elements/247 bytes and index probes.
#[tokio::test]
async fn independent_tcp_checks_inventory_and_every_selection_shape() {
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer, 255);
    assert_eq!(
        c.read_configured_history(iq()).await.unwrap().points,
        [7, 2, 255]
    );
    for kind in [
        HistoryKind::Minute,
        HistoryKind::Periodic,
        HistoryKind::Daily,
    ] {
        for timestamps in [false, true] {
            let count = if timestamps { 15 } else { 20 };
            let request =
                SelectedHistoryRequest::new(0, 0, kind, vec![7, 2, 7], count, timestamps).unwrap();
            let p = c.read_selected_history(request).await.unwrap();
            assert_eq!(p.raw.len(), 247);
            assert_eq!(p.periods.len(), usize::from(count));
            let p = c
                .read_selected_history(
                    SelectedHistoryRequest::new(0, 0, kind, vec![7; 247], 0, timestamps).unwrap(),
                )
                .await
                .unwrap();
            assert!(p.periods.is_empty());
            assert_eq!(p.raw.len(), 7);
        }
    }
    assert_eq!(calls(&peer, 0), [vec![0, 0]]);
    assert_eq!(calls(&peer, 1)[0], [1, 0, 0, 0, 0, 20, 0, 3, 7, 2, 7]);
    c.stop().await;
}
// POL-139-PAGING: whole periods obey profile budgets and known ring boundaries.
#[tokio::test]
async fn ranges_split_periods_and_wrap_only_known_geometry() {
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer, 39); // two timestamped 4-word rows
    let r = c.read_selected_history_range(range(5, true)).await.unwrap();
    assert!(r.complete);
    assert_eq!(r.next_index, Some(5));
    assert_eq!(r.remaining, 0);
    assert_eq!(
        r.pages
            .iter()
            .map(|p| (p.request.start(), p.periods.len()))
            .collect::<Vec<_>>(),
        [(0, 2), (2, 2), (4, 1)]
    );
    let policy = ConfiguredHistoryPolicy::new(10, 256, 60, 5, 3).unwrap();
    let r = c
        .read_selected_history_range(
            SelectedHistoryRange::with_policy(
                0,
                4,
                HistoryKind::Periodic,
                vec![7, 2, 7],
                5,
                true,
                &policy,
            )
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        r.pages
            .iter()
            .map(|p| (p.request.start(), p.periods.len()))
            .collect::<Vec<_>>(),
        [(4, 1), (0, 2), (2, 2)]
    );
    assert_eq!(r.next_index, Some(4));
    assert!(
        c.read_selected_history_range(range(0, true))
            .await
            .unwrap()
            .complete
    );
    let r = c
        .read_selected_history_range(
            SelectedHistoryRange::new(0, 65535, HistoryKind::Daily, vec![255], 1, false).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.next_index, None);
    c.stop().await;
}
// POL-139-BUDGET: request/reply/capability validation happens before wire use; points are not split.
#[tokio::test]
async fn impossible_budgets_and_unavailable_capability_fail_locally() {
    for (request, response, unavailable) in [
        (10, 247, false),
        (11, 6, false),
        (11, 22, false),
        (255, 247, true),
    ] {
        let peer = DiscoveryFixture::start(0).await;
        let c = Client::tcp(
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
        assert!(c.read_selected_history_range(range(1, true)).await.is_err());
        assert!(calls(&peer, 1).is_empty());
        c.stop().await;
    }
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer, 246);
    assert!(c.read_selected_history(q(15, true)).await.is_err());
    assert!(calls(&peer, 1).is_empty());
    let r = c
        .read_selected_history_range(range(15, true))
        .await
        .unwrap();
    assert_eq!(
        r.pages.iter().map(|p| p.periods.len()).collect::<Vec<_>>(),
        [14, 1]
    );
    c.stop().await;
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer, 7);
    assert!(
        c.read_selected_history_range(range(0, false))
            .await
            .unwrap()
            .complete
    );
    c.stop().await;
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer, 3);
    assert!(c.read_configured_history(iq()).await.is_err());
    c.read_clock().await.unwrap();
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
    c.stop().await;
}
// POL-139-INVENTORY: membership and before/after checks are explicit and same-generation.
#[tokio::test]
async fn inventory_checks_report_changed_or_removed_points_without_replay() {
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer, 39);
    let r = c
        .read_selected_history_range(range(5, true).with_inventory_check(true))
        .await
        .unwrap();
    assert!(r.complete);
    assert_eq!(calls(&peer, 0).len(), 2);
    assert!(r.inventory_before.is_some() && r.inventory_after.is_some());
    peer.peer
        .inventory_replies
        .lock()
        .unwrap()
        .insert(4, vec![0, 0, 2, 7, 255]);
    let e = c
        .read_selected_history_range(range(5, true).with_inventory_check(true))
        .await
        .unwrap_err();
    assert!(e.progress.inventory_changed && !e.progress.complete);
    assert_eq!(e.progress.remaining, 0);
    assert_eq!(e.progress.pages.len(), 3);
    c.read_clock().await.unwrap();
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
    peer.peer
        .inventory_replies
        .lock()
        .unwrap()
        .insert(5, vec![0, 0, 1, 7]);
    let before = calls(&peer, 1).len();
    let e = c
        .read_selected_history_range(range(5, true).with_inventory_check(true))
        .await
        .unwrap_err();
    assert!(matches!(
        e.error,
        etherbird::Error::Operation(Error::HistoryChanged(_))
    ));
    assert!(e.progress.pages.is_empty());
    assert_eq!(calls(&peer, 1).len(), before);
    c.stop().await;
}
// POL-139-INVENTORY-FAILURE: a failed final check leaves all samples but no completion claim.
#[tokio::test]
async fn failed_final_inventory_retains_all_confirmed_samples() {
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer, 39);
    peer.peer.inventory_fail_at.store(2, Ordering::SeqCst);
    peer.peer.inventory_fault.store(4, Ordering::SeqCst);
    let e = c
        .read_selected_history_range(range(5, true).with_inventory_check(true))
        .await
        .unwrap_err();
    assert_eq!(e.progress.pages.len(), 3);
    assert_eq!(e.progress.remaining, 0);
    assert!(!e.progress.complete);
    assert!(e.progress.inventory_after.is_none());
    assert_eq!(calls(&peer, 1).len(), 3);
    c.read_clock().await.unwrap();
    assert_eq!(calls(&peer, 1).len(), 3);
    c.stop().await;
}
// POL-139-SHORT: zero/short positive replies stop with explicit incomplete progress.
#[tokio::test]
async fn short_replies_do_not_guess_missing_periods_or_continue_inventory_checks() {
    for count in [0, 1] {
        let peer = DiscoveryFixture::start(0).await;
        let c = client(&peer, 39);
        peer.peer.sample_short_at.store(1, Ordering::SeqCst);
        peer.peer.sample_short_count.store(count, Ordering::SeqCst);
        let p = c
            .read_selected_history_range(range(5, true).with_inventory_check(true))
            .await
            .unwrap();
        assert!(p.short_page && !p.complete);
        assert_eq!(p.remaining, 5 - count as u32);
        assert_eq!(p.next_index, Some(count as u16));
        assert_eq!(calls(&peer, 1).len(), 1);
        assert_eq!(calls(&peer, 0).len(), 1);
        c.stop().await;
    }
}
// POL-139-CONTINUITY: strict/observe movement retains the page that exposed it.
#[tokio::test]
async fn index_movement_stops_or_continues_only_by_explicit_policy() {
    for observe in [false, true] {
        let peer = DiscoveryFixture::start(0).await;
        let c = client(&peer, 39);
        peer.peer.sample_change_at.store(2, Ordering::SeqCst);
        peer.peer.sample_change_index.store(1, Ordering::SeqCst);
        let r = if observe {
            range(5, true).with_continuity(HistoryContinuity::ObserveIndexChanges)
        } else {
            range(5, true)
        };
        let r = c.read_selected_history_range(r).await;
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
        assert_eq!(p.pages.len(), if observe { 3 } else { 2 });
        c.read_clock().await.unwrap();
        assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
        c.stop().await;
    }
}
// POL-139-PROGRESS / POL-139-NO-REPLAY: malformed, rejected, disconnected/stalled page 2.
#[tokio::test]
async fn failures_return_only_confirmed_pages_and_require_explicit_new_window() {
    for fault in [1, 3, 4, 5] {
        let peer = DiscoveryFixture::start(0).await;
        let c = client(&peer, 39);
        peer.peer.sample_fail_at.store(2, Ordering::SeqCst);
        peer.peer.sample_fault.store(fault, Ordering::SeqCst);
        let e = c
            .read_selected_history_range(range(5, true))
            .await
            .unwrap_err();
        assert_eq!(e.progress.pages.len(), 1);
        assert_eq!(e.progress.next_index, Some(2));
        assert_eq!(e.progress.remaining, 3);
        assert_eq!(calls(&peer, 1).len(), 2);
        c.read_clock().await.unwrap();
        assert_eq!(
            peer.peer.connections.load(Ordering::SeqCst),
            if fault == 3 { 1 } else { 2 }
        );
        assert_eq!(calls(&peer, 1).len(), 2);
        let next = c
            .read_selected_history_range(
                SelectedHistoryRange::new(0, 2, HistoryKind::Periodic, vec![7, 2, 7], 3, true)
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(next.complete);
        if fault != 3 {
            assert_ne!(e.progress.generation, next.generation);
        }
        c.stop().await;
    }
}
// POL-139-CANCELLATION: overall deadline, caller abort and awaited stop retire active work.
#[tokio::test]
async fn deadline_cancellation_and_stop_never_replay_pending_pages() {
    for mode in 0..3 {
        let peer = DiscoveryFixture::start(0).await;
        let c = Client::tcp(
            &peer.endpoint,
            ClientConfig {
                profile: Arc::new(Budget {
                    request: 255,
                    response: 39,
                    unavailable: false,
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
        let task =
            tokio::spawn(async move { caller.read_selected_history_range(range(5, true)).await });
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
        assert_eq!(calls(&peer, 1).len(), 2);
        c.stop().await;
    }
}
// POL-139-LATE: a delayed reply from an old stream cannot reach the replacement.
#[tokio::test]
async fn delayed_selected_reply_cannot_contaminate_a_new_generation() {
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer, 39);
    peer.peer.sample_fail_at.store(2, Ordering::SeqCst);
    peer.peer.sample_fault.store(10, Ordering::SeqCst);
    let e = c
        .read_selected_history_range(range(5, true))
        .await
        .unwrap_err();
    assert_eq!(e.progress.pages.len(), 1);
    c.read_clock().await.unwrap();
    peer.peer.release_late_sample.notify_one();
    assert_eq!(
        c.read_selected_history(q(1, true))
            .await
            .unwrap()
            .periods
            .len(),
        1
    );
    assert_eq!(calls(&peer, 1).len(), 3);
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
    c.stop().await;
}
struct ConsumerCodec;
impl ReplyCodec<SelectedHistoryRequest> for ConsumerCodec {
    type Output = usize;
    fn decode(&self, q: &SelectedHistoryRequest, b: &[u8]) -> Result<usize, DecodeError> {
        Ok(SelectedHistoryCodec.decode(q, b)?.periods.len())
    }
}
struct BadProfile;
impl ProtocolProfile for BadProfile {
    fn metadata(&self) -> ProfileMetadata {
        DefaultProfile.metadata()
    }
    fn capability(&self, op: u8) -> rocsteady::Capability {
        DefaultProfile.capability(op)
    }
    fn decode_configured_history(
        &self,
        q: &ConfiguredHistoryRequest,
        b: &[u8],
    ) -> Result<rocsteady::ConfiguredHistoryPoints, DecodeError> {
        let mut p = ConfiguredHistoryCodec.decode(q, b)?;
        p.points.resize(253, 0);
        Ok(p)
    }
    fn decode_selected_history(
        &self,
        q: &SelectedHistoryRequest,
        b: &[u8],
    ) -> Result<SelectedHistoryPage, DecodeError> {
        let mut p = SelectedHistoryCodec.decode(q, b)?;
        p.periods[0].timestamp = None;
        Ok(p)
    }
}
// POL-139-EXTENSION: public codecs/profiles, standalone and exclusive methods share the guard.
#[tokio::test]
async fn consumer_codecs_profile_invariants_sessions_and_workflows_are_supported() {
    let peer = DiscoveryFixture::start(0).await;
    let c = client(&peer, 255);
    assert_eq!(
        c.read_selected_history_with_codec(q(1, true), ConsumerCodec)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        c.read_configured_history_with_codec(iq(), ConfiguredHistoryCodec)
            .await
            .unwrap()
            .points,
        [7, 2, 255]
    );
    struct Read;
    #[etherbird::async_trait]
    impl rocsteady::Workflow for Read {
        type Output = ();
        async fn run(self, c: &mut rocsteady::ManagedContext<'_>) -> Result<(), Error> {
            assert_eq!(c.read_configured_history(iq()).await?.points.len(), 3);
            assert_eq!(
                c.read_configured_history_with_codec(iq(), ConfiguredHistoryCodec)
                    .await?
                    .points
                    .len(),
                3
            );
            assert_eq!(c.read_selected_history(q(1, true)).await?.periods.len(), 1);
            assert_eq!(
                c.read_selected_history_with_codec(q(1, true), ConsumerCodec)
                    .await?,
                1
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
    assert_eq!(
        s.read_configured_history(iq()).await.unwrap().points.len(),
        3
    );
    assert_eq!(
        s.read_configured_history_with_codec(iq(), ConfiguredHistoryCodec)
            .await
            .unwrap()
            .points
            .len(),
        3
    );
    assert_eq!(
        s.read_selected_history(q(1, true))
            .await
            .unwrap()
            .periods
            .len(),
        1
    );
    assert_eq!(
        s.read_selected_history_with_codec(q(1, true), ConsumerCodec)
            .await
            .unwrap(),
        1
    );
    for inventory in [false, true] {
        let c = Client::tcp(
            &peer.endpoint,
            ClientConfig {
                profile: Arc::new(BadProfile),
                ..config()
            },
        )
        .unwrap();
        c.connected().await.unwrap();
        let before = peer.peer.connections.load(Ordering::SeqCst);
        let result = if inventory {
            c.read_configured_history(iq()).await.map(|_| ())
        } else {
            c.read_selected_history(q(1, true)).await.map(|_| ())
        };
        assert!(result.is_err());
        c.read_clock().await.unwrap();
        assert_eq!(peer.peer.connections.load(Ordering::SeqCst), before + 1);
        c.stop().await;
    }
}
// POL-139-GEOMETRY: each requested point is checked; differing ring geometry is not combined.
#[test]
fn per_point_geometry_must_be_coherent() {
    struct Geometry;
    impl rocsteady::HistoryIndexPolicy for Geometry {
        fn bounds(
            &self,
            _: u8,
            point: u8,
            _: HistoryKind,
        ) -> Result<rocsteady::HistoryBounds, Error> {
            rocsteady::HistoryBounds::new(10, Some(256), Some(if point == 2 { 6 } else { 5 }))
        }
    }
    assert!(
        SelectedHistoryRequest::with_policy(
            0,
            0,
            HistoryKind::Periodic,
            vec![7, 2],
            1,
            false,
            &Geometry
        )
        .is_err()
    );
    assert!(
        SelectedHistoryRequest::with_policy(
            0,
            0,
            HistoryKind::Periodic,
            vec![7, 7],
            1,
            true,
            &Geometry
        )
        .is_ok()
    );
}

// POL-139-INVENTORY-SESSION: inventory decode failure also retires a standalone stream.
#[tokio::test]
async fn malformed_standalone_inventory_retires_before_reuse() {
    let peer = DiscoveryFixture::start(0).await;
    peer.peer
        .inventory_replies
        .lock()
        .unwrap()
        .insert(1, vec![0, 0, 2, 7]);
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
    assert!(matches!(
        s.read_configured_history(iq()).await,
        Err(Error::InvalidResponse(_))
    ));
    assert!(!s.is_connected());
    assert!(matches!(s.read_clock().await, Err(Error::Disconnected)));
    assert_eq!(calls(&peer, 0).len(), 1);
}
