use serde::Deserialize;
use serde_json::Value;
use crate::types::Arg;
use crate::error::{SearpcError, Result};

/// RPC Request
///
/// Serializes to: ["function_name", arg1, arg2, ...]
///
/// Good taste: simple array format, no nested objects
#[derive(Debug, Clone)]
pub struct RpcRequest {
    pub function_name: String,
    pub args: Vec<Arg>,
}

impl RpcRequest {
    pub fn new(function_name: impl Into<String>) -> Self {
        RpcRequest {
            function_name: function_name.into(),
            args: Vec::new(),
        }
    }

    pub fn with_args(function_name: impl Into<String>, args: Vec<Arg>) -> Self {
        RpcRequest {
            function_name: function_name.into(),
            args,
        }
    }

    pub fn add_arg(&mut self, arg: impl Into<Arg>) {
        self.args.push(arg.into());
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String> {
        // Build the array: [fname, arg1, arg2, ...]
        let mut arr = Vec::with_capacity(1 + self.args.len());
        arr.push(Value::String(self.function_name.clone()));

        for arg in &self.args {
            arr.push(serde_json::to_value(arg)?);
        }

        Ok(serde_json::to_string(&arr)?)
    }
}

/// RPC Response
///
/// Deserializes from: {"ret": value, "err_code": code, "err_msg": msg}
#[derive(Debug, Clone, Deserialize)]
pub struct RpcResponse {
    #[serde(default)]
    pub ret: Option<Value>,

    pub err_code: Option<i32>,
    pub err_msg: Option<String>,
}

impl RpcResponse {
    /// Parse from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Check if response contains an error
    pub fn into_result(self) -> Result<Value> {
        if let Some(code) = self.err_code {
            return Err(SearpcError::RpcError {
                code,
                message: self.err_msg.unwrap_or_else(|| "Unknown error".to_string()),
            });
        }

        // ret can be None (when key missing) or Some(Value::Null)
        // Both cases are valid - treat null as a valid empty result
        match self.ret {
            None => Ok(Value::Null),
            Some(v) => Ok(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let mut req = RpcRequest::new("get_substring");
        req.add_arg("hello");
        req.add_arg(2);

        let json = req.to_json().unwrap();
        assert_eq!(json, r#"["get_substring","hello",2]"#);
    }

    #[test]
    fn test_response_success() {
        let json = r#"{"ret": 42}"#;
        let resp = RpcResponse::from_json(json).unwrap();
        let value = resp.into_result().unwrap();
        assert_eq!(value.as_i64(), Some(42));
    }

    #[test]
    fn test_response_error() {
        let json = r#"{"err_code": 500, "err_msg": "Test error"}"#;
        let resp = RpcResponse::from_json(json).unwrap();
        let err = resp.into_result().unwrap_err();

        match err {
            SearpcError::RpcError { code, message } => {
                assert_eq!(code, 500);
                assert_eq!(message, "Test error");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_response_with_string() {
        let json = r#"{"ret": "hello world"}"#;
        let resp = RpcResponse::from_json(json).unwrap();
        let value = resp.into_result().unwrap();
        assert_eq!(value.as_str(), Some("hello world"));
    }
}
