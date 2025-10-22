///! Async example client to connect to libsearpc demo server
///!
///! First, compile and run the C demo server from libsearpc/demo:
///! ```bash
///! cd libsearpc/demo
///! make
///! ./searpc-demo-server
///! ```
///!
///! Then run this async client:
///! ```bash
///! cargo run --example async_demo_client
///! ```
use searpc::{Arg, AsyncSearpcClient, AsyncTcpTransport};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Async client connecting to searpc demo server at 127.0.0.1:12345...\n");

    // Test 1: searpc_strlen
    println!("=== Test 1: searpc_strlen (async) ===");
    let transport = AsyncTcpTransport::connect("127.0.0.1:12345").await?;
    let mut client = AsyncSearpcClient::new(transport);

    let test_str = "hello searpc";
    let args = vec![Arg::string(test_str)];

    match client.call_int("searpc_strlen", args).await {
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

    // Test 2: searpc_objlisttest
    println!("\n=== Test 2: searpc_objlisttest (async) ===");
    // Need to reconnect for second test (demo server closes connection after each request)
    let transport2 = AsyncTcpTransport::connect("127.0.0.1:12345").await?;
    let mut client2 = AsyncSearpcClient::new(transport2);

    let args2 = vec![
        Arg::int(4),                // count
        Arg::int(11),               // len
        Arg::string("A rpc test."), // str
    ];

    match client2.call_objlist("searpc_objlisttest", args2).await {
        Ok(objlist) => {
            println!("✓ Result: Got {} objects", objlist.len());
            for (i, obj) in objlist.iter().enumerate() {
                println!("  Object {}: {}", i, obj);
            }
            if objlist.len() == 4 {
                println!("✓ Test PASSED!");
            } else {
                println!("✗ Test FAILED!");
            }
        }
        Err(e) => {
            println!("✗ Error: {}", e);
        }
    }

    println!("\n✨ Async tests completed!");
    Ok(())
}
