# seaf-cli - Seafile Command Line Client (Rust)

Rust implementation of Seafile command-line client, feature-complete with the Python version.

## Features

All commands from the original Python `seaf-cli` are implemented:

- **init** - Initialize configuration directory
- **start** - Start Seafile daemon
- **stop** - Stop Seafile daemon
- **list** - List local libraries (with JSON output support)
- **list-remote** - List remote libraries from server (with JSON output support)
- **status** - Show detailed syncing status with progress
- **download** - Download a library by ID
- **download-by-name** - Download a library by name
- **sync** - Synchronize existing folder with library
- **desync** - Desynchronize a library
- **create** - Create a new library
- **config** - Get/set configuration values

## Installation

```bash
cargo build --release
sudo cp target/release/seaf-cli /usr/local/bin/
```

## Usage Examples

### Initialize and Start

```bash
# Initialize configuration (first time only)
seaf-cli init -d /path/to/parent/dir

# Start Seafile daemon
seaf-cli start

# Check status
seaf-cli status
```

### List Libraries

```bash
# List local libraries
seaf-cli list

# List with JSON output
seaf-cli list --json

# List remote libraries
seaf-cli list-remote -s https://seafile.example.com -u user@example.com
```

### Download/Sync Libraries

```bash
# Download a library by ID
seaf-cli download \
  -l LIBRARY_ID \
  -s https://seafile.example.com \
  -u user@example.com \
  -d /path/to/download

# Download by name
seaf-cli download-by-name \
  -L "My Library" \
  -s https://seafile.example.com \
  -u user@example.com

# Sync existing folder
seaf-cli sync \
  -l LIBRARY_ID \
  -s https://seafile.example.com \
  -u user@example.com \
  -d /existing/folder
```

### Manage Libraries

```bash
# Create a new library
seaf-cli create \
  -n "New Library" \
  -t "Description" \
  -s https://seafile.example.com \
  -u user@example.com

# Desynchronize a library
seaf-cli desync -d /path/to/library

# Stop daemon
seaf-cli stop
```

### Configuration Management

```bash
# Get configuration value
seaf-cli config -k sync_interval

# Set configuration value
seaf-cli config -k sync_interval -v 300
```

## User Configuration File

You can create `~/.seafile.conf` to avoid typing server/username repeatedly:

```ini
[account]
server = https://seafile.example.com
user = user@example.com
token = YOUR_AUTH_TOKEN_HERE
```

Then you can simply run:

```bash
seaf-cli list-remote
seaf-cli download -l LIBRARY_ID
```

## Authentication

The client supports multiple authentication methods:

1. **Token** - Use `-T` flag or store in `~/.seafile.conf`
2. **Password** - Use `-p` flag or prompt interactively
3. **Two-factor authentication** - Use `-a` flag for OTP code

## Encrypted Libraries

For encrypted libraries, use the `-e` flag:

```bash
seaf-cli download -l LIBRARY_ID -e LIBRARY_PASSWORD ...
```

If not provided, you'll be prompted interactively.

## Advantages over Python Version

- **Performance**: Native binary, faster startup and execution
- **Memory**: Lower memory footprint
- **Dependencies**: Single static binary, no Python runtime needed
- **Type Safety**: Compile-time error checking
- **Zero unsafe code**: Memory safe by design

## Implementation Details

This Rust implementation uses:

- **searpc-rs**: Rust implementation of Searpc RPC protocol
- **reqwest**: HTTP client for Seafile API
- **clap**: Command-line argument parsing
- **serde**: JSON serialization/deserialization
- **tokio**: Async runtime (for future async operations)

## Compatibility

Fully compatible with Seafile server 7.0+ and the original C daemon (`seaf-daemon`).

## License

MIT
