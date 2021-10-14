use actix::Addr;
use std::collections::HashMap;
use rand::Rng;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use crate::{
    types::Result,
    utils::time::get_time_since_epoch,
    resources::{Dispatcher, DispatcherMessage}
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
    services: RwLock<HashMap<String, LeaseHashMap>>,
    dispatcher: Addr<Dispatcher>
}

impl ServiceRegistry {
    /// Creates a `serviceRegistry`.
    pub fn new(dispatcher: Addr<Dispatcher>) -> ServiceRegistry {
        ServiceRegistry {
            services: RwLock::new(HashMap::new()),
            dispatcher
        }
    }

    /// Runs the service registry.
    pub async fn run(&self) -> Result<()> {
        self.evict().await?;
        Ok(())
    }

    /// Registers a new service.
    pub async fn register_instance(&self, service_id: &str, instance_info: InstanceInfo, is_replicated: bool) -> Result<()> {
        let mut services = self.services.write().await;
        if !services.contains_key(service_id) {
            services.insert(service_id.to_string(), HashMap::new());
        }

        if let Some(service) = services.get_mut(service_id) {
            service.insert(instance_info.instance_id.to_string(), LeaseInfo {
                instance_info: instance_info.clone(),
                service_id: service_id.to_string(),
                last_updated_timestamp: get_time_since_epoch()?
            });

            if !is_replicated {
                self.dispatcher.do_send(DispatcherMessage::Register(service_id.to_string(), instance_info));
            }
        }
        Ok(())
    }

    /// Renews a lease by updating its `last_updated_timestamp`.
    /// 
    /// If the lease does not exists, this method will return false.
    pub async fn renew_lease(&self, service_id: &str, instance_id: &str, is_replicated: bool) -> Result<bool> {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            if let Some(lease) = service.get_mut(instance_id) {
                lease.last_updated_timestamp = get_time_since_epoch()?;
                if !is_replicated {
                    self.dispatcher.do_send(DispatcherMessage::Renew(service_id.to_string(), lease.instance_info.clone()));
                }
                return Ok(true)
            }
        }
        Ok(false)
    }

    /// Cancels a lease by removing it from the `ServiceRegistry`.
    /// 
    /// If the lease does not exists, this method will return None.
    pub async fn cancel_lease(&self, service_id: &str, instance_id: &str, is_replicated: bool) -> Option<LeaseInfo> {
        let mut services = self.services.write().await;
        match services.get_mut(service_id) {
            Some(service) => {
                let lease_option = service.remove(instance_id);
                if !is_replicated {   
                    if let Some(_) = &lease_option {
                        self.dispatcher.do_send(DispatcherMessage::Cancel(service_id.to_string(), instance_id.to_string()));
                    }
                }
                lease_option
            },
            None => None
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
            self.cancel_lease(&lease.service_id, &lease.instance_info.instance_id, true).await;
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

    /// Returns all the `InstanceInfo` of the interested service.
    pub async fn get_all_instances(&self, service_id: &str) -> Option<Vec<InstanceInfo>> {
        let services = self.services.read().await;
        if let Some(leases) = services.get(service_id) {
            Some(leases.values().cloned().map(|lease| lease.instance_info).collect())
        } else {
            None
        }
    }
}
