---
name: cargo-release
description: Automate multi-crate Rust workspace releases with single-commit tagging strategy
license: MIT
metadata:
  author: larusso
  version: "1.0"
---

Automate releasing multiple crates in this Rust workspace. Detects changed crates, guides version bumping, and creates a single commit with multiple tags following the project's established pattern.

## Usage

Invoke with `/release` or by asking "release the changed crates".

## Workflow Phases

The skill follows a multi-phase workflow to ensure safe, reviewed releases:

### 0. Pre-Check Phase

Verify working directory state and warn if dirty.

```bash
# Check for uncommitted changes
git status --porcelain
```

**If dirty files exist:**
```
⚠️  Working Directory Status

Uncommitted changes detected:
 M uvm_install/src/lib.rs
 M uvm/src/main.rs
?? new-feature.txt

These files will NOT be included in the release commit.
Only Cargo.toml version changes will be committed.

Continue with release? (yes/no)
```

**If clean:** Proceed without warning.

### 1. Analysis Phase

Detect which workspace crates have changed since their last release.

**Algorithm:**
```bash
# For each crate, find last tag and check for changes
# Main binary (uvm): look for v* tags
git tag --list "v*" --sort=-v:refname | head -1

# Library crates: look for <crate>-v* tags
git tag --list "uvm_install-v*" --sort=-v:refname | head -1
git tag --list "unity-version-v*" --sort=-v:refname | head -1
# ... etc for each crate

# Check if files changed since tag
git diff --stat <last-tag> -- <crate-dir>/
```

**Output to user:**
```
## Changed Crates Since Last Release

1. uvm_install (v0.20.0 → ?)
   - 5 files changed since uvm_install-v0.20.0
   - lib.rs, error.rs, +3 more

2. unity-hub (v0.6.0 → ?)
   - 2 files changed since unity-hub-v0.6.0
   - Cargo.toml, src/lib.rs

3. unity-version-manager (v3.9.0 → ?)
   - 1 file changed since v3.9.0
   - Cargo.toml (dependency updates)

Unchanged: unity-types, unity-version, uvm_move_dir, uvm_gc, uvm_live_platform, uvm_detect, uvm_install_graph
```

### 2. Planning Phase

Analyze code changes and dependency usage to propose bump level, then user selects final version.

**Step 1: Analyze direct code changes**

For each changed crate, examine the diff:

```bash
git diff <last-tag> -- <crate-dir>/ | head -200
```

**Apply heuristics:**

- **MAJOR (breaking)** if:
  - Removed public items: `- pub fn`, `- pub struct`, `- pub enum`
  - Changed public signatures
  - Removed public fields
  - "BREAKING" in commits/comments

- **MINOR (feature)** if:
  - Added public items: `+ pub fn`, `+ pub struct`
  - New public fields/methods
  - `[ADD]` in commit messages

- **PATCH (fix)** if:
  - Implementation-only changes
  - Private/internal changes only
  - `[FIX]` in commit messages

**Step 2: Analyze dependency impact**

For crates that depend on other crates being released, check if breaking changes propagate:

```bash
# Check for re-exports
grep -r "pub use <dep_crate>" <crate-dir>/src/
```

**Impact rules:**

| Dependency Change | Re-export? | Suggested Bump |
|-------------------|------------|----------------|
| Major (breaking)  | Yes        | Major          |
| Major (breaking)  | No         | Minor          |
| Minor (feature)   | Yes        | Minor          |
| Minor (feature)   | No         | Patch          |
| Patch (fix)       | Any        | Patch          |

**Example:**
```
unity-version: 0.3.1 → 0.4.0 (minor - new feature)
↓
uvm_live_platform depends on unity-version
- Searching for re-exports: grep -r "pub use unity_version" uvm_live_platform/src/
- Found: pub use unity_version::Version;
- Impact: Re-exported publicly
- Suggested: MINOR (dependency feature exposed in public API)

uvm_detect depends on unity-version
- Searching for re-exports: grep -r "pub use unity_version" uvm_detect/src/
- Found: (none - internal use only)
- Impact: Internal dependency update
- Suggested: PATCH (internal dependency update)
```

**Step 3: Show analysis and prompt user**

```
## uvm_install (v0.20.0 → ?)

Direct changes:
- Added public function: install_modules_with_installer
- Added public function: write_modules_json

Suggested: MINOR (new public API)

Version bump?
  ○ minor (new features) - Recommended
  ○ patch (bug fixes)
  ○ major (breaking changes)
  ○ skip

## unity-hub (v0.6.0 → ?)

Dependency changes:
- Depends on unity-version (0.3.1 → 0.4.0, minor bump)
- Re-exports unity-version publicly: pub use unity_version::Version

Suggested: MINOR (dependency feature exposed in public API)

Version bump?
  ○ minor (new features) - Recommended
  ○ patch (bug fixes)
  ○ major (breaking changes)
  ○ skip
```

**After selections, compute dependency order:**

Use `cargo metadata` to build dependency graph:
```bash
cargo metadata --no-deps --format-version 1 | jq '.packages[] | {name: .name, deps: [.dependencies[] | select(.path) | .name]}'
```

Topological sort (leaf crates first):
```
Level 0 (leaf):  unity-types, unity-version, uvm_move_dir, uvm_gc
Level 1:         uvm_live_platform, uvm_detect
Level 2:         uvm_install_graph, unity-hub
Level 3:         uvm_install
Level 4 (top):   unity-version-manager
```

**Show release plan:**
```
## Release Plan

Will release in dependency order:

1. uvm_install: 0.20.0 → 0.21.0 (minor)
2. unity-hub: 0.6.0 → 0.7.0 (minor)
3. unity-version-manager: 3.9.0 → 3.10.0 (minor)

Single commit with 3 tags:
- uvm_install-v0.21.0
- unity-hub-v0.7.0
- v3.10.0

Proceed? (yes/no)
```

### 3. Preparation Phase

Update Cargo.toml versions for all releasing crates.

**Use cargo-edit for version bumping:**
```bash
# For each crate being released
cargo set-version --package uvm_install 0.21.0
cargo set-version --package unity-hub 0.7.0
cargo set-version --package unity-version-manager 3.10.0
```

**Also update internal dependencies:**

If a crate being released is depended on by another crate being released, update the dependency version:
```toml
# In unity-version-manager/Cargo.toml
[dependencies]
uvm_install = { version = "0.21.0", path = "../uvm_install" }
unity-hub = { version = "0.7.0", path = "../unity-hub" }
```

**Verify with:**
```bash
cargo check --workspace
```

### 4. Commit Phase

Create single commit with descriptive message, staging only Cargo.toml version changes.

**Selectively stage only version changes:**
```bash
# Stage only Cargo.toml files (avoid staging other dirty files)
git add */Cargo.toml Cargo.toml

# Verify what's staged
git diff --cached --stat

# Commit with appropriate message
```

**For 1-2 crates:**
```bash
git commit -m "Release uvm_install v0.21.0, uvm v3.9.1"
```

**For 3+ crates:**
```bash
git commit -m "$(cat <<'EOF'
Release uvm_install v0.21.0, unity-hub v0.7.0, uvm v3.10.0

- uvm_install v0.21.0
- unity-hub v0.7.0
- uvm v3.10.0
EOF
)"
```

**Important:** Using selective staging (`git add */Cargo.toml`) instead of `git add -A` prevents accidentally committing unrelated work-in-progress changes.

### 5. Tagging Phase

Create multiple tags on the single release commit.

**Tag naming conventions:**
- Main binary (`unity-version-manager`): `v<version>` (e.g., `v3.10.0`)
- Library crates: `<crate-name>-v<version>` (e.g., `uvm_install-v0.21.0`)

```bash
# Create tags
git tag v3.10.0
git tag uvm_install-v0.21.0
git tag unity-hub-v0.7.0

# Verify
git log --oneline --decorate -1
# Should show: <hash> (tag: v3.10.0, tag: uvm_install-v0.21.0, tag: unity-hub-v0.7.0) chore: Release
```

**Pattern matches:** Commit `a4d22cf` with tags `tag: v3.9.0, tag: uvm_live_platform-v0.8.0, tag: uvm_install_graph-v0.14.0, tag: uvm_install-v0.20.0, tag: unity-hub-v0.6.0`

### 6. Publish Phase (Optional)

Optionally validate and publish crates to crates.io in dependency order.

**Ask user:**
```
Publish to crates.io?
  ○ Yes, validate and publish
  ○ No, skip publishing
```

**If user chooses to publish:**

For each crate in dependency order:

1. **Validate package contents:**
```bash
cargo package --list -p uvm_install
```

Show output to user:
```
uvm_install v0.21.0 will include:
- Cargo.toml
- src/lib.rs
- src/error.rs
- ... (list continues)

Total: 15 files
```

2. **Publish:**
```bash
cargo publish -p uvm_install
```

Wait for publish to complete (check crates.io or wait 30s).

3. **Repeat for next crate:**
```bash
cargo package --list -p unity-version-manager
cargo publish -p unity-version-manager
```

**Error handling:**
- If publish fails: Show error, ask to retry/skip/abort
- If validation fails: Show error, don't attempt publish

**Important:** Must publish in dependency order so dependents can reference the newly published versions.

### 7. Push Phase (Optional)

Push commit and tags to remote.

```bash
# Ask user before pushing
git push origin <branch-name>
git push --tags origin
```

## Dry-Run Mode

When user requests dry-run, show all actions without executing:

```
## Dry-Run: Release Preview

### Version Changes
- uvm_install: 0.20.0 → 0.21.0
- unity-hub: 0.6.0 → 0.7.0
- unity-version-manager: 3.9.0 → 3.10.0

### Commands (not executed):
1. cargo set-version --package uvm_install 0.21.0
2. cargo set-version --package unity-hub 0.7.0
3. cargo set-version --package unity-version-manager 3.10.0
4. cargo check --workspace
5. git add */Cargo.toml Cargo.toml
6. git commit -m "Release uvm_install v0.21.0, unity-hub v0.7.0, uvm v3.10.0"
7. git tag -a uvm_install-v0.21.0 -m "uvm_install v0.21.0"
8. git tag -a unity-hub-v0.7.0 -m "unity-hub v0.7.0"
9. git tag -a v3.10.0 -m "uvm v3.10.0"

### Optional Publish (if requested):
10. cargo package --list -p uvm_install
11. cargo publish -p uvm_install
12. cargo package --list -p unity-hub
13. cargo publish -p unity-hub
14. cargo package --list -p unity-version-manager
15. cargo publish -p unity-version-manager

No changes made. Run without --dry-run to execute.
```

## Workspace Crate Reference

Current workspace structure (for reference):

| Crate | Current Version | Tag Pattern |
|-------|----------------|-------------|
| unity-version-manager (main binary) | 3.9.0 | `v<version>` |
| unity-types | 0.1.2 | `unity-types-v<version>` |
| unity-version | 0.3.1 | `unity-version-v<version>` |
| unity-hub | 0.6.0 | `unity-hub-v<version>` |
| uvm_live_platform | 0.8.0 | `uvm_live_platform-v<version>` |
| uvm_detect | 1.1.1 | `uvm_detect-v<version>` |
| uvm_gc | 0.2.0 | `uvm_gc-v<version>` |
| uvm_move_dir | 0.2.2 | `uvm_move_dir-v<version>` |
| uvm_install_graph | 0.14.0 | `uvm_install_graph-v<version>` |
| uvm_install | 0.20.0 | `uvm_install-v<version>` |

## Dependencies

Requires `cargo-edit` for `cargo set-version`:
```bash
cargo install cargo-edit
```

## Error Handling

**If tag detection fails:** Warn user that crate has no previous tag, treat as first release (0.1.0).

**If cargo check fails:** Stop before commit, show error, ask user to fix manually.

**If version conflicts:** Warn if bumping a dependency requires bumping dependents.

**If git push fails:** Show error, suggest manual resolution (rebase, force-push, etc.).

## Example Session

```
User: /release