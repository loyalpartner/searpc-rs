use crate::error::{Result, SearpcError};
use crate::protocol::{RpcRequest, RpcResponse};
use crate::transport::Transport;
use crate::types::Arg;
use serde_json::Value;

/// Searpc RPC Client
///
/// Good taste: simple struct, single responsibility
pub struct SearpcClient<T: Transport> {
    transport: T,
}

impl<T: Transport> SearpcClient<T> {
    pub fn new(transport: T) -> Self {
        SearpcClient { transport }
    }

    /// Low-level call: returns raw JSON Value
    pub fn call(&mut self, function_name: &str, args: Vec<Arg>) -> Result<Value> {
        // 1. Create request
        let request = RpcRequest::with_args(function_name, args);
        let request_json = request.to_json()?;

        // 2. Send via transport
        let response_bytes = self.transport.send(request_json.as_bytes())?;

        // 3. Parse response
        let response_str = std::str::from_utf8(&response_bytes).map_err(|e| {
            SearpcError::InvalidResponse(format!("Response is not valid UTF-8: {}", e))
        })?;

        let response = RpcResponse::from_json(response_str)?;

        // 4. Check for errors and return result
        response.into_result()
    }

    /// Call function expecting int return type
    pub fn call_int(&mut self, function_name: &str, args: Vec<Arg>) -> Result<i32> {
        let value = self.call(function_name, args)?;
        value
            .as_i64()
            .and_then(|v| i32::try_from(v).ok())
            .ok_or_else(|| SearpcError::TypeError(format!("Expected int, got: {:?}", value)))
    }

    /// Call function expecting int64 return type
    pub fn call_int64(&mut self, function_name: &str, args: Vec<Arg>) -> Result<i64> {
        let value = self.call(function_name, args)?;
        value
            .as_i64()
            .ok_or_else(|| SearpcError::TypeError(format!("Expected int64, got: {:?}", value)))
    }

    /// Call function expecting string return type
    pub fn call_string(&mut self, function_name: &str, args: Vec<Arg>) -> Result<String> {
        let value = self.call(function_name, args)?;
        value
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| SearpcError::TypeError(format!("Expected string, got: {:?}", value)))
    }

    /// Call function expecting object return type (returns JSON Value)
    pub fn call_object(&mut self, function_name: &str, args: Vec<Arg>) -> Result<Value> {
        let value = self.call(function_name, args)?;
        if value.is_object() || value.is_null() {
            Ok(value)
        } else {
            Err(SearpcError::TypeError(format!(
                "Expected object, got: {:?}",
                value
            )))
        }
    }

    /// Call function expecting objlist return type (returns Vec of JSON Values)
    pub fn call_objlist(&mut self, function_name: &str, args: Vec<Arg>) -> Result<Vec<Value>> {
        let value = self.call(function_name, args)?;

        // Handle null as empty array (Seafile daemon returns null for empty lists)
        if value.is_null() {
            return Ok(Vec::new());
        }

        value
            .as_array()
            .map(|arr| arr.clone())
            .ok_or_else(|| SearpcError::TypeError(format!("Expected array, got: {:?}", value)))
    }

    /// Call function expecting JSON return type
    pub fn call_json(&mut self, function_name: &str, args: Vec<Arg>) -> Result<Value> {
        self.call(function_name, args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_transport(expected_req: &str, response: &str) -> impl FnMut(&[u8]) -> Result<Vec<u8>> {
        let expected = expected_req.to_string();
        let resp = response.to_string();

        move |req: &[u8]| -> Result<Vec<u8>> {
            let req_str = std::str::from_utf8(req).unwrap();
            assert_eq!(req_str, expected);
            Ok(resp.as_bytes().to_vec())
        }
    }

    #[test]
    fn test_call_int() {
        let transport = mock_transport(r#"["searpc_strlen","hello"]"#, r#"{"ret": 5}"#);

        let mut client = SearpcClient::new(transport);
        let result = client
            .call_int("searpc_strlen", vec!["hello".into()])
            .unwrap();
        assert_eq!(result, 5);
    }

    #[test]
    fn test_call_string() {
        let transport = mock_transport(r#"["get_version"]"#, r#"{"ret": "1.0.0"}"#);

        let mut client = SearpcClient::new(transport);
        let result = client.call_string("get_version", vec![]).unwrap();
        assert_eq!(result, "1.0.0");
    }

    #[test]
    fn test_call_error() {
        let transport = mock_transport(
            r#"["bad_func"]"#,
            r#"{"err_code": 404, "err_msg": "Function not found"}"#,
        );

        let mut client = SearpcClient::new(transport);
        let result = client.call_int("bad_func", vec![]);

        match result {
            Err(SearpcError::RpcError { code, message }) => {
                assert_eq!(code, 404);
                assert_eq!(message, "Function not found");
            }
            _ => panic!("Expected RpcError"),
        }
    }
}
