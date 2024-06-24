use protos::echo::v1::{UnaryEchoRequest, UnaryEchoResponse};
use crate::database::PgPooledConnection;
use crate::errors::ApiError;
use crate::utils::validation::ValidateRequest;

pub async fn echo(
    request: UnaryEchoRequest,
    _conn: &mut PgPooledConnection,
) -> Result<UnaryEchoResponse, ApiError> {
    request.validate()?;

    Ok(UnaryEchoResponse {
        message: request.message,
    })
}