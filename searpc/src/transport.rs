use crate::error::Result;

/// Transport callback trait
///
/// This abstracts the network layer, just like the C version's TransportCB.
/// Users provide their own implementation (TCP, Unix socket, Named pipe, etc.)
///
/// Good taste: simple function signature, no complex state machine
pub trait Transport {
    /// Send request bytes and receive response bytes
    ///
    /// # Arguments
    /// * `request` - The JSON request as bytes
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - Response bytes
    /// * `Err(SearpcError)` - Transport error
    fn send(&mut self, request: &[u8]) -> Result<Vec<u8>>;
}

/// Function-based transport (for simple callbacks)
impl<F> Transport for F
where
    F: FnMut(&[u8]) -> Result<Vec<u8>>,
{
    fn send(&mut self, request: &[u8]) -> Result<Vec<u8>> {
        self(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_transport() {
        let mut transport = |req: &[u8]| -> Result<Vec<u8>> {
            // Echo transport
            Ok(req.to_vec())
        };

        let result = transport.send(b"test").unwrap();
        assert_eq!(result, b"test");
    }
}
