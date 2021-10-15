use std::net::{SocketAddr, ToSocketAddrs};

const DEFAULT_USERNAME: &str = "admin";
const DEFAULT_PASSWORD: &str = "password";
const DEFAULT_HOSTNAME: &str = "127.0.0.1:8088";

pub fn get_cluster_nodes() -> Vec<SocketAddr> {
    match std::env::var("CLUSTER_NODES") {
        Ok(var) => {
            let mut nodes: Vec<SocketAddr> = var.split(',').map(|node| node.to_socket_addrs()
                .unwrap().next().unwrap()).collect();
            nodes.retain(|node| *node != get_hostname());
            nodes
        },
        Err(_) => vec![]
    }
}

pub fn get_hostname() -> SocketAddr {
    std::env::var("HOSTNAME").unwrap_or(DEFAULT_HOSTNAME.to_string()).to_socket_addrs().unwrap().next().unwrap()
}

pub struct AuthInfo {
    pub username: String,
    pub password: String
}

pub fn get_auth_info() -> AuthInfo {
    AuthInfo {
        username: std::env::var("USERNAME").unwrap_or(DEFAULT_USERNAME.to_string()),
        password: std::env::var("PASSWORD").unwrap_or(DEFAULT_PASSWORD.to_string()), 
    }
}
