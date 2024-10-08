mod log;
mod proxy;
mod connection;

pub use log::LogItem;
use std::collections::HashMap;

use anyhow::anyhow;
use reqwest::Client;
use serde_json::json;
use std::sync::{LazyLock, Mutex};
use tokio::join;

pub use proxy::{Provider, ProviderItem, Proxy, ProxyData, ProxyItem};
pub use connection::{Connection, ConnectionItem};

#[derive(Default, Clone)]
pub struct Config {
    pub host: String,
    pub key: String,
}

pub fn get_config() -> Config {
    let config = CONFIG.lock().unwrap();
    config.clone()
}

pub static CONFIG: LazyLock<Mutex<Config>> = LazyLock::new(|| {
    Mutex::new(Config {
        host: "127.0.0.1:9090".to_string(),
        key: "".to_string(),
    })
});

pub fn load_config() {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("clash-tui.ini"))
        .build()
        .expect("无法加载配置文件");

    let host: String = settings.get("host").unwrap_or("127.0.0.1:9090".to_string());
    let key: String = settings.get("key").unwrap_or("".to_string());
    let mut config = CONFIG.lock().unwrap();
    config.host = host;
    config.key = key;
}

pub async fn load_proxy() -> anyhow::Result<ProxyData> {
    let config = get_config();
    async fn get_proxies(config: &Config) -> anyhow::Result<HashMap<String, ProxyItem>> {
        let proxies = Client::default()
            .get(format!("http://{}/proxies", config.host))
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", config.key),
            )
            .send()
            .await?
            .json::<Proxy>()
            .await?
            .proxies;
        Ok(proxies)
    }
    async fn get_providers(config: &Config) -> anyhow::Result<HashMap<String, ProviderItem>> {
        let providers = Client::default()
            .get(format!("http://{}/providers/proxies", config.host))
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", config.key),
            )
            .send()
            .await?
            .json::<Provider>()
            .await?
            .providers;
        Ok(providers)
    }

    let (proxies, providers) = join!(get_proxies(&config), get_providers(&config));
    let (proxies, providers) = (proxies?, providers?);

    let global = proxies.get("GLOBAL");
    if global.is_none() {
        return Err(anyhow!("接口返回错误，GLOBAL不存在"));
    }
    let mut groups = global
        .unwrap()
        .all
        .iter()
        .filter(|it| proxies.get(*it).map_or(false, |x| !x.all.is_empty()))
        .map(|x| x.clone())
        .collect::<Vec<String>>();
    groups.push("GLOBAL".into());

    let proxy_providers = providers
        .values()
        .filter(|it| it.name != "default" && it.vehicle_type != "Compatible")
        .map(|it| it.name.clone())
        .collect::<Vec<String>>();

    Ok(ProxyData {
        proxies,
        providers,
        groups,
        proxy_providers,
    })
}

pub async fn check_delay(group: &str) {
    let config = get_config();

    let url = format!("http://{}/group/{group}/delay", config.host);
    let params = [
        ("url", "https://www.gstatic.com/generate_204"),
        ("timeout", "5000"),
    ];
    let url = reqwest::Url::parse_with_params(&url, &params).unwrap();
    let resp = Client::default()
        .get(url)
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", config.key),
        )
        .send()
        .await
        .unwrap();
    resp.text().await.unwrap();
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
