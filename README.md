# Far Beyond Development Kit CLI (fbcli)

🚀 The official CLI tool for Far Beyond development - streamlining Horizon plugin development and repository management.

## Features

- **🔧 Horizon Plugin Development**: Create, build, and manage Horizon game server plugins
- **📦 Repository Management**: Easy cloning and management of Far-Beyond-Dev repositories
- **🎯 Cross-Platform**: Works on Windows, macOS, and Linux
- **⚡ Fast & Reliable**: Built with Rust for performance and safety

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Git](https://git-scm.com/) 
- [Cargo](https://doc.rust-lang.org/cargo/) (comes with Rust)

### Build from Source

```bash
git clone https://github.com/Far-Beyond-Dev/fbcli.git
cd fbcli
cargo build --release
```

The binary will be available at `target/release/fbcli` (or `fbcli.exe` on Windows).

## Usage

### Horizon Plugin Commands

#### Create a New Plugin

Create a new Horizon plugin from the official template:

```bash
fbcli horizon plugin new my_awesome_plugin
```

Options:
- `--path, -p <PATH>`: Specify target directory (defaults to current directory)

This command will:
1. Clone the `Horizon-Plugin-Sample` repository
2. Update the `Cargo.toml` with your plugin name
3. Generate a basic plugin template with the correct structure
4. Clean up unnecessary files

#### Build a Plugin

Build your plugin and optionally copy it to your Horizon server:

```bash
fbcli horizon plugin build
```

Options:
- `--horizon-path <PATH>`: Path to your Horizon server (defaults to `../Horizon`)
- `--no-copy`: Skip copying the built plugin to Horizon plugins directory

This command will:
1. Build your plugin in release mode
2. Locate the compiled library (`.dll`, `.so`, or `.dylib`)
3. Copy it to `<horizon-path>/plugins/` directory

### Repository Management Commands

#### List Repositories

List all repositories in the Far-Beyond-Dev organization:

```bash
fbcli repo list
```

Options:
- `--public-only`: Show only public repositories

#### Clone a Repository

Clone a repository from the Far-Beyond-Dev organization:

```bash
fbcli repo clone Horizon-Server
```

Options:
- `--path, -p <PATH>`: Target directory (defaults to repository name)
- `--ssh`: Use SSH instead of HTTPS for cloning

#### Update Repositories

Update all Far-Beyond repositories in the current directory:

```bash
fbcli repo update
```

Options:
- `--dry-run`: Show what would be updated without making changes

#### Check Repository Status

Check the status of all Far-Beyond repositories in the current directory:

```bash
fbcli repo status
```

Shows:
- Current branch
- Working directory status (clean/dirty)
- Commits ahead/behind remote

## Plugin Development Workflow

Here's a typical workflow for developing a Horizon plugin:

### 1. Create a New Plugin

```bash
fbcli horizon plugin new my_game_mode
cd my_game_mode
```

### 2. Implement Your Plugin

Edit `src/lib.rs` to implement your plugin logic:

```rust
use async_trait::async_trait;
use horizon_event_system::{
    create_simple_plugin, register_handlers, EventSystem, LogLevel,
    PluginError, ServerContext, SimplePlugin,
};
use std::sync::Arc;
use tracing::info;

pub struct MyGameModePlugin {
    name: String,
}

impl MyGameModePlugin {
    pub fn new() -> Self {
        Self {
            name: "my_game_mode".to_string(),
        }
    }
}

#[async_trait]
impl SimplePlugin for MyGameModePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    async fn register_handlers(&mut self, events: Arc<EventSystem>) -> Result<(), PluginError> {
        register_handlers!(events; core {
            "player_connected" => |event: serde_json::Value| {
                info!("Player joined the game: {:?}", event);
                Ok(())
            }
        })?;
        Ok(())
    }

    // ... implement other required methods
}

create_simple_plugin!(MyGameModePlugin);
```

### 3. Build and Test

```bash
fbcli horizon plugin build
```

Your plugin will be compiled and copied to your Horizon server's plugins directory.

## Project Structure

```
my_plugin/
├── Cargo.toml          # Package configuration
└── src/
    └── lib.rs          # Plugin implementation
```

The generated `Cargo.toml` will look like:

```toml
[package]
name = "plugin_my_plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
horizon_event_system = "0.1"  # Event system from crates.io
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"
tracing = { version = "0.1", features = ["log"] }
```

## Configuration

### Default Paths

- **Horizon Server**: `../Horizon` (relative to plugin directory)
- **Plugins Directory**: `<horizon-path>/plugins/`

### Environment Variables

- `FBCLI_HORIZON_PATH`: Override default Horizon server path
- `FBCLI_GITHUB_TOKEN`: GitHub personal access token for private repositories

## Troubleshooting

### Common Issues

**Git not found**
```
Error: Git is not installed or not available in PATH
```
Solution: Install Git and ensure it's in your system PATH.

**Cargo not found**
```
Error: Cargo is not installed or not available in PATH
```
Solution: Install Rust which includes Cargo.

**Plugin build fails**
```
Error: Cargo build failed
```
Solution: Check your plugin code for compilation errors. Ensure all dependencies are correctly specified in `Cargo.toml`.

**Cannot copy to Horizon plugins directory**
```
Error: Failed to copy plugin to <path>
```
Solution: Ensure the Horizon path exists and you have write permissions. Use `--horizon-path` to specify the correct path.

### Debug Mode

Run any command with `RUST_LOG=debug` for verbose output:

```bash
RUST_LOG=debug fbcli horizon plugin build
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.