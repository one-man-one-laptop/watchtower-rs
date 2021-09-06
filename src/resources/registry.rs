use std::collections::HashMap;
use rand::Rng;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use crate::{
    types::Result,
    utils::time::get_time_since_epoch
};

const LEASE_TTL_SECONDS: u64 = 30;
const MAX_LEASE_TO_EVICT: usize = 50;

/// An instance info.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InstanceInfo {
    pub instance_id: String,
    pub ip_addr: String,
    pub port: u16
}

/// A lease information.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LeaseInfo {
    pub service_id: String,
    pub instance_info: InstanceInfo,
    last_updated_timestamp: u64
}

impl LeaseInfo {
    /// Returns `true` if the lease is expired.
    /// 
    /// The life time of a lease is limited by `LEASE_TTL_SECONDS`.
    pub fn is_expired(&self) -> Result<bool> {
        Ok((self.last_updated_timestamp + LEASE_TTL_SECONDS) < get_time_since_epoch()?)
    }
}

/// A type alias for a hashmap with `LeaseInfo` as its values.
pub type LeaseHashMap = HashMap<String, LeaseInfo>;

/// A service registry for storing information about services and their leases.
pub struct ServiceRegistry {
    services: RwLock<HashMap<String, LeaseHashMap>>
}

impl ServiceRegistry {
    /// Creates a `serviceRegistry`.
    pub fn new() -> ServiceRegistry {
        ServiceRegistry {
            services: RwLock::new(HashMap::new())
        }
    }

    /// Runs the service registry.
    pub async fn run(&self) -> Result<()> {
        self.evict().await?;
        Ok(())
    }

    /// Registers a new service.
    pub async fn register_instance(&self, service_id: &str, instance_info: InstanceInfo) -> Result<()> {
        let mut services = self.services.write().await;
        if !services.contains_key(service_id) {
            services.insert(service_id.to_string(), HashMap::new());
        }

        if let Some(service) = services.get_mut(service_id) {
            service.insert(instance_info.instance_id.to_string(), LeaseInfo {
                instance_info,
                service_id: service_id.to_string(),
                last_updated_timestamp: get_time_since_epoch()?
            });
        }
        Ok(())
    }

    /// Renews a lease by updating its `last_updated_timestamp`.
    /// 
    /// If the lease does not exists, this method will do nothing.
    pub async fn renew_lease(&self, service_id: &str, instance_id: &str) -> Result<()> {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            if let Some(lease) = service.get_mut(instance_id) {
                lease.last_updated_timestamp = get_time_since_epoch()?;
            }
        }
        Ok(())
    }

    /// Cancels a lease by removing it from the `ServiceRegistry`.
    /// 
    /// If the lease does not exists, this method will do nothing.
    pub async fn cancel_lease(&self, service_id: &str, instance_id: &str) {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            service.remove(instance_id);
        }
    }

    /// Evicts expired instances.
    /// 
    /// The number of evict instances will be limited by `MAX_LEASE_TO_EVICT`.
    pub async fn evict(&self) -> Result<()> {
        let mut expired_leases = self.get_expired_instances().await?;
        let to_evict = std::cmp::min(expired_leases.len(), MAX_LEASE_TO_EVICT);
        for i in 0..to_evict {
            let next;
            {
                let mut rng = rand::thread_rng();
                next = rng.gen_range(i..to_evict);
            }
            expired_leases.swap(i, next);

            let lease = &expired_leases[i];
            self.cancel_lease(&lease.service_id, &lease.instance_info.instance_id).await;
        }
        Ok(())
    }

    /// Returns all expired instances.
    pub async fn get_expired_instances(&self) -> Result<Vec<LeaseInfo>> {
        let services = self.services.read().await;

        let mut expired_leases = Vec::new();
        for (_, leases) in services.iter() {
            for (_, lease) in leases.iter() {
                if lease.is_expired()? {
                    expired_leases.push(lease.clone());
                }
            }
        }
        Ok(expired_leases)
    }

    /// Returns the `LeaseInfo` of the interested lease.
    pub async fn get_instance(&self, service_id: &str, instance_id: &str) -> Option<LeaseInfo> {
        let services = self.services.read().await;
        if let Some(service) = services.get(service_id) {
            if let Some(lease) = service.get(instance_id) {
                Some(lease.clone())
            } else {
                None
            }
        } else {
            None
        }
    } 

    pub async fn get_all_instances(&self, service_id: &str) -> Option<Vec<LeaseInfo>> {
        let services = self.services.read().await;
        if let Some(leases) = services.get(service_id) {
            Some(leases.values().cloned().collect())
        } else {
            None
        }
    }
}
