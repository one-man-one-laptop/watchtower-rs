use log::error;
use pyo3::{
    exceptions::{PyException},
    PyErr
};

#[derive(Debug, PartialEq)]
pub enum WatchtowerError {
    InternalError,
    NotFound,
    Unauthorized,
    InstanceAlreadyRegistered,
    MaxRetryReached,
    InvalidPing
}

impl From<reqwest::Error> for WatchtowerError {
    fn from(error: reqwest::Error) -> Self {
        error!("Reqwest Error: {:?}", error);
        WatchtowerError::InternalError
    }
}

impl From<std::time::SystemTimeError> for WatchtowerError {
    fn from(error: std::time::SystemTimeError) -> Self {
        error!("{:?}", error);
        WatchtowerError::InternalError
    }
}

impl From<serde_json::Error> for WatchtowerError {
    fn from(error: serde_json::Error) -> Self {
        error!("{:?}", error);
        WatchtowerError::InternalError
    }
}

impl From<WatchtowerError> for PyErr {
    fn from(err: WatchtowerError) -> PyErr {
        match err {
            WatchtowerError::NotFound => PyException::new_err("NotFound"),
            WatchtowerError::Unauthorized => PyException::new_err("Unauthorized"),
            _ => PyException::new_err("Something went wrong")
        }
        
    }
}