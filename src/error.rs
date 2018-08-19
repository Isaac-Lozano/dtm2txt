use std::error::Error;
use std::fmt;
use std::io::Error as IoError;
use std::string::FromUtf8Error;
use std::num::ParseIntError;

use serde_json::error::Error as JsonError;

#[derive(Debug)]
pub enum ControllerInputParseError {
    ParseIntError(ParseIntError),
    IoError(IoError),
    MissingTokenError,
    InvalidButtonError,
}

impl fmt::Display for ControllerInputParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ControllerInputParseError::ParseIntError(ref e) => e.fmt(f),
            ControllerInputParseError::IoError(ref e) => e.fmt(f),
            ControllerInputParseError::MissingTokenError => f.write_str("missing a button or axis"),
            ControllerInputParseError::InvalidButtonError => f.write_str("invalid button value"),
        }
    }
}

#[derive(Debug)]
pub enum Dtm2txtError {
    IoError(IoError),
    FromUtf8Error(FromUtf8Error),
    JsonError(JsonError),
    ControllerInputParseError{
        reason: ControllerInputParseError,
        line: u64,
    }
}

impl fmt::Display for Dtm2txtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Dtm2txtError::IoError(ref e) => e.fmt(f),
            Dtm2txtError::FromUtf8Error(ref e) => e.fmt(f),
            Dtm2txtError::JsonError(ref e) => e.fmt(f),
            Dtm2txtError::ControllerInputParseError{ref reason, line} =>
                write!(f, "{} on line {}", reason, line),
        }
    }
}

impl Error for Dtm2txtError {
    fn cause(&self) -> Option<&Error> {
        match *self {
            Dtm2txtError::IoError(ref e) => Some(e),
            Dtm2txtError::FromUtf8Error(ref e) => Some(e),
            Dtm2txtError::JsonError(ref e) => Some(e),
            Dtm2txtError::ControllerInputParseError{..} => None,
        }
    }
}

impl From<IoError> for Dtm2txtError {
    fn from(error: IoError) -> Dtm2txtError {
        Dtm2txtError::IoError(error)
    }
}

impl From<FromUtf8Error> for Dtm2txtError {
    fn from(error: FromUtf8Error) -> Dtm2txtError {
        Dtm2txtError::FromUtf8Error(error)
    }
}

impl From<JsonError> for Dtm2txtError {
    fn from(error: JsonError) -> Dtm2txtError {
        Dtm2txtError::JsonError(error)
    }
}

pub type Dtm2txtResult<T> = Result<T, Dtm2txtError>;