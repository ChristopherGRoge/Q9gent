# Q9gent Logging Guide

Q9gent uses structured logging with visual indicators to make it easy to monitor service activity.

## Log Levels

The service supports standard Rust tracing log levels:
- `error` - Critical errors
- `warn` - Warnings and potential issues
- `info` - General information about requests and operations
- `debug` - Detailed diagnostic information
- `trace` - Very verbose output (not used)

## Configuration

Set the log level via the `RUST_LOG` environment variable:

```bash
# Default (info level)
./q9gent

# Debug level for detailed output
RUST_LOG=q9gent=debug ./q9gent

# Debug Q9gent + HTTP layer
RUST_LOG=q9gent=debug,tower_http=debug ./q9gent

# Error level only
RUST_LOG=q9gent=error ./q9gent
```

## Log Format

Logs use emoji indicators for quick visual scanning:

### Service Lifecycle
```
🎯 Q9gent v0.1.0 starting...
📂 Session directory: ./sessions
🔧 Claude CLI path: claude
🚀 Server listening on http://127.0.0.1:8080
📍 Endpoints: /health, /spawn, /message/:id, /terminate/:id, /sessions
```

### Request Handling

**Spawn Request (info level):**
```
🚀 Spawn request - agent_type: 'code_helper', create_session: true, tools: ["write_file"], prompt_length: 45 chars
📝 Created session: 550e8400-e29b-41d4-a716-446655440000
🔨 Building Claude command - 8 args
✓ Claude process spawned - PID: Some(12345)
📤 Sending session_created event for: 550e8400-e29b-41d4-a716-446655440000
📥 First output received from Claude
✅ Claude process completed - 15 output lines sent
🧹 Cleaned up process for session: 550e8400-e29b-41d4-a716-446655440000
```

**Message Request (resuming session):**
```
💬 Message request - session_id: 550e8400-e29b-41d4-a716-446655440000, tools: ["read_file", "write_file"], prompt_length: 32 chars
✓ Session found and updated: 550e8400-e29b-41d4-a716-446655440000
⚡ Resuming Claude session...
✓ Claude process resumed successfully
📥 First output received from resumed session
✅ Resumed session completed - 8 output lines sent
🧹 Cleaned up resumed session: 550e8400-e29b-41d4-a716-446655440000
```

**Terminate Request:**
```
🛑 Terminate request - session_id: 550e8400-e29b-41d4-a716-446655440000
🛑 Terminating Claude process - PID: Some(12345)
✓ Process terminated
✓ Process terminated successfully: 550e8400-e29b-41d4-a716-446655440000
```

**List Sessions:**
```
📋 List sessions request
✓ Found 3 sessions
```

**Health Check (debug level):**
```
Health check requested
```

### Debug Level Details

When `RUST_LOG=q9gent=debug`, you'll also see:

```
Spawn request details - flags: [], system_append: Some("You are a helpful..."), resume_id: None
No session requested (stateless mode)
Stored process for session: 550e8400-e29b-41d4-a716-446655440000
Command: claude ["-p", "Write hello world", "--output-format", "stream-json", "--allowedTools", "write_file"]
📥 First line from Claude stdout
Claude stdout line 1: 156 chars
Claude stdout line 2: 234 chars
...
📊 Stdout reader finished - 15 lines read
Stderr reader finished
Channel closed, stopping stdout reader
```

### Error/Warning Messages

**Process spawn failed:**
```
❌ Failed to spawn Claude process: No such file or directory
```

**Session not found:**
```
⚠️  No running process found for session: 550e8400-e29b-41d4-a716-446655440000
```

**Claude stderr:**
```
⚠️  Claude stderr: Warning: some warning message
```

**Claude process error:**
```
❌ Error from Claude process: Process exited with status 1
```

## Example Output

Starting the server:
```bash
$ RUST_LOG=q9gent=info ./q9gent
2025-10-30T12:00:00.123456Z  INFO q9gent: 🎯 Q9gent v0.1.0 starting...
2025-10-30T12:00:00.123789Z  INFO q9gent: 📂 Session directory: ./sessions
2025-10-30T12:00:00.124012Z  INFO q9gent: 🔧 Claude CLI path: claude
2025-10-30T12:00:00.125456Z  INFO q9gent::api: 🚀 Server listening on http://127.0.0.1:8080
2025-10-30T12:00:00.125567Z  INFO q9gent::api: 📍 Endpoints: /health, /spawn, /message/:id, /terminate/:id, /sessions
```

Handling a spawn request:
```bash
2025-10-30T12:01:15.234567Z  INFO q9gent::api: 🚀 Spawn request - agent_type: 'code_helper', create_session: true, tools: ["write_file"], prompt_length: 45 chars
2025-10-30T12:01:15.235678Z  INFO q9gent::api: 📝 Created session: 550e8400-e29b-41d4-a716-446655440000
2025-10-30T12:01:15.236789Z  INFO q9gent::agent: 🔨 Building Claude command - 8 args
2025-10-30T12:01:15.567890Z  INFO q9gent::agent: ✓ Claude process spawned - PID: Some(12345)
2025-10-30T12:01:15.568901Z  INFO q9gent::api: ⚡ Spawning Claude CLI process...
2025-10-30T12:01:15.569012Z  INFO q9gent::api: ✓ Claude process spawned successfully
2025-10-30T12:01:15.570123Z  INFO q9gent::api: 📤 Sending session_created event for: 550e8400-e29b-41d4-a716-446655440000
2025-10-30T12:01:17.890123Z  INFO q9gent::api: 📥 First output received from Claude
2025-10-30T12:01:25.123456Z  INFO q9gent::api: ✅ Claude process completed - 15 output lines sent
```

## Integration with Monitoring

### Prometheus/Grafana

Parse logs for metrics:
- Count `🚀 Spawn request` for request rate
- Measure time between `⚡ Spawning` and `✓ Claude process spawned` for spawn latency
- Count `✅ Claude process completed` vs `❌ Error` for success rate
- Track output line counts from completed messages

### Log Aggregation (ELK, Splunk, etc.)

Search patterns:
- `"🚀 Spawn request"` - All spawn requests
- `"❌"` - All errors
- `"session_id:"` - Track specific sessions
- `"PID:"` - Process management events
- `"output lines sent"` - Response sizes

### Alerting

Set up alerts for:
- `"❌ Failed to spawn Claude process"` - Process spawn failures
- Multiple `"⚠️  No running process found"` - Session management issues
- High frequency of `"❌ Error from Claude process"` - Claude CLI problems

## Tips

1. **Development**: Use `RUST_LOG=q9gent=debug` to see all details
2. **Production**: Use `RUST_LOG=q9gent=info` for operational visibility
3. **Performance**: Use `RUST_LOG=q9gent=warn` or `error` to minimize overhead
4. **Troubleshooting**: Enable debug temporarily when investigating issues

## Structured Fields

All log entries include:
- Timestamp (ISO 8601)
- Log level
- Module path (e.g., `q9gent::api`, `q9gent::agent`)
- Message with context

Example:
```
2025-10-30T12:00:00.123456Z  INFO q9gent::api: 🚀 Spawn request - agent_type: 'code_helper', create_session: true, tools: ["write_file"], prompt_length: 45 chars
```

This structured format makes it easy to:
- Filter by module (`q9gent::api` vs `q9gent::agent`)
- Search by time range
- Parse programmatically
- Correlate events across requests
