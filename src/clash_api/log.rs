use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogItem {
    pub r#type: String,
    pub payload: String,
}
