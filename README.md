# Watchtower-rs
## Overview
Watchtower-rs is a service discovery written in Rust.

# Getting Started
## Running the Registry
To start the service,
```
cargo run
```
## Connecting as a Client
### Rust Client
The library includes a Rust client. To include in your project, add the following to your Cargo.toml file.
```toml
watchtower-client = { git = "https://github.com/one-man-one-laptop/watchtower-rs", branch = "main" }
```
The basic functionalities of the client can be described as followed:
```rust
use watchtower_client::{WatchtowerClient, Error};

const watchtower_urls = vec!["http://localhost:8088"];
const USERNAME: &str = "admin";
const PASSWORD: &str = "password";

async fn main() {
    let watchtower_client = WatchtowerClient::new(watchtower_urls, USERNAME, PASSWORD);

    // To register a service
    let url = "127.0.0.1";
    let port = 1234;
    let service_id = "some_service_name";
    watchtower_client.register(service_id, url, port).await.unwrap();

    // To get the url of a service
    let service_url = watchtower_client.get_service_url(service_id).await.unwrap();
}
```

### Python Client
To install the python client,
```
pip install watchtower-client
```
Unlike the Rust client, in order to keep the service on the registry, you will have to manually call the ping function.
```python
from watchtower_client import PyWatchtowerClient

watchtower_client = PyWatchtowerClient(["http://127.0.0.1:8088"], "admin", "password")

# To register a service
url = "127.0.0.1"
port = 1234
service_id = "some_service_name"
watchtower_client.register(service_id, url, port)

# To keep the service on the registry, do this every 30 seconds
watchtower_client.ping()

# To get the url of a service
service_url = watchtower_client.get_service_url("traffic_control") 
```

### Custom Client
You may write your own client and make the appropriate http requests in order to register, get, and keep a service on the registry.

# Limitations
Currently, the service provider only works with http connection. This capability will be expanded in the future.