use watchtower_client::{WatchtowerClient, Error};

const WATCHTOWER_URL: &str = "http://localhost:8088";

const USERNAME: &str = "admin";
const PASSWORD: &str = "password";

fn get_watchtower_urls() -> Vec<String> {
    match std::env::var("WATCHTOWER_URLS") {
        Ok(var) => {
            var.split(',').map(|url| url.to_string()).collect()
        },
        Err(_) => vec![WATCHTOWER_URL.to_string()]
    }
}

#[actix_rt::test]
async fn test_register_and_get_service() {
    let watchtower_client = WatchtowerClient::new(get_watchtower_urls(), USERNAME, PASSWORD);

    let url = "127.0.0.1";
    let port = 1234;
    let service_id = "test_register_and_get_service";
    watchtower_client.register(service_id, url, port).await.unwrap();

    let service_url = watchtower_client.get_service_url(service_id).await.unwrap();
    assert_eq!(service_url, format!("{}:{}", url, port));
    let sleep_time = std::time::Duration::from_millis(40 * 1000);
    std::thread::sleep(sleep_time);
    let service_url = watchtower_client.get_service_url(service_id).await.unwrap();
    assert_eq!(service_url, format!("{}:{}", url, port));
    watchtower_client.cancel().await.unwrap();
}

#[actix_rt::test]
async fn test_get_non_existent_service() {
    let watchtower_client = WatchtowerClient::new(get_watchtower_urls(), USERNAME, PASSWORD);
    let maybe_service = watchtower_client.get_service_url("imaginary").await;
    assert_eq!(maybe_service, Err(Error::NotFound));
}

#[actix_rt::test]
async fn test_unauthorized() {
    let watchtower_client = WatchtowerClient::new(get_watchtower_urls(), USERNAME, "whatever");
    let maybe_service = watchtower_client.get_service_url("foo").await;
    assert_eq!(maybe_service, Err(Error::Unauthorized));
    assert_eq!(watchtower_client.register("bar", "127.0.0.1", 1234).await, Err(Error::Unauthorized));
}

#[actix_rt::test]
async fn test_register_twice() {
    let watchtower_client = WatchtowerClient::new(get_watchtower_urls(), USERNAME, PASSWORD);

    let url = "127.0.0.1";
    let port = 1234;
    let service_id = "test_register_twice";
    watchtower_client.register(service_id, url, port).await.unwrap();

    assert_eq!(watchtower_client.register("bar", "127.0.0.1", 1234).await, Err(Error::InstanceAlreadyRegistered));

    watchtower_client.cancel().await.unwrap();
    watchtower_client.register(service_id, url, port).await.unwrap();
    watchtower_client.cancel().await.unwrap();
}

#[actix_rt::test]
async fn test_register_then_cancel() {
    let watchtower_client = WatchtowerClient::new(get_watchtower_urls(), USERNAME, PASSWORD);

    let url = "127.0.0.1";
    let port = 2345;
    let service_id = "test_register_then_cancel";
    watchtower_client.register(service_id, url, port).await.unwrap();
    watchtower_client.cancel().await.unwrap();

    assert_eq!(watchtower_client.get_service_url("test_register_then_cancel").await, Err(Error::NotFound));
}
