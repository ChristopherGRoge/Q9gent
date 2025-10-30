# Q9gent API Specification

**Version:** 0.1.0  
**Base URL:** `http://localhost:8080`  
**Protocol:** HTTP/1.1  
**Content-Type:** `application/json`

---

## Table of Contents
1. [Health Check](#health-check)
2. [Spawn Agent](#spawn-agent)
3. [Message Session](#message-session)
4. [Terminate Agent](#terminate-agent)
5. [List Sessions](#list-sessions)
6. [Error Responses](#error-responses)
7. [Server-Sent Events Format](#server-sent-events-format)

---

## Health Check

Check if the server is running and responsive.

**Endpoint:** `GET /health`

**Response:** `200 OK`
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

**Example:**
```bash
curl http://localhost:8080/health
```

---

## Spawn Agent

Spawn a new Claude CLI process with specified configuration. Returns a Server-Sent Events stream.

**Endpoint:** `POST /spawn`

**Request Body:**
```json
{
  "agent_type": "string",           // Descriptive type (informational only)
  "prompt": "string",                // The prompt to send to Claude
  "flags": ["string"],               // Optional: Additional CLI flags
  "tools_allowed": ["string"],       // Optional: List of allowed tool names
  "system_append": "string",         // Optional: Additional system prompt
  "resume_id": "string",             // Optional: Session ID to resume
  "create_session": boolean          // Optional: Create new session (default: false)
}
```

**Field Details:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `agent_type` | string | Yes | Descriptive type for logging/tracking |
| `prompt` | string | Yes | The prompt to send to Claude |
| `flags` | array[string] | No | Raw CLI flags to pass to claude command |
| `tools_allowed` | array[string] | No | Tools the agent can use (passed as --allowedTools) |
| `system_append` | string | No | Additional system prompt (--append-system-prompt) |
| `resume_id` | string | No | Session ID to resume a previous conversation |
| `create_session` | boolean | No | Whether to create a new session for resumption |

**Response:** `200 OK` - Server-Sent Events stream

**Event Types:**

1. **session_created** - Emitted when `create_session: true`
```json
{
  "type": "session_created",
  "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

2. **output** - Claude's JSONL output
```json
{
  "type": "output",
  "data": "{\"event\":\"text\",\"text\":\"Response content...\"}"
}
```

3. **error** - Error during execution
```json
{
  "type": "error",
  "error": "Error message"
}
```

4. **completed** - Agent finished processing
```json
{
  "type": "completed"
}
```

**Example:**
```bash
curl -N -X POST http://localhost:8080/spawn \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "code_helper",
    "prompt": "Write a function to sort an array",
    "tools_allowed": ["write_file"],
    "system_append": "You are a coding expert",
    "create_session": true
  }'
```

**Error Responses:**

- `400 Bad Request` - Invalid request body
- `500 Internal Server Error` - Failed to spawn process

---

## Message Session

Send a message to an existing session (resume conversation).

**Endpoint:** `POST /message/{session_id}`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `session_id` | string (UUID) | The session ID to message |

**Request Body:**
```json
{
  "prompt": "string",                // The prompt to send
  "flags": ["string"],               // Optional: Additional CLI flags
  "tools_allowed": ["string"],       // Optional: List of allowed tool names
  "system_append": "string"          // Optional: Additional system prompt
}
```

**Response:** `200 OK` - Server-Sent Events stream (same format as `/spawn`)

**Example:**
```bash
curl -N -X POST http://localhost:8080/message/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Now add error handling",
    "tools_allowed": ["write_file", "read_file"]
  }'
```

**Error Responses:**

- `400 Bad Request` - Invalid request body
- `404 Not Found` - Session not found
- `500 Internal Server Error` - Failed to spawn process

---

## Terminate Agent

Forcefully terminate a running agent process.

**Endpoint:** `POST /terminate/{session_id}`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `session_id` | string (UUID) | The session ID to terminate |

**Response:** `200 OK`
```json
{
  "message": "Process terminated successfully"
}
```

**Example:**
```bash
curl -X POST http://localhost:8080/terminate/550e8400-e29b-41d4-a716-446655440000
```

**Error Responses:**

- `404 Not Found` - Session not found or not running
- `500 Internal Server Error` - Failed to terminate process

---

## List Sessions

Retrieve all stored session metadata.

**Endpoint:** `GET /sessions`

**Response:** `200 OK`
```json
{
  "sessions": [
    {
      "session_id": "550e8400-e29b-41d4-a716-446655440000",
      "agent_type": "code_helper",
      "created_at": 1698624000,
      "last_used": 1698624120
    },
    {
      "session_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
      "agent_type": "assistant",
      "created_at": 1698625000,
      "last_used": 1698625050
    }
  ]
}
```

**Field Details:**

| Field | Type | Description |
|-------|------|-------------|
| `session_id` | string (UUID) | Unique session identifier |
| `agent_type` | string | Type specified when creating session |
| `created_at` | number | Unix timestamp (seconds) when created |
| `last_used` | number | Unix timestamp (seconds) of last activity |

**Example:**
```bash
curl http://localhost:8080/sessions
```

**Error Responses:**

- `500 Internal Server Error` - Failed to read session directory

---

## Error Responses

All error responses follow this format:

```json
{
  "error": "Error message describing what went wrong"
}
```

**HTTP Status Codes:**

| Code | Meaning |
|------|---------|
| 200 | Success |
| 400 | Bad Request - Invalid input |
| 404 | Not Found - Resource doesn't exist |
| 500 | Internal Server Error - Server-side failure |

---

## Server-Sent Events Format

SSE responses use the standard SSE format:

```
data: {"type":"event_type","field":"value"}

data: {"type":"event_type","field":"value"}

```

Each event is prefixed with `data: ` and followed by two newlines.

**Consuming SSE in Different Languages:**

### JavaScript
```javascript
const eventSource = new EventSource('http://localhost:8080/spawn', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ /* request body */ })
});

eventSource.addEventListener('message', (event) => {
  const data = JSON.parse(event.data);
  console.log(data);
});
```

### Python
```python
import requests
import json

response = requests.post(url, json=payload, stream=True)
for line in response.iter_lines():
    if line and line.startswith(b'data: '):
        data = json.loads(line[6:])
        print(data)
```

### cURL
```bash
curl -N -X POST http://localhost:8080/spawn \
  -H "Content-Type: application/json" \
  -d '{ /* request body */ }'
```

---

## Rate Limiting

**Current Status:** Not implemented

Future versions may include rate limiting. Check response headers for:
- `X-RateLimit-Limit` - Requests allowed per window
- `X-RateLimit-Remaining` - Requests remaining
- `X-RateLimit-Reset` - Time when limit resets

---

## Authentication

**Current Status:** Not implemented

The server currently has no authentication. For production use:
1. Run behind a reverse proxy (nginx, caddy)
2. Implement authentication at the proxy level
3. Bind to `127.0.0.1` for local-only access

---

## CORS

**Current Status:** Permissive (all origins allowed)

CORS is configured to allow all origins for development. For production:
1. Configure specific allowed origins
2. Set up proper CORS headers in reverse proxy

---

## Versioning

API version is included in the `/health` response. Breaking changes will increment the major version number following semantic versioning.

**Current Version:** 0.1.0

---

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for detailed version history.

---

## Support

- **Issues:** https://github.com/ChristopherGRoge/Q9gent/issues
- **Discussions:** https://github.com/ChristopherGRoge/Q9gent/discussions
- **Documentation:** See [README.md](README.md) and [EXAMPLES.md](EXAMPLES.md)
