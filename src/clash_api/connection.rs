use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub host: String,
    #[serde(rename = "sniffHost")]
    #[serde(default = "Default::default")]
    pub sniff_host: String,
    #[serde(rename = "destinationIP")]
    pub destination_ip: String,
    #[serde(rename = "destinationPort")]
    pub destination_port: String,

    #[serde(rename = "sourceIP")]
    pub source_ip: String,
    #[serde(rename = "sourcePort")]
    pub source_port: String,
    #[serde(rename = "inboundName")]
    #[serde(default = "Default::default")]
    pub inbound_name: String,
    pub network: String,

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionItem {
    pub id: String,
    pub chains: Vec<String>,
    pub metadata: Metadata,
    pub rule: String,
    #[serde(rename = "rulePayload")]
    pub rule_payload: String,
    pub start: String,
    pub download: u64,
    pub upload: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_null_default")]
    pub connections: Vec<ConnectionItem>,
    #[serde(rename = "downloadTotal")]
    pub download_total: u64,
    #[serde(rename = "uploadTotal")]
    pub upload_total: u64,
}

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}
