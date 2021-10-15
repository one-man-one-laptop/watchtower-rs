use actix::Actor;
use actix_web::{middleware, web, App, HttpServer};

mod routes;
mod utils;
mod types;
mod error;
mod resources;

use crate::{
    types::{AppState, ServiceRegistry},
    resources::{spawn_runner, Dispatcher},
    utils::env
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info,watchtower=info");
    env_logger::init();

    let dispatcher = Dispatcher::new(utils::env::get_cluster_nodes()).start();
    let app_state = web::Data::new(AppState {
        service_registry: ServiceRegistry::new(dispatcher)
    });

    spawn_runner(app_state.clone());

    HttpServer::new(move || App::new()
        .wrap(middleware::Logger::default())
        .app_data(app_state.clone())
        .service(
            web::scope("/api/v1")
            .configure(routes::v1::services::config)
            .configure(routes::v1::health::config)
        )
    )
    .bind(env::get_hostname())?
    .run()
    .await
}