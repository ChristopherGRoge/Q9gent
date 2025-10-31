# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2] - 2025-10-31

### Fixed
- **CRITICAL**: Windows EPIPE (broken pipe) errors when Claude CLI process runs
  - Continue draining stdout even after SSE client disconnects to prevent pipe breakage
  - Set Node.js environment variables to disable stdout buffering on Windows
  - Prevents premature pipe closure that causes Claude CLI to crash mid-execution
  
### Changed
- stdout reader now continues reading to EOF even if channel is closed
- Added Windows-specific Node.js environment variables (NODE_NO_WARNINGS=1)
- Improved debug logging for Windows pipe handling

## [0.1.1] - 2025-10-31

### Fixed
- **CRITICAL**: Windows process spawning for npm-installed Claude CLI
  - Automatically wraps `.cmd` and `.bat` files in `cmd.exe /c` on Windows
  - Fixes issue where Windows binary would spawn processes but produce no output
  - Direct `.exe` files are still executed without wrapper
- Enhanced error logging with detailed process spawn diagnostics
- Process exit status monitoring and reporting
- Warning when zero output lines are received from Claude process

### Added
- Platform-specific process spawning logic (`prepare_platform_command`)
- Comprehensive Windows deployment guide (WINDOWS_DEPLOYMENT.md)
- Windows fix verification guide (WINDOWS_FIX_VERIFICATION.md)
- Technical fix summary (WINDOWS_FIX_SUMMARY.md)
- Implementation summary documentation (IMPLEMENTATION_SUMMARY.md)
- Additional unit tests for Windows `.cmd`/`.bat` wrapper detection
- Process exit status logging for debugging
- Explicit warnings when processes fail silently
- Enhanced documentation for multi-platform Claude CLI detection
- Docker deployment examples with Claude CLI installation

### Changed
- Stderr messages now logged at ERROR level for better visibility
- Enhanced spawn failure messages with full command details
- Improved logging throughout process lifecycle
- README updated with platform-specific deployment notes
- Developer Guide updated with comprehensive Claude CLI path scenarios
- Clarified Docker container deployment with Claude CLI

### Documentation
- Added clear guidance for finding Claude CLI on Windows, macOS, Linux
- Documented auto-discovery vs. explicit path configuration
- Added Docker-specific deployment instructions
- Enhanced troubleshooting section with platform-specific solutions

## [0.1.0] - 2025-10-30

### Added
- Initial release of Q9gent
- Core agent runner with Claude CLI process spawning
- HTTP API with Server-Sent Events (SSE) streaming
- Endpoints: `/spawn`, `/message/:session_id`, `/terminate/:session_id`, `/sessions`, `/health`
- Session metadata persistence for multi-turn conversations
- Stateless request handling by default
- Configurable tool access control per request
- Clean process supervision using Tokio
- CLI argument parsing (host, port, session-dir, claude-path)
- Cross-platform GitHub Actions workflows for Windows, Linux, and macOS
- Comprehensive documentation (README, EXAMPLES, DEVELOPMENT)
- Docker support with multi-stage builds
- Docker Compose configuration
- Makefile for common development tasks
- MIT License

### Features
- Spawn ephemeral Claude CLI processes with precise flags
- Stream JSONL output in real-time via SSE
- Resume previous conversations with session IDs
- Terminate running agents on demand
- List active sessions
- Strict least-privilege tool fences
- Zero hidden orchestration
- Pure Rust implementation

[Unreleased]: https://github.com/ChristopherGRoge/Q9gent/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/ChristopherGRoge/Q9gent/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/ChristopherGRoge/Q9gent/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ChristopherGRoge/Q9gent/releases/tag/v0.1.0
