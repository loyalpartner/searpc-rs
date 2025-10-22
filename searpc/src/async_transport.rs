//! Async transport layer for Searpc RPC
//!
//! This module provides async versions of transports using tokio.

#[cfg(feature = "async")]
use crate::Result;

/// Async transport trait for sending/receiving RPC packets
///
/// Similar to the sync [`Transport`](crate::Transport) trait,
/// but all methods are async.
#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait AsyncTransport {
    /// Send a request and receive a response
    ///
    /// This is the main method for RPC communication.
    /// It sends the request bytes and returns the response bytes.
    async fn send(&mut self, request: &[u8]) -> Result<Vec<u8>>;
}
