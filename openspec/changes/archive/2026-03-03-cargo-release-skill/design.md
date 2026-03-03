## Context

This workspace has 10 crates with independent versioning. Releases require:
1. Identifying which crates have changes since their last release
2. Determining version bump level (patch/minor/major) per crate
3. Releasing in dependency order (leaf crates first)
4. Creating correct tags (`v3.9.0` for main binary, `<crate>-v<version>` for libraries)

Currently this is manual: inspect git history, identify changes, run multiple `cargo release` commands in order.

**Tag conventions:**
- Main binary (`uvm`/`unity-version-manager`): `v<version>` (e.g., `v3.9.0`)
- Library crates: `<crate-name>-v<version>` (e.g., `uvm_install-v0.20.0`)

**Dependency order (bottom-up):**
```
Level 0 (leaf):  unity-types, unity-version, uvm_move_dir, uvm_gc
Level 1:         uvm_live_platform → unity-version
                 uvm_detect → unity-version
Level 2:         uvm_install_graph → unity-version, uvm_live_platform
                 unity-hub → unity-types, unity-version, uvm_live_platform
Level 3:         uvm_install → unity-hub, unity-version, uvm_install_graph, uvm_live_platform, uvm_move_dir
Level 4 (top):   unity-version-manager → unity-hub, unity-version, uvm_detect, uvm_gc, uvm_install, uvm_live_platform
```

## Goals / Non-Goals

**Goals:**
- Automate detection of which crates changed since last release
- Guide user through version bump decisions
- Execute releases in correct dependency order
- Create consistent tags following existing conventions
- Work across Claude, Cursor, Gemini, and Codex

**Non-Goals:**
- Automatic version bump inference from commit messages (user decides)
- Publishing to crates.io (just local release + tags)
- Changelog generation (separate concern)
- CI/CD integration (skill is for local interactive use)

## Decisions

### 1. Change detection via git tags

**Decision**: Compare each crate's directory against its last release tag.

**Rationale**: Tags are the source of truth for releases. By finding the last tag matching `<crate>-v*` or `v*` (for main binary) and checking `git diff --stat <tag> -- <crate-dir>`, we know exactly what changed.

**Algorithm**:
```bash
# For each crate, find its last tag and check for changes
git tag --list "<crate>-v*" --sort=-v:refname | head -1
git diff --stat <last-tag> -- <crate-dir>
```

### 2. AI-assisted version bump selection

**Decision**: Analyze code changes to propose bump level, with user final approval.

**Rationale**: AI can examine diffs and suggest appropriate bump level based on heuristics, but user has final say. Balances automation with human judgment.

**Analysis heuristics**:

```bash
# Get diff for analysis
git diff <last-tag> -- <crate-dir>/ | head -200
```

**MAJOR (breaking) indicators:**
- Removed public items: `- pub fn`, `- pub struct`, `- pub enum`
- Changed public signatures: `pub fn foo(old)` → `pub fn foo(new)`
- Removed public fields from structs
- Keywords: "BREAKING", "breaking change" in commits/comments

**MINOR (feature) indicators:**
- Added public items: `+ pub fn`, `+ pub struct`, `+ pub enum`
- New public fields/methods
- Commit message tags: `[ADD]`

**PATCH (fix) indicators:**
- Implementation-only changes (function bodies without signature changes)
- Private/internal-only modifications
- Test-only changes
- Commit message tags: `[FIX]`

**Presentation**:
- Show suggested bump level with reasoning
- Mark recommended option in selection prompt
- Allow user to override with any bump level

**Alternatives considered**:
- Pure conventional commits: Too rigid, doesn't analyze actual code
- User-only decision: Tedious for obvious cases (patch-only changes)
- Automatic bumping: Too risky, needs human verification

### 3. Dependency-ordered release execution

**Decision**: Topologically sort crates by internal dependencies, release leaf crates first.

**Rationale**: `cargo release` with `dependent-version = "upgrade"` (already in `uvm/release.toml`) handles version updates in dependents, but we need to release in order so published versions exist when dependents build.

**Algorithm**:
1. Parse `cargo metadata` for internal dependencies
2. Build DAG of workspace crates
3. Topological sort (Kahn's algorithm or similar)
4. Release in sorted order, skipping unchanged crates

### 4. Tag format per crate type

**Decision**: Use existing conventions:
- Main binary: `v<version>` via `tag-prefix = ""` in release.toml
- Libraries: `<crate-name>-v<version>` via `tag-prefix = "<crate-name>-"` in release.toml

**Action**: Add `release.toml` to each crate that lacks one, with appropriate `tag-prefix`.

### 5. Single-commit release strategy

**Decision**: All version bumps in one commit, multiple tags on that commit.

**Workflow**:
1. **Analysis phase**: Detect changed crates, show what changed
2. **Planning phase**: User selects bump level per crate
3. **Preparation phase**: Update all Cargo.toml versions (using `cargo set-version` or manual edits)
4. **Commit phase**: Create single commit with descriptive message
5. **Tagging phase**: Create multiple tags on that commit (one per released crate)
6. **Publish phase** (optional): Run `cargo publish -p <crate>` for each in dependency order

**Commit message format**:

For 1-2 crates:
```
Release <crate1> v<version>, <crate2> v<version>
```

For 3+ crates:
```
Release <crate1> v<version>, <crate2> v<version>, <crate3> v<version>

- <crate1> v<version>
- <crate2> v<version>
- <crate3> v<version>
```

**Rationale**:
- Keeps git history clean with one release commit instead of N commits
- Descriptive title shows what's being released at a glance
- No "chore:" prefix avoids cluttering GitHub commit lists
- Single-line for small releases, title + body for many crates
- Matches existing pattern (see commit `a4d22cf` with 5 tags)

**Alternatives considered**:
- "chore: Release" prefix: Clutters GitHub with repetitive "chore:" commits
- Version-only message: Doesn't show which crates were released
- Standard cargo-release workflow: Creates one commit per crate, clutters history

### 6. Multi-tool skill deployment

**Decision**: Single SKILL.md content, adapted to each tool's format:
- Claude: `.claude/skills/cargo-release/SKILL.md` + command wrapper
- Cursor: `.cursor/skills/cargo-release/SKILL.md` + command wrapper
- Gemini: `.gemini/skills/cargo-release/SKILL.md` + TOML command
- Codex: `.codex/skills/cargo-release/SKILL.md`

**Rationale**: Maintain one source of truth for the skill logic, only format differs.

### 7. Dependency re-export analysis

**Decision**: Analyze whether dependencies are re-exported to determine cascade impact.

**Algorithm**:

For each crate being released with changed dependencies:

```bash
# Check for public re-exports of the dependency
grep -r "pub use <dep_crate>" <crate-dir>/src/

# Example: Check if uvm_live_platform re-exports unity-version
grep -r "pub use unity_version" uvm_live_platform/src/
```

**Impact rules**:

| Dependency Change | Re-export Status | Suggested Bump |
|-------------------|------------------|----------------|
| Major (breaking)  | Re-exported publicly | Major |
| Major (breaking)  | Internal-only | Minor |
| Minor (feature)   | Re-exported publicly | Minor |
| Minor (feature)   | Internal-only | Patch |
| Patch (fix)       | Any | Patch |

**Rationale**: A breaking change in a dependency only affects the public API if that dependency is re-exported. Internal-only usage means the breaking change is contained.

**Example**:
```rust
// uvm_live_platform/src/lib.rs
pub use unity_version::Version; // RE-EXPORTED
use unity_version::parse;        // INTERNAL ONLY

// If unity_version has breaking change to Version:
// → uvm_live_platform needs MAJOR bump (public API affected)

// If unity_version has breaking change to parse:
// → uvm_live_platform needs MINOR bump (internal change only)
```

**Implementation**:
1. Detect changed dependencies from version bumps
2. Search dependent crates for `pub use <dep>::*` patterns
3. Categorize: re-exported vs internal-only
4. Suggest bump level based on impact rules
5. Show reasoning to user

### 8. Dirty working directory handling

**Decision**: Allow releases in dirty repos with warning, selectively stage only Cargo.toml files.

**Workflow**:
1. Check working directory state: `git status --porcelain`
2. If dirty files exist:
   - Warn user about uncommitted changes
   - Show list of dirty files
   - Ask for confirmation to proceed
3. When committing:
   - Only stage modified Cargo.toml files: `git add */Cargo.toml Cargo.toml`
   - Do not stage other dirty files
   - Verify staged changes before commit

**Rationale**:
- Developers often have work-in-progress changes
- Blocking releases on clean repo is too restrictive
- Selective staging prevents accidentally committing unrelated work
- Warning ensures user is aware of dirty state

**Alternatives considered**:
- Require clean repo: Too restrictive for real workflows
- Use `git add -A`: Dangerous, commits everything including WIP
- Automatic stash: Complex, risk of losing work

### 9. Publishing workflow

**Decision**: Optional publish phase with validation and dependency-ordered execution.

**Workflow**:
1. After tagging, ask user if they want to publish
2. For each crate (in dependency order):
   - Run `cargo package --list -p <crate>` to validate
   - Show files that will be published
   - Run `cargo publish -p <crate>`
   - Wait for publish to complete before next crate
3. Handle errors gracefully

**Validation steps**:
```bash
# Validate package contents
cargo package --list -p uvm_install

# Dry-run publish (optional)
cargo publish --dry-run -p uvm_install

# Actual publish
cargo publish -p uvm_install
```

**Rationale**:
- Dependency order ensures published crates can reference each other
- Validation catches packaging issues before publishing
- Optional publish allows local-only releases
- Sequential publishing avoids race conditions

**Alternatives considered**:
- Parallel publishing: Risks failures due to dependency timing
- Always publish: Not flexible for private workflows
- Skip validation: Higher risk of publishing broken packages

## Risks / Trade-offs

**[Risk] Tag parsing edge cases**: Some old tags don't follow conventions (e.g., `uvm-install2-v*`) → Mitigation: Match current crate names only, ignore legacy tags.

**[Risk] Dependency version conflicts**: Bumping a leaf crate might require bumping dependents → Mitigation: Show cascading effects in planning phase, let user adjust.

**[Risk] Partial release failure**: Release might fail mid-way → Mitigation: Show clear state after each step, allow resuming from any point.

**[Trade-off] Manual bump selection**: Slower than automatic, but more accurate. Acceptable for infrequent releases.

**[Trade-off] No crates.io publish**: Keeps skill simple, publish is a separate CI step. Can add later if needed.
