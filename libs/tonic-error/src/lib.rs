use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;
use tonic::metadata::errors::ToStrError;

pub use tonic_error_impl::*;

pub trait TonicError<'de>: Serialize + Deserialize<'de> + Display {}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("missing metadata entry")]
    MissingMetadata,

    #[error("invalid status code set")]
    InvalidStatusCode(tonic::Status),

    #[error("could not parse metadata to string")]
    MetadataParseError(#[from] ToStrError),

    #[error("serde json error")]
    JsonError(#[from] serde_json::Error),
}