//! Unix Domain Socket transport with 32-bit length header
//!
//! This matches the production protocol used by Seafile:
//! ```
//! ┌─────────────┬──────────────────┐
//! │ Length(4B)  │  Wrapped JSON    │
//! │ (uint32)    │  (variable)      │
//! └─────────────┴──────────────────┘
//! ```
//!
//! The wrapped JSON contains:
//! ```json
//! {
//!   "service": "service-name",
//!   "request": ["function_name", arg1, arg2, ...]
//! }
//! ```
//!
//! And the response is still the standard format:
//! ```json
//! {"ret": value, "err_code": code, "err_msg": msg}
//! ```

use crate::error::{Result, SearpcError};
use crate::transport::Transport;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

/// Unix Domain Socket transport
///
/// Uses 32-bit length header (matching Seafile's named pipe transport)
pub struct UnixSocketTransport {
    stream: UnixStream,
    service: String,
}

impl UnixSocketTransport {
    pub fn new(stream: UnixStream, service: impl Into<String>) -> Self {
        UnixSocketTransport {
            stream,
            service: service.into(),
        }
    }

    pub fn connect(path: impl AsRef<Path>, service: impl Into<String>) -> std::io::Result<Self> {
        let stream = UnixStream::connect(path)?;
        Ok(UnixSocketTransport {
            stream,
            service: service.into(),
        })
    }

    /// Read exactly n bytes
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.stream
            .read_exact(buf)
            .map_err(|e| SearpcError::TransportError(format!("Read failed: {}", e)))
    }

    /// Write all bytes
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.stream
            .write_all(buf)
            .map_err(|e| SearpcError::TransportError(format!("Write failed: {}", e)))
    }

    /// Send a packet with service wrapper
    fn send_packet(&mut self, rpc_request: &[u8]) -> Result<()> {
        // Wrap the RPC request with service info
        let wrapped = self.wrap_request(rpc_request)?;

        let len = wrapped.len() as u32;

        // Write length (4 bytes, native endian - matches C code using guint32)
        self.write_all(&len.to_ne_bytes())?;

        // Write data
        self.write_all(&wrapped)?;

        Ok(())
    }

    /// Receive a packet
    fn recv_packet(&mut self) -> Result<Vec<u8>> {
        // Read length (4 bytes, native endian)
        let mut len_buf = [0u8; 4];
        self.read_exact(&mut len_buf)?;
        let len = u32::from_ne_bytes(len_buf) as usize;

        if len == 0 {
            return Err(SearpcError::TransportError(
                "Received packet with zero length".to_string(),
            ));
        }

        // Read data
        let mut data = vec![0u8; len];
        self.read_exact(&mut data)?;

        Ok(data)
    }

    /// Wrap RPC request in service envelope
    ///
    /// Input: ["function_name", arg1, arg2, ...]  (as JSON string)
    /// Output: {"service": "xxx", "request": "[\"function_name\",arg1,...]"}  (request as STRING)
    ///
    /// IMPORTANT: The 'request' field must be a JSON-encoded STRING, not a JSON object!
    /// This matches Python's pysearpc implementation:
    ///   json.dumps({'service': service, 'request': fcall_str})
    /// where fcall_str is already a JSON string like '["func",arg1,arg2]'
    fn wrap_request(&self, rpc_request: &[u8]) -> Result<Vec<u8>> {
        use serde_json::json;

        let request_str = std::str::from_utf8(rpc_request).map_err(|e| {
            SearpcError::InvalidResponse(format!("Request is not valid UTF-8: {}", e))
        })?;

        // CRITICAL: Keep request as a string, don't parse it as JSON!
        // The server expects: {"service":"...", "request":"[...]"}
        // NOT: {"service":"...", "request":[...]}
        let wrapped = json!({
            "service": &self.service,
            "request": request_str  // Pass as string, not parsed JSON
        });

        Ok(serde_json::to_vec(&wrapped)?)
    }
}

impl Transport for UnixSocketTransport {
    fn send(&mut self, request: &[u8]) -> Result<Vec<u8>> {
        self.send_packet(request)?;
        self.recv_packet()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_encoding() {
        // Test that length is encoded as native endian (same as C guint32)
        let len: u32 = 0x12345678;
        let bytes = len.to_ne_bytes();

        #[cfg(target_endian = "little")]
        assert_eq!(bytes, [0x78, 0x56, 0x34, 0x12]);

        #[cfg(target_endian = "big")]
        assert_eq!(bytes, [0x12, 0x34, 0x56, 0x78]);

        let decoded = u32::from_ne_bytes(bytes);
        assert_eq!(decoded, 0x12345678);
    }

    #[test]
    fn test_wrap_request() {
        let transport = UnixSocketTransport {
            stream: UnixStream::pair().unwrap().0,
            service: "test-service".to_string(),
        };

        let rpc_request = r#"["get_version"]"#.as_bytes();
        let wrapped = transport.wrap_request(rpc_request).unwrap();
        let wrapped_str = std::str::from_utf8(&wrapped).unwrap();

        assert!(wrapped_str.contains("\"service\":\"test-service\""));
        // request 字段是 JSON 字符串，不是直接的数组
        assert!(wrapped_str.contains("\"request\":\"[\\\"get_version\\\"]\""));
    }
}
