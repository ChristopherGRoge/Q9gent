# Q9gent v0.1.1 Release Notes

**Release Date:** October 31, 2025  
**Type:** Patch Release - Critical Bug Fix  
**Git Tag:** `v0.1.1`

---

## 🎯 Overview

Version 0.1.1 is a **critical patch release** that fixes Windows deployment and enhances cross-platform Claude CLI detection. This release ensures Q9gent works seamlessly on Windows, macOS, Linux, and Docker containers.

---

## 🔥 Critical Fix: Windows Process Spawning

### The Problem
The Windows binary (`q9gent-windows-x86_64.exe`) would start successfully but failed to produce any output from Claude CLI, returning only:
```
data: {"type":"completed"}
```

### Root Cause
When Claude CLI is installed via npm on Windows, it creates a `.cmd` wrapper script. Rust's process spawning was attempting to execute this directly, which fails silently on Windows because batch scripts require explicit `cmd.exe /c` wrapper execution.

### The Solution
Q9gent now automatically:
- ✅ Detects `.cmd` and `.bat` files on Windows
- ✅ Wraps them in `cmd.exe /c` for proper execution
- ✅ Executes `.exe` files directly (no wrapper)
- ✅ Maintains Unix/Linux/macOS direct execution

### Impact
- **Before:** Windows completely non-functional
- **After:** Full Windows support for npm-installed Claude CLI

---

## 🆕 What's New

### Enhanced Cross-Platform Support

**Flexible Claude CLI Detection:**
- Auto-discovery when Claude is in PATH
- Explicit path configuration for production
- Support for custom npm prefixes
- Docker container compatibility
- Works with any Claude CLI installation method

**Example Deployments:**

```bash
# Auto-discovery (all platforms)
./q9gent

# Windows (npm install)
q9gent.exe --claude-path "C:\Users\USERNAME\AppData\Roaming\npm\claude.cmd"

# macOS/Linux (npm install)
./q9gent --claude-path /usr/local/bin/claude

# Docker container
docker run -p 8080:8080 q9gent:latest --claude-path /usr/local/bin/claude

# Custom location
./q9gent --claude-path /opt/custom/path/to/claude
```

### Improved Diagnostics

**Enhanced Logging:**
- Detailed process spawn logging
- Exit status monitoring
- Warning when zero output received
- Full command logging for debugging

**Example Log Output:**
```
🔨 Building Claude command - 5 args
📋 Executing: cmd.exe ["/c", "C:\\...\\claude.cmd", "-p", "test", ...]
✓ Claude process spawned - PID: 12345
📥 First line from Claude stdout
📊 Stdout reader finished - 42 lines read
✅ Process 12345 exited successfully with status: 0
```

---

## 📚 New Documentation

### Added
- **WINDOWS_DEPLOYMENT.md** - Comprehensive Windows setup guide
  - npm installation instructions
  - PowerShell helper scripts
  - Running as Windows service
  - Troubleshooting guide

- **WINDOWS_FIX_VERIFICATION.md** - Test suite for verification
  - Step-by-step test plan
  - Automated PowerShell test script
  - Success criteria checklist

- **WINDOWS_FIX_SUMMARY.md** - Technical analysis
  - Root cause analysis
  - Implementation details
  - Lessons learned

- **IMPLEMENTATION_SUMMARY.md** - Developer reference
  - Code changes overview
  - Testing results
  - Deployment checklist

### Updated
- **README.md** - Platform-specific deployment sections
- **DEVELOPER_GUIDE.md** - Multi-platform Claude CLI scenarios
- **CHANGELOG.md** - Comprehensive release notes

---

## 🧪 Testing

All tests pass:
```
running 6 tests
......
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured
```

New platform-specific tests:
- ✅ `test_windows_cmd_wrapper()` - Verifies `.cmd` wrapping
- ✅ `test_windows_bat_wrapper()` - Verifies `.bat` wrapping
- ✅ `test_windows_exe_direct()` - Verifies `.exe` direct execution
- ✅ `test_unix_direct_execution()` - Verifies Unix unchanged

---

## 🔧 Technical Details

### Files Modified
- `src/agent.rs` - Platform-specific process spawning
- `src/api.rs` - Exit status monitoring
- `Cargo.toml` - Version bump to 0.1.1
- `CHANGELOG.md` - Release notes
- `README.md` - Usage documentation
- `DEVELOPER_GUIDE.md` - Deployment scenarios

### Code Changes Summary
- Added `prepare_platform_command()` method
- Enhanced error messages
- Process exit logging
- Zero-output warnings
- Improved stderr handling

---

## 🚀 Deployment

### Breaking Changes
**NONE** - This release is 100% backwards compatible.

### Migration Guide
No migration needed. Simply replace the binary:

**Windows:**
```powershell
# Download new q9gent-windows-x86_64.exe
# Replace old binary
# Start with same configuration
```

**Linux/macOS:**
```bash
# Download new binary
# Replace: cp q9gent-new /path/to/q9gent
# Restart service
```

**Docker:**
```bash
# Pull new image
docker pull q9gent:0.1.1

# Restart container
docker-compose restart
```

---

## 📦 Download

### Pre-built Binaries

**Windows:**
- `q9gent-windows-x86_64.exe` (Recommended)
- `q9gent-windows-i686.exe`
- `q9gent-windows-aarch64.exe`

**Linux:**
- `q9gent-linux-x86_64` (Recommended)
- `q9gent-linux-aarch64`
- `q9gent-linux-x86_64-musl`

**macOS:**
- `q9gent-macos-x86_64` (Intel)
- `q9gent-macos-aarch64` (Apple Silicon)

### Source
```bash
git clone https://github.com/ChristopherGRoge/Q9gent.git
cd Q9gent
git checkout v0.1.1
cargo build --release
```

---

## 🔍 Verification

### Quick Test

**1. Health Check:**
```bash
curl http://localhost:8080/health
# Expected: {"status":"ok","version":"0.1.1"}
```

**2. Agent Spawn:**
```bash
curl -N -X POST http://localhost:8080/spawn \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type":"test",
    "prompt":"Say hello",
    "tools_allowed":[],
    "create_session":false
  }'

# Expected: data: {"type":"output",...} events (NOT just "completed")
```

**3. Check Logs:**
```bash
# Windows
$env:RUST_LOG="q9gent=debug"
.\q9gent-windows-x86_64.exe --claude-path "C:\...\claude.cmd"

# Look for: "🪟 Windows: Detected .cmd/.bat file, using cmd.exe wrapper"
```

---

## 🐛 Known Issues

**None** - All critical issues from 0.1.0 are resolved.

If you encounter issues:
1. Check [WINDOWS_DEPLOYMENT.md](WINDOWS_DEPLOYMENT.md) for platform-specific guidance
2. Enable debug logging: `RUST_LOG=q9gent=debug`
3. Report issues: https://github.com/ChristopherGRoge/Q9gent/issues

---

## 📈 Performance

**No performance regression:**
- Process spawn time unchanged (~50-100ms)
- Memory usage unchanged
- Network overhead unchanged
- Only addition: One string comparison for `.cmd`/`.bat` detection

---

## 🔐 Security

**No security changes:**
- Same threat model as 0.1.0
- Same process isolation
- Same tool access control
- Same network binding defaults

---

## 🙏 Acknowledgments

**Issue Reporter:** System Integration Testing  
**Platform:** Windows 11 with npm-installed Claude CLI  
**Severity:** HIGH - Complete service failure on Windows  

This release directly addresses the critical bug report submitted during initial Windows deployment testing.

---

## 📅 Next Steps

**Planned for 0.2.0:**
- WebSocket streaming alternative to SSE
- Metrics and observability
- Rate limiting middleware
- Authentication/authorization hooks
- Enhanced Docker image

---

## 🔗 Links

- **Repository:** https://github.com/ChristopherGRoge/Q9gent
- **Releases:** https://github.com/ChristopherGRoge/Q9gent/releases
- **Issues:** https://github.com/ChristopherGRoge/Q9gent/issues
- **Documentation:** [DEVELOPER_GUIDE.md](DEVELOPER_GUIDE.md)

---

## 📝 Full Changelog

See [CHANGELOG.md](CHANGELOG.md) for complete version history.

**Upgrade recommended for all users, especially Windows deployments.**

---

**Release Hash:** `git rev-parse v0.1.1`  
**Build Date:** October 31, 2025  
**Rust Version:** 1.70+
