//! Explicit consumed-alarm acknowledgements and caller-driven idle polling.
use crate::{AlarmRecord, Client, Error, LogCheckpoint, LogRequest, ManagedContext};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    sync::{broadcast, watch},
    time::{Instant, sleep_until},
};

pub const SRBX_BUFFER_CAPACITY: usize = 32;

/// Confirmation of the empty opcode-225 reply. It does not confirm SRBX was cleared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlarmAcknowledgement {
    pub index: u16,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmAcknowledgementOutcome {
    NotAttempted,
    Rejected,
    Confirmed,
    /// Send was permitted but no valid acceptance/rejection was observed; may have applied.
    Unconfirmed,
}
#[derive(Debug, thiserror::Error)]
#[error("alarm acknowledgement at index {index} failed ({outcome:?}): {error}")]
pub struct AlarmAcknowledgementError<E = etherbird::Error<Error>> {
    pub index: u16,
    pub outcome: AlarmAcknowledgementOutcome,
    #[source]
    pub error: E,
}
struct AckState {
    canceled: bool,
    outcome: AlarmAcknowledgementOutcome,
}
impl Default for AckState {
    fn default() -> Self {
        Self {
            canceled: false,
            outcome: AlarmAcknowledgementOutcome::NotAttempted,
        }
    }
}
struct AcknowledgeAlarm {
    checkpoint: LogCheckpoint<AlarmRecord>,
    state: Arc<Mutex<AckState>>,
}
fn validate_ack(
    checkpoint: &LogCheckpoint<AlarmRecord>,
    profile: &dyn crate::ProtocolProfile,
) -> Result<(), Error> {
    crate::profile::validate_profile_request_size(profile, 118, 3)?;
    crate::profile::validate_expected_response(profile, 118, 28)?;
    crate::operations::alarm_ack(checkpoint.index()).validate(profile)
}
async fn acknowledge(
    context: &mut ManagedContext<'_>,
    checkpoint: &LogCheckpoint<AlarmRecord>,
    state: &Mutex<AckState>,
) -> Result<AlarmAcknowledgement, Error> {
    validate_ack(checkpoint, context.profile())?;
    let page = context
        .read_alarms(LogRequest::new(checkpoint.index(), 1)?)
        .await?;
    if page.records().first().map(AlarmRecord::raw) != Some(checkpoint.record().raw()) {
        return Err(Error::LogChanged(
            "consumed alarm changed or is unavailable before acknowledgement",
        ));
    }
    {
        // Close the send gate on a caller deadline/stop before reporting NotAttempted.
        // A late-canceled framework job can never start opcode 225 after that report.
        let mut state = state.lock().unwrap();
        if state.canceled {
            return Err(Error::Disconnected);
        }
        state.outcome = AlarmAcknowledgementOutcome::Unconfirmed;
    }
    let result = context
        .exchange_operation(crate::operations::alarm_ack(checkpoint.index()))
        .await;
    let mut state = state.lock().unwrap();
    match &result {
        Ok(()) => state.outcome = AlarmAcknowledgementOutcome::Confirmed,
        Err(Error::Device(_)) => state.outcome = AlarmAcknowledgementOutcome::Rejected,
        Err(error) if error.kind() == crate::ErrorKind::LocalValidation => {
            state.outcome = AlarmAcknowledgementOutcome::NotAttempted
        }
        Err(_) => {}
    }
    result.map(|()| AlarmAcknowledgement {
        index: checkpoint.index(),
    })
}
#[etherbird::async_trait]
impl crate::Workflow for AcknowledgeAlarm {
    type Output = AlarmAcknowledgement;
    async fn run(self, context: &mut ManagedContext<'_>) -> Result<Self::Output, Error> {
        acknowledge(context, &self.checkpoint, &self.state).await
    }
}
impl Client {
    /// Explicitly acknowledge a caller-consumed checkpoint for the same intended controller.
    /// The caller confirms all outstanding alarms through it have been processed.
    /// Anchor validation cannot prove ingestion completeness or controller identity.
    /// One attempt verifies its raw anchor, then sends its index; never auto-replayed.
    ///
    /// ```compile_fail
    /// use rocsteady::{Client, EventRecord, LogCheckpoint};
    /// fn wrong_log(client: &Client, consumed: &LogCheckpoint<EventRecord>) {
    ///     let _ = client.acknowledge_alarms(consumed);
    /// }
    /// ```
    pub async fn acknowledge_alarms(
        &self,
        checkpoint: &LogCheckpoint<AlarmRecord>,
    ) -> Result<AlarmAcknowledgement, AlarmAcknowledgementError> {
        let index = checkpoint.index();
        validate_ack(checkpoint, self.profile()).map_err(|error| AlarmAcknowledgementError {
            index,
            outcome: AlarmAcknowledgementOutcome::NotAttempted,
            error: etherbird::Error::Operation(error),
        })?;
        let state = Arc::new(Mutex::new(AckState::default()));
        // Dropping this caller future must close the gate too, even before the worker is canceled.
        let mut guard = AckGate(state.clone());
        let result = self
            .workflow(AcknowledgeAlarm {
                checkpoint: checkpoint.clone(),
                state,
            })
            .await;
        let outcome = guard.close();
        result.map_err(|error| AlarmAcknowledgementError {
            index,
            outcome,
            error,
        })
    }
}
struct AckGate(Arc<Mutex<AckState>>);
impl AckGate {
    fn close(&mut self) -> AlarmAcknowledgementOutcome {
        let mut state = self.0.lock().unwrap();
        state.canceled = true;
        state.outcome
    }
}
impl Drop for AckGate {
    fn drop(&mut self) {
        self.close();
    }
}
impl ManagedContext<'_> {
    pub async fn acknowledge_alarms(
        &mut self,
        checkpoint: &LogCheckpoint<AlarmRecord>,
    ) -> Result<AlarmAcknowledgement, AlarmAcknowledgementError<Error>> {
        let state = Mutex::new(AckState::default());
        let result = acknowledge(self, checkpoint, &state).await;
        result.map_err(|error| AlarmAcknowledgementError {
            index: checkpoint.index(),
            outcome: state.lock().unwrap().outcome,
            error,
        })
    }
}
impl crate::Session {
    pub async fn acknowledge_alarms(
        &self,
        checkpoint: &LogCheckpoint<AlarmRecord>,
    ) -> Result<AlarmAcknowledgement, AlarmAcknowledgementError<Error>> {
        validate_ack(checkpoint, self.profile.as_ref()).map_err(|error| {
            AlarmAcknowledgementError {
                index: checkpoint.index(),
                outcome: AlarmAcknowledgementOutcome::NotAttempted,
                error,
            }
        })?;
        let mut context = self.managed().await;
        let result = context.acknowledge_alarms(checkpoint).await;
        context.complete = true;
        result
    }
}

/// Bounded raw notification delivery. Lag counts are advisory, never durable log counts.
#[derive(Debug, Default)]
pub struct SrbxNotifications {
    pub signals: Vec<crate::protocol::Frame>,
    pub lagged: u64,
}
#[derive(Debug)]
pub struct SrbxPoll {
    pub current_alarm_index: u16,
    pub notifications: SrbxNotifications,
}
#[derive(Debug, thiserror::Error)]
#[error("SRBX poll failed: {error}")]
pub struct SrbxPollError {
    pub notifications: SrbxNotifications,
    #[source]
    pub error: etherbird::Error<Error>,
}
/// Opt-in, caller-driven polling; no background task or acknowledgement is spawned.
/// First poll is immediate. Later polls wait interval after the previous completion.
/// Each index-only exchange has the Client's ordinary admission/call/wire deadlines.
pub struct SrbxPoller {
    client: Client,
    events: broadcast::Receiver<crate::protocol::Frame>,
    stopping: watch::Receiver<bool>,
    interval: Duration,
    due: Option<Instant>,
}
impl SrbxPoller {
    pub(crate) fn new(
        client: Client,
        stopping: watch::Receiver<bool>,
        interval: Duration,
    ) -> Result<Self, Error> {
        if interval.is_zero() || Instant::now().checked_add(interval).is_none() {
            return Err(Error::InvalidConfig(
                "SRBX polling interval must be positive and representable",
            ));
        }
        crate::profile::validate_profile_request_size(client.profile(), 118, 3)?;
        crate::profile::validate_expected_response(client.profile(), 118, 5)?;
        let events = client.subscribe_events();
        Ok(Self {
            client,
            events,
            stopping,
            interval,
            due: Some(Instant::now()),
        })
    }
    fn drain(&mut self) -> SrbxNotifications {
        let mut received = SrbxNotifications::default();
        // A producer storm cannot turn draining into an unbounded synchronous loop.
        for _ in 0..=SRBX_BUFFER_CAPACITY {
            if received.signals.len() == SRBX_BUFFER_CAPACITY {
                break;
            }
            match self.events.try_recv() {
                Ok(frame) => received.signals.push(frame),
                Err(broadcast::error::TryRecvError::Lagged(n)) => {
                    received.lagged = received.lagged.saturating_add(n)
                }
                Err(_) => break,
            }
        }
        received
    }
    pub async fn poll(&mut self) -> Result<SrbxPoll, SrbxPollError> {
        let result = async {
            if *self.stopping.borrow_and_update() {
                return Err(etherbird::Error::Stopped);
            }
            let due = self
                .due
                .ok_or(etherbird::Error::Operation(Error::InvalidConfig(
                    "SRBX polling interval is no longer representable",
                )))?;
            tokio::select! {
                biased;
                _ = self.stopping.changed() => return Err(etherbird::Error::Stopped),
                _ = sleep_until(due) => {}
            }
            self.client
                .read_alarms(LogRequest::new(0, 0).expect("fixed index-only request"))
                .await
        }
        .await;
        self.due = Instant::now().checked_add(self.interval);
        let notifications = self.drain();
        match result {
            Ok(page) => Ok(SrbxPoll {
                current_alarm_index: page.current_index(),
                notifications,
            }),
            Err(error) => Err(SrbxPollError {
                notifications,
                error,
            }),
        }
    }
}
