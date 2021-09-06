use actix::prelude::Actor;
use actix_web::{middleware, web, App, HttpServer};

mod auth;
mod routes;
mod utils;
mod types;
mod error;
mod resources;

use crate::{
    types::{AppState, ServiceRegistry},
    resources::task_runner::TaskRunner
};

const HOST_ADDR: &str = env!("HOST_ADDR");

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=trace,actix_redis=trace");
    env_logger::init();

    let app_state = web::Data::new(AppState {
        service_registry: ServiceRegistry::new()
    });

    let task_runner = TaskRunner::new(app_state.clone());
    task_runner.start();

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api/v1")
                .configure(routes::v1::services::config)
            )
    })
    .bind(HOST_ADDR)?
    .run()
    .await
}