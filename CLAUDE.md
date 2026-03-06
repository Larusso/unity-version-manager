# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Build
cargo build --workspace           # Build all crates
cargo build -p uvm                # Build main binary only
cargo build --release --workspace # Release build

# Test
cargo test --workspace            # Run all tests
cargo test -p <crate>             # Test specific crate
cargo test --workspace -- --nocapture  # Tests with output

# Lint & Format
cargo fmt                         # Format code
cargo clippy --workspace          # Run linter

# Run
cargo run --bin uvm -- <command>  # Run development binary
cargo install --path ./uvm        # Install locally
```

## Architecture

This is a Cargo workspace with the main binary (`uvm`) and supporting library crates.

### Crate Dependency Hierarchy

```
uvm (CLI binary)
 ├── uvm_install (installation logic)
 │    ├── uvm_install_graph (dependency resolution)
 │    └── uvm_live_platform (Unity release API)
 ├── uvm_detect (project detection)
 ├── uvm_gc (garbage collection)
 ├── unity-hub (Hub integration, paths)
 │    └── unity-version (version parsing)
 │         └── unity-types (base types)
 └── uvm_move_dir (cross-platform directory ops)
```

Lower-level crates don't depend on higher-level ones.

### Key Entry Points

- **CLI commands**: `uvm/src/commands/` - each subcommand (install, list, launch, etc.)
- **Version parsing**: `unity-version/src/` - Unity version string handling
- **Installation**: `uvm_install/src/` - core install/uninstall logic
- **Hub integration**: `unity-hub/src/` - Unity Hub paths and installations

### Deep Dives

- [Installer Architecture](docs/installer-architecture.md) - execution flow, platform-specific logic, external dependencies

## Code Conventions

- **Rust Editions**: mixed (2018, 2021, 2024); check each crate's `Cargo.toml` for its edition (e.g., `unity-version` is 2021; `uvm_detect`/`uvm_gc` are 2024)
- **Error handling**: `anyhow::Result` for application errors, `thiserror` for library errors
- **CLI parsing**: `clap` v4 with derive macros
- **Logging**: `log` crate with `flexi_logger`
- Use workspace dependencies from root `Cargo.toml` - don't duplicate versions

## Commit Messages

Use imperative mood: "Fix bug" not "Fixed bug" or "Fixes bug"

Do NOT add Co-Authored-By lines.

```
Short summary (50 chars or less)

## Description

Detailed explanation of the change.

## Changes

* ![ADD] new capability or file
* ![FIX] bug or broken behaviour
* ![UPDATE] changed existing behaviour
* ![REMOVE] deleted capability or file
```

Use `![ADD]`, `![FIX]`, `![UPDATE]`, or `![REMOVE]` icon tags for each change entry.

## Pull Requests

Use the PR template at `.github/PULL_REQUEST_TEMPLATE.md`. Fill out:
- Description of changes
- Review checklist items
- Change log with icons

When updating branches: `git pull origin master --rebase`
