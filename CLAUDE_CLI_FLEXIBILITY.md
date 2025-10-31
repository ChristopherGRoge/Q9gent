# Q9gent Claude CLI Detection - Flexibility Confirmation

**Date:** October 31, 2025  
**Version:** 0.1.1  
**Status:** ✅ CONFIRMED - Maximum Flexibility Achieved

---

## Executive Summary

Q9gent is designed with **maximum flexibility** to locate and use Claude CLI installations across all platforms and deployment scenarios. The `--claude-path` flag provides complete control while maintaining sensible defaults.

---

## ✅ Supported Scenarios

### 1. Auto-Discovery (PATH-based)

**How it works:**
- Default `--claude-path` value is `"claude"`
- OS resolves the executable from system PATH
- No configuration needed if Claude is in PATH

**Platforms:**
- ✅ Windows (if `claude.cmd` is in PATH)
- ✅ macOS (if `claude` is in PATH)
- ✅ Linux (if `claude` is in PATH)
- ✅ Docker (if `claude` is in PATH inside container)

**Example:**
```bash
# All platforms - Claude in PATH
./q9gent
```

**Advantage:** Zero configuration, "just works" for default installations.

---

### 2. Explicit Path (Production Recommended)

**How it works:**
- Specify exact path to Claude CLI executable
- Bypasses PATH resolution
- Predictable, no surprises

**Platforms:**
- ✅ Windows npm install: `--claude-path "C:\Users\...\npm\claude.cmd"`
- ✅ macOS npm install: `--claude-path "/usr/local/bin/claude"`
- ✅ Linux npm install: `--claude-path "/usr/local/bin/claude"`
- ✅ Custom location: `--claude-path "/opt/claude/bin/claude"`
- ✅ Docker: `--claude-path "/usr/local/bin/claude"`

**Example:**
```bash
# Production deployment - explicit path
./q9gent --claude-path /usr/local/bin/claude
```

**Advantage:** Predictable, auditable, production-ready.

---

### 3. npm Custom Prefix

**How it works:**
- npm can install to custom locations via `npm config set prefix`
- Specify the custom path to Q9gent

**Platforms:**
- ✅ Windows custom prefix
- ✅ macOS custom prefix
- ✅ Linux custom prefix

**Example:**
```bash
# npm installed to $HOME/.npm-global
./q9gent --claude-path "$HOME/.npm-global/bin/claude"
```

**Advantage:** Supports non-standard npm configurations.

---

### 4. Docker Container

**Scenario A: Claude in Container Image**

```dockerfile
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y nodejs npm
RUN npm install -g @anthropic/claude-cli
COPY q9gent /usr/local/bin/
EXPOSE 8080

# Claude is in PATH at /usr/local/bin/claude
CMD ["q9gent", "--host", "0.0.0.0"]
```

**Scenario B: Explicit Path in Container**

```dockerfile
# ... same setup ...
CMD ["q9gent", "--host", "0.0.0.0", "--claude-path", "/usr/local/bin/claude"]
```

**Scenario C: Mount from Host**

```bash
# Mount host's Claude CLI into container
docker run -v /usr/local/bin/claude:/app/claude \
  q9gent:latest --claude-path /app/claude
```

**Advantage:** Full Docker compatibility, multiple mounting strategies.

---

### 5. Symlink Resolution

**How it works:**
- OS automatically resolves symlinks
- Q9gent executes the resolved target

**Example:**
```bash
# /usr/local/bin/claude -> /opt/claude-v1.2.3/bin/claude
./q9gent --claude-path /usr/local/bin/claude
# Executes: /opt/claude-v1.2.3/bin/claude
```

**Advantage:** Version management via symlinks supported.

---

### 6. Environment-Specific Paths

**Development:**
```bash
./q9gent --claude-path "$HOME/dev/claude/bin/claude"
```

**Staging:**
```bash
./q9gent --claude-path "/opt/staging/claude"
```

**Production:**
```bash
./q9gent --claude-path "/opt/production/claude"
```

**Advantage:** Environment-specific Claude CLI versions.

---

## 🔍 Platform-Specific Detection

### Windows

**Automatic Detection:**
- File extension check: `.cmd`, `.bat` → wrapped in `cmd.exe /c`
- File extension check: `.exe` → direct execution
- Case-insensitive: `CLAUDE.CMD` and `claude.cmd` both work

**Example:**
```powershell
# npm global install
q9gent.exe --claude-path "C:\Users\USERNAME\AppData\Roaming\npm\claude.cmd"

# Custom install
q9gent.exe --claude-path "C:\Tools\claude\claude.exe"
```

**Result:**
```
🪟 Windows: Detected .cmd/.bat file, using cmd.exe wrapper
📋 Executing: cmd.exe ["/c", "C:\\...\\claude.cmd", "-p", ...]
```

---

### macOS

**Typical Locations:**
- `/usr/local/bin/claude` (npm global)
- `/opt/homebrew/bin/claude` (Homebrew)
- `$HOME/.npm-global/bin/claude` (custom prefix)

**Example:**
```bash
# npm install
./q9gent --claude-path /usr/local/bin/claude

# Homebrew install (if available)
./q9gent --claude-path /opt/homebrew/bin/claude
```

**Result:**
```
🐧 Unix: Direct execution of /usr/local/bin/claude
✓ Claude process spawned - PID: 12345
```

---

### Linux

**Typical Locations:**
- `/usr/local/bin/claude` (npm global)
- `/usr/bin/claude` (system package)
- `$HOME/.npm-global/bin/claude` (custom prefix)
- `/opt/claude/bin/claude` (manual install)

**Example:**
```bash
# System install
./q9gent --claude-path /usr/local/bin/claude

# Containerized
./q9gent --claude-path /app/bin/claude
```

**Result:**
```
🐧 Unix: Direct execution of /usr/local/bin/claude
✓ Claude process spawned - PID: 67890
```

---

## 🐳 Docker Deployment Flexibility

### Strategy 1: Build-Time Installation

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y nodejs npm ca-certificates
RUN npm install -g @anthropic/claude-cli

COPY --from=builder /app/target/release/q9gent /usr/local/bin/

EXPOSE 8080
CMD ["q9gent", "--host", "0.0.0.0", "--claude-path", "/usr/local/bin/claude"]
```

**Advantage:** Self-contained, reproducible builds.

---

### Strategy 2: Runtime Volume Mount

```yaml
# docker-compose.yml
version: '3.8'
services:
  q9gent:
    image: q9gent:0.1.1
    ports:
      - "8080:8080"
    volumes:
      - /usr/local/bin/claude:/app/claude:ro
    command: ["--host", "0.0.0.0", "--claude-path", "/app/claude"]
```

**Advantage:** Use host's Claude installation, easier updates.

---

### Strategy 3: Environment Variable Configuration

```dockerfile
ENV CLAUDE_PATH=/usr/local/bin/claude
CMD ["sh", "-c", "q9gent --host 0.0.0.0 --claude-path $CLAUDE_PATH"]
```

```bash
docker run -e CLAUDE_PATH=/custom/path/claude q9gent:latest
```

**Advantage:** Runtime configurability without rebuilding.

---

## 🧪 Verification Methods

### 1. Check Which Claude is Used

```bash
# Enable debug logging
RUST_LOG=q9gent=debug ./q9gent --claude-path /path/to/claude

# Look for log line:
# "🔧 Claude CLI path: /path/to/claude"
# "📋 Executing: ..."
```

### 2. Test Claude Path Resolution

**Windows:**
```powershell
where.exe claude
# Output: C:\Users\...\npm\claude.cmd
```

**macOS/Linux:**
```bash
which claude
# Output: /usr/local/bin/claude
```

### 3. Verify in Container

```bash
docker exec -it q9gent-container which claude
# Output: /usr/local/bin/claude

docker exec -it q9gent-container ls -l /usr/local/bin/claude
# Output: -rwxr-xr-x ... /usr/local/bin/claude
```

---

## ✅ Flexibility Confirmation Checklist

- [x] **Windows npm install** - Works via `.cmd` wrapper
- [x] **Windows .exe** - Works via direct execution
- [x] **macOS npm install** - Works via direct execution
- [x] **Linux npm install** - Works via direct execution
- [x] **Docker build-time** - Works with Dockerfile RUN npm install
- [x] **Docker runtime mount** - Works with volume mounts
- [x] **Custom install paths** - Works with any absolute path
- [x] **npm custom prefix** - Works with $HOME/.npm-global
- [x] **Symlink resolution** - Works via OS symlink following
- [x] **PATH auto-discovery** - Works if claude in PATH
- [x] **Explicit path** - Works with --claude-path flag
- [x] **Environment variables** - Works with $CLAUDE_PATH in scripts

---

## 🎯 Design Goals Achieved

### ✅ Goal 1: Zero-Config Default
**Status:** ACHIEVED  
- Default `--claude-path "claude"` works if Claude in PATH
- No configuration needed for standard installations

### ✅ Goal 2: Production Flexibility
**Status:** ACHIEVED  
- Explicit path configuration for production
- No guessing, fully deterministic

### ✅ Goal 3: Multi-Platform Support
**Status:** ACHIEVED  
- Windows: Automatic `.cmd` wrapper detection
- macOS/Linux: Direct execution
- Docker: Multiple deployment strategies

### ✅ Goal 4: Container-First Design
**Status:** ACHIEVED  
- Build-time installation supported
- Runtime volume mounting supported
- Environment variable configuration supported

### ✅ Goal 5: Developer-Friendly
**Status:** ACHIEVED  
- Auto-discovery for local development
- Explicit paths for CI/CD
- Comprehensive logging for debugging

---

## 📋 Recommended Practices

### For Local Development
```bash
# Let Q9gent find Claude automatically
./q9gent
```

### For Production Deployment
```bash
# Always use explicit paths
./q9gent --claude-path /usr/local/bin/claude
```

### For Docker Production
```dockerfile
# Pin the exact path in Dockerfile
CMD ["q9gent", "--host", "0.0.0.0", "--claude-path", "/usr/local/bin/claude"]
```

### For Multi-Environment
```bash
# Use environment-specific config files
./q9gent --claude-path "$(cat /etc/q9gent/claude-path)"
```

---

## 🔮 Future Enhancements (Not Needed, But Possible)

### Auto-Discovery Improvements
- Search common installation paths
- Detect multiple Claude versions
- Warn if Claude not found

**Status:** Not implemented - current design is sufficient.

### Configuration File Support
```yaml
# q9gent.yml
claude_path: /usr/local/bin/claude
host: 0.0.0.0
port: 8080
```

**Status:** Not needed - CLI flags are sufficient.

### Health Check with Claude Version
```json
{
  "status": "ok",
  "version": "0.1.1",
  "claude_version": "1.2.3",
  "claude_path": "/usr/local/bin/claude"
}
```

**Status:** Could be added in future release.

---

## ✅ Final Confirmation

**Q9gent v0.1.1 provides MAXIMUM FLEXIBILITY for Claude CLI detection:**

✅ **Auto-discovers** Claude in PATH (all platforms)  
✅ **Accepts explicit paths** for production  
✅ **Handles npm installations** on Windows, macOS, Linux  
✅ **Works in Docker containers** (build-time or runtime)  
✅ **Supports custom locations** via `--claude-path`  
✅ **Platform-aware** (Windows `.cmd` wrapper, Unix direct)  
✅ **Symlink-compatible** (OS handles resolution)  
✅ **Environment-flexible** (dev, staging, prod)  

**Conclusion:** The current implementation achieves all flexibility goals without additional complexity. No further changes needed.

---

**Verified by:** Development Team  
**Date:** October 31, 2025  
**Version:** 0.1.1
