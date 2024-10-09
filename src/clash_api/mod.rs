mod log;
mod proxy;
mod connection;

pub use log::LogItem;
use std::collections::HashMap;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tokio::join;
use crate::app_config::get_config;
pub use connection::{Connection, ConnectionItem};
pub use proxy::{Provider, ProviderItem, Proxy, ProxyData, ProxyItem};

async fn http_get<U: AsRef<str>, T: DeserializeOwned>(uri: U, params: &[(U, U)]) -> anyhow::Result<T> {
    let config = get_config();
    let mut url = format!("http://{}{}", config.host, uri.as_ref());
    if !params.is_empty() {
        url = reqwest::Url::parse_with_params(&url, params).unwrap().to_string();
    }
    let mut proxies = Client::default().get(url);
    if !config.key.is_empty() {
        proxies = proxies.header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", config.key),
        )
    }

    let value = proxies
        .send()
        .await?
        .json::<T>()
        .await?;
    Ok(value)
}

pub async fn load_proxy() -> anyhow::Result<ProxyData> {
    async fn get_proxies() ->anyhow::Result<HashMap<String, ProxyItem>> {
        Ok(http_get::<&str, Proxy>("/proxies", &[]).await?.proxies)
    }
    async fn get_providers() -> anyhow::Result<HashMap<String, ProviderItem>> {
        Ok(http_get::<&str, Provider>("/providers/proxies", &[]).await?.providers)
    }

    let result = join!(
        get_proxies(),
        get_providers()
    );

    Ok(ProxyData {
        proxies: result.0?,
        providers: result.1?,
    })
}

pub async fn check_delay(group: &str) -> anyhow::Result<()> {
    let params = [
        ("url", "https://www.gstatic.com/generate_204"),
        ("timeout", "5000"),
    ];
    let _: Value = http_get(format!("/group/{group}/delay").as_str(), &params).await?;
    Ok(())
}

pub async fn select_group_current(group: &str, current: &str) {
    let config = get_config();
    let url = format!("http://{}/proxies/{group}", config.host);
    let resp = Client::default()
        .put(url)
        .json(&json!({"name":current}))
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", config.key),
        )
        .send()
        .await
        .unwrap();
    resp.text().await.unwrap();
}
