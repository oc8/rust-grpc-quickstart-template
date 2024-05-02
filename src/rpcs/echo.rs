use tonic::{Status};
use protos::grpc::examples::echo::{EchoRequest, EchoResponse};
use crate::validations::{validate_echo_request};
use crate::database::PgPooledConnection;

pub fn echo(
    request: EchoRequest,
    _conn: &mut PgPooledConnection,
    _r_conn: &mut redis::Connection,
) -> Result<EchoResponse, Status> {
    validate_echo_request(&request)?;

    Ok(EchoResponse {
        message: request.message,
    })
}