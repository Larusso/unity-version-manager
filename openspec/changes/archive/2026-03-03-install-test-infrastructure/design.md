## Context

`install_modules_with_installer` is the core function that:
1. Iterates through `InstallGraph` nodes in topological order
2. Calls the installer for each missing module
3. Updates `is_installed` flag based on success/failure
4. Writes `modules.json` after each module

Currently untestable because `InstallGraph` requires a real `Release` from Unity's API.

## Goals / Non-Goals

**Goals:**
- Enable unit testing of `install_modules_with_installer`
- Test error collection and continuation behavior
- Test incremental `modules.json` writes
- Keep test fixtures minimal and maintainable

**Non-Goals:**
- Mocking the entire Unity API
- Testing actual download/install behavior (that's integration testing)
- Changing production code structure significantly

## Decisions

### 1. Create Release from JSON fixtures

**Decision**: Use JSON deserialization to create test `Release` instances.

**Rationale**: `Release`, `Download`, and `Module` all derive `Deserialize`. We can create minimal JSON fixtures that produce valid `InstallGraph` instances without hitting Unity's API.

### 2. Test fixture location

**Decision**: Place fixtures inline in test code as `const` strings or in a `test_fixtures` module.

**Rationale**: Keeps tests self-contained and readable. External fixture files add complexity for minimal benefit.

### 3. Use existing MockModuleInstaller

**Decision**: The `MockModuleInstaller` already exists with the right interface. Wire it up to actual tests.

**Rationale**: No new abstractions needed - just connect existing pieces.

## Risks / Trade-offs

**[Risk] Fixture maintenance**: If `Release` structure changes, fixtures break → Mitigation: Minimal fixtures with only required fields, rely on `#[serde(default)]`.

**[Risk] Test coverage gaps**: Unit tests can't catch all integration issues → Mitigation: These complement, not replace, real installation testing.

## Future Improvements

### Extract test utilities to separate crate

**When to consider**: If test fixtures and mocks grow significantly, or if multiple crates need to share test infrastructure.

**Approach**: Create a `uvm-test-utils` crate with test fixtures and mocks. To avoid circular dependencies, the `ModuleInstaller` trait would need to move to a lower-level crate (e.g., `uvm-install-core` or `uvm_install_graph`), or use a feature flag approach where `uvm_install` exposes test utilities behind a `test-utils` feature.

**Current state**: Test utilities live inline in `uvm_install/src/lib.rs` within `#[cfg(test)]` modules. This is sufficient for the current scope and avoids premature abstraction.
