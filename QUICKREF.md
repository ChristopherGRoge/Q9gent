# Q9gent Quick Reference

## Installation

```bash
# From source
git clone https://github.com/yourusername/Q9gent.git
cd Q9gent
cargo build --release

# Or download from releases
# https://github.com/yourusername/Q9gent/releases
```

## Start Server

```bash
# Default (127.0.0.1:8080)
./q9gent

# Custom
./q9gent --host 0.0.0.0 --port 3000 --claude-path /usr/bin/claude
```

## Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Health check |
| `/spawn` | POST | Create agent |
| `/message/:id` | POST | Resume session |
| `/terminate/:id` | POST | Kill agent |
| `/sessions` | GET | List sessions |

## Request Examples

### Spawn Agent (One-shot)
```bash
curl -N -X POST http://localhost:8080/spawn \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "assistant",
    "prompt": "Your prompt here",
    "tools_allowed": ["read_file", "write_file"],
    "create_session": false
  }'
```

### Spawn with Session
```bash
curl -N -X POST http://localhost:8080/spawn \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "assistant",
    "prompt": "Your prompt here",
    "tools_allowed": ["read_file"],
    "create_session": true
  }'
# Returns: session_id in first event
```

### Resume Session
```bash
curl -N -X POST http://localhost:8080/message/SESSION_ID \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Follow-up prompt",
    "tools_allowed": ["read_file", "write_file"]
  }'
```

### Terminate Agent
```bash
curl -X POST http://localhost:8080/terminate/SESSION_ID
```

### List Sessions
```bash
curl http://localhost:8080/sessions | jq
```

## Response Format

Server-Sent Events (SSE):

```
data: {"type":"session_created","session_id":"..."}

data: {"type":"output","data":"{\"event\":\"text\",\"text\":\"...\"}"}

data: {"type":"completed"}
```

## Event Types

| Type | Description |
|------|-------------|
| `session_created` | New session created |
| `output` | Claude JSONL output |
| `error` | Error occurred |
| `completed` | Agent finished |

## Tool Control

```json
{
  "tools_allowed": [
    "read_file",
    "write_file",
    "list_files",
    "search_files",
    "run_command",
    // ... other tools
  ]
}
```

**Empty array = no tools allowed**

## Common Patterns

### Read-Only Agent
```json
{
  "tools_allowed": ["read_file", "list_files", "search_files"]
}
```

### Code Generator
```json
{
  "tools_allowed": ["write_file"],
  "system_append": "Write clean, well-documented code"
}
```

### Full-Access Assistant
```json
{
  "tools_allowed": [
    "read_file", "write_file", "list_files", 
    "search_files", "run_command"
  ]
}
```

## Environment Variables

```bash
# Logging level
export RUST_LOG=q9gent=debug

# Start server
./q9gent
```

Log levels: `trace`, `debug`, `info`, `warn`, `error`

## Build Commands

```bash
# Debug
cargo build

# Release
cargo build --release

# Test
cargo test

# Format
cargo fmt

# Lint
cargo clippy
```

## Docker

```bash
# Build image
docker build -t q9gent .

# Run container
docker run -p 8080:8080 q9gent

# With docker-compose
docker-compose up -d
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| `claude: not found` | Install Claude CLI or use `--claude-path` |
| `Address in use` | Use different `--port` |
| `Permission denied` | Check session directory permissions |
| `Connection refused` | Check if server is running |

## Files & Directories

```
./sessions/          # Session metadata (JSON files)
./q9gent            # Binary
```

## Security Notes

- Default: localhost only (`127.0.0.1`)
- No authentication (use reverse proxy)
- Tool access controlled per request
- Each agent isolated in separate process

## For More Info

- Full docs: `README.md`
- API spec: `API.md`
- Examples: `EXAMPLES.md`
- Development: `DEVELOPMENT.md`

---

**Q9gent v0.1.0** | MIT License | Pure Rust
