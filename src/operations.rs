//! Shared preparation and decoding for standalone, managed and workflow calls.
use crate::{
    Error, IoPointRequest, Login, Operation, Parameter, ParameterBlock, PointTypeRequest,
    ProtocolProfile,
};
use std::sync::Arc;

/// The decoder remains owned by the request until the exchange guard validates it.
/// Keep preparation generic so forwarding does not box decoder closures.
pub(crate) struct PreparedOperation<D> {
    pub opcode: u8,
    pub payload: Vec<u8>,
    pub decode: D,
    expected_response: Option<usize>,
}
type Preparation<D> = Result<PreparedOperation<D>, Error>;

impl<D> PreparedOperation<D> {
    fn new(opcode: u8, payload: Vec<u8>, expected_response: Option<usize>, decode: D) -> Self {
        Self {
            opcode,
            payload,
            decode,
            expected_response,
        }
    }
    pub fn validate(&self, profile: &dyn ProtocolProfile) -> Result<(), Error> {
        crate::profile::validate_profile_request(profile, self.opcode, &self.payload)?;
        if let Some(size) = self.expected_response {
            crate::profile::validate_expected_response(profile, self.opcode, size)?;
        }
        Ok(())
    }
}
pub(crate) fn custom<O: Operation>(
    operation: O,
) -> Preparation<impl FnOnce(&[u8]) -> Result<O::Output, Error> + Send> {
    let (opcode, payload) = operation.encode()?;
    Ok(PreparedOperation::new(
        opcode,
        payload,
        None,
        move |bytes: &[u8]| operation.decode(bytes).map_err(Error::from),
    ))
}
pub(crate) fn raw(
    opcode: u8,
    payload: Vec<u8>,
) -> PreparedOperation<impl FnOnce(&[u8]) -> Result<Vec<u8>, Error> + Send> {
    PreparedOperation::new(opcode, payload, None, |bytes: &[u8]| Ok(bytes.to_vec()))
}
pub(crate) fn configuration(
    profile: Arc<dyn ProtocolProfile>,
) -> PreparedOperation<impl FnOnce(&[u8]) -> Result<crate::SystemConfiguration, Error> + Send> {
    PreparedOperation::new(6, vec![], None, move |bytes: &[u8]| {
        profile.decode_configuration(bytes).map_err(Error::from)
    })
}
pub(crate) fn clock(
    profile: Arc<dyn ProtocolProfile>,
) -> PreparedOperation<impl FnOnce(&[u8]) -> Result<crate::Clock, Error> + Send> {
    PreparedOperation::new(7, vec![], None, move |bytes: &[u8]| {
        profile.decode_clock(bytes).map_err(Error::from)
    })
}
pub(crate) fn login(
    login: &Login,
) -> PreparedOperation<impl FnOnce(&[u8]) -> Result<(), Error> + Send + use<>> {
    PreparedOperation::new(17, login.payload(), Some(0), |bytes: &[u8]| {
        if bytes.is_empty() {
            Ok(())
        } else {
            Err(Error::InvalidResponse(
                "login acknowledgement must be empty",
            ))
        }
    })
}
pub(crate) fn parameters(
    parameters: &[Parameter],
) -> Preparation<impl FnOnce(&[u8]) -> Result<Vec<crate::Value>, Error> + Send + use<>> {
    let (payload, budget) = crate::protocol::prepare_parameter_request(parameters)?;
    let parameters = parameters.to_vec();
    Ok(PreparedOperation::new(
        180,
        payload,
        Some(budget.response),
        move |bytes: &[u8]| crate::protocol::decode_parameters(&parameters, bytes),
    ))
}
pub(crate) fn block(
    block: &ParameterBlock,
) -> PreparedOperation<impl FnOnce(&[u8]) -> Result<Vec<crate::Value>, Error> + Send + use<>> {
    let block = block.clone();
    PreparedOperation::new(
        167,
        block.payload(),
        Some(block.response_size()),
        move |bytes: &[u8]| block.decode(bytes),
    )
}
pub(crate) fn io(
    request: IoPointRequest,
    profile: Arc<dyn ProtocolProfile>,
) -> PreparedOperation<impl FnOnce(&[u8]) -> Result<crate::IoPointReply, Error> + Send> {
    PreparedOperation::new(
        50,
        request.payload(),
        Some(request.layout.len()),
        move |bytes: &[u8]| {
            profile
                .decode_io_points(&request, bytes)
                .map_err(Error::from)
        },
    )
}
pub(crate) fn point_types(
    request: PointTypeRequest,
    profile: Arc<dyn ProtocolProfile>,
) -> PreparedOperation<impl FnOnce(&[u8]) -> Result<crate::PointTypePage, Error> + Send> {
    PreparedOperation::new(
        100,
        request.payload(),
        Some(request.response_size()),
        move |bytes: &[u8]| {
            profile
                .decode_point_types(&request, bytes)
                .map_err(Error::from)
        },
    )
}
