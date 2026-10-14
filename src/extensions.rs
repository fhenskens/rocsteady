//! Scoped managed extension interfaces.
use crate::{DecodeError, Error, Session};
use std::any::Any;

/// Exclusively owned state, installed only after successful setup.
pub type SessionState = Box<dyn Any + Send + Sync>;

/// Encode validates locally; decode runs before the exchange retirement guard completes.
pub trait Operation: Send + Sync + 'static {
    type Output: Send + 'static;
    fn encode(&self) -> Result<(u8, Vec<u8>), Error>;
    fn decode(&self, payload: &[u8]) -> Result<Self::Output, DecodeError>;
}
/// Exclusive context for one generation. No transport handle can escape.
///
/// ```compile_fail
/// use rocsteady::ManagedContext;
/// fn escape(context: &mut ManagedContext<'_>) -> &'static u64 {
///     context.state::<u64>().unwrap()
/// }
/// ```
pub struct ManagedContext<'a> {
    pub(crate) session: &'a Session,
    pub(crate) complete: bool,
    pub(crate) state: tokio::sync::MutexGuard<'a, Option<SessionState>>,
    pub(crate) _ownership: tokio::sync::MutexGuard<'a, ()>,
}
impl ManagedContext<'_> {
    pub async fn read_configuration(&mut self) -> Result<crate::SystemConfiguration, Error> {
        self.exchange_operation(crate::operations::configuration(
            self.session.profile.clone(),
        ))
        .await
    }
    pub async fn read_io_points(
        &mut self,
        request: crate::IoPointRequest,
    ) -> Result<crate::IoPointReply, Error> {
        self.exchange_operation(crate::operations::io(request, self.session.profile.clone()))
            .await
    }
    pub async fn read_point_types(
        &mut self,
        request: crate::PointTypeRequest,
    ) -> Result<crate::PointTypePage, Error> {
        self.exchange_operation(crate::operations::point_types(
            request,
            self.session.profile.clone(),
        ))
        .await
    }
    pub fn profile(&self) -> &dyn crate::ProtocolProfile {
        self.session.profile.as_ref()
    }
    pub async fn read_alarms(
        &mut self,
        request: crate::LogRequest,
    ) -> Result<crate::AlarmPage, Error> {
        self.exchange_operation(crate::operations::logs::<crate::AlarmRecord>(
            request,
            self.session.profile.clone(),
        ))
        .await
    }
    pub async fn read_events(
        &mut self,
        request: crate::LogRequest,
    ) -> Result<crate::EventPage, Error> {
        self.exchange_operation(crate::operations::logs::<crate::EventRecord>(
            request,
            self.session.profile.clone(),
        ))
        .await
    }
    pub async fn read_clock(&mut self) -> Result<crate::Clock, Error> {
        self.exchange_operation(crate::operations::clock(self.session.profile.clone()))
            .await
    }
    pub fn generation(&self) -> u64 {
        self.session.generation
    }
    pub fn state<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.state.as_ref()?.as_ref().downcast_ref()
    }
    /// Fail clearly when a workflow's required setup state is absent.
    pub fn require_state<T: Any + Send + Sync>(&self) -> Result<&T, Error> {
        self.state()
            .ok_or(Error::CapabilityUnavailable("workflow setup state"))
    }
    /// Bind a remote handle or progress marker to the current generation.
    pub fn bind<T>(&self, value: T) -> GenerationValue<T> {
        GenerationValue {
            generation: self.generation(),
            value,
        }
    }
    pub async fn exchange(&mut self, opcode: u8, data: Vec<u8>) -> Result<Vec<u8>, Error> {
        self.exchange_operation(crate::operations::raw(opcode, data))
            .await
    }
    pub async fn operation<O: Operation>(&mut self, operation: O) -> Result<O::Output, Error> {
        self.exchange_operation(crate::operations::custom(operation)?)
            .await
    }
    pub(crate) async fn exchange_operation<T, D>(
        &mut self,
        operation: crate::operations::PreparedOperation<D>,
    ) -> Result<T, Error>
    where
        D: FnOnce(&[u8]) -> Result<T, Error>,
    {
        self.session.exchange_operation(operation).await
    }
}
/// One exclusive attempt. Never replayed across replacement.
#[etherbird::async_trait]
pub trait Workflow: Send + 'static {
    type Output: Send + 'static;
    async fn run(self, context: &mut ManagedContext<'_>) -> Result<Self::Output, Error>;
}
/// Runs before readiness on every fresh connection, bounded by setup_timeout.
/// Return only generation-local state. Fetch credentials here; never log secrets.
#[etherbird::async_trait]
pub trait SetupStrategy: Send + Sync + 'static {
    async fn setup(&self, context: &mut ManagedContext<'_>) -> Result<SessionState, Error>;
}
/// Legacy login and clock readiness, using the same managed boundary as custom setup.
pub struct LegacySetup {
    pub login: Option<crate::Login>,
}
#[etherbird::async_trait]
impl SetupStrategy for LegacySetup {
    async fn setup(&self, context: &mut ManagedContext<'_>) -> Result<SessionState, Error> {
        if let Some(login) = &self.login {
            context
                .exchange_operation(crate::operations::login(login))
                .await
                .map_err(|error| match error {
                    Error::Device(_) => {
                        Error::Authentication(crate::AuthenticationFailure::Rejected)
                    }
                    error => error,
                })?;
        }
        context.read_clock().await?;
        Ok(Box::new(()))
    }
}

/// A handle is accessible only in the generation that created it. No Debug output
/// exposes the value. Extensions must not extract/copy it for unchecked reuse.
pub struct GenerationValue<T> {
    generation: u64,
    value: T,
}
impl<T> GenerationValue<T> {
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn get<'a>(&'a self, context: &'a ManagedContext<'_>) -> Result<&'a T, Error> {
        if self.generation != context.generation() {
            return Err(Error::StaleGeneration {
                expected: self.generation,
                actual: context.generation(),
            });
        }
        if !context.session.is_connected() {
            return Err(Error::Disconnected);
        }
        Ok(&self.value)
    }
}

impl Drop for ManagedContext<'_> {
    fn drop(&mut self) {
        if !self.complete {
            self.session.connected.send_replace(false);
        }
    }
}
