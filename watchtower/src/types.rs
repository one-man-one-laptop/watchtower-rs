use crate::error::WatchtowerError;
pub use crate::resources::{ServiceRegistry, InstanceInfo};
pub use crate::utils::auth::AuthorizedReq;

pub type Error = WatchtowerError;
pub type Result<T> = std::result::Result<T, Error>;

pub struct AppState {
    pub service_registry: ServiceRegistry
}
