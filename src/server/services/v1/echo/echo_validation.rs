use protos::echo::v1::UnaryEchoRequest;
use crate::errors::{ApiError, List, ValidationErrorKind};
use crate::errors::ApiError::ValidationError;
use crate::utils::validation::ValidateRequest;

impl ValidateRequest for UnaryEchoRequest {
    fn validate(&self) -> Result<(), ApiError> {
        if self.message.is_empty() {
            return Err(ValidationError(List(vec![ValidationErrorKind::MissingField("message".to_string())])));
        }

        Ok(())
    }
}