use crate::error::WatchtowerError;
pub use crate::resources::InstanceInfo;

pub type Error = WatchtowerError;
pub type Result<T> = std::result::Result<T, Error>;
