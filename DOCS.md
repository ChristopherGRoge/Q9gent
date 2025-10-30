# Q9gent Documentation Index

Welcome to Q9gent - a lightweight Rust CLI Assistant Server for spawning Claude Code Headless processes.

## 📚 Documentation Guide

### For Users

1. **[README.md](README.md)** - Start here!
   - Installation instructions
   - Usage guide
   - API overview
   - Examples
   - Troubleshooting

2. **[QUICKREF.md](QUICKREF.md)** - Quick reference card
   - Common commands
   - Quick examples
   - Endpoint summary
   - Troubleshooting tips

3. **[API.md](API.md)** - Complete API specification
   - Full endpoint documentation
   - Request/response formats
   - Error codes
   - SSE event types

4. **[EXAMPLES.md](EXAMPLES.md)** - Code examples
   - Python client examples
   - JavaScript/Node.js examples
   - curl examples
   - Common use cases

### For Developers

5. **[DEVELOPMENT.md](DEVELOPMENT.md)** - Developer guide
   - Setup instructions
   - Project structure
   - Testing guide
   - Contributing guidelines

6. **[PROJECT_SUMMARY.md](PROJECT_SUMMARY.md)** - Project overview
   - Architecture
   - Technology stack
   - Design principles
   - Project status

7. **[CHANGELOG.md](CHANGELOG.md)** - Version history
   - Release notes
   - Feature additions
   - Breaking changes

### For DevOps

8. **[Dockerfile](Dockerfile)** - Container image definition
   - Multi-stage build
   - Runtime configuration

9. **[docker-compose.yml](docker-compose.yml)** - Container orchestration
   - Service configuration
   - Volume mappings

10. **[Makefile](Makefile)** - Build automation
    - Common tasks
    - Quality checks
    - Cross-compilation

## 🎯 Quick Navigation

### I want to...

- **Install Q9gent** → [README.md#installation](README.md#installation)
- **Start the server** → [README.md#usage](README.md#usage)
- **Use the API** → [API.md](API.md) or [QUICKREF.md](QUICKREF.md)
- **See code examples** → [EXAMPLES.md](EXAMPLES.md)
- **Contribute** → [DEVELOPMENT.md#contributing](DEVELOPMENT.md#contributing)
- **Build from source** → [DEVELOPMENT.md#quick-start](DEVELOPMENT.md#quick-start)
- **Deploy with Docker** → [README.md#docker](README.md#docker)
- **Understand the architecture** → [PROJECT_SUMMARY.md](PROJECT_SUMMARY.md)
- **Check version history** → [CHANGELOG.md](CHANGELOG.md)

## 📖 Reading Order

### New Users
1. README.md
2. QUICKREF.md
3. EXAMPLES.md

### Developers
1. README.md
2. PROJECT_SUMMARY.md
3. DEVELOPMENT.md
4. API.md

### Contributors
1. DEVELOPMENT.md
2. PROJECT_SUMMARY.md
3. Source code in `src/`

## 🔍 Source Code

```
src/
├── main.rs          - Entry point, CLI argument parsing
├── api.rs           - HTTP server and endpoint handlers
├── agent.rs         - Claude process spawning and management
├── session.rs       - Session metadata persistence
├── session/
│   └── tests.rs     - Session storage tests
├── error.rs         - Error types and handling
└── config.rs        - Server configuration
```

## 🛠️ Build & Deploy

```bash
# Quick start
make build          # Build debug
make release        # Build release
make test           # Run tests
make run            # Run server

# See all commands
make help
```

## 🌐 Community

- **Issues:** Report bugs or request features
- **Discussions:** Ask questions or share ideas
- **Pull Requests:** Contribute code improvements

## 📄 License

This project is licensed under the MIT License - see [LICENSE](LICENSE) file.

---

**Q9gent v0.1.0** - Built with ❤️ in Rust
