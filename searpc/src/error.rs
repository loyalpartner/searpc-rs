use thiserror::Error;

pub type Result<T> = std::result::Result<T, SearpcError>;

/// Transport error code (matches C TRANSPORT_ERROR_CODE)
pub const TRANSPORT_ERROR_CODE: i32 = 500;

/// Transport error message (matches C TRANSPORT_ERROR)
pub const TRANSPORT_ERROR_MSG: &str = "Transport Error";

#[derive(Debug, Error)]
pub enum SearpcError {
    /// RPC function returned an error
    #[error("RPC error {code}: {message}")]
    RpcError { code: i32, message: String },

    /// Transport layer error (network, timeout, etc.)
    /// Matches C's TRANSPORT_ERROR (code 500)
    #[error("Transport error: {0}")]
    TransportError(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Invalid response format
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    /// Type conversion error
    #[error("Type error: {0}")]
    TypeError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Environment variable error
    #[error("Environment variable error: {0}")]
    EnvVarError(#[from] std::env::VarError),
}
