use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    pub id: String,
    pub command: String,
    pub nonce: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl IpcResponse {
    pub fn ok(id: &str, data: serde_json::Value) -> Self {
        Self {
            id: id.to_string(),
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(id: &str, message: &str) -> Self {
        Self {
            id: id.to_string(),
            success: false,
            data: None,
            error: Some(message.to_string()),
        }
    }
}
