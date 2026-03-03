## 1. Create core skill file

- [x] 1.1 Create `.claude/skills/cargo-release/SKILL.md` with full skill logic
- [x] 1.2 Document analysis phase (detect changed crates via git tags)
- [x] 1.3 Document planning phase (user selects bump levels, show release order)
- [x] 1.3.1 Add code analysis for proposing bump levels (MAJOR/MINOR/PATCH heuristics)
- [x] 1.3.2 Add dependency re-export analysis to determine cascade impact
- [x] 1.3.3 Document showing analysis reasoning to user before selection
- [x] 1.4 Document preparation phase (use `cargo set-version` to bump all versions)
- [x] 1.5 Document commit phase (single "chore: Release" commit with all bumps)
- [x] 1.6 Document tagging phase (create multiple tags on single commit)
- [x] 1.7 Include dry-run mode instructions
- [x] 1.8 Include optional publish phase (cargo publish per crate)

## 2. Deploy skill to all AI tools

- [x] 2.1 Copy skill to `.cursor/skills/cargo-release/SKILL.md`
- [x] 2.2 Copy skill to `.gemini/skills/cargo-release/SKILL.md`
- [x] 2.3 Copy skill to `.codex/skills/cargo-release/SKILL.md`

## 3. Create command wrappers

- [x] 3.1 Create `.claude/commands/release.md` command wrapper
- [x] 3.2 Create `.cursor/commands/release.md` command wrapper
- [x] 3.3 Create `.gemini/commands/release.toml` command wrapper (TOML format)

## 4. Verification

- [x] 4.1 Test skill detection: verify it finds changed crates correctly
- [x] 4.2 Test dry-run mode: verify version bumps and tags shown without execution
- [x] 4.3 Verify dependency order is correct in release plan
- [x] 4.4 Verify single-commit multi-tag approach matches existing pattern
