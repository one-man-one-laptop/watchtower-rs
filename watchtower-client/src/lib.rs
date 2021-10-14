use std::{
    collections::HashMap,
    sync::Arc
};
use tokio::sync::Mutex;
use uuid::Uuid;

mod utils;
mod resources;
mod error;
mod types;

pub use crate::{
    resources::{InstanceInfo, Service, HttpClient, load_balancer},
    types::{Result, Error},
};

pub struct WatchtowerClient {
    http_client: Arc<HttpClient>,
    services: Mutex<HashMap<String, Service>>
}

const HEARTBEAT_INTVERAL_SEC: u64 = 30;

impl WatchtowerClient {
    pub fn new(watchtower_urls: Vec<String>, username: &str, password: &str) -> Self {
        let http_client = Arc::new(HttpClient::new(watchtower_urls, username.to_string(), password.to_string()));
        WatchtowerClient {
            http_client,
            services: Mutex::new(HashMap::new())
        }
    }

    /// Register a new service
    /// 
    /// This will spawn a child process to ping the service registry
    pub async fn register(&self, service_id: &str, ip_addr: &str, port: u16) -> Result<()> {
        let instance_id = Uuid::new_v4().to_string();
        let service_id = service_id.to_string();

        let client = self.http_client.clone();
        let instance_info = InstanceInfo {
            instance_id: instance_id.to_string(),
            ip_addr: ip_addr.to_string(),
            port
        };
        client.register(&service_id, &instance_info).await?;
        actix::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(HEARTBEAT_INTVERAL_SEC));
            loop {
                interval.tick().await;
                client.renew(&service_id, &instance_info).await.unwrap();
            }
        });
        Ok(())
    }

    async fn refetch_service(&self, service_id: &str) -> Result<InstanceInfo> {
        let instance_infos: Vec<InstanceInfo> = self.http_client.get_all_instances(service_id).await?;

        let mut service = Service::new(instance_infos);
        let instance_info = service.get_next_instance()?;

        self.services.lock().await.insert(service_id.to_string(), service);
        Ok(instance_info)
    }

    /// Get the url of the service
    pub async fn get_service_url(&self, service_id: &str) -> Result<String> {
        let maybe_instance_info = match self.services.lock().await.get_mut(service_id) {
            Some(service) => {
                if service.is_expired()? {
                    None
                } else {
                    Some(service.get_next_instance()?)
                }
            }
            None => None
        };

        let instance_info = match maybe_instance_info {
            Some(service) => service,
            None => self.refetch_service(service_id).await?
        };
        
        Ok(format!("{}:{}", instance_info.ip_addr, instance_info.port))
    }
}
