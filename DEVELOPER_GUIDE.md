# Q9gent Developer Guide

**Version:** 0.1.0  
**Last Updated:** October 30, 2025

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Core Design Principles](#core-design-principles)
3. [System Components](#system-components)
4. [Request Flow](#request-flow)
5. [API Reference](#api-reference)
6. [Data Models](#data-models)
7. [Process Management](#process-management)
8. [Session Management](#session-management)
9. [Error Handling](#error-handling)
10. [Security Model](#security-model)
11. [Extension Points](#extension-points)
12. [Testing Strategy](#testing-strategy)
13. [Performance Considerations](#performance-considerations)
14. [Deployment Guide](#deployment-guide)
15. [Troubleshooting](#troubleshooting)

---

## Architecture Overview

### What is Q9gent?

Q9gent is a **lightweight, stateless HTTP server** that acts as a process supervisor for Claude CLI instances. It does **NOT** make agentic decisions itself—it simply:

1. Receives HTTP requests with agent configuration
2. Spawns a Claude CLI process with precise flags
3. Streams the JSONL output back to the client via Server-Sent Events
4. Terminates the process when complete
5. Optionally persists minimal session metadata for conversation resumption

### Core Philosophy

```
┌─────────────────────────────────────────────────────────────┐
│                Q9gent is NOT an Agent                       │
│                                                             │
│  Q9gent is a PROCESS RUNNER that spawns agents on demand   │
│                                                             │
│  • No hidden orchestration                                 │
│  • No decision-making                                      │
│  • No agent state management                               │
│  • Just process supervision + HTTP API                     │
└─────────────────────────────────────────────────────────────┘
```

### Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Runtime** | Tokio | Async I/O, process management |
| **Web Framework** | Axum | HTTP server, routing, SSE |
| **Serialization** | Serde | JSON encoding/decoding |
| **CLI Parsing** | Clap | Command-line argument parsing |
| **Logging** | Tracing | Structured logging |
| **Session IDs** | UUID v4 | Unique session identifiers |

---

## Core Design Principles

### 1. Ephemerality First

**Processes are short-lived by default.**

- Each agent request spawns a fresh Claude CLI process
- Process exits when the turn completes
- No persistent state unless explicitly requested
- Zero shared memory between processes

```rust
// Process lifecycle
spawn() -> stream_output() -> exit()
```

### 2. Least Privilege Tool Fencing

**Strict per-request tool access control.**

- Caller specifies exactly which tools are allowed
- Passed directly to Claude via `--allowedTools`
- No server-side tool execution
- No privilege escalation

```json
{
  "tools_allowed": ["read_file", "write_file"]
  // Agent can ONLY use these two tools
}
```

### 3. Zero Hidden Orchestration

**The server makes no decisions.**

- No prompt modification
- No tool filtering beyond what's specified
- No result transformation
- Pure passthrough of Claude's output

### 4. Stateless by Default

**State is optional, not required.**

- Stateless requests: spawn → stream → done
- Stateful requests: create session → spawn with session_id → stream → done
- Session metadata is minimal (ID, timestamps, agent_type)
- No conversation history stored by Q9gent

### 5. Composability

**Q9gent is a building block.**

- Can be integrated into larger systems
- Can be chained with other services
- Can be used for any Claude CLI use case
- No opinions about how it's used

---

## System Components

### Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                         Client                              │
│              (HTTP requests, SSE streams)                   │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                      Q9gent Server                          │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                   API Layer (Axum)                   │  │
│  │   /spawn  /message  /terminate  /sessions  /health  │  │
│  └──────────────────────┬───────────────────────────────┘  │
│                         │                                   │
│  ┌──────────────────────▼───────────────────────────────┐  │
│  │              AgentRunner (Process Mgmt)              │  │
│  │  • Builds CLI command from request                   │  │
│  │  • Spawns child process                             │  │
│  │  • Streams stdout/stderr                            │  │
│  │  • Manages process lifecycle                        │  │
│  └──────────────────────┬───────────────────────────────┘  │
│                         │                                   │
│  ┌──────────────────────▼───────────────────────────────┐  │
│  │         SessionStore (Optional Persistence)          │  │
│  │  • Creates session metadata                          │  │
│  │  • Saves/loads session files                        │  │
│  │  • Updates timestamps                               │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   Claude CLI Process                        │
│         (Ephemeral, spawned per request)                    │
└─────────────────────────────────────────────────────────────┘
```

### Module Breakdown

#### `src/main.rs`
- Entry point
- CLI argument parsing
- Server initialization
- Configuration setup

#### `src/api.rs`
- HTTP endpoint handlers
- SSE stream creation
- Request/response types
- State management

#### `src/agent.rs`
- Claude CLI command building
- Process spawning (via `tokio::process`)
- stdout/stderr stream management
- Process termination

#### `src/session.rs`
- Session metadata CRUD operations
- File-based persistence
- Session ID generation
- Timestamp management

#### `src/error.rs`
- Error type definitions
- HTTP status code mapping
- Error serialization

#### `src/config.rs`
- Server configuration struct
- Runtime settings

---

## Request Flow

### Stateless Request Flow

```
Client                    Q9gent                    Claude CLI
  │                         │                           │
  │  POST /spawn           │                           │
  ├────────────────────────>│                           │
  │  {prompt, tools, ...}  │                           │
  │                         │                           │
  │                         │  Build CLI command        │
  │                         │  (add flags, tools, etc)  │
  │                         │                           │
  │                         │  spawn process           │
  │                         ├──────────────────────────>│
  │                         │                           │
  │                         │  JSONL output            │
  │                         │<──────────────────────────┤
  │  SSE stream            │                           │
  │<────────────────────────┤                           │
  │  data: {"event":...}   │                           │
  │  data: {"event":...}   │                           │
  │  data: {"event":...}   │                           │
  │                         │                           │
  │                         │  process exit            │
  │                         │<──────────────────────────┤
  │  data: {"type":        │                           │
  │        "completed"}    │                           │
  │<────────────────────────┤                           │
  │                         │                           │
  │  [Connection closed]   │                           │
```

### Stateful Request Flow (Multi-turn)

```
Client                    Q9gent                    SessionStore
  │                         │                           │
  │  POST /spawn           │                           │
  │  {create_session:true} │                           │
  ├────────────────────────>│                           │
  │                         │  create_session()        │
  │                         ├──────────────────────────>│
  │                         │  session_id + metadata   │
  │                         │<──────────────────────────┤
  │                         │                           │
  │  SSE: session_created  │                           │
  │<────────────────────────┤                           │
  │  {session_id: "..."}   │                           │
  │                         │                           │
  │  [Claude process runs, streams output...]          │
  │                         │                           │
  │  POST /message/        │                           │
  │       {session_id}     │                           │
  ├────────────────────────>│                           │
  │                         │  touch_session()         │
  │                         ├──────────────────────────>│
  │                         │  updated timestamp       │
  │                         │<──────────────────────────┤
  │                         │                           │
  │  [Claude resumes with --resume flag...]            │
```

---

## API Reference

### Endpoint: `GET /health`

**Purpose:** Health check endpoint for monitoring and load balancers.

**Request:**
```http
GET /health HTTP/1.1
Host: localhost:8080
```

**Response:**
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

**Status Codes:**
- `200 OK` - Server is healthy

---

### Endpoint: `POST /spawn`

**Purpose:** Spawn a new Claude CLI process with specified configuration.

**Request:**
```json
{
  "agent_type": "string",           // Descriptive type (logging only)
  "prompt": "string",                // Required: The prompt
  "flags": ["string"],               // Optional: Raw CLI flags
  "tools_allowed": ["string"],       // Optional: Allowed tools
  "system_append": "string",         // Optional: System prompt addition
  "resume_id": "string",             // Optional: Resume session
  "create_session": boolean          // Optional: Create new session
}
```

**Field Details:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `agent_type` | string | Yes | - | Descriptive label for logging/tracking |
| `prompt` | string | Yes | - | The prompt to send to Claude |
| `flags` | array[string] | No | `[]` | Additional CLI flags (advanced) |
| `tools_allowed` | array[string] | No | `[]` | Whitelist of allowed tools |
| `system_append` | string | No | `null` | Additional system prompt text |
| `resume_id` | string | No | `null` | Session ID to resume |
| `create_session` | boolean | No | `false` | Whether to create a session |

**Response:** Server-Sent Events (SSE) stream

**Event Types:**

1. **session_created** (only if `create_session: true`)
   ```json
   {
     "type": "session_created",
     "session_id": "550e8400-e29b-41d4-a716-446655440000"
   }
   ```

2. **output** (repeated for each Claude output line)
   ```json
   {
     "type": "output",
     "data": "{\"event\":\"text\",\"text\":\"Claude's response...\"}"
   }
   ```
   Note: `data` is a **string containing JSONL** from Claude

3. **error** (if process fails)
   ```json
   {
     "type": "error",
     "error": "Process execution failed: <reason>"
   }
   ```

4. **completed** (final event)
   ```json
   {
     "type": "completed"
   }
   ```

**Status Codes:**
- `200 OK` - Stream started successfully
- `500 Internal Server Error` - Failed to spawn process

**Example:**
```bash
curl -N -X POST http://localhost:8080/spawn \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "code_helper",
    "prompt": "Write a hello world in Python",
    "tools_allowed": ["write_file"],
    "create_session": true
  }'
```

---

### Endpoint: `POST /message/{session_id}`

**Purpose:** Send a message to an existing session (resume conversation).

**Path Parameters:**
- `session_id` (UUID) - The session to resume

**Request:**
```json
{
  "prompt": "string",                // Required: Follow-up prompt
  "flags": ["string"],               // Optional: CLI flags
  "tools_allowed": ["string"],       // Optional: Allowed tools
  "system_append": "string"          // Optional: System prompt
}
```

**Response:** SSE stream (same format as `/spawn`, minus `session_created`)

**Status Codes:**
- `200 OK` - Stream started
- `404 Not Found` - Session doesn't exist
- `500 Internal Server Error` - Process spawn failed

**Example:**
```bash
curl -N -X POST http://localhost:8080/message/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Now add error handling",
    "tools_allowed": ["read_file", "write_file"]
  }'
```

---

### Endpoint: `POST /terminate/{session_id}`

**Purpose:** Forcefully terminate a running Claude process.

**Path Parameters:**
- `session_id` (UUID) - The session to terminate

**Request:** No body

**Response:**
```json
{
  "message": "Process terminated successfully"
}
```

**Status Codes:**
- `200 OK` - Process terminated
- `404 Not Found` - No running process for this session

---

### Endpoint: `GET /sessions`

**Purpose:** List all stored session metadata.

**Request:** No parameters

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

**Status Codes:**
- `200 OK` - Sessions retrieved

---

## Data Models

### AgentRequest

```rust
pub struct AgentRequest {
    pub agent_type: String,           // Descriptive type
    pub prompt: String,                // The prompt
    pub flags: Vec<String>,            // CLI flags
    pub tools_allowed: Vec<String>,    // Allowed tools
    pub system_append: Option<String>, // System prompt
    pub resume_id: Option<String>,     // Session to resume
}
```

**Maps to Claude CLI:**
```bash
claude \
  -p "{prompt}" \
  --output-format stream-json \
  --allowedTools {tools_allowed.join(",")} \
  --append-system-prompt "{system_append}" \
  --resume "{resume_id}" \
  {flags.join(" ")}
```

### SessionMetadata

```rust
pub struct SessionMetadata {
    pub session_id: String,      // UUID v4
    pub agent_type: String,       // From request
    pub created_at: u64,          // Unix timestamp (seconds)
    pub last_used: u64,           // Unix timestamp (seconds)
}
```

**Persistence:**
- Stored as JSON files in `{session_dir}/{session_id}.json`
- Read on demand, not cached in memory
- Updated on every message to session

---

## Process Management

### Process Lifecycle

1. **Spawn:**
   ```rust
   Command::new(&claude_path)
       .args(&args)
       .stdout(Stdio::piped())
       .stderr(Stdio::piped())
       .kill_on_drop(true)  // Critical: auto-cleanup
       .spawn()?
   ```

2. **Stream stdout:**
   ```rust
   let reader = BufReader::new(stdout);
   let mut lines = reader.lines();
   
   while let Some(line) = lines.next_line().await? {
       // Send via SSE channel
       tx.send(Ok(line)).await?;
   }
   ```

3. **Log stderr:**
   ```rust
   // Stderr is logged, not sent to client
   while let Some(line) = stderr_lines.next_line().await? {
       warn!("Claude stderr: {}", line);
   }
   ```

4. **Cleanup:**
   ```rust
   // Automatic via kill_on_drop(true)
   // Or manual: child.kill().await?
   ```

### Concurrency Model

- **One process per request**
- **Multiple concurrent requests supported** (via Tokio)
- **No shared state between processes**
- **Process isolation via OS**

```rust
// Multiple concurrent agents
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│  Request 1  │  │  Request 2  │  │  Request 3  │
│  (spawn)    │  │  (spawn)    │  │  (spawn)    │
└──────┬──────┘  └──────┬──────┘  └──────┬──────┘
       │                │                │
       ▼                ▼                ▼
   ┌───────┐        ┌───────┐        ┌───────┐
   │Claude1│        │Claude2│        │Claude3│
   │  PID  │        │  PID  │        │  PID  │
   │ 12345 │        │ 12346 │        │ 12347 │
   └───────┘        └───────┘        └───────┘
```

### Process Supervision

**Tokio Features Used:**
- `tokio::process::Command` - Async process spawning
- `tokio::io::BufReader` - Async line reading
- `tokio::sync::mpsc` - Channel for output streaming
- `tokio::spawn` - Concurrent task execution

**Error Handling:**
- Process spawn failures → `ProcessSpawnFailed` error
- Process crashes → Logged, stream ends with error event
- Timeout → Not implemented (intentional - client can terminate)

---

## Session Management

### Session Storage

**Location:** `{session_dir}/{session_id}.json`

**Example:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "agent_type": "code_helper",
  "created_at": 1698624000,
  "last_used": 1698624120
}
```

### Session Operations

1. **Create:**
   ```rust
   let session = SessionStore::create_session(agent_type).await?;
   // Generates UUID, sets timestamps, saves to disk
   ```

2. **Load:**
   ```rust
   let session = SessionStore::load_session(session_id).await?;
   // Reads from disk, deserializes JSON
   ```

3. **Touch (update timestamp):**
   ```rust
   SessionStore::touch_session(session_id).await?;
   // Updates last_used, saves to disk
   ```

### What Sessions DON'T Store

- ❌ Conversation history
- ❌ Tool call logs
- ❌ User information
- ❌ Authentication tokens
- ❌ Agent state

**Sessions only store metadata for Claude's `--resume` feature.**

---

## Error Handling

### Error Types

```rust
pub enum AppError {
    ProcessSpawnFailed(String),      // Can't start Claude
    ProcessExecutionError(String),   // Claude crashed
    SessionNotFound(String),         // Session doesn't exist
    IoError(std::io::Error),         // File I/O failed
    SerializationError(serde_json::Error), // JSON parsing failed
}
```

### HTTP Status Code Mapping

| Error | HTTP Status | Description |
|-------|-------------|-------------|
| `ProcessSpawnFailed` | 500 | Failed to start process |
| `ProcessExecutionError` | 500 | Process crashed |
| `SessionNotFound` | 404 | Session ID not found |
| `IoError` | 500 | File system error |
| `SerializationError` | 500 | JSON error |

### Error Response Format

```json
{
  "error": "Human-readable error message"
}
```

### Error Recovery

**Q9gent does NOT retry.**
- Failed process spawn → Return error to client
- Process crash → End stream with error event
- Session not found → Return 404

**Retry logic is the client's responsibility.**

---

## Security Model

### Threat Model

Q9gent assumes:
1. **Trusted network** - Default binding is `127.0.0.1` (localhost)
2. **Trusted clients** - No authentication (add via reverse proxy)
3. **Trusted Claude CLI** - Process execution is inherently trusted
4. **Untrusted prompts** - But Claude handles prompt injection

### Security Features

#### 1. Tool Access Control

```json
{
  "tools_allowed": ["read_file"]
  // Agent CANNOT use write_file, run_command, etc.
}
```

**Enforcement:** Passed directly to Claude via `--allowedTools`

#### 2. Process Isolation

- Each agent runs in a separate OS process
- No shared memory between agents
- No inter-agent communication
- Process dies when request completes

#### 3. Network Binding

**Default:** `127.0.0.1` (localhost only)
```bash
./q9gent --host 127.0.0.1 --port 8080
```

**Production:** Use reverse proxy (nginx, caddy) for:
- TLS/HTTPS
- Authentication
- Rate limiting
- IP whitelisting

#### 4. Session Storage

**Unencrypted by design:**
- Session files contain only metadata
- No sensitive data stored
- File permissions: 644 (user read/write)

**Recommendations:**
- Secure the session directory (`chmod 700`)
- Use encrypted filesystem if needed
- Rotate session files periodically

---

## Extension Points

### Adding New Endpoints

**Location:** `src/api.rs`

```rust
// 1. Define request/response types
#[derive(Deserialize)]
struct MyRequest {
    field: String,
}

#[derive(Serialize)]
struct MyResponse {
    result: String,
}

// 2. Implement handler
async fn my_endpoint(
    State(state): State<AppState>,
    Json(payload): Json<MyRequest>,
) -> AppResult<Json<MyResponse>> {
    // Implementation
    Ok(Json(MyResponse {
        result: "success".to_string(),
    }))
}

// 3. Add route
fn app(state: AppState) -> Router {
    Router::new()
        .route("/my-endpoint", post(my_endpoint))
        // ... existing routes
        .with_state(state)
}
```

### Custom Process Management

**Location:** `src/agent.rs`

Extend `AgentRunner`:
```rust
impl AgentRunner {
    pub async fn spawn_with_timeout(
        &self,
        request: AgentRequest,
        timeout: Duration,
    ) -> AppResult<(Child, Receiver<AppResult<String>>)> {
        // Custom spawn logic
    }
}
```

### Custom Session Storage

**Location:** `src/session.rs`

Implement custom backend:
```rust
pub trait SessionBackend {
    async fn save(&self, metadata: &SessionMetadata) -> AppResult<()>;
    async fn load(&self, session_id: &str) -> AppResult<SessionMetadata>;
}

// Redis backend
pub struct RedisSessionStore {
    client: redis::Client,
}

// Database backend
pub struct DbSessionStore {
    pool: sqlx::PgPool,
}
```

### Middleware

Add Axum middleware:
```rust
use tower_http::{
    auth::RequireAuthorizationLayer,
    compression::CompressionLayer,
    timeout::TimeoutLayer,
};

Router::new()
    .route("/spawn", post(spawn))
    .layer(TimeoutLayer::new(Duration::from_secs(30)))
    .layer(CompressionLayer::new())
    .layer(RequireAuthorizationLayer::bearer("token"))
```

---

## Testing Strategy

### Unit Tests

**Location:** `src/*/tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = SessionStore::new(temp_dir.path());
        
        let session = store
            .create_session("test".to_string())
            .await
            .unwrap();
        
        assert!(!session.session_id.is_empty());
    }
}
```

**Run:** `cargo test`

### Integration Tests

Create `tests/integration_test.rs`:
```rust
#[tokio::test]
async fn test_spawn_endpoint() {
    let server = spawn_test_server().await;
    
    let response = reqwest::Client::new()
        .post(format!("{}/spawn", server.addr()))
        .json(&json!({
            "agent_type": "test",
            "prompt": "Hello",
            "create_session": false,
        }))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
}
```

### Load Testing

Use `wrk` or `k6`:
```bash
# wrk
wrk -t4 -c100 -d30s \
  -s spawn.lua \
  http://localhost:8080/spawn

# k6
k6 run load_test.js
```

---

## Performance Considerations

### Bottlenecks

1. **Process Spawn Time:** ~50-100ms per spawn
2. **Claude CLI Startup:** ~1-2s for first token
3. **File I/O:** Session metadata reads/writes
4. **Network:** SSE streaming overhead

### Optimization Strategies

#### 1. Process Pooling (Not Implemented)

**Concept:** Pre-spawn idle Claude processes
```rust
// Not recommended - violates ephemerality principle
struct ProcessPool {
    idle_processes: Vec<Child>,
    max_size: usize,
}
```

**Why not:** Breaks statelessness, complicates cleanup

#### 2. Session Caching (Possible)

```rust
use moka::future::Cache;

struct CachedSessionStore {
    cache: Cache<String, SessionMetadata>,
    disk: SessionStore,
}
```

**Trade-off:** Memory vs. disk I/O

#### 3. Connection Pooling

Already handled by Axum/Tokio - no action needed.

### Resource Limits

**Recommended:**
```bash
# Max file descriptors (processes)
ulimit -n 4096

# Max processes per user
ulimit -u 2048
```

**Monitoring:**
```rust
// Add metrics
use prometheus::{Counter, Histogram};

static SPAWN_COUNT: Counter = Counter::new("q9gent_spawns_total", "Total spawns");
static SPAWN_DURATION: Histogram = Histogram::new("q9gent_spawn_duration_seconds", "Spawn duration");
```

---

## Deployment Guide

### Docker Deployment

**Build:**
```bash
docker build -t q9gent:0.1.0 .
```

**Run:**
```bash
docker run -d \
  --name q9gent \
  -p 8080:8080 \
  -v /path/to/sessions:/app/sessions \
  -v ~/.config/claude:/home/q9gent/.config/claude:ro \
  -e RUST_LOG=q9gent=info \
  q9gent:0.1.0
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: q9gent
spec:
  replicas: 3
  selector:
    matchLabels:
      app: q9gent
  template:
    metadata:
      labels:
        app: q9gent
    spec:
      containers:
      - name: q9gent
        image: q9gent:0.1.0
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "q9gent=info"
        volumeMounts:
        - name: sessions
          mountPath: /app/sessions
      volumes:
      - name: sessions
        persistentVolumeClaim:
          claimName: q9gent-sessions
```

### Systemd Service

```ini
[Unit]
Description=Q9gent Claude CLI Server
After=network.target

[Service]
Type=simple
User=q9gent
WorkingDirectory=/opt/q9gent
ExecStart=/opt/q9gent/q9gent --host 127.0.0.1 --port 8080
Restart=on-failure
Environment="RUST_LOG=q9gent=info"

[Install]
WantedBy=multi-user.target
```

### Reverse Proxy (Nginx)

```nginx
upstream q9gent {
    server 127.0.0.1:8080;
}

server {
    listen 443 ssl http2;
    server_name q9gent.example.com;

    ssl_certificate /etc/ssl/certs/q9gent.crt;
    ssl_certificate_key /etc/ssl/private/q9gent.key;

    location / {
        proxy_pass http://q9gent;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        
        # SSE-specific settings
        proxy_buffering off;
        proxy_cache off;
        proxy_set_header Connection '';
        proxy_http_version 1.1;
        chunked_transfer_encoding off;
    }
}
```

---

## Troubleshooting

### Problem: Process Spawn Fails

**Symptom:**
```json
{"error": "Process spawn failed: No such file or directory"}
```

**Diagnosis:**
1. Check Claude CLI is installed: `which claude`
2. Check PATH: `echo $PATH`
3. Check permissions: `ls -l $(which claude)`

**Solution:**
```bash
# Install Claude CLI
# OR specify path explicitly
./q9gent --claude-path /usr/local/bin/claude
```

### Problem: Session Not Found

**Symptom:**
```json
{"error": "Session not found: 550e8400-..."}
```

**Diagnosis:**
1. Check session directory exists: `ls ./sessions`
2. Check session file exists: `ls ./sessions/*.json`
3. Check file permissions: `ls -l ./sessions`

**Solution:**
```bash
# Ensure directory exists
mkdir -p ./sessions
chmod 755 ./sessions

# Check file ownership
chown -R $USER:$USER ./sessions
```

### Problem: High Memory Usage

**Symptom:** Server memory grows continuously

**Diagnosis:**
1. Check running processes: `ps aux | grep claude`
2. Check file descriptors: `lsof -p $(pgrep q9gent)`
3. Check session files: `du -sh ./sessions`

**Solution:**
- Ensure `kill_on_drop(true)` is set
- Implement session cleanup (delete old sessions)
- Monitor with: `RUST_LOG=q9gent=debug cargo run`

### Problem: SSE Stream Hangs

**Symptom:** Client receives no events

**Diagnosis:**
1. Check Claude process is running: `ps aux | grep claude`
2. Check network connection: `netstat -an | grep 8080`
3. Check logs: `RUST_LOG=q9gent=debug`

**Solution:**
- Disable proxy buffering (nginx: `proxy_buffering off`)
- Check firewall rules
- Verify client SSE implementation

---

## Advanced Topics

### Custom Tool Definitions

Q9gent passes tools to Claude, but doesn't define them:

```json
{
  "tools_allowed": ["custom_tool_1", "custom_tool_2"]
}
```

**Claude CLI must recognize these tools.**

### Multi-Region Deployment

```
        ┌─────────────┐
        │ Load Balancer│
        └──────┬───────┘
               │
       ┌───────┴───────┐
       │               │
   ┌───▼────┐     ┌───▼────┐
   │Q9gent  │     │Q9gent  │
   │Region 1│     │Region 2│
   └────────┘     └────────┘
```

**Considerations:**
- Session files must be replicated
- Use shared storage (S3, NFS)
- Or implement session routing

### Metrics & Observability

**Add Prometheus metrics:**
```rust
use prometheus::{Encoder, TextEncoder, Registry};

let registry = Registry::new();

// Register metrics
registry.register(Box::new(SPAWN_COUNT)).unwrap();
registry.register(Box::new(SPAWN_DURATION)).unwrap();

// Expose /metrics endpoint
async fn metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
```

---

## Conclusion

Q9gent is intentionally simple:

✅ Spawns processes  
✅ Streams output  
✅ Manages sessions (optionally)  
✅ Nothing more

This simplicity is a feature, not a limitation. It makes Q9gent:
- **Composable** - Easy to integrate
- **Debuggable** - Clear execution path
- **Reliable** - Fewer moving parts
- **Maintainable** - Small codebase

For complex orchestration, build it **around** Q9gent, not **inside** it.

---

## References

- **Repository:** https://github.com/ChristopherGRoge/Q9gent
- **API Documentation:** [API.md](API.md)
- **Examples:** [EXAMPLES.md](EXAMPLES.md)
- **Contributing:** [DEVELOPMENT.md](DEVELOPMENT.md)
- **Tokio Docs:** https://tokio.rs/
- **Axum Docs:** https://docs.rs/axum/
- **Claude CLI:** https://www.anthropic.com/claude-cli

---

**Version History:**
- 0.1.0 (2025-10-30) - Initial release
