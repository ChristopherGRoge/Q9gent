# Development Guide

## Prerequisites

1. **Rust** (1.70+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Claude CLI** - Required for running the server
   - Install from: https://www.anthropic.com/claude-cli
   - Ensure it's in your PATH or note the installation path

## Quick Start

### 1. Clone and Build
```bash
git clone https://github.com/yourusername/Q9gent.git
cd Q9gent
cargo build --release
```

### 2. Run the Server
```bash
# Default configuration (127.0.0.1:8080)
cargo run

# With custom configuration
cargo run -- --host 0.0.0.0 --port 3000 --claude-path /usr/local/bin/claude

# With debug logging
RUST_LOG=q9gent=debug cargo run
```

### 3. Test the API
```bash
# Health check
curl http://localhost:8080/health

# Spawn an agent
curl -N -X POST http://localhost:8080/spawn \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "test",
    "prompt": "Say hello",
    "tools_allowed": [],
    "create_session": false
  }'
```

## Project Structure

```
Q9gent/
├── src/
│   ├── main.rs           # Entry point, CLI argument parsing
│   ├── api.rs            # HTTP server and endpoint handlers
│   ├── agent.rs          # Claude process spawning and management
│   ├── session.rs        # Session metadata persistence
│   ├── error.rs          # Error types and handling
│   └── config.rs         # Server configuration
├── .github/
│   └── workflows/
│       └── build.yml     # CI/CD for cross-platform builds
├── Cargo.toml            # Dependencies and project metadata
├── README.md             # User documentation
├── EXAMPLES.md           # API usage examples
└── DEVELOPMENT.md        # This file
```

## Development Workflow

### Running Tests
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_build_command
```

### Linting and Formatting
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run clippy (linter)
cargo clippy

# Clippy with all warnings as errors
cargo clippy -- -D warnings
```

### Build Variants
```bash
# Debug build (faster compile, slower runtime)
cargo build

# Release build (optimized)
cargo build --release

# Check without building
cargo check
```

## Adding New Features

### 1. Add a New Endpoint

**Step 1:** Define request/response types in `src/api.rs`
```rust
#[derive(Debug, Deserialize)]
struct MyRequest {
    field: String,
}

#[derive(Debug, Serialize)]
struct MyResponse {
    result: String,
}
```

**Step 2:** Implement the handler
```rust
async fn my_endpoint(
    State(state): State<AppState>,
    Json(payload): Json<MyRequest>,
) -> AppResult<Json<MyResponse>> {
    // Implementation
    Ok(Json(MyResponse {
        result: "success".to_string(),
    }))
}
```

**Step 3:** Add route in `app()` function
```rust
fn app(state: AppState) -> Router {
    Router::new()
        // ... existing routes
        .route("/my-endpoint", post(my_endpoint))
        .with_state(state)
}
```

### 2. Add Session Metadata Fields

Edit `src/session.rs`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: String,
    pub agent_type: String,
    pub created_at: u64,
    pub last_used: u64,
    // Add new fields here
    pub my_field: String,
}
```

### 3. Add New CLI Options

Edit `src/main.rs`:
```rust
#[derive(Parser, Debug)]
struct Args {
    // ... existing args
    
    #[arg(long, default_value = "default_value")]
    my_option: String,
}
```

Then update `ServerConfig` in `src/config.rs` to include the new option.

## Testing with Mock Claude CLI

For development without the real Claude CLI:

**Create a mock script** (`mock-claude.sh`):
```bash
#!/bin/bash
echo '{"event":"text","text":"Mock response"}'
echo '{"event":"completed"}'
```

**Make it executable:**
```bash
chmod +x mock-claude.sh
```

**Run server with mock:**
```bash
cargo run -- --claude-path ./mock-claude.sh
```

## Debugging

### Enable Debug Logging
```bash
RUST_LOG=q9gent=trace,axum=debug,tower_http=debug cargo run
```

### Attach Debugger (VS Code)

Create `.vscode/launch.json`:
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Q9gent",
      "cargo": {
        "args": ["build", "--bin=q9gent", "--package=q9gent"]
      },
      "args": ["--host", "127.0.0.1", "--port", "8080"],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

### Print Debugging
```rust
use tracing::{debug, info, warn, error};

debug!("Variable value: {:?}", var);
info!("Processing request");
warn!("Unexpected condition");
error!("Fatal error: {}", err);
```

## Performance Profiling

### Using cargo-flamegraph
```bash
# Install
cargo install flamegraph

# Generate flame graph
cargo flamegraph --root

# Open flamegraph.svg in browser
```

### Using perf (Linux)
```bash
# Build with symbols
cargo build --release

# Record performance data
perf record -g target/release/q9gent

# Generate report
perf report
```

## Cross-Platform Testing

### Test on Windows (via Docker)
```bash
docker run --rm -v $(pwd):/app -w /app rust:latest \
  cargo build --target x86_64-pc-windows-gnu
```

### Test on Linux (via Docker)
```bash
docker run --rm -v $(pwd):/app -w /app rust:latest \
  cargo build --target x86_64-unknown-linux-musl
```

## Common Issues

### Issue: "claude not found"
**Solution:** Install Claude CLI or use `--claude-path` flag

### Issue: "Address already in use"
**Solution:** Change port with `--port` or kill existing process
```bash
lsof -ti:8080 | xargs kill -9
```

### Issue: Session directory permission denied
**Solution:** Ensure directory is writable
```bash
mkdir -p ./sessions
chmod 755 ./sessions
```

## Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Update version in `README.md` if needed
- [ ] Run full test suite: `cargo test`
- [ ] Run clippy: `cargo clippy -- -D warnings`
- [ ] Check formatting: `cargo fmt -- --check`
- [ ] Build release binary: `cargo build --release`
- [ ] Test release binary manually
- [ ] Update CHANGELOG (if exists)
- [ ] Create git tag: `git tag -a v0.1.0 -m "Release v0.1.0"`
- [ ] Push tag: `git push origin v0.1.0`
- [ ] GitHub Actions will build and create release

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Format code: `cargo fmt`
6. Run clippy: `cargo clippy`
7. Commit: `git commit -am 'Add my feature'`
8. Push: `git push origin feature/my-feature`
9. Create Pull Request

## Useful Commands

```bash
# Watch for changes and rebuild
cargo watch -x run

# Check dependencies for updates
cargo outdated

# Update dependencies
cargo update

# Generate documentation
cargo doc --open

# Show dependency tree
cargo tree

# Benchmark (if benchmarks exist)
cargo bench

# Clean build artifacts
cargo clean
```

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [Serde Guide](https://serde.rs/)
- [Tracing Guide](https://docs.rs/tracing/latest/tracing/)
