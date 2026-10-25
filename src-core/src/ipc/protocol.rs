use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    pub id: String,
    pub command: String,
    pub nonce: String,
    pub payload: serde_json::Value,
}

#[derive(Clone, Serialize, Deserialize)]
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

impl std::fmt::Debug for IpcRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("IpcRequest")
            .field("id", &self.id)
            .field("command", &self.command)
            .field("payload", &"[REDACTED]")
            .finish()
    }
}
impl std::fmt::Debug for IpcResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("IpcResponse")
            .field("id", &self.id)
            .field("success", &self.success)
            .field("data", &"[REDACTED]")
            .finish()
    }
}
pub fn wipe_value(v: &mut serde_json::Value) {
    use zeroize::Zeroize;
    match v {
        serde_json::Value::String(s) => s.zeroize(),
        serde_json::Value::Array(a) => {
            for v in a {
                wipe_value(v)
            }
        }
        serde_json::Value::Object(o) => {
            for v in o.values_mut() {
                wipe_value(v)
            }
        }
        _ => {}
    }
}
impl Drop for IpcRequest {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.nonce.zeroize();
        wipe_value(&mut self.payload);
    }
}
impl Drop for IpcResponse {
    fn drop(&mut self) {
        if let Some(data) = &mut self.data {
            wipe_value(data)
        }
    }
}
