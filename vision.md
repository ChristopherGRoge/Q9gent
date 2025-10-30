# Vision ✅ IMPLEMENTED

Design a lightweight Rust CLI Assistant Server that does one job: spawn short-lived Claude Code Headless processes on demand with precise CLI flags, then stream results back to the caller. The server itself makes no agentic decisions; the "orchestrator" is just another self-contained subagent the caller may request, not a resident process. For each request, the server receives {agent_type, prompt, flags, tools_allowed, system_append, optional_resume_id}, launches claude -p --output-format stream-json [--allowedTools …] [--append-system-prompt …] [--resume <session-id>], streams JSONL stdout to the client, and exits the process when the turn completes. Persist only minimal metadata the caller asks for (e.g., last session-id) so the caller can later provide --resume to reattach; otherwise treat agents as ephemeral. Prioritize: clean process supervision (Tokio), strict least-privilege tool fences, stateless request handling by default, simple HTTP/gRPC endpoints (/spawn, /message, /terminate optional), and zero hidden orchestration. The outcome is a reusable, composable "agent runner" that the caller can use to invoke any subagent—briefly, repeatably, and safely.

Must be pure Rust, with git action to compile for Windows, Mac and Linux, priorize Windows.

---

## ✅ Implementation Status

**Status:** COMPLETE  
**Version:** 0.1.0  
**Date:** October 30, 2025

### Implemented Features

✅ Pure Rust implementation  
✅ Short-lived Claude process spawning  
✅ Precise CLI flag control  
✅ JSONL streaming via Server-Sent Events  
✅ Minimal session metadata persistence  
✅ Stateless by default  
✅ Clean Tokio-based process supervision  
✅ HTTP endpoints: /spawn, /message/:id, /terminate/:id, /sessions, /health  
✅ Strict least-privilege tool fences  
✅ Zero hidden orchestration  
✅ Cross-platform GitHub Actions (Windows, Mac, Linux)  
✅ Windows builds prioritized (3 variants)  

### Project Documentation

- **[README.md](README.md)** - Complete user guide
- **[API.md](API.md)** - Full API specification  
- **[EXAMPLES.md](EXAMPLES.md)** - Code examples
- **[DEVELOPMENT.md](DEVELOPMENT.md)** - Developer guide
- **[PROJECT_SUMMARY.md](PROJECT_SUMMARY.md)** - Architecture overview
- **[QUICKREF.md](QUICKREF.md)** - Quick reference
- **[DOCS.md](DOCS.md)** - Documentation index

### Quick Start

```bash
# Build
cargo build --release

# Run
./target/release/q9gent --host 127.0.0.1 --port 8080

# Test
curl http://localhost:8080/health
```

See [README.md](README.md) for complete documentation.
