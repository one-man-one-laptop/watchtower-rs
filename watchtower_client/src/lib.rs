use std::{
    collections::HashMap,
    sync::Arc
};
use pyo3::prelude::*;
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

#[pyclass]
pub struct PyWatchtowerClient {
    client: Arc<WatchtowerClient>,
}

#[pymethods]
impl PyWatchtowerClient {
    #[new]
    pub fn new(watchtower_urls: Vec<String>, username: &str, password: &str) -> Self {
        let client = Arc::new(WatchtowerClient::new(
            watchtower_urls,
            username,
            password
        ));
        PyWatchtowerClient {
            client
        }
    }

    pub fn register(self_: PyRef<Self>, service_id: &str, ip_addr: &str, port: u16) -> PyResult<()> {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        let client = self_.client.clone();
        rt.block_on(async {
            client.register_without_pinging(&service_id, ip_addr, port).await
        })?;
        Ok(())
    }

    pub fn ping(self_: PyRef<Self>) -> PyResult<()> {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        let client = self_.client.clone();
        rt.block_on(async {
            client.ping().await
        })?;
        Ok(())
    }

    pub fn cancel(self_: PyRef<Self>) -> PyResult<()> {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        let client = self_.client.clone();
        rt.block_on(async {
            client.cancel().await
        })?;
        Ok(())
    }

    pub fn get_service_url(self_: PyRef<Self>, service_id: &str) -> PyResult<String> {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        let client = self_.client.clone();
        let service_url = rt.block_on(async {
            client.get_service_url(service_id).await
        })?;
        Ok(service_url)
    }
}

#[pymodule]
fn watchtower_client(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyWatchtowerClient>()?;
    Ok(())
}

pub struct WatchtowerClient {
    http_client: Arc<HttpClient>,
    services: Mutex<HashMap<String, Service>>,
    instance_info: Arc<Mutex<Option<(String, InstanceInfo)>>>,
}

const HEARTBEAT_INTERVAL_SEC: u64 = 15;

impl WatchtowerClient {
    pub fn new(watchtower_urls: Vec<String>, username: &str, password: &str) -> Self {
        let http_client = Arc::new(HttpClient::new(watchtower_urls, username.to_string(), password.to_string()));
        WatchtowerClient {
            http_client,
            services: Mutex::new(HashMap::new()),
            instance_info: Arc::new(Mutex::new(None))
        }
    }

    fn generate_new_instance(ip_addr: &str, port: u16) -> InstanceInfo {
        let instance_id = Uuid::new_v4().to_string();
        InstanceInfo {
            instance_id: instance_id.to_string(),
            ip_addr: ip_addr.to_string(),
            port
        }
    }

    async fn register_helper(&self, service_id: &str, new_instance_info: &InstanceInfo) -> Result<()> {
        if let Some(_) = *self.instance_info.lock().await {
            return Err(Error::InstanceAlreadyRegistered);
        }
        *self.instance_info.lock().await = Some((service_id.to_string(), new_instance_info.clone()));
        self.http_client.register(&service_id, &new_instance_info).await
    }

    /// Register a new service
    /// 
    /// This will spawn a child process to ping the service registry
    /// Note: only one service can be registered at a time
    pub async fn register(&self, service_id: &str, ip_addr: &str, port: u16) -> Result<()> {
        let new_instance_info = Self::generate_new_instance(ip_addr, port);
        self.register_helper(&service_id, &new_instance_info).await?;

        let client = self.http_client.clone();
        let instance_info = self.instance_info.clone();
        let service_id = service_id.to_string();
        actix::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(HEARTBEAT_INTERVAL_SEC));
            loop {
                interval.tick().await;
                if let Some((_, instance_info)) = &*instance_info.lock().await {
                    if instance_info.instance_id != new_instance_info.instance_id {
                        // the instance info does not match the current instance info
                        return;
                    }
                } else {
                    // the instance info is no longer there
                    return;
                }
                client.renew(&service_id, &new_instance_info).await.unwrap();
            }
        });
        Ok(())
    }

    pub async fn register_without_pinging(&self, service_id: &str, ip_addr: &str, port: u16) -> Result<()> {
        let new_instance_info = Self::generate_new_instance(ip_addr, port);
        self.register_helper(&service_id, &new_instance_info).await
    }
    
    pub async fn ping(&self) -> Result<()> {
        let my_instance_info;
        let my_service_id;
        if let Some((service_id, instance_info)) = &*self.instance_info.lock().await {
            my_instance_info = instance_info.clone();
            my_service_id = service_id.to_string();
        } else {
            // the instance info is no longer there
            return Err(Error::InvalidPing);
        }
        self.http_client.renew(&my_service_id, &my_instance_info).await
    }

    /// Cancel a lease for a service
    pub async fn cancel(&self) -> Result<()> {
        let (service_id, instance_info) = match &*self.instance_info.lock().await {
            Some((service_id, instance_info)) => (service_id.to_string(), instance_info.clone()),
            None => return Err(Error::NotFound)
        };
        self.http_client.cancel(&service_id, &instance_info).await?;
        *self.instance_info.lock().await = None;
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
