use actix_web::{web, HttpResponse};
use crate::types::{Result, AppState, InstanceInfo, AuthorizedReq};

pub async fn get_all_instances(_: AuthorizedReq, path: web::Path<(String,)>, data: web::Data<AppState>) -> Result<HttpResponse> {
    let (service_id,) = path.into_inner();
    if let Some(leases) = data.service_registry.get_all_instances(&service_id).await {
        Ok(HttpResponse::Ok().content_type("application/json").body(serde_json::to_string(&leases)?))
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

pub async fn register_instance(req: AuthorizedReq, instance_info: web::Json<InstanceInfo>, path: web::Path<(String,)>, data: web::Data<AppState>) -> Result<HttpResponse> {
    let (service_id,) = path.into_inner();
    data.service_registry.register_instance(&service_id, instance_info.into_inner(), req.is_replicated).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn renew_lease(req: AuthorizedReq, path: web::Path<(String, String)>, data: web::Data<AppState>) -> Result<HttpResponse> {
    let (service_id, instance_id) = path.into_inner();

    if data.service_registry.renew_lease(&service_id, &instance_id, req.is_replicated).await? {
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

pub async fn cancel_lease(req: AuthorizedReq, path: web::Path<(String, String)>, data: web::Data<AppState>) -> Result<HttpResponse> {
    let (service_id, instance_id) = path.into_inner();
    match data.service_registry.cancel_lease(&service_id, &instance_id, req.is_replicated).await? {
        Some(_) => Ok(HttpResponse::Ok().finish()),
        None => Ok(HttpResponse::NotFound().finish())
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/services/{service_id}")
            .route(web::get().to(get_all_instances))
            .route(web::post().to(register_instance))
    ).service(
        web::resource("/services/{service_id}/{instance_id}")
            .route(web::put().to(renew_lease))
            .route(web::delete().to(cancel_lease))
    );
}
