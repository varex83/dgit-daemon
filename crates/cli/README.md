# dgit CLI

A command-line interface for interacting with the decentralized git daemon.

## Installation

From the workspace root:

```bash
cargo install --path crates/cli
```

## Usage

### Global Options

- `--daemon-url <URL>`: Override the daemon URL (default: http://localhost:3000)
- `-v, --verbose`: Increase verbosity (can be used multiple times)

### Commands

#### Daemon Management

Start the daemon:

```bash
dgit daemon [--port <PORT>]
```

Check daemon health:

```bash
dgit health
```

#### Account Management

Add a new account:

```bash
# Interactive mode
dgit account add

# With arguments
dgit account add --name alice --private-key <PK> --address <ETH_ADDRESS>
```

List all accounts:

```bash
dgit account list
```

Switch active account:

```bash
# Interactive selection
dgit account switch

# Direct switch
dgit account switch alice
```

Show current active account:

```bash
dgit account current
```

Remove an account:

```bash
dgit account remove alice
```

#### Repository Management

Create a new repository:

```bash
dgit repo create my-repo
```

##### Role Management

Grant pusher role:

```bash
# To active account
dgit repo role grant-pusher --repo my-repo

# To specific address
dgit repo role grant-pusher --repo my-repo --address 0x123...
```

Revoke pusher role:

```bash
dgit repo role revoke-pusher --repo my-repo [--address 0x123...]
```

Grant admin role:

```bash
dgit repo role grant-admin --repo my-repo [--address 0x123...]
```

Revoke admin role:

```bash
dgit repo role revoke-admin --repo my-repo [--address 0x123...]
```

Check roles:

```bash
# Check pusher role
dgit repo role check-pusher --repo my-repo [--address 0x123...]

# Check admin role
dgit repo role check-admin --repo my-repo [--address 0x123...]
```

## Configuration

The CLI stores configuration in `~/.config/dgit/config.toml`. This includes:

- Account information (names, addresses, encrypted private keys)
- Active account selection

## Examples

### Complete workflow

1. Start the daemon:
   ```bash
   dgit daemon
   ```

2. Add an account:
   ```bash
   dgit account add --name alice --address 0x123...
   ```

3. Create a repository:
   ```bash
   dgit repo create my-project
   ```

4. Grant yourself pusher rights:
   ```bash
   dgit repo role grant-pusher --repo my-project
   ```

5. Clone and work with the repository using git:
   ```bash
   git clone http://localhost:3000/my-project
   ```

## Environment Variables

- `DGIT_DAEMON_URL`: Default daemon URL (overrides the default http://localhost:3000)
- Standard Rust/Cargo environment variables for logging (e.g., `RUST_LOG`)

## Troubleshooting

### Daemon not responding

Check if the daemon is running:

```bash
dgit health
```

### Permission denied errors

Ensure you have the appropriate role:

```bash
dgit repo role check-pusher --repo <repo-name>
dgit repo role check-admin --repo <repo-name>
``` 