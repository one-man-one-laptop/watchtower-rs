use crate::error::WatchTowerError;
pub use crate::resources::registry::{ServiceRegistry, InstanceInfo};
pub use crate::auth::AuthorizedReq;

pub type Error = WatchTowerError;
pub type Result<T> = std::result::Result<T, Error>;

pub struct AppState {
    pub service_registry: ServiceRegistry
}
