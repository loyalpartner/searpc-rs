use crate::error::{Result, SearpcError};
use crate::transport::Transport;
///! TCP transport with packet protocol
///!
///! Packet format (matching libsearpc demo):
///! ```
///! ┌─────────────┬──────────────────┐
///! │ Length(2B)  │  JSON Data       │
///! │ (uint16_t)  │  (variable)      │
///! └─────────────┴──────────────────┘
///! ```
///! Length is in network byte order (big-endian)
use std::io::{Read, Write};
use std::net::TcpStream;

const MAX_PACKET_SIZE: usize = 65535; // uint16 max

/// TCP transport using the packet protocol
pub struct TcpTransport {
    stream: TcpStream,
}

impl TcpTransport {
    pub fn new(stream: TcpStream) -> Self {
        TcpTransport { stream }
    }

    pub fn connect(addr: impl std::net::ToSocketAddrs) -> std::io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        Ok(TcpTransport { stream })
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

    /// Send a packet
    fn send_packet(&mut self, data: &[u8]) -> Result<()> {
        if data.len() > MAX_PACKET_SIZE {
            return Err(SearpcError::TransportError(format!(
                "Packet too large: {} > {}",
                data.len(),
                MAX_PACKET_SIZE
            )));
        }

        // Write length (2 bytes, big-endian)
        let len = data.len() as u16;
        self.write_all(&len.to_be_bytes())?;

        // Write data
        self.write_all(data)?;

        Ok(())
    }

    /// Receive a packet
    fn recv_packet(&mut self) -> Result<Vec<u8>> {
        // Read length (2 bytes, big-endian)
        let mut len_buf = [0u8; 2];
        self.read_exact(&mut len_buf)?;
        let len = u16::from_be_bytes(len_buf) as usize;

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
}

impl Transport for TcpTransport {
    fn send(&mut self, request: &[u8]) -> Result<Vec<u8>> {
        self.send_packet(request)?;
        self.recv_packet()
    }
}

#[cfg(test)]
mod tests {
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
