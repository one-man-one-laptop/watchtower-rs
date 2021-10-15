use actix_web::{
    dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse,
};
use derive_more::{Display, Error};
use log::error;

#[derive(Debug, Display, Error)]
pub enum WatchtowerError {
    #[display(fmt = "Internal Server Error")]
    InternalError
}

impl From<std::time::SystemTimeError> for WatchtowerError {
    fn from(error: std::time::SystemTimeError) -> Self {
        error!("{}", error);
        WatchtowerError::InternalError
    }
}

impl From<serde_json::Error> for WatchtowerError {
    fn from(error: serde_json::Error) -> Self {
        error!("{}", error);
        WatchtowerError::InternalError
    }
}

impl From<actix::MailboxError> for WatchtowerError {
    fn from(error: actix::MailboxError) -> Self {
        error!("{}", error);
        WatchtowerError::InternalError
    }
}

impl error::ResponseError for WatchtowerError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            WatchtowerError::InternalError => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}