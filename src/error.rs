use actix_web::{
  HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse,
};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum WatchTowerError {
    #[display(fmt = "Internal Server Error")]
    InternalError,

    #[display(fmt = "bad request")]
    BadClientData,

    #[display(fmt = "timeout")]
    Timeout,
}

impl From<std::time::SystemTimeError> for WatchTowerError {
    fn from(error: std::time::SystemTimeError) -> Self {
        println!("{:?}", error);
        WatchTowerError::InternalError
    }
}

impl From<serde_json::Error> for WatchTowerError {
    fn from(error: serde_json::Error) -> Self {
        println!("{:?}", error);
        WatchTowerError::InternalError
    }
}

impl error::ResponseError for WatchTowerError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .insert_header((header::CONTENT_TYPE, "text/html; charset=utf-8"))
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            WatchTowerError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            WatchTowerError::BadClientData => StatusCode::BAD_REQUEST,
            WatchTowerError::Timeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }
}