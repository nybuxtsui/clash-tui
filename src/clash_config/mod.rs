mod subs_url;

use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use subs_url::SubsUrl;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subs: Vec<Subscription>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub connection: Vec<Connection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Subscription {
    pub id: String,
    pub name: String,
    pub url: String,
}

fn is_zero(value: &u16) -> bool {
    *value == 0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    #[serde(rename = "type")]
    pub conn_type: String,
    pub tag: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub server: String,
    #[serde(skip_serializing_if = "is_zero")]
    pub server_port: u16,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub method: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub password: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<Tls>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tls {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub server_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insecure: Option<bool>,
}

#[allow(dead_code)]
pub async fn subscribe<T: AsRef<str>>(url: T) -> anyhow::Result<Vec<Connection>> {
    let resp = Client::default().get(url.as_ref()).send().await?;
    if resp.status().as_u16() >= 300 {
        return Err(anyhow::anyhow!(
            "http请求错误: code={}, resp={}",
            resp.status(),
            resp.text().await.unwrap_or("未知错误".to_string())
        ));
    }
    decode(resp.text().await?)
}

fn decode<T: AsRef<[u8]>>(data: T) -> anyhow::Result<Vec<Connection>> {
    let data = String::from_utf8(BASE64_STANDARD_NO_PAD.decode(data)?)?;
    Ok(data
        .lines()
        .filter_map(|line| {
            decode_item(line).unwrap_or_else(|e| {
                println!("decode_item failed: {e}");
                None
            })
        })
        .collect())
}

fn decode_item<T: AsRef<str>>(line: T) -> anyhow::Result<Option<Connection>> {
    let url = SubsUrl::new(line.as_ref())?;

    let value = match url.scheme().as_ref() {
        "trojan" => Some(Connection {
            conn_type: "trojan".to_string(),
            tag: url.fragment(),
            server: url.server(),
            server_port: url.port(),
            password: url.username(),
            method: "".to_owned(),
            tls: Some(Tls {
                enabled: Some(true),
                server_name: url.query("sni"),
                insecure: Some(url.query("allowInsecure") == "1"),
            }),
        }),
        "ss" => Some(Connection {
            conn_type: String::from("shadowsocks"),
            tag: url.fragment(),
            server: url.server(),
            server_port: url.port(),
            method: url.username(),
            password: url.password(),
            tls: None,
        }),
        _ => None,
    };
    Ok(value)
}
