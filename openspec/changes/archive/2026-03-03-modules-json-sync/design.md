## Context

Currently, `modules.json` is written once at the end of `Installer::install()` (lib.rs:363). The installation flow is:

1. Build dependency graph from Unity release
2. Call `install_module_and_dependencies()` which loops through modules
3. Each module: download → create installer → install
4. After ALL modules succeed, update `is_installed` flags and write `modules.json`

If any module fails in step 3, the function returns an error immediately and `modules.json` is never written. Physically installed modules are not reflected in the state file.

## Goals / Non-Goals

**Goals:**
- Write `modules.json` after each module installation completes (success or failure)
- Maintain accurate installation state even when individual modules fail
- Continue installing remaining modules when one fails (collect errors)

**Non-Goals:**
- Changing the modules.json file format
- Adding retry logic for failed modules
- Parallel module installation (current sequential flow is maintained)

## Decisions

### 1. Incremental Write Strategy

**Decision**: Write `modules.json` after each module completes installation.

**Alternatives considered**:
- Write on error only: Simpler but loses sync on crash
- Write at checkpoints: More complex, unclear what checkpoints mean

**Rationale**: Writing after each module is simple, ensures state is always current, and the performance cost is negligible (one small file write per module).

### 2. Error Handling Approach

**Decision**: Collect all module installation errors and continue with remaining modules.

**Alternatives considered**:
- Fail fast (current behavior): Loses progress information
- Interactive mode asking user to skip/retry: Too complex for CLI tool

**Rationale**: Users can see what succeeded and retry just the failed modules. This matches user expectations for package managers.

### 3. State Initialization

**Decision**: Load or create `modules.json` before the installation loop starts, then mutate and write incrementally.

**Rationale**: The modules list needs to exist before we can update individual module states. Current code already handles this (lines 340-354), we just need to move the write into the loop.

### 4. Installation Reference

**Decision**: Create the `UnityInstallation` handle before the installation loop instead of after.

**Rationale**: We need the installation handle to call `write_modules()` inside the loop. Currently it's created at line 339 after installation completes. Moving it earlier means we need to handle the case where the base directory doesn't exist yet - we can create the directory first if needed.

## Risks / Trade-offs

**[Risk] More disk I/O** → Mitigation: One small JSON write per module is negligible compared to module download/install time.

**[Risk] Partial state on crash during write** → Mitigation: File writes are typically atomic on modern filesystems. Could add write-to-temp-then-rename pattern if needed.

**[Risk] Installation continues despite failures** → Mitigation: Clearly report all failures at the end. Users who want fail-fast can use existing behavior via a flag (future work).
