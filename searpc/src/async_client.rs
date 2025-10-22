//! Async RPC client implementation
//!
//! Provides async versions of all RPC call methods.

#[cfg(feature = "async")]
use crate::{async_transport::AsyncTransport, protocol::*, types::Arg, Result};
#[cfg(feature = "async")]
use serde_json::Value;

/// Async Searpc RPC client
///
/// This is the async version of [`SearpcClient`](crate::SearpcClient).
/// All methods are async and require a tokio runtime.
///
/// ## Example
///
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use searpc::{AsyncSearpcClient, AsyncTcpTransport, Arg};
///
/// let transport = AsyncTcpTransport::connect("127.0.0.1:12345").await?;
/// let mut client = AsyncSearpcClient::new(transport);
///
/// let result = client.call_int("strlen", vec![Arg::string("hello")]).await?;
/// println!("Length: {}", result);
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "async")]
pub struct AsyncSearpcClient<T: AsyncTransport> {
    transport: T,
}

#[cfg(feature = "async")]
impl<T: AsyncTransport> AsyncSearpcClient<T> {
    /// Create a new async RPC client with the given transport
    pub fn new(transport: T) -> Self {
        AsyncSearpcClient { transport }
    }

    /// Make an RPC call expecting an integer result
    pub async fn call_int(&mut self, fname: &str, args: Vec<Arg>) -> Result<i32> {
        let request = RpcRequest {
            function_name: fname.to_string(),
            args,
        };

        let request_json = request.to_json()?;
        let response_data = self.transport.send(request_json.as_bytes()).await?;

        let response_str = std::str::from_utf8(&response_data)
            .map_err(|e| crate::SearpcError::InvalidResponse(e.to_string()))?;
        let response = RpcResponse::from_json(response_str)?;

        let value = response.into_result()?;
        value
            .as_i64()
            .map(|v| v as i32)
            .ok_or_else(|| crate::SearpcError::TypeError("Expected int".to_string()))
    }

    /// Make an RPC call expecting a 64-bit integer result
    pub async fn call_int64(&mut self, fname: &str, args: Vec<Arg>) -> Result<i64> {
        let request = RpcRequest {
            function_name: fname.to_string(),
            args,
        };

        let request_json = request.to_json()?;
        let response_data = self.transport.send(request_json.as_bytes()).await?;

        let response_str = std::str::from_utf8(&response_data)
            .map_err(|e| crate::SearpcError::InvalidResponse(e.to_string()))?;
        let response = RpcResponse::from_json(response_str)?;

        let value = response.into_result()?;
        value
            .as_i64()
            .ok_or_else(|| crate::SearpcError::TypeError("Expected int64".to_string()))
    }

    /// Make an RPC call expecting a string result
    pub async fn call_string(&mut self, fname: &str, args: Vec<Arg>) -> Result<String> {
        let request = RpcRequest {
            function_name: fname.to_string(),
            args,
        };

        let request_json = request.to_json()?;
        let response_data = self.transport.send(request_json.as_bytes()).await?;

        let response_str = std::str::from_utf8(&response_data)
            .map_err(|e| crate::SearpcError::InvalidResponse(e.to_string()))?;
        let response = RpcResponse::from_json(response_str)?;

        let value = response.into_result()?;
        value
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| crate::SearpcError::TypeError("Expected string".to_string()))
    }

    /// Make an RPC call expecting a JSON object result
    pub async fn call_object(&mut self, fname: &str, args: Vec<Arg>) -> Result<Value> {
        let request = RpcRequest {
            function_name: fname.to_string(),
            args,
        };

        let request_json = request.to_json()?;
        let response_data = self.transport.send(request_json.as_bytes()).await?;

        let response_str = std::str::from_utf8(&response_data)
            .map_err(|e| crate::SearpcError::InvalidResponse(e.to_string()))?;
        let response = RpcResponse::from_json(response_str)?;

        response.into_result()
    }

    /// Make an RPC call expecting a list of JSON objects
    pub async fn call_objlist(&mut self, fname: &str, args: Vec<Arg>) -> Result<Vec<Value>> {
        let request = RpcRequest {
            function_name: fname.to_string(),
            args,
        };

        let request_json = request.to_json()?;
        let response_data = self.transport.send(request_json.as_bytes()).await?;

        let response_str = std::str::from_utf8(&response_data)
            .map_err(|e| crate::SearpcError::InvalidResponse(e.to_string()))?;
        let response = RpcResponse::from_json(response_str)?;

        let value = response.into_result()?;
        value
            .as_array()
            .cloned()
            .ok_or_else(|| crate::SearpcError::TypeError("Expected array".to_string()))
    }

    /// Make an RPC call expecting a JSON value result
    pub async fn call_json(&mut self, fname: &str, args: Vec<Arg>) -> Result<Value> {
        let request = RpcRequest {
            function_name: fname.to_string(),
            args,
        };

        let request_json = request.to_json()?;
        let response_data = self.transport.send(request_json.as_bytes()).await?;

        let response_str = std::str::from_utf8(&response_data)
            .map_err(|e| crate::SearpcError::InvalidResponse(e.to_string()))?;
        let response = RpcResponse::from_json(response_str)?;

        response.into_result()
    }
}
