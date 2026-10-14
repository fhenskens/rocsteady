include!("../examples/support/discovery_fixture.rs");
include!("../examples/support/log_literals.rs");
use rocsteady::{
    AlarmCodec, AlarmData, AlarmRecord, Client, ClientConfig, DataType, DecodeError, Error,
    EventCodec, EventData, EventDataType, EventRecord, LogRequest, LoggedValue, ProtocolProfile,
    ReplyCodec, Tlp, Value,
};
use std::time::Duration;

async fn fixture() -> DiscoveryFixture {
    let peer = DiscoveryFixture::start(0).await;
    peer.peer
        .alarm_records
        .lock()
        .unwrap()
        .extend((0..450).map(|i| (i, alarm_literal(i))));
    peer.peer
        .event_records
        .lock()
        .unwrap()
        .extend((0..450).map(|i| (i, event_literal(i))));
    peer
}
fn client(peer: &DiscoveryFixture) -> Client {
    Client::tcp(
        &peer.endpoint,
        ClientConfig {
            heartbeat_interval: None,
            response_timeout: Duration::from_secs(1),
            call_timeout: Duration::from_secs(3),
            recovery: etherbird::Config {
                retry_delay: Duration::from_millis(1),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap()
}
#[test]
fn table_2_16_alarm_variants_flags_reason_and_float_widths_are_literal() {
    for i in 0..6 {
        let bytes = alarm_literal(i);
        let record = AlarmRecord::decode(&bytes).unwrap();
        assert_eq!(record.raw(), &bytes);
        assert_eq!(record.timestamp(), 0x04030201);
        match (i, record.data()) {
            (0, AlarmData::None) => {}
            (
                1,
                AlarmData::Parameter {
                    reason,
                    tlp,
                    description,
                    value,
                },
            ) => {
                assert!(record.srbx() && record.is_set());
                assert_eq!(reason.0, 33);
                assert_eq!(reason.key(), Some("market_pulse"));
                assert_eq!(*tlp, Tlp::new(101, 32, 3));
                assert_eq!(description.printable_ascii(), Some("STATUS ON "));
                assert_eq!(*value, 1.0); // Alarm values are floats even for byte-status TLPs.
            }
            (
                2,
                AlarmData::Fst {
                    number,
                    description,
                    value,
                },
            ) => {
                assert!(!record.srbx() && record.is_set());
                assert_eq!(*number, 6);
                assert_eq!(description.printable_ascii(), Some("FST CONDITION"));
                assert_eq!(value.to_bits(), 0x7fc00001);
            }
            (3, AlarmData::UserText(text)) => {
                assert!(record.srbx() && !record.is_set());
                assert_eq!(text.printable_ascii(), Some("USER TEXT ALARM   "));
            }
            (4, AlarmData::UserValue { description, value }) => {
                assert_eq!(description.printable_ascii(), Some("USER VALUE    "));
                assert_eq!(*value, -12.5);
            }
            (5, AlarmData::Unknown { kind, bytes }) => {
                assert_eq!(*kind, 63);
                assert_eq!(*bytes, [0xee; 18]);
            }
            _ => panic!("unexpected alarm variant"),
        }
    }
    for packed in 0..=255 {
        let mut bytes = alarm_literal(0);
        bytes[0] = packed;
        let record = AlarmRecord::decode(&bytes).unwrap();
        assert_eq!(record.raw()[0], packed);
        assert_eq!(record.srbx(), packed & 128 != 0);
        assert_eq!(record.is_set(), packed & 64 != 0);
    }
    assert!(rocsteady::AlarmReason(34).key().is_none());
    assert!(
        rocsteady::LogText([b'A', 0, b'B'])
            .printable_ascii()
            .is_none()
    );
    for length in 0..23 {
        assert!(AlarmRecord::decode(&alarm_literal(1)[..length]).is_err());
    }
    assert!(AlarmRecord::decode(&[0; 24]).is_err());
}
#[test]
fn table_2_17_all_eight_event_kinds_and_unknown_kind_preserve_spare_bytes() {
    for i in 0..9 {
        let bytes = event_literal(i);
        let record = EventRecord::decode(&bytes).unwrap();
        assert_eq!(record.raw(), &bytes);
        assert_eq!(record.timestamp(), 0x12345678);
        match (i, record.data()) {
            (0, EventData::None) => {}
            (
                1,
                EventData::ParameterChange {
                    operator,
                    tlp,
                    data_type,
                    new_value,
                    old_value,
                },
            ) => {
                assert_eq!(operator, b"ABC");
                assert_eq!(*tlp, Tlp::new(103, 16, 21));
                assert_eq!(*data_type, EventDataType::Known(DataType::Float));
                assert_eq!(*new_value, LoggedValue::Complete(Value::Float(12.5)));
                assert_eq!(*old_value, Some(Value::Float(1.0)));
                assert_eq!(record.raw()[20..22], [0xdd; 2]);
            }
            (
                2,
                EventData::System {
                    reason,
                    description,
                },
            ) => {
                assert_eq!(reason.0, 154);
                assert_eq!(reason.key(), Some("module_inserted"));
                assert_eq!(description.printable_ascii(), Some("MODULE INSERTED "));
            }
            (
                3,
                EventData::Fst {
                    number,
                    value,
                    description,
                },
            ) => {
                assert_eq!(*number, 6);
                assert_eq!(*value, 12.5);
                assert_eq!(description.printable_ascii(), Some("FST EVENT "));
            }
            (
                4,
                EventData::User {
                    operator,
                    reason,
                    description,
                },
            ) => {
                assert_eq!(operator, b"XYZ");
                assert_eq!(reason.0, 249);
                assert_eq!(description.printable_ascii(), Some("CONFIG CHANGE"));
            }
            (5, EventData::PowerLost { time }) | (6, EventData::ClockSet { time }) => {
                assert_eq!(*time, 0x04030201)
            }
            (
                7,
                EventData::CalibrationVerification {
                    operator,
                    tlp,
                    raw_value,
                    calibrated_value,
                },
            ) => {
                assert_eq!(operator, b"ABC");
                assert_eq!(*tlp, Tlp::new(103, 16, 21));
                assert_eq!(*raw_value, 1.0);
                assert_eq!(*calibrated_value, 12.5);
                assert_eq!(record.raw()[19..22], [0xdd; 3]);
            }
            (8, EventData::Unknown { kind, bytes }) => {
                assert_eq!(*kind, 200);
                assert_eq!(*bytes, [0xdd; 17]);
            }
            _ => panic!("unexpected event variant"),
        }
    }
    assert_eq!(rocsteady::SystemEventReason(151).key(), Some("reserved"));
    assert!(rocsteady::SystemEventReason(149).key().is_none());
    for length in 0..22 {
        assert!(EventRecord::decode(&event_literal(1)[..length]).is_err());
    }
    assert!(EventRecord::decode(&[0; 23]).is_err());
}
#[test]
fn event_specific_tags_decode_old_new_and_overlaps_without_inventing_large_values() {
    let cases = [
        (0, vec![255], Value::Binary(255)),
        (1, vec![254], Value::Int8(-2)),
        (2, vec![254, 255], Value::Int16(-2)),
        (3, vec![254, 255, 255, 255], Value::Int32(-2)),
        (4, vec![254], Value::UInt8(254)),
        (5, vec![254, 255], Value::UInt16(65534)),
        (6, vec![255; 4], Value::UInt32(u32::MAX)),
        (7, vec![0, 0, 0x80, 0x3f], Value::Float(1.0)),
        (8, vec![103, 16, 21], Value::Tlp(Tlp::new(103, 16, 21))),
        (9, b"NEW".to_vec(), Value::Ascii("NEW".into())),
        (10, b"NEWTEXT".to_vec(), Value::Ascii("NEWTEXT".into())),
        (
            11,
            b"NEW TEXT  ".to_vec(),
            Value::Ascii("NEW TEXT  ".into()),
        ),
        (16, vec![0, 0, 0, 0, 0, 0, 0x29, 0x40], Value::Double(12.5)),
        (17, vec![255; 4], Value::Time(u32::MAX)),
    ];
    for (tag, bytes, expected) in cases {
        let mut record = event_literal(1);
        record[11] = tag;
        record[12..12 + bytes.len()].copy_from_slice(&bytes);
        if bytes.len() <= 4 {
            record[16..16 + bytes.len()].copy_from_slice(&bytes);
        }
        let decoded = EventRecord::decode(&record).unwrap();
        let EventData::ParameterChange {
            new_value,
            old_value,
            ..
        } = decoded.data()
        else {
            panic!()
        };
        assert_eq!(*new_value, LoggedValue::Complete(expected.clone()));
        assert_eq!(
            *old_value,
            if bytes.len() <= 4 {
                Some(expected)
            } else {
                None
            }
        );
    }
    for (tag, width) in [(12, 12), (13, 20), (14, 30), (15, 40)] {
        let mut bytes = event_literal(1);
        bytes[11] = tag;
        bytes[12..22].copy_from_slice(b"PREFIX    ");
        let record = EventRecord::decode(&bytes).unwrap();
        let EventData::ParameterChange {
            new_value,
            old_value,
            ..
        } = record.data()
        else {
            panic!()
        };
        assert_eq!(
            *new_value,
            LoggedValue::Truncated {
                data_type: DataType::Ascii(width),
                bytes: *b"PREFIX    "
            }
        );
        assert!(old_value.is_none());
    }
    let mut bytes = event_literal(1);
    bytes[11] = 255;
    let record = EventRecord::decode(&bytes).unwrap();
    let EventData::ParameterChange {
        new_value,
        old_value,
        data_type,
        ..
    } = record.data()
    else {
        panic!()
    };
    assert_eq!(*data_type, EventDataType::Unknown(255));
    assert!(matches!(new_value, LoggedValue::Unknown { tag: 255, .. }));
    assert!(old_value.is_none());
    for offset in [12, 16] {
        let mut bytes = event_literal(1);
        bytes[11] = 9;
        bytes[12..15].copy_from_slice(b"NEW");
        bytes[16..19].copy_from_slice(b"OLD");
        bytes[offset] = 0;
        assert!(EventRecord::decode(&bytes).is_err());
    }
}
#[test]
fn page_headers_record_limits_and_checkpoints_are_strict() {
    for (start, count) in [(450, 0), (65535, 1), (0, 11), (449, 2)] {
        assert!(LogRequest::new(start, count).is_err());
    }
    let request = LogRequest::new(449, 1).unwrap();
    assert_eq!(request.payload(), [1, 0xc1, 1]);
    let mut alarms = vec![1, 0xc1, 1, 0, 0];
    alarms.extend(alarm_literal(1));
    let mut events = vec![1, 0xc1, 1, 0, 0];
    events.extend(event_literal(1));
    for n in 0..alarms.len() {
        assert!(AlarmCodec.decode(&request, &alarms[..n]).is_err());
    }
    for n in 0..events.len() {
        assert!(EventCodec.decode(&request, &events[..n]).is_err());
    }
    for (offset, value) in [(0, 2), (1, 0), (4, 2)] {
        let mut a = alarms.clone();
        a[offset] = value;
        assert!(AlarmCodec.decode(&request, &a).is_err());
        let mut e = events.clone();
        e[offset] = value;
        assert!(EventCodec.decode(&request, &e).is_err());
    }
    let page = AlarmCodec.decode(&request, &alarms).unwrap();
    let checkpoint = page.checkpoint(1).unwrap();
    assert_eq!(checkpoint.index(), 449);
    assert_eq!(checkpoint.next_index(), 0);
    assert!(
        rocsteady::LogCheckpoint::new(450, AlarmRecord::decode(&alarm_literal(1)).unwrap())
            .is_err()
    );
    assert!(
        rocsteady::LogCheckpoint::new(0, AlarmRecord::decode(&alarm_literal(0)).unwrap()).is_err()
    );
    let restored = rocsteady::LogCheckpoint::new(
        checkpoint.index(),
        AlarmRecord::decode(checkpoint.record().raw()).unwrap(),
    )
    .unwrap();
    assert_eq!(restored.record().raw(), checkpoint.record().raw());
    assert!(page.checkpoint(0).is_err());
    assert!(page.checkpoint(2).is_err());
    alarms.push(0);
    assert!(AlarmCodec.decode(&request, &alarms).is_err());
    events.push(0);
    assert!(EventCodec.decode(&request, &events).is_err());
    let empty = LogRequest::new(449, 0).unwrap();
    let page = AlarmCodec.decode(&empty, &[0, 0xc1, 1, 0xc1, 1]).unwrap();
    assert!(page.records().is_empty());
    assert!(page.checkpoint(1).is_err());
    let mut no_alarm = vec![1, 0xc1, 1, 0, 0];
    no_alarm.extend(alarm_literal(0));
    assert!(
        AlarmCodec
            .decode(&request, &no_alarm)
            .unwrap()
            .checkpoint(1)
            .is_err()
    );
    // The returned count may be smaller than requested, but its record boundary remains exact.
    assert!(
        AlarmCodec
            .decode(&LogRequest::new(0, 10).unwrap(), &[0, 0, 0, 0, 0])
            .unwrap()
            .records()
            .is_empty()
    );
}
#[tokio::test]
async fn independent_tcp_reads_all_variants_and_zero_count_headers() {
    let peer = fixture().await;
    let client = client(&peer);
    let a = client
        .read_alarms(LogRequest::new(0, 6).unwrap())
        .await
        .unwrap();
    let e = client
        .read_events(LogRequest::new(0, 9).unwrap())
        .await
        .unwrap();
    for (i, r) in a.records().iter().enumerate() {
        assert_eq!(r.raw(), &alarm_literal(i as u16));
    }
    for (i, r) in e.records().iter().enumerate() {
        assert_eq!(r.raw(), &event_literal(i as u16));
    }
    assert!(
        client
            .read_events(LogRequest::new(449, 0).unwrap())
            .await
            .unwrap()
            .records()
            .is_empty()
    );
    assert!(
        peer.peer
            .requests
            .lock()
            .unwrap()
            .contains(&(118, vec![6, 0, 0]))
    );
    assert!(
        peer.peer
            .requests
            .lock()
            .unwrap()
            .contains(&(119, vec![0, 0xc1, 1]))
    );
    client.stop().await;
}
struct SmallPages;
impl ProtocolProfile for SmallPages {
    fn metadata(&self) -> rocsteady::ProfileMetadata {
        rocsteady::DefaultProfile.metadata()
    }
    fn capability(&self, opcode: u8) -> rocsteady::Capability {
        rocsteady::DefaultProfile.capability(opcode)
    }
    fn limits(&self, opcode: u8) -> rocsteady::PayloadLimits {
        rocsteady::PayloadLimits {
            request: 255,
            response: match opcode {
                118 => 51,
                119 => 49,
                _ => 255,
            },
        }
    }
}
#[tokio::test]
async fn bounded_paging_splits_profile_budgets_and_ring_boundary_and_stops_on_short_page() {
    let peer = fixture().await;
    let client = Client::tcp(
        &peer.endpoint,
        ClientConfig {
            profile: Arc::new(SmallPages),
            heartbeat_interval: None,
            ..Default::default()
        },
    )
    .unwrap();
    let progress = client.read_alarm_range(448, 5).await.unwrap();
    assert_eq!(
        progress
            .pages
            .iter()
            .map(|p| (p.request().start(), p.request().count()))
            .collect::<Vec<_>>(),
        [(448, 2), (0, 2), (2, 1)]
    );
    assert_eq!(progress.records().count(), 5);
    assert!(!progress.anchor_verified);
    peer.peer.short_log_at.store(4, Ordering::SeqCst);
    let events = client.read_event_range(1, 6).await.unwrap();
    assert_eq!(events.pages.len(), 1);
    assert_eq!(events.records().count(), 1);
    assert_eq!(events.checkpoint(1).unwrap().next_index(), 2);
    assert!(events.checkpoint(2).is_err());
    let all = client.read_event_range(449, 450).await.unwrap();
    assert_eq!(all.records().count(), 450);
    assert!(all.pages.len() <= 226);
    assert!(client.read_event_range(0, 451).await.is_err());
    client.stop().await;
}
#[tokio::test]
async fn checkpoint_resume_checks_raw_anchor_including_nan_and_observes_growth() {
    let peer = fixture().await;
    let client = client(&peer);
    let page = client
        .read_alarms(LogRequest::new(2, 1).unwrap())
        .await
        .unwrap();
    let checkpoint = page.checkpoint(1).unwrap(); // FST record contains NaN: compare raw bytes.
    peer.peer.grow_log_at.store(3, Ordering::SeqCst);
    let progress = client.resume_alarms(checkpoint.clone(), 4).await.unwrap();
    assert!(progress.anchor_verified);
    assert_eq!(progress.records().count(), 4);
    assert_eq!(progress.pages[0].current_index(), 1);
    assert_eq!(peer.peer.log_reads.load(Ordering::SeqCst), 4);
    assert!(client.resume_alarms(checkpoint, 450).await.is_err());
    let event = client
        .read_events(LogRequest::new(1, 1).unwrap())
        .await
        .unwrap()
        .checkpoint(1)
        .unwrap();
    assert!(
        client
            .resume_events(event, 0)
            .await
            .unwrap()
            .anchor_verified
    );
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
    client.stop().await;
}
#[tokio::test]
async fn overwritten_or_changed_anchor_reports_gap_without_retiring_healthy_connection() {
    for change_during in [false, true] {
        let peer = fixture().await;
        let client = client(&peer);
        let checkpoint = client
            .read_events(LogRequest::new(1, 1).unwrap())
            .await
            .unwrap()
            .checkpoint(1)
            .unwrap();
        if change_during {
            peer.peer.change_log_at.store(3, Ordering::SeqCst);
            peer.peer.change_log_index.store(1, Ordering::SeqCst);
        } else {
            peer.peer.event_records.lock().unwrap().get_mut(&1).unwrap()[1] ^= 1;
        }
        let error = client.resume_events(checkpoint, 4).await.unwrap_err();
        assert!(matches!(
            error.error,
            etherbird::Error::Operation(Error::LogChanged(_))
        ));
        assert_eq!(error.progress.pages.len(), usize::from(change_during));
        assert!(!error.progress.anchor_verified);
        client.read_clock().await.unwrap();
        assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
        client.stop().await;
    }
}
#[tokio::test]
async fn interrupted_second_page_returns_confirmed_progress_and_explicit_resume_after_reconnect() {
    let peer = fixture().await;
    let client = client(&peer);
    peer.peer.log_fault_at.store(2, Ordering::SeqCst);
    peer.peer.log_fault.store(4, Ordering::SeqCst);
    let error = client.read_alarm_range(0, 25).await.unwrap_err();
    assert_eq!(error.progress.pages.len(), 1);
    assert_eq!(error.progress.records().count(), 10);
    let checkpoint = error.progress.checkpoint(10).unwrap();
    assert_eq!(checkpoint.next_index(), 10);
    client.read_clock().await.unwrap();
    assert_eq!(peer.peer.log_reads.load(Ordering::SeqCst), 2);
    let progress = client.resume_alarms(checkpoint, 15).await.unwrap();
    assert!(progress.anchor_verified);
    assert_eq!(progress.records().count(), 15);
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
    client.stop().await;
}
#[tokio::test]
async fn device_rejections_keep_connection_but_malformed_pages_and_tagged_values_retire() {
    let peer = fixture().await;
    let client = client(&peer);
    peer.peer.log_fault_at.store(1, Ordering::SeqCst);
    peer.peer.log_fault.store(3, Ordering::SeqCst);
    assert!(matches!(
        client.read_alarms(LogRequest::new(1, 1).unwrap()).await,
        Err(etherbird::Error::Operation(Error::Device(_)))
    ));
    client
        .read_alarms(LogRequest::new(1, 1).unwrap())
        .await
        .unwrap();
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 1);
    let mut malformed = event_literal(1);
    malformed[11] = 9;
    malformed[12] = 0;
    peer.peer.event_records.lock().unwrap().insert(1, malformed);
    assert!(matches!(
        client.read_events(LogRequest::new(1, 1).unwrap()).await,
        Err(etherbird::Error::Operation(Error::InvalidResponse(_)))
    ));
    client.read_clock().await.unwrap();
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
    peer.peer.log_fault_at.store(4, Ordering::SeqCst);
    peer.peer.log_fault.store(1, Ordering::SeqCst);
    assert!(
        client
            .read_alarms(LogRequest::new(1, 1).unwrap())
            .await
            .is_err()
    );
    client.read_clock().await.unwrap();
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 3);
    client.stop().await;
}
struct LogPolicy {
    response: usize,
    request: usize,
    disabled: bool,
}
impl ProtocolProfile for LogPolicy {
    fn metadata(&self) -> rocsteady::ProfileMetadata {
        rocsteady::DefaultProfile.metadata()
    }
    fn capability(&self, opcode: u8) -> rocsteady::Capability {
        if self.disabled && matches!(opcode, 118 | 119) {
            rocsteady::Capability::Unavailable
        } else {
            rocsteady::DefaultProfile.capability(opcode)
        }
    }
    fn limits(&self, opcode: u8) -> rocsteady::PayloadLimits {
        if matches!(opcode, 118 | 119) {
            rocsteady::PayloadLimits {
                request: self.request,
                response: self.response,
            }
        } else {
            Default::default()
        }
    }
}
#[tokio::test]
async fn unsupported_services_and_impossible_budgets_fail_before_log_exchange() {
    for policy in [
        LogPolicy {
            response: 4,
            request: 3,
            disabled: false,
        },
        LogPolicy {
            response: 5,
            request: 3,
            disabled: false,
        },
        LogPolicy {
            response: 255,
            request: 2,
            disabled: false,
        },
        LogPolicy {
            response: 255,
            request: 3,
            disabled: true,
        },
    ] {
        let peer = fixture().await;
        let client = Client::tcp(
            &peer.endpoint,
            ClientConfig {
                profile: Arc::new(policy),
                heartbeat_interval: None,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(client.read_alarm_range(0, 1).await.is_err());
        assert!(
            client
                .read_events(LogRequest::new(0, 1).unwrap())
                .await
                .is_err()
        );
        assert_eq!(peer.peer.log_reads.load(Ordering::SeqCst), 0);
        client.stop().await;
    }
}
#[tokio::test]
async fn canceled_log_range_cannot_complete_late_or_replay() {
    let peer = fixture().await;
    let client = client(&peer);
    peer.peer.log_fault_at.store(2, Ordering::SeqCst);
    peer.peer.log_fault.store(5, Ordering::SeqCst);
    let task = tokio::spawn({
        let client = client.clone();
        async move { client.read_event_range(0, 25).await }
    });
    tokio::time::timeout(Duration::from_secs(2), async {
        while peer.peer.log_reads.load(Ordering::SeqCst) < 2 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    client.read_clock().await.unwrap();
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
    assert_eq!(peer.peer.log_reads.load(Ordering::SeqCst), 2);
    client.stop().await;
}
#[tokio::test]
async fn log_deadline_returns_confirmed_pages_and_retires_active_exchange() {
    let peer = fixture().await;
    let client = Client::tcp(
        &peer.endpoint,
        ClientConfig {
            heartbeat_interval: None,
            call_timeout: Duration::from_secs(1),
            response_timeout: Duration::from_secs(2),
            ..Default::default()
        },
    )
    .unwrap();
    peer.peer.log_fault_at.store(2, Ordering::SeqCst);
    peer.peer.log_fault.store(5, Ordering::SeqCst);
    client.connected().await.unwrap();
    let error = client.read_alarm_range(0, 25).await.unwrap_err();
    assert!(matches!(error.error, etherbird::Error::Timeout));
    assert_eq!(error.progress.records().count(), 10);
    // Framework cancellation is asynchronous; observe the replacement before a new attempt.
    tokio::time::timeout(Duration::from_secs(3), async {
        while peer.peer.connections.load(Ordering::SeqCst) < 2 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    client.connected().await.unwrap();
    client.read_clock().await.unwrap();
    assert_eq!(peer.peer.connections.load(Ordering::SeqCst), 2);
    assert_eq!(peer.peer.log_reads.load(Ordering::SeqCst), 2);
    client.stop().await;
}
struct OverrideCodec(Arc<AtomicUsize>);
impl ProtocolProfile for OverrideCodec {
    fn metadata(&self) -> rocsteady::ProfileMetadata {
        rocsteady::DefaultProfile.metadata()
    }
    fn capability(&self, opcode: u8) -> rocsteady::Capability {
        rocsteady::DefaultProfile.capability(opcode)
    }
    fn decode_events(
        &self,
        request: &LogRequest,
        payload: &[u8],
    ) -> Result<rocsteady::EventPage, DecodeError> {
        assert_eq!(request.start(), 1);
        self.0.fetch_add(1, Ordering::SeqCst);
        EventCodec.decode(request, payload)
    }
}
struct ReadBoth;
#[etherbird::async_trait]
impl rocsteady::Workflow for ReadBoth {
    type Output = (rocsteady::AlarmPage, rocsteady::EventPage);
    async fn run(self, context: &mut rocsteady::ManagedContext<'_>) -> Result<Self::Output, Error> {
        Ok((
            context.read_alarms(LogRequest::new(1, 1)?).await?,
            context.read_events(LogRequest::new(1, 1)?).await?,
        ))
    }
}
#[tokio::test]
async fn contextual_profile_codecs_and_managed_and_standalone_interfaces_work() {
    use rocsteady::transport::Connector;
    let peer = fixture().await;
    let calls = Arc::new(AtomicUsize::new(0));
    let client = Client::tcp(
        &peer.endpoint,
        ClientConfig {
            profile: Arc::new(OverrideCodec(calls.clone())),
            heartbeat_interval: None,
            ..Default::default()
        },
    )
    .unwrap();
    let (alarm, event) = client.workflow(ReadBoth).await.unwrap();
    assert_eq!(alarm.records()[0].raw(), &alarm_literal(1));
    assert_eq!(event.records()[0].raw(), &event_literal(1));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    client.stop().await;
    let stream = rocsteady::transport::TcpConnector::new(&peer.endpoint)
        .connect()
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
            .read_events(LogRequest::new(1, 1).unwrap())
            .await
            .unwrap()
            .records()[0]
            .raw(),
        &event_literal(1)
    );
    session
        .read_alarms(LogRequest::new(1, 1).unwrap())
        .await
        .unwrap();
    session.disconnect().await;
}
