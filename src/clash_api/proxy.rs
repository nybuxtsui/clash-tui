use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyData {
    pub proxies: HashMap<String, ProxyItem>,
    pub providers: HashMap<String, ProviderItem>,
    // 可切换代理
    pub groups: Vec<String>,
    // 代理提供者
    pub proxy_providers: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub providers: HashMap<String, ProviderItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderItem {
    pub name: String,
    #[serde(rename = "vehicleType")]
    pub vehicle_type: String,
    #[serde(default = "String::new")]
    pub now: String,
    pub proxies: Vec<ProviderProxy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderProxy {
    pub name: String,
    #[serde(default = "Vec::new")]
    pub history: Vec<HistoryItem>,
}

impl ProxyItem {
    pub fn get_delay(&self, proxies: &HashMap<String, ProxyItem>) -> String {
        if self.now == "" {
            self.history.last().map_or("".to_string(), |x| {
                if x.delay == 0 {
                    "-".to_string()
                } else {
                    format!("{}ms", x.delay)
                }
            })
        } else {
            proxies.get(&self.now).unwrap().get_delay(proxies)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proxy {
    pub proxies: HashMap<String, ProxyItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub time: String,
    pub delay: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyItem {
    pub name: String,
    #[serde(default = "Vec::new")]
    pub all: Vec<String>,
    #[serde(default = "String::new")]
    pub now: String,
    pub r#type: String,
    #[serde(default = "Vec::new")]
    pub history: Vec<HistoryItem>,
}

impl ProxyData {
    pub fn to_group_items(&self, name: &str) -> Vec<Vec<String>> {
        let proxy = self.proxies.get(name).unwrap();
        let now = &proxy.now;

        self.proxies
            .get(name)
            .unwrap()
            .all
            .iter()
            .map(|v| {
                vec![
                    v.clone(),
                    self.proxies.get(v).unwrap().get_delay(&self.proxies),
                    if now == v { "✓" } else { "" }.to_string(),
                ]
            })
            .collect()
    }

    pub fn to_groups(&self) -> Vec<Vec<String>> {
        self.groups
            .iter()
            .map(|x| {
                let proxy = self.proxies.get(x).unwrap();
                let now = proxy.now.clone();
                let delay = self.proxies.get(&now).unwrap().get_delay(&self.proxies);
                vec![proxy.name.clone(), now, delay]
            })
            .collect()
    }
}
