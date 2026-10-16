//! OP-8, D301180X012 §2.4/Table 2-4, printed p.2-11.
use crate::{Client, Clock, DecodeError, Error, Operation, WriteOutcome};

/// A validated calendar without a weekday. The controller computes weekday and
/// clears microseconds. No timezone conversion, save or read-back is automatic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockSet {
    bytes: [u8; 7],
}
impl ClockSet {
    /// Gregorian calendar policy matches Clock reads: years 1–65535, seconds 0–59.
    /// The wire year width does not establish a controller's actual accepted range.
    pub fn new(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, Error> {
        let clock = Clock {
            year,
            month,
            day,
            hour,
            minute,
            second,
            weekday: 1,
        };
        let bytes = clock
            .encode()
            .map_err(|_| Error::InvalidRequest("invalid clock-set calendar"))?;
        Ok(Self {
            bytes: bytes[..7].try_into().unwrap(),
        })
    }
    pub fn payload(self) -> [u8; 7] {
        self.bytes
    }
    #[cfg(feature = "simulator")]
    pub(crate) fn simulated_clock(self) -> Clock {
        let [second, minute, hour, day, month, lo, hi] = self.bytes;
        let year = u16::from_le_bytes([lo, hi]);
        // Gregorian weekday, Sunday=1. Fixture calculation, not a wire field.
        let y = i32::from(year) - i32::from(month < 3);
        let offsets = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let weekday =
            ((y + y / 4 - y / 100 + y / 400 + offsets[usize::from(month) - 1] + i32::from(day)) % 7
                + 1) as u8;
        Clock {
            year,
            month,
            day,
            hour,
            minute,
            second,
            weekday,
        }
    }
}
impl Operation for ClockSet {
    type Output = ();
    fn encode(&self) -> Result<(u8, Vec<u8>), Error> {
        Ok((8, self.bytes.to_vec()))
    }
    fn decode(&self, bytes: &[u8]) -> Result<(), DecodeError> {
        if bytes.is_empty() {
            Ok(())
        } else {
            Err(DecodeError("clock-set acknowledgement must be empty"))
        }
    }
}
#[derive(Debug, thiserror::Error)]
#[error("clock set failed ({outcome:?}): {error}")]
pub struct ClockSetError {
    pub outcome: WriteOutcome,
    #[source]
    pub error: etherbird::Error<Error>,
}
impl Client {
    /// Set once, with a synchronized send gate and explicit remote uncertainty.
    /// A valid device rejection does not establish that no clock change occurred.
    pub async fn set_clock(&self, clock: ClockSet) -> Result<WriteOutcome, ClockSetError> {
        let plan = crate::WritePlan::single(8, clock.payload().to_vec());
        self.write_plan(plan)
            .await
            .map(|p| p.batches[0])
            .map_err(|failure| ClockSetError {
                outcome: failure.progress.batches[0],
                error: failure.error,
            })
    }
}
impl crate::Session {
    /// Standalone exchange; any error after transmission may leave remote effects.
    pub async fn set_clock(&self, clock: ClockSet) -> Result<(), Error> {
        self.request_operation(crate::operations::write_batch(8, clock.payload().to_vec()))
            .await
    }
}
impl crate::ManagedContext<'_> {
    /// Use inside an exclusive workflow; failures do not trigger replay.
    pub async fn set_clock(&mut self, clock: ClockSet) -> Result<(), Error> {
        self.operation(clock).await
    }
}
