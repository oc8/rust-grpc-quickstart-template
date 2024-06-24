use std::fmt;
use redis::RedisError;
use serde::{Deserialize, Serialize};
use serde_variant::to_variant_name;
use thiserror::Error;
use tonic_error::TonicError;
use crate::report_error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ValidationErrorMessage { }

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ValidationErrorKind {
    #[error("{0} must be between {1} and {2}, field: {0}")]
    InvalidLength(String, usize, usize),
    #[error("{0} {1}, field: {0}")]
    InvalidFormat(String, ValidationErrorMessage),
    #[error("{0} must be between {1} and {2}, field: {0}")]
    InvalidRange(String, usize, usize),
    #[error("{0} is missing, field: {0}")]
    MissingField(String),
    #[error("{1} is an invalid type, field: {0}")]
    InvalidType(String, String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct List<T>(pub Vec<T>);

impl<T> fmt::Display for List<T>
where
    T: fmt::Display + serde::Serialize,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        for (i, item) in self.0.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            s.push_str(&item.to_string());
        }
        write!(f, "{}", s)
    }
}

#[derive(Error, Debug, Serialize, Deserialize, TonicError)]
#[non_exhaustive]
pub enum ApiError {
    #[error("internal server error")]
    InternalServerError,
    #[error("the request was invalid: {0}")]
    InvalidRequest(String),
    #[error("redis connection failure")]
    RedisConnectionFailure,
    #[error("cache error")]
    CacheError,
    #[error("database connection failure")]
    DatabaseConnectionFailure,
    #[error("database error: {0}")]
    DatabaseError(String),
    #[error("already exists: {0}")]
    AlreadyExists(String),
    #[error("not found {0}")]
    NotFound(String),
    #[error("parsing error: {0}")]
    ParsingError(String),
    #[error("validation error: {0}")]
    ValidationError(List<ValidationErrorKind>),
}

impl ApiError {
    pub fn code(&self) -> tonic::Code {
        match self {
            ApiError::InvalidRequest(_) => tonic::Code::InvalidArgument,
            ApiError::RedisConnectionFailure => tonic::Code::Unavailable,
            ApiError::CacheError => tonic::Code::Unavailable,
            ApiError::DatabaseConnectionFailure => tonic::Code::Unavailable,
            ApiError::DatabaseError(_) => tonic::Code::Internal,
            ApiError::AlreadyExists(_) => tonic::Code::AlreadyExists,
            ApiError::NotFound(_) => tonic::Code::NotFound,
            ApiError::ValidationError(_) => tonic::Code::InvalidArgument,
            _ => tonic::Code::Internal,
        }
    }

    pub fn is_list(&self) -> bool {
        match self {
            ApiError::ValidationError(_) => true,
            _ => false,
        }
    }

    pub fn errors(&self) -> serde_json::Value {
        match self {
            ApiError::ValidationError(errors) => {
                let mut v = Vec::new();
                for e in &errors.0 {
                    let type_name = to_variant_name(e).unwrap();
                    let data = e.to_string();
                    let (message, field) = data.split_once(", field: ").unwrap_or(("", ""));
                    v.push(serde_json::json!({
                        "message": message,
                        "field": field,
                        "type": type_name,
                    }));
                }

                serde_json::json!(v)
            }
            _ => serde_json::json!([])
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => {
                // TODO: find a way to return the table name
                ApiError::NotFound("".to_string())
            },
            sqlx::Error::Database(e) => {
                if e.is_unique_violation() {
                    ApiError::AlreadyExists(e.message().to_string())
                } else {
                    report_error(&e);
                    ApiError::DatabaseError(e.message().to_string())
                }
            }
            _ => {
                report_error(&error);
                ApiError::InternalServerError
            }
        }
    }
}

impl From<RedisError> for ApiError {
    fn from(error: RedisError) -> Self {
        report_error(&error);
        ApiError::CacheError
    }
}
