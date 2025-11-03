# Q9gent

**Bring First-Class Subagents to Your Application**

Q9gent is a production-ready HTTP server that brings Claude Code's advanced "thinking" capabilities—including autonomous tool use, multi-turn conversations, and streaming responses—to any application via a simple REST API.

Spawn Claude CLI processes on demand, stream real-time JSONL output via SSE, and let your application leverage Claude's full power without managing the complexity of process supervision, session state, or tool execution.

**Latest Version:** v0.1.2 | **Status:** ✅ Production Ready (Windows, macOS, Linux tested)

---

## Why Q9gent?

**Add Advanced AI Capabilities to Any Application:**
- 🧠 **True "Thinking" AI** - Leverage Claude Code's autonomous reasoning and tool use
- 🔌 **Universal Integration** - Simple HTTP API works with any language or framework
- 🚀 **Production Ready** - Comprehensive Windows/macOS/Linux testing, battle-tested process management
- 🎯 **Zero Complexity** - No agent frameworks, no hidden orchestration, just clean process supervision
- 🔒 **Security First** - Strict per-request tool access control, process isolation, least-privilege design

**Perfect For:**
- Adding AI coding assistance to IDEs, editors, and dev tools
- Building autonomous code review, testing, and documentation systems
- Creating interactive AI assistants with file access and code execution
- Prototyping complex multi-agent systems with isolated subagents
- Any application that needs Claude Code's capabilities via API

> 📘 **New to Q9gent?** Start with the [Developer Guide](DEVELOPER_GUIDE.md) for comprehensive documentation, examples, and best practices.

---

## Features

- 🚀 **Stateless by Default** - Each request spawns an ephemeral Claude process
- 🔒 **Least-Privilege Tool Fences** - Strict control over allowed tools per agent
- 📡 **Server-Sent Events** - Real-time JSONL streaming via SSE
- 💾 **Optional Session Resumption** - Minimal metadata persistence for multi-turn conversations
- 🎯 **Zero Hidden Orchestration** - The server makes no agentic decisions
- 🔧 **Clean Process Supervision** - Built on Tokio for robust async process management
- 🌐 **Simple HTTP API** - Easy integration with any client
- ✅ **Windows Verified** - Comprehensive testing on Windows 11 with npm-installed Claude CLI

---

## Quick Start

### Installation

### From Source

```bash
git clone https://github.com/ChristopherGRoge/Q9gent.git
cd Q9gent
cargo build --release
```

> 📘 **Complete Setup Guide:** See the [Developer Guide](DEVELOPER_GUIDE.md) for detailed installation instructions, configuration options, and platform-specific considerations.

### From Releases

Download pre-built binaries from the [Releases](https://github.com/ChristopherGRoge/Q9gent/releases) page:

- **Windows**: `q9gent-windows-x86_64.exe`, `q9gent-windows-i686.exe`, `q9gent-windows-aarch64.exe`
- **Linux**: `q9gent-linux-x86_64`, `q9gent-linux-aarch64`, `q9gent-linux-x86_64-musl`
- **macOS**: `q9gent-macos-x86_64`, `q9gent-macos-aarch64`

> **Windows Deployment**: ✅ **Fully tested and verified on Windows 11 with npm-installed Claude CLI**. Version 0.1.2 includes critical fixes for Windows process spawning and pipe handling. See [WINDOWS_DEPLOYMENT.md](WINDOWS_DEPLOYMENT.md) for complete setup instructions.

> **SmartScreen Warning**: Windows may show a security warning because the binaries are not code-signed. This is normal for open-source software. Click "More info" → "Run anyway" to proceed. The binaries are built automatically via GitHub Actions and are safe to use. You can verify the build process in [`.github/workflows/build.yml`](.github/workflows/build.yml).

## Usage

### Prerequisites

**Claude CLI Installation:**

Q9gent requires Claude CLI to be installed. Choose your platform:

- **Windows:** `npm install -g @anthropic/claude-cli`
- **macOS:** `npm install -g @anthropic/claude-cli`
- **Linux:** `npm install -g @anthropic/claude-cli`
- **Docker:** Include `RUN npm install -g @anthropic/claude-cli` in your Dockerfile

**Authentication:**
```bash
claude auth login
```

### Starting the Server

**Auto-Discovery (Claude in PATH):**
```bash
# Q9gent automatically finds 'claude' command
./q9gent
```

**Explicit Path (Recommended for Production):**
```bash
# Windows (npm install)
q9gent.exe --claude-path "C:\Users\USERNAME\AppData\Roaming\npm\claude.cmd"

# macOS/Linux (npm install)
./q9gent --claude-path /usr/local/bin/claude

# Custom installation
./q9gent --claude-path /path/to/claude
```

**Full Configuration:**
```bash
./q9gent \
  --host 0.0.0.0 \
  --port 3000 \
  --session-dir /var/lib/q9gent/sessions \
  --claude-path /usr/local/bin/claude
```

**View Help:**
```bash
./q9gent --help
```

**Command-line Options:**
- `-h, --host <HOST>` - Server bind address (default: `127.0.0.1`)
- `-p, --port <PORT>` - Server port (default: `8080`)
- `-s, --session-dir <SESSION_DIR>` - Session storage directory (default: `./sessions`)
- `-c, --claude-path <CLAUDE_PATH>` - Path to Claude CLI executable (default: `claude`)

### Platform-Specific Notes

**Windows (✅ Production Ready - v0.1.2):**
- **Fully tested** on Windows 11 with npm-installed Claude CLI
- Automatic `.cmd`/`.bat` wrapper detection and `cmd.exe /c` execution
- Fixes for EPIPE (broken pipe) errors in v0.1.2
- Node.js buffering prevention for stable output streaming
- See [WINDOWS_DEPLOYMENT.md](WINDOWS_DEPLOYMENT.md) for complete setup guide
- **Verified scenarios:**
  - npm global install (`C:\Users\...\AppData\Roaming\npm\claude.cmd`)
  - Custom npm prefix installations
  - Process spawning with SSE streaming
  - Multi-turn conversations with session resumption

**macOS/Linux:**
- npm typically installs to `/usr/local/bin/claude`
- Direct execution (no wrapper needed)
- Symlinks are resolved automatically

**Docker:**
- Install Claude CLI in your container image
- Specify `--host 0.0.0.0` to accept external connections
- Mount session directory as volume for persistence

---

## How It Works

Q9gent is a **process supervisor** that gives your application access to Claude Code's full capabilities:

```
Your Application  →  HTTP Request  →  Q9gent  →  Claude CLI Process
                                         ↓
                ←  SSE Stream (real-time)  ←  JSONL Output
```

**The Flow:**

1. **Your app sends an HTTP request** with a prompt, tool permissions, and optional session ID
2. **Q9gent spawns a Claude CLI process** with precise flags (stateless or resuming a session)
3. **Claude "thinks" and acts** - reads files, writes code, uses allowed tools autonomously
4. **Real-time streaming** - Q9gent forwards Claude's JSONL output via Server-Sent Events
5. **Process completes** - Claude finishes the task, Q9gent sends completion event
6. **Optional session persistence** - Minimal metadata saved for multi-turn conversations

**What Makes This Powerful:**

- **Autonomous Tool Use**: Claude can read files, write code, execute commands (with your permission)
- **Multi-Turn Reasoning**: Resume conversations across requests for iterative refinement
- **Streaming Responses**: See Claude's thinking in real-time, not just the final result
- **Flexible Integration**: Any language/framework can use the HTTP API
- **Production Ready**: Battle-tested process management, comprehensive error handling

> 📘 **Learn More:** The [Developer Guide](DEVELOPER_GUIDE.md) includes:
> - Detailed architecture diagrams
> - 8+ real-world use cases with code examples
> - API reference with all endpoints and parameters
> - Session management strategies
> - Security model and best practices
> - Performance tuning and deployment guides

---

### Docker Deployment

**Example Dockerfile:**
```dockerfile
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y nodejs npm
RUN npm install -g @anthropic/claude-cli
COPY q9gent /usr/local/bin/
EXPOSE 8080
CMD ["q9gent", "--host", "0.0.0.0"]
```

**Run:**
```bash
docker run -p 8080:8080 -v ./sessions:/app/sessions q9gent:latest
```

### API Endpoints

#### Health Check
```bash
GET /health
```
**Response:**
```json
{
  "status": "ok",
  "version": "0.1.2"
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

> 📘 **More Examples:** The [Developer Guide](DEVELOPER_GUIDE.md) includes comprehensive examples:
> - Multi-turn interactive coding sessions
> - Restricted tool access patterns
> - Custom system prompts for specialized agents
> - JavaScript/TypeScript client implementations
> - Parallel agent spawning patterns
> - Session cleanup strategies
> - Production integration patterns

---

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

**Windows-specific:**
```powershell
# Find Claude CLI location
where.exe claude

# Use the full path to .cmd file
.\q9gent.exe --claude-path "C:\Users\USERNAME\AppData\Roaming\npm\claude.cmd"
```

### Windows EPIPE Errors (Fixed in v0.1.2)
```
Error: EPIPE: broken pipe, write
Process exited with code: 1
```
**Solution:** Update to v0.1.2 or later. This version includes:
- Continued stdout draining to prevent pipe breakage
- Node.js buffering prevention
- Proper Windows pipe handling

If you're on v0.1.2+ and still see issues:
- Ensure you're using the correct Claude CLI path
- Check that Claude CLI works standalone: `claude --version`
- Enable debug logging: `$env:RUST_LOG="q9gent=debug"`

### Permission Denied (Session Directory)
```
Error: IO error: Permission denied
```
**Solution:** Ensure the session directory is writable:
```bash
mkdir -p ./sessions
chmod 755 ./sessions
```

**Windows:**
```powershell
New-Item -ItemType Directory -Path ".\sessions" -Force
```

### Port Already in Use
```
Error: Address already in use
```
**Solution:** Use a different port:
```bash
./q9gent --port 8081
```

---

## Why Choose Q9gent?

### The Power of Claude Code, Available to Any Application

Q9gent is **not just another API wrapper**. It's a production-grade bridge that brings Claude Code's full autonomous capabilities—the same AI that powers advanced IDEs and development tools—to your application via a simple HTTP interface.

**What You Get:**

🧠 **True AI "Thinking"**
- Claude doesn't just answer questions—it reasons, plans, and executes
- Autonomous tool use: read files, write code, analyze projects
- Multi-step problem solving with persistent context

🔌 **Universal Integration**
- Works with any language: Python, JavaScript, Go, Ruby, Java, etc.
- Simple REST API + Server-Sent Events
- No SDK lock-in, no vendor-specific frameworks

🏗️ **Production Ready**
- Comprehensive Windows/macOS/Linux testing
- Battle-tested process management with Tokio
- Proper error handling, logging, and monitoring hooks
- Resource isolation and clean process supervision

🔒 **Security First**
- Strict per-request tool access control
- Process-level isolation between agents
- Least-privilege by default
- No hidden behavior or orchestration

**Perfect For Building:**
- **AI-Powered IDEs**: Add Claude's coding assistance to editors and development tools
- **Autonomous DevOps**: Code review, testing, documentation generation systems
- **Interactive Assistants**: File-aware chatbots with code execution capabilities
- **Multi-Agent Systems**: Coordinate specialized subagents with isolated permissions
- **Research Platforms**: Experiment with agentic AI in controlled environments

> 📘 **Start Building:** The [Developer Guide](DEVELOPER_GUIDE.md) provides everything you need:
> - Complete API documentation
> - 8+ production use cases with examples
> - Architecture deep-dive
> - Security model and best practices
> - Performance tuning guides
> - Deployment strategies for all platforms

---

## Documentation

- **[Developer Guide](DEVELOPER_GUIDE.md)** - Comprehensive documentation (START HERE)
- **[Windows Deployment](WINDOWS_DEPLOYMENT.md)** - Windows-specific setup and troubleshooting
- **[Testing Summary](TESTING_SUMMARY.md)** - Production readiness verification
- **[Changelog](CHANGELOG.md)** - Version history and release notes

---

## Version History

- **v0.1.2** (2025-10-31) - Windows EPIPE fix, pipe handling improvements
- **v0.1.1** (2025-10-31) - Windows .cmd wrapper detection, cross-platform enhancements
- **v0.1.0** (2025-10-30) - Initial release

---

## Roadmap

**Near-term:**
- [ ] Enhanced metrics and observability (Prometheus integration)
- [ ] WebSocket streaming as SSE alternative
- [ ] Docker Hub published images
- [ ] Kubernetes deployment manifests

**Future:**
- [ ] gRPC endpoint support for high-performance clients
- [ ] Built-in rate limiting middleware
- [ ] Authentication/authorization hooks
- [ ] Multi-region deployment examples
- [ ] Advanced session storage backends (Redis, PostgreSQL)

---

## Acknowledgments

Built with:
- [Tokio](https://tokio.rs/) - Async runtime
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Serde](https://serde.rs/) - Serialization
- [Claude CLI](https://www.anthropic.com/claude) - AI assistant by Anthropic

**Special Thanks:**
- The Anthropic team for Claude and Claude CLI
- The Rust community for exceptional tooling
- All contributors and testers
