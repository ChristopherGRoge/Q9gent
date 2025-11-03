# Q9gent Windows Deployment Guide

**Version:** 0.1.2  
**Last Updated:** October 31, 2025  
**Status:** ✅ Production Ready - Comprehensively Tested on Windows 11

---

## Overview

This guide addresses Windows-specific considerations for deploying and running Q9gent. Q9gent v0.1.2 has been **comprehensively tested and verified** on Windows 11 with npm-installed Claude CLI. All critical issues have been resolved, including `.cmd` wrapper detection and EPIPE (broken pipe) errors.

---

## Prerequisites

### 1. Claude CLI Installation

Q9gent requires Claude CLI to be installed and accessible. On Windows, Claude CLI is typically installed via npm:

```powershell
# Install Claude CLI globally
npm install -g @anthropic/claude-cli

# Verify installation
claude --version

# Find the installation path
where.exe claude
```

**Expected output:**
```
C:\Users\YourUsername\AppData\Roaming\npm\claude.cmd
```

### 2. Authentication

Ensure you're logged in to Claude:

```powershell
claude auth login
```

---

## Installation

### Option 1: Download Pre-built Binary

Download `q9gent-windows-x86_64.exe` from the [Releases](https://github.com/ChristopherGRoge/Q9gent/releases) page.

**Windows SmartScreen Warning:**
- Windows may show a security warning because the binary is not code-signed
- This is normal for open-source software
- Click "More info" → "Run anyway" to proceed
- The binaries are built automatically via GitHub Actions and are safe to use

### Option 2: Build from Source

```powershell
# Clone the repository
git clone https://github.com/ChristopherGRoge/Q9gent.git
cd Q9gent

# Build with Cargo
cargo build --release

# Binary will be at: target\release\q9gent.exe
```

---

## Configuration

### Finding Your Claude CLI Path

Q9gent needs to know where Claude CLI is installed. Find the path:

```powershell
where.exe claude
```

This typically returns one of:
- `C:\Users\YourUsername\AppData\Roaming\npm\claude.cmd` (npm global install)
- `C:\Users\YourUsername\tools\npm-global-22\claude.cmd` (custom npm prefix)
- `C:\Program Files\nodejs\claude.cmd` (system-wide install)

### Starting Q9gent

**Basic usage (if `claude` is in PATH):**
```powershell
.\q9gent-windows-x86_64.exe
```

**With explicit Claude path (recommended):**
```powershell
.\q9gent-windows-x86_64.exe --claude-path "C:\Users\YourUsername\AppData\Roaming\npm\claude.cmd"
```

**With custom port and session directory:**
```powershell
.\q9gent-windows-x86_64.exe `
  --host 127.0.0.1 `
  --port 3000 `
  --session-dir "C:\ProgramData\q9gent\sessions" `
  --claude-path "C:\Users\YourUsername\AppData\Roaming\npm\claude.cmd"
```

**With logging:**
```powershell
$env:RUST_LOG="q9gent=info"
.\q9gent-windows-x86_64.exe --claude-path "C:\Path\To\claude.cmd"
```

---

## Windows-Specific Technical Details

### Process Spawning

Q9gent automatically detects when Claude CLI is installed as a `.cmd` or `.bat` file (typical for npm packages on Windows) and wraps the execution in `cmd.exe /c`.

**What happens internally:**

When you configure:
```powershell
--claude-path "C:\Users\...\npm\claude.cmd"
```

Q9gent executes:
```
cmd.exe /c "C:\Users\...\npm\claude.cmd" -p "your prompt" --output-format stream-json ...
```

This ensures proper shell context and output streaming on Windows.

### File Extensions Handled

- `.cmd` files → Wrapped in `cmd.exe /c`
- `.bat` files → Wrapped in `cmd.exe /c`
- `.exe` files → Executed directly
- Other files → Executed directly (may fail if not executable)

### Logging Behavior

Windows console encoding may affect log output. If you see garbled characters:

```powershell
# Set console to UTF-8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

# Then run Q9gent
$env:RUST_LOG="q9gent=info"
.\q9gent-windows-x86_64.exe
```

---

## Common Issues and Solutions

### Issue 1: "Process spawn failed: No such file or directory"

**Cause:** Q9gent cannot find the Claude CLI executable.

**Solution:**
```powershell
# Find Claude path
where.exe claude

# Use explicit path
.\q9gent-windows-x86_64.exe --claude-path "C:\Path\From\Where\Command\claude.cmd"
```

### Issue 2: EPIPE (broken pipe) errors - Process crashes mid-execution

**Symptoms:**
```
Error: EPIPE: broken pipe, write
Process exited with code: 1
```

**Cause:** Windows pipe handling issue in versions prior to v0.1.2. The process would start successfully but crash when writing output.

**Solution:** ✅ **FIXED in v0.1.2**
- Upgrade to Q9gent v0.1.2 or later
- This version includes continued stdout draining to prevent pipe breakage
- Node.js buffering prevention environment variables
- Proper Windows pipe lifecycle management

**Verify fix:**
```powershell
# Check version
.\q9gent-windows-x86_64.exe --version  # Should show 0.1.2 or later

# Test with debug logging
$env:RUST_LOG="q9gent=debug"
.\q9gent-windows-x86_64.exe --claude-path "C:\Path\To\claude.cmd"

# Look for successful completion without EPIPE errors
```

### Issue 3: Server starts but returns only `data: {"type":"completed"}` with no output

**Cause:** This was a bug in v0.1.0 where `.cmd` files weren't properly wrapped.

**Solution:** ✅ **FIXED in v0.1.1+**
- Upgrade to Q9gent v0.1.1 or later
- Automatic `.cmd` and `.bat` file detection with `cmd.exe /c` wrapper

**Verify fix:**
```powershell
# Enable debug logging
$env:RUST_LOG="q9gent=debug"
.\q9gent-windows-x86_64.exe --claude-path "C:\Path\To\claude.cmd"

# Look for log line:
# "🪟 Windows: Detected .cmd/.bat file, using cmd.exe wrapper"
```

### Issue 4: "Access Denied" when creating session directory

**Cause:** Insufficient permissions for the session directory.

**Solution:**
```powershell
# Create directory with proper permissions
New-Item -ItemType Directory -Path ".\sessions" -Force

# Or use a user-writable location
.\q9gent-windows-x86_64.exe --session-dir "$env:USERPROFILE\q9gent\sessions"
```

### Issue 4: Port already in use

**Cause:** Another process is using port 8080.

**Solution:**
```powershell
# Use a different port
.\q9gent-windows-x86_64.exe --port 8081

# Or find what's using port 8080
netstat -ano | findstr :8080
```

### Issue 5: Firewall blocking connections

**Cause:** Windows Firewall may block incoming connections.

**Solution:**
```powershell
# For localhost-only access (default), no action needed

# For network access, add firewall rule
New-NetFirewallRule -DisplayName "Q9gent" -Direction Inbound -Protocol TCP -LocalPort 8080 -Action Allow
```

---

## Testing Your Installation

### 1. Health Check

```powershell
Invoke-WebRequest -Uri "http://localhost:8080/health"
```

**Expected response:**
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

### 2. Simple Agent Spawn

```powershell
$body = @{
    agent_type = "test"
    prompt = "What is 2+2?"
    tools_allowed = @()
    create_session = $false
} | ConvertTo-Json

Invoke-WebRequest -Uri "http://localhost:8080/spawn" -Method Post -ContentType "application/json" -Body $body
```

**Expected response:**
```
data: {"type":"output","data":"{\"event\":\"text\",\"text\":\"2+2 equals 4.\"}"}

data: {"type":"completed"}
```

### 3. With Logging Enabled

```powershell
# Terminal 1: Start server with debug logging
$env:RUST_LOG="q9gent=debug"
.\q9gent-windows-x86_64.exe --claude-path "C:\Path\To\claude.cmd"

# Terminal 2: Send request
$body = @{
    agent_type = "test"
    prompt = "Hello, Claude!"
    tools_allowed = @()
    create_session = $false
} | ConvertTo-Json

Invoke-WebRequest -Uri "http://localhost:8080/spawn" -Method Post -ContentType "application/json" -Body $body
```

**Check Terminal 1 logs for:**
- `🪟 Windows: Detected .cmd/.bat file, using cmd.exe wrapper`
- `✓ Claude process spawned - PID: ...`
- `📥 First line from Claude stdout`
- `✅ Process ... exited successfully`

---

## PowerShell Helper Script

Create a `start-q9gent.ps1` script for easy launching:

```powershell
# start-q9gent.ps1

# Configuration
$ClaudePath = "C:\Users\$env:USERNAME\AppData\Roaming\npm\claude.cmd"
$Port = 8080
$SessionDir = ".\sessions"

# Enable logging
$env:RUST_LOG = "q9gent=info"

# Check if Claude exists
if (-not (Test-Path $ClaudePath)) {
    Write-Error "Claude CLI not found at: $ClaudePath"
    Write-Host "Run: where.exe claude" -ForegroundColor Yellow
    exit 1
}

# Create session directory
New-Item -ItemType Directory -Path $SessionDir -Force | Out-Null

# Start Q9gent
Write-Host "🎯 Starting Q9gent..." -ForegroundColor Green
Write-Host "📂 Session directory: $SessionDir" -ForegroundColor Cyan
Write-Host "🔧 Claude CLI path: $ClaudePath" -ForegroundColor Cyan
Write-Host "🚀 Port: $Port" -ForegroundColor Cyan

.\q9gent-windows-x86_64.exe `
    --host 127.0.0.1 `
    --port $Port `
    --session-dir $SessionDir `
    --claude-path $ClaudePath
```

**Usage:**
```powershell
.\start-q9gent.ps1
```

---

## Running as a Windows Service

### Option 1: NSSM (Non-Sucking Service Manager)

**Install NSSM:**
```powershell
# Using Chocolatey
choco install nssm

# Or download from: https://nssm.cc/download
```

**Create service:**
```powershell
# Install the service
nssm install Q9gent "C:\Path\To\q9gent-windows-x86_64.exe"

# Set arguments
nssm set Q9gent AppParameters "--host 127.0.0.1 --port 8080 --claude-path C:\Path\To\claude.cmd"

# Set working directory
nssm set Q9gent AppDirectory "C:\Path\To\Q9gent"

# Set environment variable for logging
nssm set Q9gent AppEnvironmentExtra "RUST_LOG=q9gent=info"

# Start the service
nssm start Q9gent
```

**Manage service:**
```powershell
# Check status
nssm status Q9gent

# Stop service
nssm stop Q9gent

# Remove service
nssm remove Q9gent confirm
```

### Option 2: Task Scheduler (Auto-start on login)

```powershell
# Create scheduled task
$action = New-ScheduledTaskAction -Execute "C:\Path\To\q9gent-windows-x86_64.exe" -Argument "--claude-path C:\Path\To\claude.cmd"
$trigger = New-ScheduledTaskTrigger -AtLogOn
$principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -LogonType Interactive
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries

Register-ScheduledTask -TaskName "Q9gent" -Action $action -Trigger $trigger -Principal $principal -Settings $settings

# Run task
Start-ScheduledTask -TaskName "Q9gent"
```

---

## Performance Considerations

### Process Limits

Windows has default limits on child processes:

```powershell
# Check current job object limits (if applicable)
# No direct command, but typically:
# - Max processes: ~2000 per user
# - Max handles: ~10000 per process
```

### File Descriptor Limits

Windows handles this differently than Unix. Monitor with:

```powershell
# Process Explorer (from Sysinternals)
# Shows handle count per process
```

### Memory Usage

Q9gent itself is lightweight (~5-10MB base), but each Claude process may use 100-500MB.

**Monitor memory:**
```powershell
Get-Process q9gent-windows-x86_64 | Select-Object Name, WS, PM
```

---

## Security Recommendations

### 1. Network Binding

**Default (localhost only):**
```powershell
# Safe - only accessible from local machine
.\q9gent-windows-x86_64.exe --host 127.0.0.1
```

**Network access (use with caution):**
```powershell
# Accessible from network - secure with firewall/reverse proxy
.\q9gent-windows-x86_64.exe --host 0.0.0.0
```

### 2. Firewall Configuration

**Allow only localhost (default):**
```powershell
# No firewall rule needed - Windows allows localhost by default
```

**Allow specific IP:**
```powershell
New-NetFirewallRule -DisplayName "Q9gent" `
    -Direction Inbound `
    -Protocol TCP `
    -LocalPort 8080 `
    -RemoteAddress "192.168.1.100" `
    -Action Allow
```

### 3. Session Directory Permissions

```powershell
# Secure session directory
icacls ".\sessions" /inheritance:r
icacls ".\sessions" /grant:r "$env:USERNAME:(OI)(CI)F"
```

### 4. Use Reverse Proxy for Production

For production deployments, use a reverse proxy (IIS, nginx for Windows) with:
- HTTPS/TLS encryption
- Authentication
- Rate limiting
- Access logs

---

## Troubleshooting Checklist

If Q9gent isn't working on Windows:

- [ ] Claude CLI installed? (`where.exe claude`)
- [ ] Claude CLI authenticated? (`claude auth login`)
- [ ] Correct path to `claude.cmd` specified?
- [ ] Firewall not blocking port 8080?
- [ ] Session directory writable?
- [ ] Debug logging enabled? (`$env:RUST_LOG="q9gent=debug"`)
- [ ] Check server logs for errors
- [ ] Test Claude CLI works standalone
- [ ] Version 0.1.0 or later? (`.cmd` wrapper support)

---

## Known Limitations

### 1. PowerShell ISE Compatibility

The binary works best in **Windows Terminal** or **PowerShell 7+**. PowerShell ISE may have encoding issues.

### 2. Long Paths

Windows has a 260-character path limit (unless enabled via registry). Keep installation paths short:

**Enable long paths (requires admin):**
```powershell
New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" `
    -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force
```

### 3. Antivirus False Positives

Some antivirus software may flag the binary. Add an exception if needed:

```powershell
# Windows Defender exclusion
Add-MpPreference -ExclusionPath "C:\Path\To\q9gent-windows-x86_64.exe"
```

---

## Support

For Windows-specific issues:

1. **Check logs:** Run with `$env:RUST_LOG="q9gent=debug"`
2. **GitHub Issues:** https://github.com/ChristopherGRoge/Q9gent/issues
3. **Include:** OS version, PowerShell version, Claude CLI path, full logs

---

## Version History

- **0.1.2** (2025-10-31) - ✅ **Current: Production Ready**
  - **Fixed:** EPIPE (broken pipe) errors causing process crashes
  - **Fixed:** Continued stdout draining to prevent pipe breakage
  - **Added:** Node.js buffering prevention (NODE_NO_WARNINGS=1)
  - **Verified:** Comprehensive testing on Windows 11 with npm Claude CLI
  - **Status:** Full Windows support confirmed and stable

- **0.1.1** (2025-10-31)
  - Fixed: Windows `.cmd` wrapper detection and execution
  - Added: Automatic `cmd.exe /c` wrapper for `.cmd` and `.bat` files
  - Added: Platform-specific process spawning logic
  - Added: Enhanced logging for process spawning
  - Added: Exit status monitoring

- **0.1.0** (2025-10-30)
  - Initial release
  - Basic Claude CLI process spawning

---

**End of Windows Deployment Guide**
