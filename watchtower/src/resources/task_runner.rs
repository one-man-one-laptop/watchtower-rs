use actix_web::web::Data;
use std::time::Duration;

use crate::types::AppState;

const RUN_INTERVAL_SEC: u64 = 15;

/// Generate a background task to evict expired leases 
pub fn spawn_runner (app_state: Data<AppState>) {
    actix::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(RUN_INTERVAL_SEC));
        loop {
            interval.tick().await;
            app_state.service_registry.run().await.expect("Service registry failed to execute!");
        }
    });
}