use actix_web::{web, HttpResponse};
use crate::types::{Result, AuthorizedReq};

pub async fn health_check(_: AuthorizedReq) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/healthcheck")
            .route(web::get().to(health_check))
    );
}