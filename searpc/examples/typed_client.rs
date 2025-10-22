///! Type-safe RPC client example using the #[rpc] macro
///!
///! This example demonstrates the macro-based API with full type safety.
///!
///! First, run the C demo server from libsearpc/demo:
///! ```bash
///! cd libsearpc/demo
///! make
///! ./searpc-demo-server
///! ```
///!
///! Then run this client:
///! ```bash
///! cargo run --example typed_client
///! ```

use searpc::{rpc, Result, SearpcClient, TcpTransport};
use serde::Deserialize;

// Define the RPC trait with type-safe methods
// Using prefix="searpc" - method names automatically prefixed!
#[rpc(prefix = "searpc")]
trait DemoRpc {
    /// Call searpc_strlen to get string length
    fn strlen(&mut self, s: &str) -> Result<i32>;

    /// Call searpc_objlisttest to get list of objects
    fn objlisttest(&mut self, count: i32, len: i32, s: &str) -> Result<Vec<TestObject>>;
}

// Define the object type returned by objlist_test
#[derive(Debug, Deserialize)]
struct TestObject {
    count: i32,
    len: i32,
    str: String,
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== Type-Safe RPC Client Example ===\n");
    println!("Connecting to searpc demo server at 127.0.0.1:12345...");

    // Connect to the demo server
    let transport = TcpTransport::connect("127.0.0.1:12345")?;
    let mut client = SearpcClient::new(transport);

    // Test 1: Type-safe strlen call
    println!("\n=== Test 1: Type-safe strlen ===");
    let test_str = "hello searpc";

    match client.strlen(test_str) {
        Ok(len) => {
            println!("✓ Result: {}", len);
            println!("✓ Expected: {}", test_str.len());
            if len as usize == test_str.len() {
                println!("✓ Test PASSED!");
            } else {
                println!("✗ Test FAILED!");
            }
        }
        Err(e) => {
            println!("✗ Error: {}", e);
        }
    }

    // Test 2: Type-safe objlist call with deserialization
    println!("\n=== Test 2: Type-safe objlist ===");
    // Reconnect for second test
    let transport2 = TcpTransport::connect("127.0.0.1:12345")?;
    let mut client2 = SearpcClient::new(transport2);

    match client2.objlisttest(4, 11, "A rpc test.") {
        Ok(objects) => {
            println!("✓ Result: Got {} objects", objects.len());
            for (i, obj) in objects.iter().enumerate() {
                println!("  Object {}: count={}, len={}, str={}",
                    i, obj.count, obj.len, obj.str);
            }
            if objects.len() == 4 {
                println!("✓ Test PASSED!");
            } else {
                println!("✗ Test FAILED! Expected 4 objects, got {}", objects.len());
            }
        }
        Err(e) => {
            println!("✗ Error: {}", e);
        }
    }

    println!("\n=== Comparison: Manual vs Macro API ===\n");
    println!("Manual API (old way):");
    println!("  let len = client.call_int(\"searpc_strlen\", vec![Arg::string(s)])?;");
    println!("\nMacro API with prefix (new way):");
    println!("  #[rpc(prefix = \"searpc\")]");
    println!("  trait DemoRpc {{");
    println!("      fn strlen(&mut self, s: &str) -> Result<i32>;  // Calls: searpc_strlen");
    println!("  }}");
    println!("  let len = client.strlen(s)?;");
    println!("\n✅ Benefits:");
    println!("  • Type-safe: compiler catches errors at compile time");
    println!("  • Auto-deserialize: Vec<Value> → Vec<TestObject>");
    println!("  • DRY: prefix defined once, not repeated");
    println!("  • Clean: no manual Arg construction");

    Ok(())
}
