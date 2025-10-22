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
Phase 2 (✅ **DONE**): Type-safe macro API for convenience
Phase 3 (✅ **DONE**): Async support with tokio

## Features

- ✅ **Protocol-compatible** with libsearpc C and Python implementations
- ✅ **Type-safe macro API**: `#[rpc]` trait-based client generation
- ✅ **Async support**: Full tokio integration (optional)
- ✅ **Two transport layers**:
  - `TcpTransport`: 16-bit length header (for demo compatibility)
  - `UnixSocketTransport`: 32-bit length header (for Seafile production)
- ✅ **All RPC types supported**: `int`, `int64`, `string`, `object`, `objlist`, `json`
- ✅ **Auto-deserialization**: JSON → custom types via serde
- ✅ **Zero unsafe code**
- ✅ **Comprehensive examples**

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

### Macro-based API (Recommended)

The `#[rpc]` macro provides complete type safety with automatic serialization/deserialization.

**DRY Principle**: Use `prefix` to avoid repeating function name prefixes!

```rust
use searpc::{rpc, Result, SearpcClient, TcpTransport};
use serde::Deserialize;

// Define your data types
#[derive(Deserialize)]
struct Repo {
    id: String,
    name: String,
    owner: String,
}

// Define the RPC interface with prefix
#[rpc(prefix = "seafile")]
trait SeafileRpc {
    fn get_version(&mut self) -> Result<String>;
    // Automatically calls: seafile_get_version

    fn list_repos(&mut self, offset: i32, limit: i32) -> Result<Vec<Repo>>;
    // Automatically calls: seafile_list_repos

    #[rpc(name = "get_commit")]  // Can still override
    fn get_specific_commit(&mut self, id: &str) -> Result<Commit>;
    // Calls: get_commit (not seafile_get_specific_commit)
}

// Use it with compile-time type checking!
let transport = TcpTransport::connect("127.0.0.1:12345")?;
let mut client = SearpcClient::new(transport);

let version: String = client.get_version()?;  // → seafile_get_version
let repos: Vec<Repo> = client.list_repos(0, 100)?;  // → seafile_list_repos

// Compiler ensures:
// - Correct parameter types (i32, &str, etc.)
// - Correct return types (String, Vec<Repo>)
// - Auto-deserialization from JSON
```

**Benefits**:
- ✅ **DRY**: Prefix defined once, not repeated for every method
- ✅ **Type safety**: Compiler catches parameter/return type errors
- ✅ **Auto-deserialization**: `Vec<Value>` → `Vec<Repo>` automatically
- ✅ **Clean API**: No manual `Arg` construction
- ✅ **IDE support**: Full autocomplete and documentation
- ✅ **Flexibility**: Can override individual method names with `#[rpc(name = "...")]`

### Manual API (Still Available)

For maximum control, use the manual API:

```rust
// int return type
let count: i32 = client.call_int("count_repos", vec![])?;

// string return type
let name: String = client.call_string("get_repo_name", vec![
    Arg::string("repo-id-here")
])?;

// object return type (returns JSON Value)
let repo = client.call_object("get_repo", vec![
    Arg::string("repo-id")
])?;

// objlist return type (returns Vec<Value>)
let repos: Vec<Value> = client.call_objlist("list_repos", vec![])?;
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
├── searpc/                       # Main crate (re-exports everything)
│   ├── src/
│   │   ├── protocol.rs          # JSON request/response serialization
│   │   ├── types.rs             # Arg enum + IntoArg trait
│   │   ├── client.rs            # SearpcClient with typed methods
│   │   ├── transport.rs         # Transport trait
│   │   ├── tcp_transport.rs     # 16-bit header transport
│   │   ├── unix_transport.rs    # 32-bit header transport (Unix only)
│   │   ├── async_client.rs      # Async RPC client (tokio)
│   │   ├── async_transport.rs   # Async transport trait
│   │   └── async_tcp_transport.rs
│   ├── examples/
│   │   ├── demo_client.rs       # Manual API example
│   │   ├── typed_client.rs      # Macro API example
│   │   └── async_demo_client.rs # Async API example
│   └── Cargo.toml
├── searpc-macro/                 # Procedural macros (#[rpc])
│   ├── src/lib.rs               # Macro implementation (~350 lines)
│   └── Cargo.toml
└── README.md
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

- [x] **Phase 1: Core Protocol** (DONE)
  - [x] JSON serialization/deserialization
  - [x] Client implementation with all RPC types
  - [x] TCP transport (16-bit header, demo compatible)
  - [x] Unix socket transport (32-bit header, Seafile production)
  - [x] Comprehensive error handling

- [x] **Phase 2: Type Safety & Ergonomics** (DONE)
  - [x] `#[rpc]` procedural macro for trait-based clients
  - [x] Auto-deserialization to custom types via serde
  - [x] `IntoArg` trait for parameter conversion
  - [x] Type-safe examples and documentation

- [x] **Phase 3: Async Support** (DONE)
  - [x] Async RPC client with tokio
  - [x] Async TCP transport
  - [x] Optional feature flags (`async`, `macro`)
  - [x] Async examples

- [ ] **Phase 4: Production Features** (Future)
  - [ ] Connection pooling
  - [ ] Timeout and retry mechanisms
  - [ ] Server implementation
  - [ ] Performance benchmarks
  - [ ] Comprehensive integration tests with Seafile

## License

MIT

## Credits

- Original libsearpc by [Haiwen](https://github.com/haiwen/libsearpc)
- Inspired by Linus Torvalds' philosophy of "good taste" in code

---

**"Talk is cheap. Show me the code."** - Linus Torvalds

We showed the code. It's simple, it works, and it has zero bullshit.
