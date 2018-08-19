use std::error::Error;
use std::fmt;
use std::io::Error as IoError;
use std::string::FromUtf8Error;

use serde_json::error::Error as JsonError;

#[derive(Debug)]
pub enum Dtm2txtError {
    IoError(IoError),
    FromUtf8Error(FromUtf8Error),
    JsonError(JsonError),
    ControllerInputParseError{
        line: u64,
        reason: &'static str,
    }
}

impl fmt::Display for Dtm2txtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Dtm2txtError::IoError(_) => f.write_str("io error"),
            Dtm2txtError::FromUtf8Error(_) => f.write_str("utf8 error"),
            Dtm2txtError::JsonError(_) => f.write_str("json error"),
            Dtm2txtError::ControllerInputParseError{..} => f.write_str("controller input parse error"),
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