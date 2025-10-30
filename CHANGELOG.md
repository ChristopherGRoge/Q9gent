# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/yourusername/Q9gent/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/Q9gent/releases/tag/v0.1.0
