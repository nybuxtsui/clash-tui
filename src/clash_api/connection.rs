use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub host: String,
    #[serde(rename = "sniffHost")]
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
    pub download: u32,
    pub upload: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub connections: Vec<ConnectionItem>,
    #[serde(rename = "downloadTotal")]
    pub download_total: u32,
    #[serde(rename = "uploadTotal")]
    pub upload_total: u32,
}
