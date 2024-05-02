use tonic::{Code, Status};
use protos::grpc::examples::echo::EchoRequest;
use crate::errors;

pub fn validate_echo_request(req: &EchoRequest) -> Result<(), Status> {
    if req.message.is_empty() {
        return Err(Status::new(Code::InvalidArgument, errors::INVALID_MESSAGE));
    }

    Ok(())
}
