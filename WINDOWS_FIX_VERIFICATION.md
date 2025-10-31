# Windows Fix Verification Guide

**Version:** 0.1.0+  
**Date:** October 31, 2025  
**Purpose:** Verify that the Windows `.cmd` wrapper fix resolves the silent failure issue

---

## Overview

This document provides step-by-step instructions for testing the Windows process spawning fix that addresses the critical bug where Q9gent would spawn Claude CLI processes but produce no output.

---

## Prerequisites

1. **Windows 10/11** with PowerShell 5.1 or PowerShell 7+
2. **Claude CLI** installed via npm
3. **Q9gent binary** version 0.1.0 or later (with the fix)

---

## Test Plan

### Test 1: Verify Claude CLI Installation

**Objective:** Confirm Claude CLI is accessible and functional.

**Steps:**
```powershell
# 1. Find Claude CLI path
where.exe claude

# Expected: C:\Users\...\npm\claude.cmd (or similar)

# 2. Verify it's a .cmd file
(where.exe claude) -match "\.cmd$"

# Expected: True

# 3. Test Claude CLI directly
claude --version

# Expected: Output showing Claude version
```

**Success Criteria:**
- ✅ `claude` command is found
- ✅ Path ends with `.cmd` or `.bat`
- ✅ `claude --version` produces output

---

### Test 2: Verify Fix is Present

**Objective:** Confirm the binary includes the Windows wrapper fix.

**Steps:**
```powershell
# 1. Enable debug logging
$env:RUST_LOG = "q9gent=debug"

# 2. Start Q9gent with explicit Claude path
$claudePath = (where.exe claude)
.\q9gent-windows-x86_64.exe --claude-path $claudePath

# 3. Look for log line indicating wrapper detection:
# "🪟 Windows: Detected .cmd/.bat file, using cmd.exe wrapper"
```

**Success Criteria:**
- ✅ Server starts without errors
- ✅ Log shows: `🪟 Windows: Detected .cmd/.bat file, using cmd.exe wrapper`
- ✅ Log shows: `📋 Executing: cmd.exe ["/c", "C:\\..\\claude.cmd", ...]`

---

### Test 3: Simple Agent Spawn (No Tools)

**Objective:** Test basic prompt processing with zero tools.

**Setup:**
```powershell
# Terminal 1: Start Q9gent with debug logging
$env:RUST_LOG = "q9gent=debug"
$claudePath = (where.exe claude)
.\q9gent-windows-x86_64.exe --claude-path $claudePath
```

**Test:**
```powershell
# Terminal 2: Send test request
$body = @{
    agent_type = "test"
    prompt = "What is the capital of France? Answer in one word."
    tools_allowed = @()
    create_session = $false
} | ConvertTo-Json

Invoke-WebRequest -Uri "http://localhost:8080/spawn" `
    -Method Post `
    -ContentType "application/json" `
    -Body $body
```

**Expected Output:**
```
data: {"type":"output","data":"{\"event\":\"text\",\"text\":\"Paris\"}"}

data: {"type":"completed"}
```

**Success Criteria:**
- ✅ Response contains `"type":"output"` events
- ✅ Response contains Claude's actual answer
- ✅ Response ends with `"type":"completed"`
- ✅ Server logs show:
  - `📥 First line from Claude stdout`
  - `📊 Stdout reader finished - X lines read` (X > 0)
  - `✅ Process ... exited successfully`
  - NO warning: `⚠️ Stdout reader finished with ZERO lines`

---

### Test 4: Agent with Tools (Read/Write Files)

**Objective:** Test process spawning with tool restrictions.

**Test:**
```powershell
# Create a test file
"Test content" | Out-File -FilePath ".\test.txt" -Encoding utf8

# Send request with tools
$body = @{
    agent_type = "file_agent"
    prompt = "Read the file test.txt and tell me what it contains"
    tools_allowed = @("read_file")
    create_session = $false
} | ConvertTo-Json

Invoke-WebRequest -Uri "http://localhost:8080/spawn" `
    -Method Post `
    -ContentType "application/json" `
    -Body $body
```

**Expected Output:**
```
data: {"type":"output","data":"{\"event\":\"tool_use\",\"name\":\"read_file\",\"input\":{\"path\":\"test.txt\"}}"}

data: {"type":"output","data":"{\"event\":\"text\",\"text\":\"The file contains: Test content\"}"}

data: {"type":"completed"}
```

**Success Criteria:**
- ✅ Claude uses the `read_file` tool
- ✅ Output includes tool_use event
- ✅ Output includes file content
- ✅ Process completes successfully

---

### Test 5: Multi-Turn Session

**Objective:** Test session creation and resumption.

**Test:**
```powershell
# First message - create session
$body = @{
    agent_type = "conversation"
    prompt = "My favorite color is blue. Remember this."
    tools_allowed = @()
    create_session = $true
} | ConvertTo-Json

$response = Invoke-WebRequest -Uri "http://localhost:8080/spawn" `
    -Method Post `
    -ContentType "application/json" `
    -Body $body

# Extract session_id from response
# (Parse SSE manually or use a script)

# Second message - resume session
$sessionId = "SESSION_ID_FROM_ABOVE"
$body2 = @{
    prompt = "What is my favorite color?"
    tools_allowed = @()
} | ConvertTo-Json

Invoke-WebRequest -Uri "http://localhost:8080/message/$sessionId" `
    -Method Post `
    -ContentType "application/json" `
    -Body $body2
```

**Expected Output (Second Message):**
```
data: {"type":"output","data":"{\"event\":\"text\",\"text\":\"Your favorite color is blue.\"}"}

data: {"type":"completed"}
```

**Success Criteria:**
- ✅ First request creates session and returns `session_id`
- ✅ Session file created in `.\sessions\` directory
- ✅ Second request resumes session successfully
- ✅ Claude remembers context from first message
- ✅ Both requests produce output (not just `completed`)

---

### Test 6: Error Handling

**Objective:** Verify errors are properly logged and reported.

**Test:**
```powershell
# Test with invalid Claude path
.\q9gent-windows-x86_64.exe --claude-path "C:\NonExistent\claude.cmd"

# Then send a request (server should fail gracefully)
```

**Expected Behavior:**
- ✅ Server logs warning: `❌ Failed to spawn Claude process`
- ✅ Client receives error response (not silent failure)
- ✅ Error message is descriptive

---

### Test 7: Stress Test (Multiple Concurrent Requests)

**Objective:** Verify Windows process spawning works under load.

**Test:**
```powershell
# Send 5 concurrent requests
$jobs = 1..5 | ForEach-Object {
    Start-Job -ScriptBlock {
        param($i)
        $body = @{
            agent_type = "stress_test"
            prompt = "Count to 3"
            tools_allowed = @()
            create_session = $false
        } | ConvertTo-Json
        
        Invoke-WebRequest -Uri "http://localhost:8080/spawn" `
            -Method Post `
            -ContentType "application/json" `
            -Body $body
    } -ArgumentList $_
}

# Wait for all jobs
$jobs | Wait-Job | Receive-Job
```

**Success Criteria:**
- ✅ All 5 requests complete successfully
- ✅ All 5 requests produce output (not silent failures)
- ✅ Server logs show 5 successful process spawns and exits
- ✅ No resource leaks (check Task Manager for zombie processes)

---

## Regression Test: Verify Old Bug is Fixed

**The Original Bug:**
- Symptom: Server returns only `data: {"type":"completed"}` with no output
- Cause: `.cmd` files not wrapped in `cmd.exe /c`

**Regression Test:**
```powershell
# 1. Ensure using .cmd file (not .exe)
$claudePath = (where.exe claude)
if (-not ($claudePath -match "\.cmd$")) {
    Write-Warning "Not using .cmd file - test may not be valid"
}

# 2. Start server
$env:RUST_LOG = "q9gent=info"
.\q9gent-windows-x86_64.exe --claude-path $claudePath

# 3. Send simple request
$body = @{
    agent_type = "regression_test"
    prompt = "Say hello"
    tools_allowed = @()
    create_session = $false
} | ConvertTo-Json

$response = Invoke-WebRequest -Uri "http://localhost:8080/spawn" `
    -Method Post `
    -ContentType "application/json" `
    -Body $body

# 4. Verify response contains MORE than just completed
$responseText = $response.Content

if ($responseText -match 'data: {"type":"output"') {
    Write-Host "✅ PASS: Output detected - bug is FIXED" -ForegroundColor Green
} else {
    Write-Host "❌ FAIL: Only 'completed' event - bug still present" -ForegroundColor Red
}
```

**Success Criteria:**
- ✅ Response contains `"type":"output"` events
- ✅ Response is NOT only `{"type":"completed"}`
- ✅ Server logs show stdout lines received

---

## Automated Test Script

Save this as `test-windows-fix.ps1`:

```powershell
# test-windows-fix.ps1
# Automated test suite for Windows fix verification

param(
    [string]$Q9gentPath = ".\q9gent-windows-x86_64.exe",
    [string]$ClaudePath = ""
)

Write-Host "🧪 Q9gent Windows Fix Test Suite" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan

# Detect Claude CLI path if not provided
if (-not $ClaudePath) {
    $ClaudePath = (where.exe claude)
    if (-not $ClaudePath) {
        Write-Error "Claude CLI not found. Install with: npm install -g @anthropic/claude-cli"
        exit 1
    }
}

Write-Host "📍 Claude CLI: $ClaudePath" -ForegroundColor Yellow

# Test 1: Check if .cmd file
Write-Host "`n[Test 1] Verifying Claude CLI is .cmd file..." -ForegroundColor White
if ($ClaudePath -match "\.cmd$") {
    Write-Host "✅ PASS: Using .cmd file" -ForegroundColor Green
} else {
    Write-Warning "Claude CLI is not a .cmd file. Fix may not be exercised."
}

# Test 2: Start Q9gent
Write-Host "`n[Test 2] Starting Q9gent server..." -ForegroundColor White
$env:RUST_LOG = "q9gent=info"
$process = Start-Process -FilePath $Q9gentPath `
    -ArgumentList "--claude-path", $ClaudePath `
    -NoNewWindow `
    -PassThru

Start-Sleep -Seconds 2

if ($process.HasExited) {
    Write-Host "❌ FAIL: Q9gent process exited immediately" -ForegroundColor Red
    exit 1
}
Write-Host "✅ PASS: Q9gent is running (PID: $($process.Id))" -ForegroundColor Green

# Test 3: Health check
Write-Host "`n[Test 3] Health check..." -ForegroundColor White
try {
    $health = Invoke-RestMethod -Uri "http://localhost:8080/health"
    if ($health.status -eq "ok") {
        Write-Host "✅ PASS: Health check OK" -ForegroundColor Green
    } else {
        Write-Host "❌ FAIL: Unexpected health response" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ FAIL: Health check failed - $_" -ForegroundColor Red
}

# Test 4: Simple agent spawn
Write-Host "`n[Test 4] Testing agent spawn with output..." -ForegroundColor White
$body = @{
    agent_type = "test"
    prompt = "Say the word 'hello' and nothing else"
    tools_allowed = @()
    create_session = $false
} | ConvertTo-Json

try {
    $response = Invoke-WebRequest -Uri "http://localhost:8080/spawn" `
        -Method Post `
        -ContentType "application/json" `
        -Body $body `
        -TimeoutSec 30
    
    $content = $response.Content
    
    if ($content -match '"type":"output"') {
        Write-Host "✅ PASS: Output events received - FIX IS WORKING" -ForegroundColor Green
    } else {
        Write-Host "❌ FAIL: No output events - only 'completed'" -ForegroundColor Red
        Write-Host "Response: $content" -ForegroundColor Yellow
    }
} catch {
    Write-Host "❌ FAIL: Request failed - $_" -ForegroundColor Red
}

# Cleanup
Write-Host "`n[Cleanup] Stopping Q9gent..." -ForegroundColor White
Stop-Process -Id $process.Id -Force
Write-Host "✅ Tests complete" -ForegroundColor Cyan
```

**Usage:**
```powershell
.\test-windows-fix.ps1
```

---

## Success Checklist

After running all tests, verify:

- [ ] `claude.cmd` detected and wrapped in `cmd.exe /c`
- [ ] Simple prompts produce output (not just `completed`)
- [ ] Tool-using agents work correctly
- [ ] Multi-turn sessions work
- [ ] Process exit status is logged
- [ ] Errors are properly reported
- [ ] No zombie processes left behind
- [ ] No "zero output lines" warnings for successful requests
- [ ] Concurrent requests work without issues

---

## Reporting Test Results

If any test fails, collect:

1. **Q9gent version:** `.\q9gent-windows-x86_64.exe --version`
2. **Claude CLI path:** `where.exe claude`
3. **Claude CLI version:** `claude --version`
4. **Full server logs:** Run with `$env:RUST_LOG="q9gent=debug"`
5. **Request/response pair:** Include exact request body and full response
6. **Windows version:** `winver` or `systeminfo | findstr /B /C:"OS Name" /C:"OS Version"`

---

## Known Issues (Expected Failures)

These are NOT bugs:

1. **Timeout on very long responses:** SSE may timeout after 60+ seconds (client-side)
2. **Unicode encoding:** Some emoji/special chars may not display correctly in PowerShell 5.1
3. **Antivirus interference:** Some AV software may block process spawning (configure exclusion)

---

**End of Verification Guide**
