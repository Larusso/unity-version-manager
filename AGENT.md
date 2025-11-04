# AI Agent Guide

This document provides essential information for AI agents working on the unity-version-manager project.

## Project Overview

**unity-version-manager** (uvm) is a command-line tool written in Rust for managing Unity installations and projects. It allows users to install, uninstall, list, and launch Unity versions from the command line, with compatibility for Unity Hub installations.

### Key Features
- Install/uninstall Unity versions and modules
- List installed Unity versions (Hub, system, or all)
- Launch Unity projects with specific versions
- Detect Unity version from project files
- Query available Unity versions and modules
- Cross-platform support (Windows, macOS, Linux)

## Project Structure

This is a **Cargo workspace** containing multiple crates:

### Main Binary
- **`uvm/`** - Main CLI application that produces the `uvm` binary

### Core Libraries
- **`unity-version/`** - Unity version parsing, validation, and management
- **`unity-types/`** - Base Unity data types and structures
- **`unity-hub/`** - Unity Hub integration and installation path detection

### Feature Modules
- **`uvm_install/`** - Unity installation logic and module management
- **`uvm_live_platform/`** - Unity release platform API integration
- **`uvm_install_graph/`** - Installation dependency graph resolution
- **`uvm_detect/`** - Unity project detection and version extraction
- **`uvm_move_dir/`** - Cross-platform directory operations
- **`uvm_gc/`** - Garbage collection for Unity installations

### Legacy Code
- **`legacy/`** - Contains older implementation code (may be deprecated)

## Development Guidelines

### Commit Messages

**IMPORTANT**: Follow the commit message guidelines in [`.github/CONTRIBUTING.md`](.github/CONTRIBUTING.md).

Key points:
- Use imperative mood: "Fix bug" not "Fixed bug" or "Fixes bug"
- Keep first line short (?50 chars ideally, can go higher if needed)
- Add detailed description after blank line if needed
- Write in imperative: "Add feature" not "Added feature"
- See the contributing guide for full formatting details

### Pull Requests

**IMPORTANT**: Use the pull request template in [`.github/PULL_REQUEST_TEMPLATE.md`](.github/PULL_REQUEST_TEMPLATE.md) when creating PRs.

The template includes:
- Description and feature overview
- Review checklist items
- Change notation with icons
- Follow-up tasks section

Always fill out the PR template completely, especially:
- Description of changes
- Items to review
- Appropriate checklist items
- Change log with proper icons

### Git Workflow

- Use `git pull --rebase` when updating branches (see contributing guide)
- Keep PRs focused on one topic
- Prefer multiple small PRs over one large PR
- Maintain clean git history

## Rust-Specific Guidelines

### Edition
- **Rust Edition**: 2018
- All crates in the workspace use edition 2018

### Code Style
- Follow standard Rust conventions
- Use `rustfmt` for formatting (run `cargo fmt`)
- Use `clippy` for linting (run `cargo clippy`)
- Prefer `Result<T, E>` and `Option<T>` for error handling
- Use `thiserror` and `anyhow` for error types (workspace dependencies)

### Workspace Dependencies

Common dependencies available to all crates:
- `anyhow` - Error handling
- `serde` - Serialization (with derive feature)
- `log` - Logging
- `thiserror` - Error types
- `semver` - Semantic versioning
- `itertools` - Iterator utilities
- `reqwest` - HTTP client (with specific features)
- `console` - Terminal utilities
- `dirs-2` - Directory utilities
- `ssri` - Integrity checking
- `cfg-if` - Conditional compilation
- `serde_json` - JSON support

### Common Patterns

1. **Error Handling**: Use `anyhow::Result` for application-level errors, `thiserror` for library errors
2. **CLI Parsing**: Use `clap` with derive feature (version 4.x)
3. **Logging**: Use `log` crate with `flexi_logger` in main binary
4. **Version Parsing**: Use `unity-version` crate for Unity version strings
5. **Progress**: Use `indicatif` for progress bars
6. **Terminal Output**: Use `console` crate for colored output

## Building and Testing

### Build Commands
```bash
# Build entire workspace
cargo build --workspace

# Build specific crate
cargo build -p uvm

# Build release version
cargo build --release --workspace
```

### Test Commands
```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p uvm

# Run tests with output
cargo test --workspace -- --nocapture
```

### Running Development Version
```bash
# Run binary directly
cargo run --bin uvm -- --help

# Run with specific command
cargo run --bin uvm -- install 2023.1.4f1
```

## Key Entry Points

### Main Application
- **`uvm/src/main.rs`** - CLI entry point
- **`uvm/src/commands/`** - Command implementations
  - `install.rs` - Installation command
  - `launch.rs` - Launch command
  - `list.rs` - List command
  - `version.rs` - Version utilities
  - etc.

### Important Modules

**Unity Version Management:**
- `unity-version/src/` - Version parsing and validation
- `unity-hub/src/` - Hub integration and paths

**Installation:**
- `uvm_install/src/` - Core installation logic
- `uvm_install_graph/` - Dependency resolution
- `uvm_live_platform/` - Unity API integration

**Project Detection:**
- `uvm_detect/src/lib.rs` - Project detection using builder pattern

## Common Tasks

### Adding a New Command
1. Create new file in `uvm/src/commands/`
2. Implement command struct with `clap::Parser`
3. Add command handler function
4. Register in `uvm/src/main.rs` command matching
5. Add tests in `uvm/tests/` or inline tests

### Adding a New Crate
1. Create new directory with `Cargo.toml`
2. Add to workspace `members` in root `Cargo.toml`
3. Add workspace dependencies as needed
4. Update any dependent crates

### Modifying Installation Logic
- Main logic in `uvm_install/src/`
- Graph resolution in `uvm_install_graph/`
- Platform API in `uvm_live_platform/`
- Hub integration in `unity-hub/src/`

### Cross-Platform Considerations
- Use `uvm_move_dir` for directory operations
- Test on multiple platforms (Windows, macOS, Linux)
- Use `cfg-if` for platform-specific code
- Path handling via `unity-hub` for Unity Hub paths

## Testing Strategy

- Unit tests alongside source files (`#[cfg(test)]` modules)
- Integration tests in `uvm/tests/` directory
- Property-based tests in `unity-version` (using proptest)
- Test data in `legacy/uvm-generate-modules-json/` for module generation

## Important Files

- **`Cargo.toml`** - Workspace configuration
- **`uvm/Cargo.toml`** - Main binary configuration
- **`modules.json`** - Unity module definitions
- **`Makefile`** - Build and install scripts
- **`.github/workflows/`** - CI/CD configuration
- **`flake.nix`** - Nix development environment

## Code Organization Principles

1. **Separation of Concerns**: Each crate has a focused responsibility
2. **Dependency Direction**: Lower-level crates don't depend on higher-level ones
   - `unity-types` ? `unity-version` ? `unity-hub`
   - Feature crates depend on core types but not each other
3. **CLI Parsing**: Command definitions use `clap` derive macros
4. **Error Propagation**: Use `?` operator and `anyhow::Context` for error handling
5. **Builder Patterns**: Used in `uvm_detect` for configuration

## Version Management

- Project version: 3.8.0 (in `uvm/Cargo.toml`)
- Individual crates have their own versions
- Version numbers follow semantic versioning
- Unity versions parsed using `unity-version` crate (supports formats like `2023.1.4f1`)

## Additional Resources

- **README.md** - User-facing documentation
- **`.github/CONTRIBUTING.md`** - Contribution guidelines (read before committing)
- **`.github/PULL_REQUEST_TEMPLATE.md`** - PR template (use for all PRs)
- **`.github/workflows/rust.yml`** - CI/CD pipeline configuration

## Notes for AI Agents

1. **Always check** `.github/CONTRIBUTING.md` before making commits
2. **Always use** `.github/PULL_REQUEST_TEMPLATE.md` for PRs
3. **Test changes** with `cargo test --workspace` before committing
4. **Format code** with `cargo fmt` before committing
5. **Check linting** with `cargo clippy` before committing
6. **Respect workspace structure** - don't create unnecessary cross-crate dependencies
7. **Follow Rust idioms** - use `Result`, `Option`, pattern matching appropriately
8. **Consider cross-platform** - test assumptions about paths, file systems, etc.
9. **Use workspace dependencies** - don't duplicate dependency versions
10. **Document public APIs** - add doc comments for public functions/types

## Quick Reference

```bash
# Setup
cargo build --workspace

# Test
cargo test --workspace

# Format
cargo fmt

# Lint
cargo clippy --workspace

# Run
cargo run --bin uvm -- <command>

# Install locally
cargo install --path ./uvm
```
