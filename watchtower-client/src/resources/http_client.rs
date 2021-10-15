use tokio::sync::Mutex;
use log::error;
use crate::{
    types::{InstanceInfo, Result, Error},
    load_balancer::{LoadBalancer, RoundRobinLoadBalancer}
};

pub struct HttpClient {
    client: reqwest::Client,
    urls: Vec<String>,
    username: String,
    password: String,
    load_balancer: Mutex<RoundRobinLoadBalancer>
}

const MAX_ATTEMPT: u16 = 3;

impl HttpClient {
    pub fn new(urls: Vec<String>, username: String, password: String) -> Self {
        HttpClient {
            client: reqwest::Client::new(),
            load_balancer: Mutex::new(RoundRobinLoadBalancer::new(urls.len())),
            urls,
            username,
            password
        }
    }

    async fn get_new_url(&self) -> &str {
        let current_index = self.load_balancer.lock().await.get_next_index();
        &self.urls[current_index]
    }

    pub async fn register(&self, service_id: &str, instance_info: &InstanceInfo) -> Result<()> {
        let mut base_url = self.get_new_url().await;
        let mut attempt = 0;

        let instance_info = serde_json::to_string(&instance_info).expect("Fails to serialize instance_info");
        while attempt < MAX_ATTEMPT {
            let url = format!("{}/api/v1/services/{}", base_url, service_id);
            match self.client.post(&url).body(instance_info.clone())
                .basic_auth(&self.username, Some(&self.password))
                .header("content-type", "application/json")
                .send().await {
                Ok(res) => {
                    if res.status() == reqwest::StatusCode::NO_CONTENT {
                        return Ok(())
                    } else if res.status() == reqwest::StatusCode::UNAUTHORIZED {
                        return Err(Error::Unauthorized);
                    } else {
                        error!("Unexpected status code {}", res.status());
                    }
                }
                Err(err) => {
                    error!("Register request error: {}", err);
                    base_url = self.get_new_url().await;
                    attempt += 1;
                }
            }
        }
        Err(Error::MaxRetryReached)
    }

    pub async fn renew(&self, service_id: &str, instance_info: &InstanceInfo) -> Result<()> {
        let mut base_url = self.get_new_url().await;
        let mut attempt = 0;

        while attempt < MAX_ATTEMPT {
            let url = format!("{}/api/v1/services/{}/{}", base_url, service_id, instance_info.instance_id);
            match self.client.put(&url)
                .basic_auth(&self.username, Some(&self.password))
                .send().await {
                Ok(res) => {
                    if res.status() == reqwest::StatusCode::OK {
                        return Ok(());
                    } else if res.status() == reqwest::StatusCode::NOT_FOUND {
                        // If the instance does not exist, register the instance instead
                        return self.register(service_id, &instance_info).await;
                    } else if res.status() == reqwest::StatusCode::UNAUTHORIZED {
                        return Err(Error::Unauthorized);
                    } else {
                        error!("Unexpected status code: {}", res.status());
                    }
                }
                Err(err) => {
                    error!("Renew request error: {}", err);
                    base_url = self.get_new_url().await;
                    attempt += 1;
                }
            }
        }
        Err(Error::MaxRetryReached)
    }

    pub async fn cancel(&self, service_id: &str, instance_info: &InstanceInfo) -> Result<()> {
        let mut base_url = self.get_new_url().await;
        let mut attempt = 0;

        while attempt < MAX_ATTEMPT {
            let url = format!("{}/api/v1/services/{}/{}", base_url, service_id, instance_info.instance_id);
            match self.client.delete(&url)
                .basic_auth(&self.username, Some(&self.password))
                .send().await {
                Ok(res) => {
                    if res.status() == reqwest::StatusCode::OK {
                        return Ok(());
                    } else {
                        error!("Unexpected status code: {}", res.status());
                    }
                }
                Err(err) => {
                    error!("Renew request error: {}", err);
                    base_url = self.get_new_url().await;
                    attempt += 1;
                }
            }
        }
        Err(Error::MaxRetryReached)    
    }

    pub async fn get_all_instances(&self, service_id: &str) -> Result<Vec<InstanceInfo>> {
        let mut base_url = self.get_new_url().await;
        let mut attempt = 0;

        while attempt < MAX_ATTEMPT {
            let url = format!("{}/api/v1/services/{}", base_url, service_id);
            match self.client.get(&url)
                .basic_auth(&self.username, Some(&self.password))
                .send().await {
                Ok(res) => {
                    if res.status() == reqwest::StatusCode::OK {
                        return Ok(res.json().await?);
                    } else if res.status() == reqwest::StatusCode::NOT_FOUND {
                        return Err(Error::NotFound);
                    } else if res.status() == reqwest::StatusCode::UNAUTHORIZED {
                        return Err(Error::Unauthorized);
                    } else {
                        error!("Unexpected status code: {}", res.status());
                    }
                }
                Err(err) => {
                    error!("Get all instances request error: {}", err);
                    base_url = self.get_new_url().await;
                    attempt += 1;
                }
            }
        }
        Err(Error::MaxRetryReached)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_get_new_url() {
        let http_client = HttpClient::new(vec!["a".to_string(), "b".to_string()], "admin".to_string(), "password".to_string());
        let url = http_client.get_new_url().await;
        match url {
            "a" => {
                assert_eq!(http_client.get_new_url().await, "b");
            }
            "b" => {
                assert_eq!(http_client.get_new_url().await, "a"); 
            }
            _ => {
                panic!("Unexpected url value");
            }
        }
    }
}