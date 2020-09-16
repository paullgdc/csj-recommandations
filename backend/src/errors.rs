use juniper::{graphql_value, FieldError, IntoFieldError};
use std::fmt;

macro_rules! from_error {
    ($variant:ident, $error:ty) => {
        impl From<$error> for ApiError {
            fn from(error: $error) -> Self {
                eprintln!("Error: {}", error);
                ApiError::$variant
            }
        }
    };
}

#[derive(Debug, Clone, Copy)]
pub enum ApiError {
    Database,
    UnrecognizedMediaValue,
    InvalidId,
    UnauthorizedOperation,
    NoUserFound,
}

impl fmt::Display for ApiError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use ApiError::*;
        match self {
            Database => write!(fmt, "Internal database error"),
            UnrecognizedMediaValue => write!(
                fmt,
                "The media associated with this object is not recognized"
            ),
            InvalidId => write!(fmt, "The id is invalid"),
            UnauthorizedOperation => {
                write!(fmt, "You don't have the permissions to perform this action")
            }
            NoUserFound => write!(fmt, "No user found associted with your id"),
        }
    }
}

from_error!(Database, r2d2::Error);
from_error!(Database, rusqlite::Error);

impl IntoFieldError for ApiError {
    fn into_field_error(self) -> FieldError {
        use ApiError::*;
        match self {
            Database => FieldError::new(
                self,
                graphql_value!({
                    "type": "DATABASE"
                }),
            ),
            UnrecognizedMediaValue => FieldError::new(
                self,
                graphql_value!({
                    "type": "UNRECOGNIZED_MEDIA_VALUE"
                }),
            ),
            InvalidId => FieldError::new(
                self,
                graphql_value!({
                    "type": "INVALID_ID"
                }),
            ),
            UnauthorizedOperation => FieldError::new(
                self,
                graphql_value!({
                    "type": "UNAUTHORIZED_OPERATION"
                }),
            ),
            ApiError::NoUserFound => FieldError::new(
                self,
                graphql_value!({
                    "type": "NO_USER_FOUND"
                }),
            ),
        }
    }
}

pub trait IntoApiResult<T> {
    fn into_api(self) -> ApiResult<T>;
}

pub type ApiResult<T> = std::result::Result<T, ApiError>;

impl<T, E> IntoApiResult<T> for std::result::Result<T, E>
where
    E: Into<ApiError>,
{
    fn into_api(self) -> ApiResult<T> {
        self.map_err(|e| e.into())
    }
}
