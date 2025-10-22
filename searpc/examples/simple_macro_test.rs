///! Minimal test for the #[rpc] macro with prefix
use searpc::{rpc, Result, SearpcClient, TcpTransport};

// Using prefix - no need to repeat "my_service_" everywhere!
#[rpc(prefix = "my_service")]
trait SimpleRpc {
    fn test(&mut self, value: i32) -> Result<String>;
    // Automatically calls: my_service_test

    #[rpc(name = "custom_name")] // Can still override
    fn another_test(&mut self, s: &str) -> Result<i32>;
    // Calls: custom_name (not my_service_another_test)
}

fn main() {
    println!("Macro compilation test - if this compiles, the macro works!");

    // Just to use the trait
    let transport = TcpTransport::connect("127.0.0.1:12345").unwrap();
    let mut client = SearpcClient::new(transport);

    // This should compile even if it fails at runtime
    if let Ok(result) = client.test(42) {
        println!("Result: {}", result);
    }
}
