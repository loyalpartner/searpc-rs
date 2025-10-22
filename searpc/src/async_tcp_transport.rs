//! Async TCP transport implementation (16-bit header, big-endian)
//!
//! This transport is compatible with the libsearpc C demo server.
//! Uses tokio for async I/O.

#[cfg(feature = "async")]
use crate::{async_transport::AsyncTransport, error::SearpcError, Result};
#[cfg(feature = "async")]
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(feature = "async")]
use tokio::net::TcpStream;

/// Async TCP transport with 16-bit big-endian length header
///
/// Compatible with libsearpc C demo server protocol.
/// Maximum packet size: 64KB (u16 limit)
///
/// ## Example
///
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use searpc::{AsyncTcpTransport, AsyncSearpcClient, Arg};
///
/// let transport = AsyncTcpTransport::connect("127.0.0.1:12345").await?;
/// let mut client = AsyncSearpcClient::new(transport);
///
/// let result = client.call_int("strlen", vec![Arg::string("hello")]).await?;
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "async")]
pub struct AsyncTcpTransport {
    stream: TcpStream,
}

#[cfg(feature = "async")]
impl AsyncTcpTransport {
    /// Connect to a TCP server
    pub async fn connect(addr: impl tokio::net::ToSocketAddrs) -> Result<Self> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|e| SearpcError::TransportError(e.to_string()))?;

        Ok(AsyncTcpTransport { stream })
    }

    /// Send a packet with 16-bit big-endian length header
    async fn send_packet(&mut self, data: &[u8]) -> Result<()> {
        let len = data.len();
        if len > u16::MAX as usize {
            return Err(SearpcError::TransportError(format!(
                "Packet too large: {} > {}",
                len,
                u16::MAX
            )));
        }

        // Send 16-bit big-endian length
        let len_bytes = (len as u16).to_be_bytes();
        self.stream
            .write_all(&len_bytes)
            .await
            .map_err(|e| SearpcError::TransportError(e.to_string()))?;

        // Send data
        self.stream
            .write_all(data)
            .await
            .map_err(|e| SearpcError::TransportError(e.to_string()))?;

        Ok(())
    }

    /// Receive a packet with 16-bit big-endian length header
    async fn recv_packet(&mut self) -> Result<Vec<u8>> {
        // Read 16-bit big-endian length
        let mut len_bytes = [0u8; 2];
        self.stream
            .read_exact(&mut len_bytes)
            .await
            .map_err(|e| SearpcError::TransportError(e.to_string()))?;

        let len = u16::from_be_bytes(len_bytes) as usize;

        // Read data
        let mut data = vec![0u8; len];
        self.stream
            .read_exact(&mut data)
            .await
            .map_err(|e| SearpcError::TransportError(e.to_string()))?;

        Ok(data)
    }
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl AsyncTransport for AsyncTcpTransport {
    async fn send(&mut self, request: &[u8]) -> Result<Vec<u8>> {
        self.send_packet(request).await?;
        self.recv_packet().await
    }
}

#[cfg(all(test, feature = "async"))]
mod tests {
    use super::*;

    #[test]
    fn test_packet_encoding() {
        // Test that packet length is encoded as big-endian
        let len: u16 = 0x1234;
        let bytes = len.to_be_bytes();
        assert_eq!(bytes, [0x12, 0x34]);

        let decoded = u16::from_be_bytes(bytes);
        assert_eq!(decoded, 0x1234);
    }
}
