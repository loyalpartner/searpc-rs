# searpc-rs

Rust implementation of the Searpc RPC protocol - compatible with [libsearpc](https://github.com/haiwen/libsearpc) and Seafile.

## Features

- ✅ **Type-safe RPC**: `#[rpc]` macro generates trait implementations
- ✅ **Protocol-compatible**: Works with existing Seafile daemons
- ✅ **Auto type conversion**: `bool`, `Vec<T>`, `Option<T>` handled automatically
- ✅ **Async support**: Full tokio integration (optional)
- ✅ **Zero unsafe code**: Memory-safe by design

## Quick Start

```toml
[dependencies]
searpc = { git = "https://github.com/loyalpartner/searpc-rs" }
searpc-macro = { git = "https://github.com/loyalpartner/searpc-rs" }
```

### Define RPC Interface

```rust
use searpc::Result;
use searpc_macro::rpc;
use serde::Deserialize;

#[derive(Deserialize)]
struct Repo {
    id: String,
    name: String,
}

#[rpc(prefix = "seafile")]
trait SeafileRpc {
    fn get_version(&mut self) -> Result<String>;
    fn get_repo_list(&mut self, start: i32, limit: i32) -> Result<Vec<Repo>>;
    fn is_auto_sync_enabled(&mut self) -> Result<bool>;  // i32 → bool auto-conversion
}
```

### Use the Client

```rust
use searpc::{SearpcClient, UnixSocketTransport};

let transport = UnixSocketTransport::connect(
    "/path/to/seafile.sock",
    "seafile-rpcserver"
)?;
let mut client = SearpcClient::new(transport);

// Type-safe RPC calls
let version: String = client.get_version()?;
let repos: Vec<Repo> = client.get_repo_list(-1, -1)?;
let enabled: bool = client.is_auto_sync_enabled()?;
```

## seaf-cli

Command-line client for Seafile:

```bash
cargo install --git https://github.com/loyalpartner/searpc-rs seaf-cli

# List repositories
seaf-cli list

# Show sync status
seaf-cli status

# Get/set config
seaf-cli config -k key
seaf-cli config -k key -v value
```

## Architecture

### Protocol

**Request:**
```json
["function_name", arg1, arg2, ...]
```

**Response:**
```json
{
  "ret": value,
  "err_code": 500,
  "err_msg": "error"
}
```

### Transport Protocols

libsearpc has **two different packet protocols**:

#### 1. TCP Transport (16-bit header)

```
┌─────────────┬──────────────────┐
│ Length(2B)  │  JSON Request    │
│ (uint16_t)  │  (variable)      │
│ big-endian  │                  │
└─────────────┴──────────────────┘
```

- **Header**: 16-bit big-endian length
- **Max packet**: 64KB
- **Usage**: libsearpc demo server, simple testing
- **Format**: Direct JSON request `["function_name", arg1, ...]`

#### 2. Unix Socket Transport (32-bit header + wrapper)

```
┌─────────────┬──────────────────────────────────┐
│ Length(4B)  │  Service-wrapped JSON Request    │
│ (uint32_t)  │  (variable)                      │
│ native-endian│                                 │
└─────────────┴──────────────────────────────────┘
```

- **Header**: 32-bit native-endian length
- **Max packet**: 4GB
- **Usage**: Seafile production, pysearpc
- **Format**: Wrapped with service identifier
  ```json
  {
    "service": "seafile-rpcserver",
    "request": "[\"function_name\", arg1, ...]"
  }
  ```

**Critical**: Do not mix these protocols! Seafile requires Unix Socket protocol.

### Type Conversion

| RPC Type | Rust Type |
|----------|-----------|
| `int` | `i32` |
| `int64` | `i64` |
| `string` | `String` |
| `object` | `T: Deserialize` |
| `objlist` | `Vec<T: Deserialize>` |

**Auto conversions:**
- `i32` → `bool` (0 = false, non-zero = true)
- `null` → `None` for `Option<T>`
- `null` → `[]` for `Vec<T>`

## Project Structure

```
searpc-rs/
├── searpc/           # Core RPC library
├── searpc-macro/     # #[rpc] procedural macro
└── seaf-cli/         # Seafile CLI tool
```

## Examples

See [`searpc/examples/`](searpc/examples/) for more examples.

## License

MIT
