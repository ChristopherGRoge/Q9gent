# Q9gent

A lightweight Rust CLI Assistant Server that spawns short-lived Claude Code Headless processes on demand with precise CLI flags and streams results back to the caller.

## Features

- 🚀 **Stateless by Default** - Each request spawns an ephemeral Claude process
- 🔒 **Least-Privilege Tool Fences** - Strict control over allowed tools per agent
- 📡 **Server-Sent Events** - Real-time JSONL streaming via SSE
- 💾 **Optional Session Resumption** - Minimal metadata persistence for multi-turn conversations
- 🎯 **Zero Hidden Orchestration** - The server makes no agentic decisions
- 🔧 **Clean Process Supervision** - Built on Tokio for robust async process management
- 🌐 **Simple HTTP API** - Easy integration with any client

## Architecture

The server is a thin wrapper around Claude CLI that:
1. Receives requests with `{agent_type, prompt, flags, tools_allowed, system_append, optional_resume_id}`
2. Spawns `claude -p --output-format stream-json [--allowedTools ...] [--append-system-prompt ...] [--resume <session-id>]`
3. Streams JSONL stdout to the client via Server-Sent Events
4. Exits the Claude process when the turn completes
5. Optionally persists minimal session metadata for resumption

## Installation

### From Source

```bash
git clone https://github.com/ChristopherGRoge/Q9gent.git
cd Q9gent
cargo build --release
```

### From Releases

Download pre-built binaries from the [Releases](https://github.com/ChristopherGRoge/Q9gent/releases) page:

- **Windows**: `q9gent-windows-x86_64.exe`, `q9gent-windows-i686.exe`, `q9gent-windows-aarch64.exe`
- **Linux**: `q9gent-linux-x86_64`, `q9gent-linux-aarch64`, `q9gent-linux-x86_64-musl`
- **macOS**: `q9gent-macos-x86_64`, `q9gent-macos-aarch64`

## Usage

### Starting the Server

```bash
# Basic usage (defaults: 127.0.0.1:8080, ./sessions)
./q9gent

# Custom configuration
./q9gent --host 0.0.0.0 --port 3000 --session-dir /var/lib/q9gent/sessions --claude-path /usr/local/bin/claude

# View all options
./q9gent --help
```

**Command-line Options:**
- `-h, --host <HOST>` - Server bind address (default: `127.0.0.1`)
- `-p, --port <PORT>` - Server port (default: `8080`)
- `-s, --session-dir <SESSION_DIR>` - Session storage directory (default: `./sessions`)
- `-c, --claude-path <CLAUDE_PATH>` - Path to claude CLI executable (default: `claude`)

### API Endpoints

#### Health Check
```bash
GET /health
```
**Response:**
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

#### Spawn Agent
```bash
POST /spawn
```
**Request Body:**
```json
{
  "agent_type": "code_helper",
  "prompt": "Write a hello world program in Rust",
  "flags": [],
  "tools_allowed": ["read_file", "write_file"],
  "system_append": "You are a helpful coding assistant",
  "resume_id": null,
  "create_session": true
}
```

**Response:** Server-Sent Events stream with JSONL output from Claude

**Event Types:**
- `session_created` - Emitted when a new session is created
  ```json
  {"type": "session_created", "session_id": "550e8400-e29b-41d4-a716-446655440000"}
  ```
- `output` - Claude's JSONL output
  ```json
  {"type": "output", "data": "{\"event\":\"text\",\"text\":\"Here's a simple...\"}"}
  ```
- `error` - Error occurred during execution
  ```json
  {"type": "error", "error": "Process execution failed"}
  ```
- `completed` - Agent turn completed
  ```json
  {"type": "completed"}
  ```

#### Send Message to Session
```bash
POST /message/{session_id}
```
**Request Body:**
```json
{
  "prompt": "Now make it print the current time too",
  "flags": [],
  "tools_allowed": ["read_file", "write_file"],
  "system_append": null
}
```

**Response:** Server-Sent Events stream (same format as `/spawn`)

#### Terminate Agent
```bash
POST /terminate/{session_id}
```
**Response:**
```json
{
  "message": "Process terminated successfully"
}
```

#### List Sessions
```bash
GET /sessions
```
**Response:**
```json
{
  "sessions": [
    {
      "session_id": "550e8400-e29b-41d4-a716-446655440000",
      "agent_type": "code_helper",
      "created_at": 1698624000,
      "last_used": 1698624120
    }
  ]
}
```

## Example Client (JavaScript)

```javascript
const eventSource = new EventSource('http://localhost:8080/spawn', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    agent_type: 'assistant',
    prompt: 'Hello, Claude!',
    create_session: true,
    tools_allowed: ['read_file', 'write_file']
  })
});

eventSource.addEventListener('message', (event) => {
  const data = JSON.parse(event.data);
  
  switch (data.type) {
    case 'session_created':
      console.log('Session ID:', data.session_id);
      break;
    case 'output':
      console.log('Claude output:', data.data);
      break;
    case 'error':
      console.error('Error:', data.error);
      break;
    case 'completed':
      console.log('Agent completed');
      eventSource.close();
      break;
  }
});
```

## Example Client (curl)

```bash
# Spawn an agent
curl -N -X POST http://localhost:8080/spawn \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "assistant",
    "prompt": "Write a simple calculator in Python",
    "tools_allowed": ["write_file"],
    "create_session": true
  }'

# Send a follow-up message (use session_id from previous response)
curl -N -X POST http://localhost:8080/message/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Add support for square root"
  }'

# List sessions
curl http://localhost:8080/sessions

# Terminate a running agent
curl -X POST http://localhost:8080/terminate/550e8400-e29b-41d4-a716-446655440000
```

## Example Client (Python)

```python
import requests
import json

def spawn_agent(prompt, tools_allowed=None, create_session=True):
    url = "http://localhost:8080/spawn"
    payload = {
        "agent_type": "assistant",
        "prompt": prompt,
        "tools_allowed": tools_allowed or [],
        "create_session": create_session
    }
    
    response = requests.post(url, json=payload, stream=True)
    session_id = None
    
    for line in response.iter_lines():
        if line:
            # Parse SSE format: "data: {...}"
            if line.startswith(b'data: '):
                data = json.loads(line[6:])
                
                if data['type'] == 'session_created':
                    session_id = data['session_id']
                    print(f"Session created: {session_id}")
                elif data['type'] == 'output':
                    print(f"Output: {data['data']}")
                elif data['type'] == 'error':
                    print(f"Error: {data['error']}")
                elif data['type'] == 'completed':
                    print("Completed")
                    break
    
    return session_id

# Example usage
session_id = spawn_agent(
    "Create a hello world program in Rust",
    tools_allowed=["write_file"]
)
```

## Configuration

### Environment Variables

```bash
# Set log level
export RUST_LOG=q9gent=debug,tower_http=debug

# Start server
./q9gent
```

### Session Persistence

Sessions are stored as JSON files in the session directory:
```
./sessions/
  ├── 550e8400-e29b-41d4-a716-446655440000.json
  └── 7c9e6679-7425-40de-944b-e07fc1f90ae7.json
```

Each session file contains:
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "agent_type": "code_helper",
  "created_at": 1698624000,
  "last_used": 1698624120
}
```

## Building from Source

### Prerequisites
- Rust 1.70 or later
- Claude CLI installed and available in PATH (or specify with `--claude-path`)

### Build
```bash
cargo build --release
```

### Run Tests
```bash
cargo test
```

### Run with Logging
```bash
RUST_LOG=q9gent=debug cargo run
```

## Cross-Platform Compilation

The project includes GitHub Actions workflows for automatic cross-platform builds.

### Manual Cross-Compilation

**For Windows (from Linux/macOS):**
```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

**For Linux (from macOS/Windows):**
```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

**For macOS (from Linux - requires osxcross):**
```bash
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

## Design Philosophy

Q9gent adheres to these principles:

1. **Composability** - The server is a reusable building block, not a monolith
2. **Transparency** - No hidden orchestration or agentic decision-making
3. **Ephemerality** - Processes are short-lived by default
4. **Least Privilege** - Strict tool access control per request
5. **Simplicity** - Minimal API surface, clear semantics

## Contributing

Contributions are welcome! Please ensure:
- Code passes `cargo test`
- Code passes `cargo clippy`
- Code is formatted with `cargo fmt`

## License

MIT License - see LICENSE file for details

## Security Considerations

- **Tool Access Control**: Always specify `tools_allowed` to limit agent capabilities
- **Process Isolation**: Each agent runs in a separate process with no shared state
- **Session Storage**: Session metadata is stored unencrypted; secure the session directory
- **Network Binding**: Default binding is `127.0.0.1` (localhost only); be cautious when binding to `0.0.0.0`

## Troubleshooting

### Claude CLI Not Found
```
Error: Process spawn failed: No such file or directory
```
**Solution:** Ensure Claude CLI is installed and in PATH, or specify the path:
```bash
./q9gent --claude-path /path/to/claude
```

### Permission Denied (Session Directory)
```
Error: IO error: Permission denied
```
**Solution:** Ensure the session directory is writable:
```bash
mkdir -p ./sessions
chmod 755 ./sessions
```

### Port Already in Use
```
Error: Address already in use
```
**Solution:** Use a different port:
```bash
./q9gent --port 8081
```

## Roadmap

- [ ] gRPC endpoint support
- [ ] WebSocket streaming alternative to SSE
- [ ] Metrics and observability
- [ ] Rate limiting
- [ ] Authentication/authorization
- [ ] Docker image
- [ ] Kubernetes manifests

## Acknowledgments

Built with:
- [Tokio](https://tokio.rs/) - Async runtime
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Serde](https://serde.rs/) - Serialization
- Claude CLI - AI assistant by Anthropic
