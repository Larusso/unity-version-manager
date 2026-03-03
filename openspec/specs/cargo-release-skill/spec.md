## ADDED Requirements

### Requirement: Skill detects changed crates since last release

The skill SHALL detect which workspace crates have changes since their last release by comparing against git tags.

#### Scenario: Crate has changes since last tag

- **WHEN** the skill runs analysis
- **AND** crate `uvm_install` has commits since tag `uvm_install-v0.20.0`
- **THEN** the skill reports `uvm_install` as changed
- **AND** shows the files modified

#### Scenario: Crate has no changes since last tag

- **WHEN** the skill runs analysis
- **AND** crate `uvm_gc` has no commits since tag `uvm_gc-v0.2.0`
- **THEN** the skill does not include `uvm_gc` in the changed list

#### Scenario: Crate has never been tagged

- **WHEN** the skill runs analysis
- **AND** a crate has no matching tag
- **THEN** the skill reports it as "new/untagged"
- **AND** includes it in the release candidates

### Requirement: Skill presents release plan for user approval

The skill SHALL present a release plan showing changed crates and their proposed release order, allowing user to select version bump level.

#### Scenario: User selects bump levels

- **WHEN** the skill shows changed crates [uvm_install, unity-hub]
- **THEN** user can select bump level (patch/minor/major/skip) for each
- **AND** the skill shows the resulting new versions

#### Scenario: User skips a crate

- **WHEN** user selects "skip" for a changed crate
- **THEN** that crate is excluded from the release
- **AND** dependent crates are warned if they depend on skipped crate changes

### Requirement: Skill analyzes code changes to propose bump level

The skill SHALL analyze code changes for each crate and suggest an appropriate version bump level based on heuristics.

#### Scenario: Breaking change detected

- **WHEN** analyzing `uvm_install` changes since last tag
- **AND** git diff shows removed public API (`pub fn foo` deleted)
- **THEN** the skill suggests "major (breaking changes)" as recommended option

#### Scenario: New features detected

- **WHEN** analyzing `uvm_install` changes since last tag
- **AND** git diff shows new public functions added
- **THEN** the skill suggests "minor (new features)" as recommended option

#### Scenario: Bug fixes detected

- **WHEN** analyzing `uvm_gc` changes since last tag
- **AND** git diff shows only implementation changes without API modifications
- **THEN** the skill suggests "patch (bug fixes)" as recommended option

#### Scenario: Ambiguous changes default to user choice

- **WHEN** analyzing changes that don't clearly match heuristics
- **THEN** the skill presents options without a recommendation
- **AND** shows summary of changes for user to decide

### Requirement: Skill computes correct release order from dependencies

The skill SHALL determine release order using topological sort of workspace dependencies, releasing leaf crates before their dependents.

#### Scenario: Release order respects dependencies

- **WHEN** the release plan includes `unity-version` and `uvm_live_platform`
- **AND** `uvm_live_platform` depends on `unity-version`
- **THEN** `unity-version` is scheduled before `uvm_live_platform`

#### Scenario: Independent crates can release in any order

- **WHEN** the release plan includes `uvm_gc` and `uvm_move_dir`
- **AND** neither depends on the other
- **THEN** either order is acceptable

### Requirement: Skill creates single commit with descriptive message

The skill SHALL update all Cargo.toml versions and create a single commit with a message listing all released crates, then apply multiple tags to that commit.

#### Scenario: Single commit with multiple version bumps

- **WHEN** releasing 3 crates: `unity-version`, `uvm_live_platform`, `uvm_install`
- **THEN** the skill updates all 3 Cargo.toml files
- **AND** creates one commit with message listing the releases
- **AND** applies 3 tags to that commit: `unity-version-v0.4.0`, `uvm_live_platform-v0.9.0`, `uvm_install-v0.21.0`

#### Scenario: Commit message format for few crates

- **WHEN** creating release commit for 1-2 crates
- **THEN** commit message lists all releases in title:
  ```
  Release uvm_install v0.21.0, uvm v3.9.1
  ```

#### Scenario: Commit message format for many crates

- **WHEN** creating release commit for 3+ crates
- **THEN** commit message uses multi-line format:
  ```
  Release uvm_install v0.21.0, unity-hub v0.7.0, uvm v3.10.0

  - uvm_install v0.21.0
  - unity-hub v0.7.0
  - uvm v3.10.0
  ```

### Requirement: Skill creates tags with correct naming convention

The skill SHALL create tags following the established pattern for main binary vs library crates.

#### Scenario: Main binary uses v-prefix tags

- **WHEN** releasing `unity-version-manager` (uvm) version 3.10.0
- **THEN** the skill creates tag `v3.10.0` (no crate name prefix)

#### Scenario: Library crates use name-prefixed tags

- **WHEN** releasing library crate `uvm_install` version 0.21.0
- **THEN** the skill creates tag `uvm_install-v0.21.0` (with crate name prefix)

### Requirement: Skill provides dry-run mode

The skill SHALL support a dry-run mode that shows what would happen without making changes.

#### Scenario: Dry-run shows planned actions

- **WHEN** user requests dry-run
- **THEN** the skill shows all commands that would be executed
- **AND** shows version changes and tags that would be created
- **AND** does not execute any commands or create any tags

### Requirement: Skill is available across all AI tools

The skill SHALL be deployed to all AI tool directories with appropriate format for each.

#### Scenario: Claude Code skill available

- **WHEN** user invokes `/release` in Claude Code
- **THEN** the skill executes from `.claude/skills/cargo-release/SKILL.md`

#### Scenario: Cursor skill available

- **WHEN** user invokes the release command in Cursor
- **THEN** the skill executes from `.cursor/skills/cargo-release/SKILL.md`

#### Scenario: Gemini skill available

- **WHEN** user invokes the release command in Gemini
- **THEN** the skill executes from `.gemini/skills/cargo-release/SKILL.md`

#### Scenario: Codex skill available

- **WHEN** user invokes the release skill in Codex
- **THEN** the skill executes from `.codex/skills/cargo-release/SKILL.md`

### Requirement: Skill analyzes dependency usage to determine cascade impact

The skill SHALL analyze whether dependencies are re-exported publicly or used internally to determine if breaking changes affect dependent crates.

#### Scenario: Breaking dependency change with public re-export

- **WHEN** `unity-version` has a major bump (breaking change)
- **AND** `uvm_live_platform` depends on `unity-version`
- **AND** `uvm_live_platform` re-exports types from `unity-version` (`pub use unity_version::Version`)
- **THEN** the skill suggests major bump for `uvm_live_platform` (breaking change propagates)

#### Scenario: Breaking dependency change with internal-only usage

- **WHEN** `unity-version` has a major bump (breaking change)
- **AND** `uvm_detect` depends on `unity-version`
- **AND** `uvm_detect` uses `unity-version` internally only (no `pub use`)
- **THEN** the skill suggests minor bump for `uvm_detect` (dependency update, not breaking)

#### Scenario: Dependency cascade warning with analysis

- **WHEN** user bumps `unity-version` to major version
- **THEN** the skill analyzes all dependents
- **AND** shows which crates re-export it (need major bump)
- **AND** shows which crates use it internally (need minor bump)

### Requirement: Skill supports resuming after failure

The skill SHALL allow resuming a release sequence if a step fails.

#### Scenario: Resume after cargo-release failure

- **WHEN** `cargo release` fails for one crate
- **THEN** the skill shows which crates succeeded and which failed
- **AND** provides option to retry failed crate or skip and continue

### Requirement: Skill handles dirty working directory

The skill SHALL allow releases in a dirty working directory but warn the user and only commit version changes.

#### Scenario: Dirty repo with unrelated changes

- **WHEN** the working directory has uncommitted changes to non-Cargo.toml files
- **THEN** the skill warns user about dirty state
- **AND** asks for confirmation to proceed
- **AND** only stages Cargo.toml files for the release commit

#### Scenario: Clean repo

- **WHEN** the working directory is clean
- **THEN** the skill proceeds without warnings
- **AND** stages all Cargo.toml changes

#### Scenario: Selective staging of version changes

- **WHEN** creating the release commit
- **THEN** the skill only stages modified Cargo.toml files
- **AND** does not stage other uncommitted changes in the workspace

### Requirement: Skill validates and publishes crates

The skill SHALL optionally validate and publish crates to crates.io in dependency order.

#### Scenario: Validate before publish

- **WHEN** user chooses to publish crates
- **THEN** the skill runs `cargo package --list` for each crate
- **AND** shows what files will be included
- **AND** asks for confirmation before publishing

#### Scenario: Publish in dependency order

- **WHEN** publishing multiple crates
- **THEN** the skill publishes leaf crates first
- **AND** waits for each publish to complete before publishing dependents
- **AND** handles publish failures gracefully

#### Scenario: Skip publish option

- **WHEN** user chooses not to publish
- **THEN** the skill completes without publishing
- **AND** reminds user they can publish manually later
