# searpc-rs

**Rust implementation of the Searpc RPC framework** - Compatible with [libsearpc](https://github.com/haiwen/libsearpc) and Seafile.

## What is this?

This is a **clean, zero-dependency** Rust client for the Searpc RPC protocol used by Seafile.

### Why rewrite in Rust?

**Linus Torvalds would approve:**

- ✅ **Good Taste**: Simple data structures (`["fname", args...]`), no complex type hierarchies
- ✅ **Zero Magic**: No code generation required for basic usage
- ✅ **Type Safety**: Rust's type system replaces C's `strcmp(type, "int")` nonsense
- ✅ **Memory Safety**: No manual `g_free()` or `g_object_unref()`
- ✅ **Practical**: Solves real problem - communicate with Seafile from Rust

### Design Philosophy

```
Make it work → Make it right → Make it fast
```

Phase 1 (✅ **DONE**): Core protocol that works with existing C/Python servers
Phase 2 (Future): High-level macro API for convenience
Phase 3 (Future): Async support with tokio

## Features

- ✅ **Protocol-compatible** with libsearpc C and Python implementations
- ✅ **Two transport layers**:
  - `TcpTransport`: 16-bit length header (for demo compatibility)
  - `UnixSocketTransport`: 32-bit length header (for Seafile production)
- ✅ **All RPC types supported**: `int`, `int64`, `string`, `object`, `objlist`, `json`
- ✅ **Zero unsafe code**
- ✅ **Comprehensive tests**

## Quick Start

### Add to your `Cargo.toml`

```toml
[dependencies]
searpc-core = { path = "path/to/searpc-rs/searpc-core" }
```

### Basic Usage

```rust
use searpc_core::{SearpcClient, UnixSocketTransport, Arg};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Seafile via Unix socket
    let transport = UnixSocketTransport::connect(
        "/path/to/seafile.sock",
        "seafile-rpcserver"  // service name
    )?;

    let mut client = SearpcClient::new(transport);

    // Call RPC function
    let version = client.call_string("get_version", vec![])?;
    println!("Seafile version: {}", version);

    // Call with arguments
    let repos = client.call_objlist("get_repo_list", vec![
        Arg::int(0),    // offset
        Arg::int(100),  // limit
    ])?;

    println!("Found {} repositories", repos.len());
    Ok(())
}
```

### Type-safe Calls

```rust
// int return type
let count: i32 = client.call_int("count_repos", vec![])?;

// string return type
let name: String = client.call_string("get_repo_name", vec![
    Arg::string("repo-id-here")
])?;

// object return type
let repo = client.call_object("get_repo", vec![
    Arg::string("repo-id")
])?;

// objlist return type
let repos: Vec<serde_json::Value> = client.call_objlist("list_repos", vec![])?;
```

## Architecture

### Protocol Format

**Request:**
```json
["function_name", arg1, arg2, ...]
```

**Response:**
```json
{
  "ret": return_value,
  "err_code": error_code,    // only if error
  "err_msg": "error message" // only if error
}
```

### Transport Protocols

#### 1. TCP Transport (Demo-compatible)

```
┌─────────────┬──────────────────┐
│ Length(2B)  │  JSON Data       │
│ (uint16_t)  │  (variable)      │
└─────────────┴──────────────────┘
```

- 16-bit big-endian length header
- Max packet size: 64KB
- Use for testing with libsearpc demos

#### 2. Unix Socket Transport (Seafile production)

```
┌─────────────┬──────────────────┐
│ Length(4B)  │  Wrapped JSON    │
│ (uint32)    │  (variable)      │
└─────────────┴──────────────────┘
```

- 32-bit native-endian length header
- Request wrapped with service name:
  ```json
  {
    "service": "service-name",
    "request": ["function_name", arg1, ...]
  }
  ```
- Max packet size: 4GB
- **This is what Seafile actually uses**

## Project Structure

```
searpc-rs/
├── searpc-core/          # Core protocol implementation
│   ├── src/
│   │   ├── protocol.rs   # JSON request/response serialization
│   │   ├── types.rs      # Arg enum (replaces C's string types)
│   │   ├── client.rs     # SearpcClient with typed methods
│   │   ├── transport.rs  # Transport trait
│   │   ├── tcp_transport.rs    # 16-bit header transport
│   │   └── unix_transport.rs   # 32-bit header transport
│   └── Cargo.toml
├── searpc-macro/         # Future: proc macros for convenience
├── searpc/               # Future: high-level API
└── examples/
    └── demo_client.rs    # Example connecting to libsearpc demo server
```

## Compatibility

### Tested with

- ✅ libsearpc C demo server
- ⏳ Seafile (in progress)

### Type Mapping

| C/Python Type | Rust Type                  |
|---------------|----------------------------|
| `int`         | `i32`                      |
| `int64`       | `i64`                      |
| `string`      | `String`                   |
| `object`      | `serde_json::Value`        |
| `objlist`     | `Vec<serde_json::Value>`   |
| `json`        | `serde_json::Value`        |

## Development

### Run Tests

```bash
cargo test --package searpc-core
```

### Test with libsearpc demo server

1. Build C demo server:
```bash
cd libsearpc
./autogen.sh
./configure
make
cd demo
./searpc-demo-server
```

2. Run Rust client (in another terminal):
```bash
cargo run --example demo_client
```

## Roadmap

- [x] Phase 1: Core protocol (MVP)
  - [x] JSON serialization/deserialization
  - [x] Client implementation
  - [x] All RPC types
  - [x] TCP transport (demo)
  - [x] Unix socket transport (production)

- [ ] Phase 2: Ergonomics
  - [ ] Procedural macros for function definitions
  - [ ] Better error types
  - [ ] Connection pooling

- [ ] Phase 3: Production Features
  - [ ] Async support (tokio)
  - [ ] Timeouts and retries
  - [ ] Server implementation
  - [ ] Performance benchmarks

## License

MIT

## Credits

- Original libsearpc by [Haiwen](https://github.com/haiwen/libsearpc)
- Inspired by Linus Torvalds' philosophy of "good taste" in code

---

**"Talk is cheap. Show me the code."** - Linus Torvalds

We showed the code. It's simple, it works, and it has zero bullshit.
