use log::error;

#[derive(Debug, PartialEq)]
pub enum WatchtowerError {
    InternalError,
    NotFound,
    Unauthorized,
    InstanceAlreadyRegistered,
    MaxRetryReached
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