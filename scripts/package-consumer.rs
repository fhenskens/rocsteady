use rocsteady::{
    Client, ClientConfig, DecodeError, Error, LegacySetup, ManagedContext, Operation, SessionState,
    SetupStrategy, Workflow,
    simulator::{Model, Simulator},
    transport::TcpConnector,
};
use std::sync::Arc;

struct Year;
impl Operation for Year {
    type Output = u16;
    fn encode(&self) -> Result<(u8, Vec<u8>), Error> {
        Ok((7, vec![]))
    }
    fn decode(&self, bytes: &[u8]) -> Result<u16, DecodeError> {
        rocsteady::Clock::decode(bytes)
            .map(|clock| clock.year)
            .map_err(|_| DecodeError("clock fixture"))
    }
}
struct Setup;
#[etherbird::async_trait]
impl SetupStrategy for Setup {
    async fn setup(&self, context: &mut ManagedContext<'_>) -> Result<SessionState, Error> {
        LegacySetup { login: None }.setup(context).await?;
        Ok(Box::new(context.generation()))
    }
}
struct Pair;
struct Catalog;
impl rocsteady::SchemaProvider for Catalog {
    fn lookup(
        &self,
        context: &rocsteady::SchemaContext,
        tlp: rocsteady::Tlp,
    ) -> Result<Option<rocsteady::ParameterSchema>, Error> {
        if context.device_family.as_deref() != Some("consumer-fixture") {
            return Err(Error::CapabilityUnavailable("consumer identity"));
        }
        Ok(
            (tlp == rocsteady::Tlp::new(200, 0, 1)).then_some(rocsteady::ParameterSchema {
                data_type: rocsteady::DataType::UInt16,
                access: rocsteady::Access::ReadOnly,
                logical_range: Some((0, 0)),
                numeric_range: None,
            }),
        )
    }
}
#[etherbird::async_trait]
impl Workflow for Pair {
    type Output = u16;
    async fn run(self, context: &mut ManagedContext<'_>) -> Result<u16, Error> {
        assert_eq!(*context.require_state::<u64>()?, context.generation());
        let handle = context.bind(1u8);
        assert_eq!(*handle.get(context)?, 1);
        context.operation(Year).await?;
        context.operation(Year).await
    }
}
#[tokio::main]
async fn main() {
    let mut model = Model::default();
    model.io_point_types[16] = 103;
    model.io_point_types[48] = 102;
    model.io_point_types[64] = 104;
    model.io_point_types[80] = 105;
    model.parameters.insert(rocsteady::Tlp::new(105,80,4),rocsteady::Value::UInt32(16000000));
    model.parameters.insert(rocsteady::Tlp::new(105,80,6),rocsteady::Value::UInt32(u32::MAX));
    model.parameters.insert(rocsteady::Tlp::new(105,80,10),rocsteady::Value::Float(42.5));
    model.parameters.insert(rocsteady::Tlp::new(105,80,23),rocsteady::Value::Binary(255));
    model.parameters.insert(rocsteady::Tlp::new(105,80,24),rocsteady::Value::Float(123.0));
    model.parameters.insert(rocsteady::Tlp::new(105,80,25),rocsteady::Value::Float(456.0));
    model.parameters.insert(rocsteady::Tlp::new(105,80,26),rocsteady::Value::Float(999999.0));
    model.parameters.insert(rocsteady::Tlp::new(105,80,27),rocsteady::Value::Float(12.5));
    model.parameters.insert(rocsteady::Tlp::new(102,48,25),rocsteady::Value::UInt8(3));
    model.parameters.insert(rocsteady::Tlp::new(102,48,6),rocsteady::Value::Binary(255));
    model.parameters.insert(rocsteady::Tlp::new(104,64,13),rocsteady::Value::UInt16(32768));
    model.parameters.insert(rocsteady::Tlp::new(104,64,16),rocsteady::Value::Float(42.5));
    model.parameters.insert(
        rocsteady::Tlp::new(103, 16, 21),
        rocsteady::Value::Float(12.5),
    );
    model.parameters.insert(
        rocsteady::Tlp::new(136, 0, 5),
        rocsteady::Value::UInt16(2026),
    );
    model.parameters.insert(
        rocsteady::Tlp::new(136, 0, 7),
        rocsteady::Value::Time(0x12345678),
    );
    model.parameters.insert(
        rocsteady::Tlp::new(200, 0, 1),
        rocsteady::Value::UInt16(123),
    );
    model.parameters.insert(rocsteady::Tlp::new(95, 1, 36), rocsteady::Value::UInt16(42));
    model.parameters.insert(rocsteady::Tlp::new(139, 26, 3), rocsteady::Value::Ascii("2.02      ".into()));
    model.parameters.insert(rocsteady::Tlp::new(103, 16, 32), rocsteady::Value::Binary(0xff));
    model.parameters.insert(rocsteady::Tlp::new(103, 16, 39), rocsteady::Value::Float(20.0));
    let mut history=rocsteady::simulator::HistoryMetadataModel::default();
    // Independent Table 2-14 body, including all four five-byte occurrence regions.
    history.history_min_max.insert((0,7),[128,103,16,21,
        0,0,0x48,0x41,0,0,0x80,0xbf,0,0,0xc8,0x42,
        1,2,3,4,5,6,7,8,9,10,0,0,0x20,0xc1,0,0,0x48,0x43,
        11,12,13,14,2,15,16,17,18,12,0,0,0xc0,0x3f]);
    history.history_tags.insert((0,7),*b"FLOW      ");
    history.history_periodic_indices.insert(0,65535);
    // Table 2-18: explicit FLOAT words and TIME words, with a fixture ring of length five.
    history.history_samples.insert((0,7,1),vec![[0,0,0x48,0x41];5]);
    history.history_samples.insert((0,7,3),vec![[0x78,0x56,0x34,0x12];5]);
    history.history_sample_indices.insert((0,1),4);
    history.multi_history_timestamps.insert((0,1),vec![[0x78,0x56,0x34,0x12];5]);
    history.history_samples.insert((0,8,1),vec![[0,0,0x80,0xbf];5]);
    history.history_sample_indices.insert((0,3),4);
    // Literal Table 2-16 parameter alarm and Table 2-17 clock-set event.
    model.alarm_log.insert(0, [0xc1,1,2,3,4,0,103,16,21,
        b'A',b'L',b'A',b'R',b'M',b' ',b' ',b' ',b' ',b' ',0,0,0x48,0x41]);
    model.alarm_index = 0;
    model.event_log.insert(0, [6,1,2,3,4,0x78,0x56,0x34,0x12,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    // Tables 2-20/21 explicit day fields/words; independently chosen fixture geometry.
    history.day_indices.insert((0,15,10),[4,0,3,0,2,0,2,0]);
    history.day_history.insert((0,7,15,10),rocsteady::simulator::DayHistoryData{periodic:vec![[0,0,0x48,0x41];3],daily:vec![[0,0,0x80,0xbf];2]});
    history.day_history.insert((0,255,15,10),rocsteady::simulator::DayHistoryData{periodic:vec![[0x78,0x56,0x34,0x12];3],daily:vec![[1,0,0,0];2]});
    history.history_samples.insert((0,7,2),vec![[0,0,0x80,0xbf];3]);
    history.history_samples.insert((0,7,4),vec![[1,0,0,0];3]);
    history.history_sample_indices.insert((0,2),2);
    history.history_sample_indices.insert((0,4),2);
    history.configured_history_points.insert(0,vec![7,2,8]);
    history.history_samples.insert((0,2,1),vec![[0,0,0,0xc0];5]);
    let server = Simulator::bind_with_history("127.0.0.1:0".parse().unwrap(), model, history)
        .await
        .unwrap();
    let client = Client::with_setup(
        Arc::new(TcpConnector::new(server.endpoint().to_string())),
        ClientConfig {
            heartbeat_interval: None,
            ..Default::default()
        },
        Arc::new(Setup),
    )
    .unwrap();
    assert_eq!(client.operation(Year).await.unwrap(), 2026);
    assert_eq!(client.workflow(Pair).await.unwrap(), 2026);
    assert_eq!(
        client
            .operation(rocsteady::DecodedOperation {
                request: (),
                opcode: 7,
                payload: vec![],
                codec: rocsteady::ClockCodec
            })
            .await
            .unwrap()
            .year,
        2026
    );
    assert_eq!(
        client.profile().capability(180),
        rocsteady::Capability::Verified
    );
    assert_eq!(
        client
            .read_schema(
                &Catalog,
                &rocsteady::SchemaContext {
                    device_family: Some("consumer-fixture".into()),
                    firmware: None
                },
                &[rocsteady::Tlp::new(200, 0, 1)]
            )
            .await
            .unwrap(),
        [rocsteady::Value::UInt16(123)]
    );
    client.stop().await;
    // Public discovery/catalog APIs against the extracted publication payload.
    let discovery_client = Client::tcp(
        server.endpoint().to_string(),
        ClientConfig {
            profile: Arc::new(rocsteady::Roc800Profile {
                layout: rocsteady::IoLayout::Compatibility160,
            }),
            heartbeat_interval: None,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        discovery_client
            .scan_point_types(0, 256)
            .await
            .unwrap()
            .len(),
        2
    );
    let io = discovery_client
        .discover_io(rocsteady::FirmwareVersion::new(3, 10).unwrap())
        .await
        .unwrap();
    let ai = io.by_logical(103, 0).unwrap().tlp(21).unwrap();
    assert_eq!(ai, rocsteady::Tlp::new(103, 16, 21));
    let catalog = rocsteady::Roc800Catalog::from_discovery(io).unwrap();
    assert_eq!(
        discovery_client
            .read_schema(&catalog, &catalog.context(), &[
                ai, rocsteady::Tlp::new(136, 0, 5), rocsteady::Tlp::new(136, 0, 7),
                rocsteady::Tlp::new(95, 1, 36), rocsteady::Tlp::new(139, 26, 3),
                rocsteady::Tlp::new(103, 16, 32), rocsteady::Tlp::new(103, 16, 39)
            ])
            .await
            .unwrap(),
        [rocsteady::Value::Float(12.5), rocsteady::Value::UInt16(2026),
         rocsteady::Value::Time(0x12345678), rocsteady::Value::UInt16(42),
         rocsteady::Value::Ascii("2.02      ".into()), rocsteady::Value::Binary(0xff),
         rocsteady::Value::Float(20.0)]
    );
    // REQ-OUTPUT-PACKAGE: Tables 3-14/3-16 through the packaged public API.
    assert_eq!(discovery_client.read_schema(&catalog,&catalog.context(),&[
        rocsteady::Tlp::new(102,48,25),rocsteady::Tlp::new(102,48,6),
        rocsteady::Tlp::new(104,64,13),rocsteady::Tlp::new(104,64,16)
    ]).await.unwrap(),[rocsteady::Value::UInt8(3),rocsteady::Value::Binary(255),
        rocsteady::Value::UInt16(32768),rocsteady::Value::Float(42.5)]);
    assert_eq!(rocsteady::Roc800Catalog::entry(102,25).unwrap().metadata.enum_meaning(3).unwrap().key,"relay_6_point");
    assert_eq!(rocsteady::Roc800Catalog::entry(104,6).unwrap().metadata.uninterpreted_bits(255),0x1f);
    // REQ-PI-PACKAGE: Table 3-17 through the extracted package's public schema API.
    let pulse_tlps: Vec<_> = [4,6,10,23,24,25,26,27].into_iter()
        .map(|p| rocsteady::Tlp::new(105,80,p)).collect();
    assert_eq!(discovery_client.read_schema(&catalog,&catalog.context(),&pulse_tlps).await.unwrap(),[
        rocsteady::Value::UInt32(16000000),rocsteady::Value::UInt32(u32::MAX),
        rocsteady::Value::Float(42.5),rocsteady::Value::Binary(255),
        rocsteady::Value::Float(123.0),rocsteady::Value::Float(456.0),
        rocsteady::Value::Float(999999.0),rocsteady::Value::Float(12.5)]);
    assert_eq!(rocsteady::Roc800Catalog::entry(105,7).unwrap().metadata.enum_meaning(2).unwrap().key,"eu_per_hour");
    assert_eq!(rocsteady::Roc800Catalog::entry(105,23).unwrap().metadata.uninterpreted_bits(255),0x20);
    assert!(rocsteady::Roc800Catalog::entry(105,9).unwrap().metadata.write_caveat.is_some());
    let alarms = rocsteady::Roc800Catalog::entry(103, 32).unwrap().metadata;
    assert_eq!(alarms.active_bits(255).count(), 7);
    assert_eq!(alarms.uninterpreted_bits(255), 0x20);
    let cal = rocsteady::Roc800Catalog::entry(103, 34).unwrap().metadata;
    assert_eq!(cal.enum_meaning(4).unwrap().key, "stop");
    assert!(cal.enum_meaning(255).is_none());
    assert!(cal.write_caveat.is_some());
    // REQ-HISTORY-PACKAGE: usable defaults, prebuilt and external codec choices.
    let hq=rocsteady::MinMaxRequest::new(0,7).unwrap();
    let hr=discovery_client.read_min_max(hq).await.unwrap();
    assert_eq!(hr.current,12.5);
    assert!(matches!(hr.today.minimum_time,rocsteady::OccurrenceTime::Raw([1,2,3,4,5])));
    let calendar=discovery_client.read_min_max_with_codec(hq,rocsteady::MinMaxCodec(rocsteady::CalendarOccurrenceCodec)).await.unwrap();
    assert!(matches!(calendar.today.minimum_time,rocsteady::OccurrenceTime::Calendar {month:5,..}));
    let epoch=discovery_client.read_min_max_with_codec(hq,rocsteady::MinMaxCodec(rocsteady::PrefixedEpochOccurrenceCodec)).await.unwrap();
    assert!(matches!(epoch.today.minimum_time,rocsteady::OccurrenceTime::PrefixedEpoch {seconds:0x05040302,..}));
    struct CustomOccurrence;
    impl rocsteady::ReplyCodec<()> for CustomOccurrence {
        type Output=u8;
        fn decode(&self,_:&(),b:&[u8])->Result<u8,rocsteady::DecodeError> {b.first().copied().ok_or(rocsteady::DecodeError("missing occurrence"))}
    }
    assert_eq!(discovery_client.read_min_max_with_codec(hq,rocsteady::MinMaxCodec(CustomOccurrence)).await.unwrap().today.minimum_time,1);
    let tags=discovery_client.read_history_tags(rocsteady::HistoryTagsRequest::new(0,vec![7,7]).unwrap()).await.unwrap();
    assert_eq!(tags.periodic_index,65535);assert_eq!(tags.tags[0].tag.0,*b"FLOW      ");
    // REQ-139-PACKAGE: inventory, selected point order/duplicates, optional times,
    // public codecs/Operation and bounded generation-exclusive inventory rechecks.
    let inventory_request=rocsteady::ConfiguredHistoryRequest::new(0).unwrap();
    assert_eq!(discovery_client.read_configured_history(inventory_request).await.unwrap().points,[7,2,8]);
    let inventory_op=rocsteady::DecodedOperation{request:inventory_request,opcode:139,payload:vec![0,0],codec:rocsteady::ConfiguredHistoryCodec};
    assert_eq!(discovery_client.operation(inventory_op).await.unwrap().points,[7,2,8]);
    let selected=rocsteady::SelectedHistoryRequest::new(0,0,rocsteady::HistoryKind::Periodic,vec![2,7,2],2,true).unwrap();
    let selected_page=discovery_client.read_selected_history(selected.clone()).await.unwrap();
    assert_eq!(selected_page.periods[0].values,[-2.0,12.5,-2.0]);assert_eq!(selected_page.periods[0].timestamp,Some(0x12345678));
    struct SelectedCount;
    impl rocsteady::ReplyCodec<rocsteady::SelectedHistoryRequest> for SelectedCount{
        type Output=usize;
        fn decode(&self,q:&rocsteady::SelectedHistoryRequest,b:&[u8])->Result<usize,rocsteady::DecodeError>{Ok(rocsteady::ReplyCodec::decode(&rocsteady::SelectedHistoryCodec,q,b)?.periods.len())}
    }
    assert_eq!(discovery_client.read_selected_history_with_codec(selected,SelectedCount).await.unwrap(),2);
    let selected_geometry=rocsteady::ConfiguredHistoryPolicy::new(10,200,60,5,3).unwrap();
    let selected_window=rocsteady::SelectedHistoryRange::with_policy(0,4,rocsteady::HistoryKind::Periodic,vec![2,7,2],5,true,&selected_geometry).unwrap().with_inventory_check(true);
    let selected_progress=discovery_client.read_selected_history_range(selected_window).await.unwrap();
    assert!(selected_progress.complete);assert_eq!(selected_progress.pages.len(),2);assert_eq!(selected_progress.remaining,0);assert!(selected_progress.inventory_after.is_some());
    let without_times=rocsteady::SelectedHistoryRequest::new(0,0,rocsteady::HistoryKind::Periodic,vec![7],1,false).unwrap();
    assert_eq!(discovery_client.read_selected_history(without_times).await.unwrap().periods[0].timestamp,None);
    // REQ-DAY-PACKAGE / POL-DAY-EXTENSION: source defaults and caller interpretation use the same public codecs.
    let date=rocsteady::HistoryDay::new(15,10).unwrap();
    let iq=rocsteady::DayIndicesRequest::new(0,date).unwrap();
    let indices=discovery_client.read_day_indices(iq).await.unwrap();
    assert_eq!((indices.periodic_start,indices.periodic_count,indices.daily_start,indices.daily_count),(4,3,Some(2),2));
    // Request-layout replacements can use Operation/DecodedOperation; this explicit
    // example still follows Table 2-20 instead of inventing a disputed extra point field.
    let op=rocsteady::DecodedOperation{request:iq,opcode:137,payload:vec![0,15,10],codec:rocsteady::DayIndicesCodec};
    assert_eq!(discovery_client.operation(op).await.unwrap().periodic_count,3);
    let day=rocsteady::DayHistoryRequest::new(0,date,rocsteady::DayHistoryTarget::Values(7)).unwrap();
    let reply=discovery_client.read_history_day(day).await.unwrap();
    assert_eq!(reply.periodic,rocsteady::HistorySamples::Values(vec![12.5;3]));
    assert_eq!(reply.daily,rocsteady::HistorySamples::Values(vec![-1.0;2]));
    struct LabelledDay(u16);
    impl rocsteady::ReplyCodec<rocsteady::DayHistoryRequest> for LabelledDay{
        type Output=(u16,rocsteady::DayHistoryReply);
        fn decode(&self,q:&rocsteady::DayHistoryRequest,b:&[u8])->Result<Self::Output,rocsteady::DecodeError>{Ok((self.0,rocsteady::ReplyCodec::decode(&rocsteady::DayHistoryCodec,q,b)?))}
    }
    assert_eq!(discovery_client.read_history_day_with_codec(day,LabelledDay(2026)).await.unwrap().0,2026);
    let timestamp_day=rocsteady::DayHistoryRequest::new(0,date,rocsteady::DayHistoryTarget::Timestamps).unwrap();
    assert_eq!(discovery_client.read_history_day(timestamp_day).await.unwrap().daily,rocsteady::HistorySamples::Timestamps(vec![1;2]));
    let geometry=std::sync::Arc::new(rocsteady::ConfiguredHistoryPolicy::new(10,200,60,5,3).unwrap());
    for plan in [rocsteady::IndexedDayHistoryRequest::new(day,geometry.clone()),rocsteady::IndexedDayHistoryRequest::new(timestamp_day,geometry).with_timestamp_point(7)]{
        let progress=discovery_client.read_history_day_indexed(plan).await.unwrap();assert!(progress.complete);assert!(progress.indices_after.is_some());assert_eq!(progress.periodic_remaining,Some(0));assert_eq!(progress.daily_remaining,Some(0));
    }
    // REQ-136-PACKAGE: typed time-major rows, custom codec and bounded range through public API.
    let multi=rocsteady::MultiHistoryRequest::new(0,0,rocsteady::HistoryKind::Periodic,7,2,2).unwrap();
    let page=discovery_client.read_multi_history(multi).await.unwrap();
    assert_eq!(page.periods.len(),2);assert_eq!(page.periods[0].timestamp,0x12345678);
    assert_eq!(page.periods[0].values,[12.5,-1.0]);
    struct MultiCodec;
    impl rocsteady::ReplyCodec<rocsteady::MultiHistoryRequest> for MultiCodec {
        type Output=usize;
        fn decode(&self,q:&rocsteady::MultiHistoryRequest,b:&[u8])->Result<usize,rocsteady::DecodeError>{
            Ok(rocsteady::ReplyCodec::decode(&rocsteady::MultiHistoryCodec,q,b)?.periods.len())
        }
    }
    assert_eq!(discovery_client.read_multi_history_with_codec(multi,MultiCodec).await.unwrap(),2);
    let multi_geometry=rocsteady::ConfiguredHistoryPolicy::new(10,200,60,5,3).unwrap();
    let multi_window=rocsteady::MultiHistoryRange::with_policy(0,4,rocsteady::HistoryKind::Periodic,7,2,5,&multi_geometry).unwrap();
    let multi_progress=discovery_client.read_multi_history_range(multi_window).await.unwrap();
    assert!(multi_progress.complete);assert_eq!(multi_progress.pages.len(),2);assert_eq!(multi_progress.next_index,Some(4));
    // REQ-135-PACKAGE: default pages, timestamps, explicit geometry and bounded paging.
    let hp=discovery_client.read_history(rocsteady::HistoryRequest::new(0,7,rocsteady::HistoryKind::Periodic,0,2).unwrap()).await.unwrap();
    assert_eq!(hp.samples,rocsteady::HistorySamples::Values(vec![12.5;2]));
    let ht=discovery_client.read_history(rocsteady::HistoryRequest::new(0,7,rocsteady::HistoryKind::PeriodicTimestamps,0,1).unwrap()).await.unwrap();
    assert_eq!(ht.samples,rocsteady::HistorySamples::Timestamps(vec![0x12345678]));
    let geometry=rocsteady::ConfiguredHistoryPolicy::new(10,200,60,5,3).unwrap();
    let window=rocsteady::HistoryRange::with_policy(0,7,rocsteady::HistoryKind::Periodic,4,5,&geometry).unwrap();
    let history_progress=discovery_client.read_history_range(window).await.unwrap();
    assert!(history_progress.complete);assert_eq!(history_progress.pages.len(),2);assert_eq!(history_progress.next_index,Some(4));
    struct RawHistory;
    impl rocsteady::ReplyCodec<rocsteady::HistoryRequest> for RawHistory {
        type Output=usize;
        fn decode(&self,q:&rocsteady::HistoryRequest,b:&[u8])->Result<usize,rocsteady::DecodeError> {
            Ok(rocsteady::ReplyCodec::decode(&rocsteady::HistoryCodec,q,b)?.samples.len())
        }
    }
    assert_eq!(discovery_client.read_history_with_codec(rocsteady::HistoryRequest::new(0,7,rocsteady::HistoryKind::Periodic,0,1).unwrap(),RawHistory).await.unwrap(),1);
    let alarms = discovery_client.read_alarm_range(0, 1).await.unwrap();
    assert_eq!(alarms.records().count(), 1);
    let checkpoint = alarms.checkpoint(1).unwrap();
    assert!(discovery_client.resume_alarms(checkpoint.clone(), 0).await.unwrap().anchor_verified);
    server.raise_srbx(0).unwrap();
    let mut poller = discovery_client.srbx_poller(std::time::Duration::from_millis(1)).unwrap();
    assert_eq!(poller.poll().await.unwrap().current_alarm_index, 0);
    assert_eq!(discovery_client.acknowledge_alarms(&checkpoint).await.unwrap().index, 0);
    assert!(!server.srbx_status().pending);
    let events = discovery_client.read_events(rocsteady::LogRequest::new(0, 1).unwrap()).await.unwrap();
    assert!(matches!(events.records()[0].data(), rocsteady::EventData::ClockSet { time: 0x12345678 }));
    discovery_client.stop().await;
    server.stop().await.unwrap();
    println!("packaged external consumer passed");
}
