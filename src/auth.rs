use actix_web::{dev, Error, HttpRequest, FromRequest};
use actix_web::error::ErrorUnauthorized;
use futures_util::future::{ok, err, Ready};
use serde::Deserialize;
use base64::decode;

#[derive(Debug, Deserialize)]
pub struct AuthorizedReq;

const UNAUTHORIZED: &str = "Unauthorized";
const AUTH_USERNAME: &str = env!("AUTH_USERNAME");
const AUTH_PASSWORD: &str = env!("AUTH_PASSWORD");

impl FromRequest for AuthorizedReq {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        match check_auth(req) {
            Ok(_) => ok(AuthorizedReq {}),
            Err(error) => err(error)
        }
    }
}

fn check_auth(req: &HttpRequest) -> Result<(), Error> {
    match req.headers().get("Authorization") {
        Some(auth) => {
            let mut iter = auth.to_str().map_err(|_| ErrorUnauthorized(UNAUTHORIZED))?.splitn(2, ' ');
            let auth_type = iter.next().ok_or(ErrorUnauthorized(UNAUTHORIZED))?;
            let hashed_creds = iter.next().ok_or(ErrorUnauthorized(UNAUTHORIZED))?;
            let creds = std::str::from_utf8(&decode(hashed_creds).map_err(|_| ErrorUnauthorized(UNAUTHORIZED))?)
                .map_err(|_| ErrorUnauthorized(UNAUTHORIZED))?.to_string();
            let mut iter = creds.splitn(2, ':');
            let username = iter.next().ok_or(ErrorUnauthorized(UNAUTHORIZED))?;
            let password = iter.next().ok_or(ErrorUnauthorized(UNAUTHORIZED))?;

            if auth_type == "Basic" && username == AUTH_USERNAME && password == AUTH_PASSWORD {
                Ok(())
            } else {
                Err(ErrorUnauthorized(UNAUTHORIZED))
            }
        }
        None => Err(ErrorUnauthorized(UNAUTHORIZED))
    }
}