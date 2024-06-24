use crate::errors::{ApiError};

pub trait ValidateRequest {
    fn validate(&self) -> Result<(), ApiError>;
}
