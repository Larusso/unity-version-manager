---
name: uvm-info
description: Query Unity version information using uvm - list installed versions, available modules, detect project versions.
license: MIT
metadata:
  author: larusso
  version: "1.0"
---

Use this skill to query Unity version information without making changes.

## Available Commands

Run these via the development binary or installed `uvm`:

```bash
# List installed Unity versions
cargo run --bin uvm -- list

# Show available modules for a specific version
cargo run --bin uvm -- modules <version>

# Detect Unity version from current project
cargo run --bin uvm -- detect

# Show all available Unity versions from Unity's API
cargo run --bin uvm -- versions
```

## When to Use

- **Exploring what's installed**: Use `list` to see current Unity installations
- **Planning an installation**: Use `modules <version>` to see what modules are available
- **Understanding a project**: Use `detect` to find which Unity version a project needs
- **Finding versions**: Use `versions` to see what's available to install

## Examples

```bash
# What Unity versions are installed?
cargo run --bin uvm -- list

# What modules can I install for Unity 2022.3.0f1?
cargo run --bin uvm -- modules 2022.3.0f1

# What version does this project need?
cargo run --bin uvm -- detect

# What LTS versions are available?
cargo run --bin uvm -- versions --lts
```

## Notes

- These commands are read-only and safe to run
- Use the development binary (`cargo run --bin uvm --`) when working on the codebase
- Output helps understand the current state before making changes
