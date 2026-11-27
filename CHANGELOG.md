# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Workspace structure with centralized dependency management
- `rust-toolchain.toml` for consistent Rust version across team
- `.cargo/config.toml` with convenient aliases (`xt`, `test-all`, `clippy-all`, etc.)
- `docs/ARCHITECTURE.md` with current state → target crates mapping

### Changed
- **BREAKING**: Moved `rust-server/` → `crates/server/` for workspace structure
- Updated package name from `brainstorm` to `forgebrain-server` (binary name)
- Library name remains `brainstorm` for backward compatibility
- Updated CI workflow to work with workspace (`--workspace` flags)
- Updated Dockerfile to use workspace context and build specific package
- Updated docker-compose.yml build context to workspace root
- Centralized common dependencies in workspace `Cargo.toml`
- Moved profile configurations to workspace level

### Migration Guide
If you were using the old structure:
```bash
# Old
cd rust-server
cargo build

# New
cd crates/server  # or use workspace root
cargo build --package forgebrain-server
# or from workspace root
cargo build -p forgebrain-server
```

### Fixed
- CI workflow now correctly uses workspace commands
- Docker build context updated for workspace structure

## [0.1.0] - 2024-11-27

### Added
- Initial Rust server implementation
- 8 game modes: classic, ffa, lives, blitz, team, coop, svoyaigra, sandbox
- AI-powered question generation (OpenAI, GigaChat, Ollama)
- Real-time WebSocket multiplayer
- Prometheus metrics and observability
- Graceful shutdown and reconnect improvements
- Integration test suite
- Comprehensive documentation

### Infrastructure
- PostgreSQL for persistence
- Redis support (experimental)
- Docker multi-stage builds
- GitHub Actions CI/CD
