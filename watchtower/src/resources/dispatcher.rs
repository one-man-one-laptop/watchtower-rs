use actix::{Actor, Context, Handler, Message, prelude::ResponseFuture};
use std::{
    net::SocketAddr,
    sync::Arc
};
use futures_util::future::join_all;
use log::error;

use crate::{
    types::{InstanceInfo, Result},
    utils::{env, auth::REPLICATION_HEADER}
};

pub enum DispatcherMessage {
    Register(String, InstanceInfo),
    Renew(String, InstanceInfo),
    Cancel(String, String),
}

impl Message for DispatcherMessage {
    type Result = Result<bool>;
}

const USER_AGENT_KEY: &str = "User-Agent";
const USER_AGENT_VALUE: &str = "WatchtowerDispatcher";

pub struct Node {
    client: reqwest::Client,
    url: SocketAddr,
    username: String,
    password: String
}

impl Node {
    pub fn new(url: SocketAddr) -> Self {
        let auth = env::get_auth_info();
        Node {
            client: reqwest::Client::new(),
            url,
            username: auth.username,
            password: auth.password
        }
    }

    /// Sends an instance register request to the node.
    pub async fn register(&self, service_id: &str, instance_info: &InstanceInfo) {
        let url = format!("http://{}/api/v1/services/{}", self.url, service_id);
        let instance_info = serde_json::to_string(&instance_info).expect("Fails to serialize instance_info");
        match self.client.post(&url).body(instance_info)
            .basic_auth(&self.username, Some(&self.password))
            .header("content-type", "application/json")
            .header(REPLICATION_HEADER, "true")
            .header(USER_AGENT_KEY, USER_AGENT_VALUE)
            .send().await {
            Ok(res) => {
                if res.status() != reqwest::StatusCode::NO_CONTENT {
                    error!("Unexpected status code {}", res.status());
                }
            },
            Err(err) => {
                error!("Unable to replicate register request: {}", err);
            }
        }
    }

    /// Sends an instance renew request to the node.
    /// 
    /// If the instance does not exist on the node, it will subsequently send an instance register request.
    pub async fn renew(&self, service_id: &str, instance_info: &InstanceInfo) {
        let url = format!("http://{}/api/v1/services/{}/{}", self.url, service_id, instance_info.instance_id);
        match self.client.put(&url)
            .basic_auth(&self.username, Some(&self.password))
            .header(REPLICATION_HEADER, "true")
            .header(USER_AGENT_KEY, USER_AGENT_VALUE)
            .send().await {
            Ok(res) => {
                if res.status() == reqwest::StatusCode::OK {
                } else if res.status() == reqwest::StatusCode::NOT_FOUND {
                    // If the instance does not exist, register the instance instead
                    self.register(service_id, &instance_info).await;
                } else {
                    error!("Unexpected status code: {}", res.status());
                }
            },
            Err(err) => {
                error!("Unable to replicate renew request: {}", err);
            }
        }
    }

    /// Sends an lease cancel request to the node.
    pub async fn cancel(&self, service_id: &str, instance_id: &str) {
        let url = format!("http://{}/api/v1/services/{}/{}", self.url, service_id, instance_id);
        match self.client.delete(&url)
            .basic_auth(&self.username, Some(&self.password))
            .header(REPLICATION_HEADER, "true")
            .header(USER_AGENT_KEY, USER_AGENT_VALUE)
            .send().await {
            Ok(res) => {
                if res.status() != reqwest::StatusCode::OK {
                    error!("Unexpected status code: {}", res.status());
                }
            },
            Err(err) => {
                error!("Unable to replicate cancel request: {}", err);
            }
        }
    }
}


pub struct Dispatcher {
    nodes: Vec<Arc<Node>>
}

impl Dispatcher {
    pub fn new(nodes_urls: Vec<SocketAddr>) -> Dispatcher {
        Dispatcher {
            nodes: nodes_urls.iter().map(|url| Arc::new(Node::new(*url))).collect()
        }
    }
}

impl Actor for Dispatcher {
    type Context = Context<Self>;
}

impl Handler<DispatcherMessage> for Dispatcher {
    type Result = ResponseFuture<Result<bool>>;

    fn handle(&mut self, event: DispatcherMessage, _ctx: &mut Context<Self>) -> Self::Result {
        let nodes: Vec<Arc<Node>> = self.nodes.iter().map(|node| node.clone()).collect();
        Box::pin(async move {
            match event {
                DispatcherMessage::Register(service_id, instance_info) => {
                    join_all(nodes.iter().map(|node| node.register(&service_id, &instance_info))).await;
                }
                DispatcherMessage::Renew(service_id, instance_info) => {
                    join_all(nodes.iter().map(|node| node.renew(&service_id, &instance_info))).await;
                }
                DispatcherMessage::Cancel(service_id, instance_id) => {
                    join_all(nodes.iter().map(|node| node.cancel(&service_id, &instance_id))).await;
                }
            };
            Ok(true)
        })
    }
}
