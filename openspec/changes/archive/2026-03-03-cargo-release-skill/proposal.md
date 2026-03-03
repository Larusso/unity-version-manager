## Why

Releasing this multi-crate workspace requires manually determining which crates changed, what version bump level each needs, the correct release order based on dependencies, and running multiple `cargo release` commands with appropriate tags. This is error-prone and time-consuming, especially when changes cascade through the dependency graph.

## What Changes

- Add a Claude Code skill `/release` that automates workspace release management
- Skill analyzes git changes since last release to identify affected crates
- Skill determines release order from dependency graph (bottom-up)
- Skill prompts for version bump level per crate (or suggests based on conventional commits)
- Skill generates and executes cargo-release commands with correct tag prefixes
- Skill handles the main binary (`uvm`) vs library crate tag conventions

## Capabilities

### New Capabilities

- `cargo-release-skill`: Skill for automating multi-crate Rust workspace releases, including change detection, dependency-ordered release sequencing, and tag management

### Modified Capabilities

None

## Impact

- **Code**: New skill files across all AI tool directories:
  - `.claude/skills/cargo-release/SKILL.md` + `.claude/commands/release.md`
  - `.cursor/skills/cargo-release/SKILL.md` + `.cursor/commands/release.md`
  - `.gemini/skills/cargo-release/SKILL.md` + `.gemini/commands/release.toml`
  - `.codex/skills/cargo-release/SKILL.md`
- **Dependencies**: Requires `cargo-release` to be installed
- **Configuration**: May need per-crate `release.toml` files for tag prefixes
- **Workflow**: Replaces manual release process with guided automation

### Current State Reference

**Workspace crates and versions:**
| Crate | Version | Tag Pattern |
|-------|---------|-------------|
| unity-version-manager (uvm) | 3.9.0 | `v3.9.0` |
| unity-types | 0.1.2 | `unity-types-v0.1.2` |
| unity-version | 0.3.1 | `unity-version-v0.3.1` |
| unity-hub | 0.6.0 | `unity-hub-v0.6.0` |
| uvm_live_platform | 0.8.0 | `uvm_live_platform-v0.8.0` |
| uvm_detect | 1.1.1 | `uvm_detect-v1.1.1` |
| uvm_gc | 0.2.0 | `uvm_gc-v0.2.0` |
| uvm_move_dir | 0.2.2 | `uvm_move_dir-v0.2.2` |
| uvm_install_graph | 0.14.0 | `uvm_install_graph-v0.14.0` |
| uvm_install | 0.20.0 | `uvm_install-v0.20.0` |

**Dependency order (release bottom-up):**
1. Leaf: `unity-types`, `unity-version`, `uvm_move_dir`, `uvm_gc`
2. Mid: `uvm_live_platform`, `uvm_detect`
3. Mid: `uvm_install_graph`, `unity-hub`
4. High: `uvm_install`
5. Top: `unity-version-manager`
