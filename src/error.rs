use std::{str::Utf8Error, string::FromUtf8Error};

use num_enum::{TryFromPrimitive, TryFromPrimitiveError};

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    DecompressionError(miniz_oxide::inflate::TINFLStatus),
    ShapeError(ndarray::ShapeError),
    OxifiveError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::OxifiveError(message) => write!(fmt, "oxifive::Error({:?})", message),
            x => std::fmt::Debug::fmt(&x, fmt),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<miniz_oxide::inflate::TINFLStatus> for Error {
    fn from(err: miniz_oxide::inflate::TINFLStatus) -> Error {
        Error::DecompressionError(err)
    }
}

impl From<ndarray::ShapeError> for Error {
    fn from(err: ndarray::ShapeError) -> Error {
        Error::ShapeError(err)
    }
}

impl<T: TryFromPrimitive> From<TryFromPrimitiveError<T>> for Error {
    fn from(error: TryFromPrimitiveError<T>) -> Self {
        Error::OxifiveError(format!(
            "Unexpected data found for {:?}: {:?}",
            stringify!(T),
            error
        ))
    }
}

impl From<Utf8Error> for Error {
    fn from(_error: Utf8Error) -> Self {
        Error::OxifiveError("Could not convert string from UTF8 bytes".to_string())
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_error: FromUtf8Error) -> Self {
        Error::OxifiveError("Could not convert string from UTF8 bytes".to_string())
    }
}
