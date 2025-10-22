///! Example client to connect to libsearpc demo server
///!
///! First, compile and run the C demo server from libsearpc/demo:
///! ```bash
///! cd libsearpc/demo
///! make
///! ./searpc-demo-server
///! ```
///!
///! Then run this client:
///! ```bash
///! cargo run --example demo_client
///! ```
use searpc::{Arg, SearpcClient, TcpTransport};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to searpc demo server at 127.0.0.1:12345...");

    // Connect to the demo server
    let transport = TcpTransport::connect("127.0.0.1:12345")?;
    let mut client = SearpcClient::new(transport);

    println!("\n=== Test 1: searpc_strlen ===");
    let test_str = "hello searpc";
    let args = vec![Arg::string(test_str)];

    match client.call_int("searpc_strlen", args) {
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

    println!("\n=== Test 2: searpc_objlisttest ===");
    // Need to reconnect for second test (demo server closes connection after each request)
    let transport2 = TcpTransport::connect("127.0.0.1:12345")?;
    let mut client2 = SearpcClient::new(transport2);

    let args2 = vec![
        Arg::int(4),                // count
        Arg::int(11),               // len
        Arg::string("A rpc test."), // str
    ];

    match client2.call_objlist("searpc_objlisttest", args2) {
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

    Ok(())
}
