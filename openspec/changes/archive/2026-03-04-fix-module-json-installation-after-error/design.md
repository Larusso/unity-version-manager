## Context

The current installation flow in `uvm_install/src/lib.rs` iterates through components in topological order (Editor first, then modules) and continues past failures, collecting errors for final reporting. This resilient behavior is valuable when adding modules to an existing Editor, but problematic for fresh installs: if the Editor fails, modules install anyway, leaving a broken partial installation that breaks version detection and requires manual cleanup.

The recent change (commit 6d4c7b5) to write `modules.json` incrementally after each module exacerbated this issue by persisting state even when the Editor never successfully installed.

**Current flow:**
1. Create base directory (`/path/to/Unity-X.Y.Z/`)
2. Check if installation exists, mark installed components in graph
3. Keep only requested components in graph (line 348: `graph.keep(&all_components)`)
4. Install loop: iterate graph in topological order, continue on failures
5. Write final `modules.json`
6. Create `UnityInstallation` (reads `Info.plist` - fails if Editor missing)

**Key insight:** The graph already encodes whether Editor installation is needed:
- **Fresh install**: Editor not installed → "Unity" is in the graph → will be attempted
- **Adding modules**: Editor already installed → "Unity" marked installed (line 255), filtered by `keep()` → not in graph

## Goals / Non-Goals

**Goals:**
- Fail-fast when Editor installation fails (instead of collecting error and continuing)
- Cleanup entire installation directory when Editor installation fails
- Preserve resilient module installation behavior (continue on module failures)
- Prevent broken partial installations from appearing in `uvm list`

**Non-Goals:**
- Detecting whether Editor exists before installation (graph already knows)
- Changing the topological order of installation
- Rolling back individual module installations
- Changing the incremental `modules.json` writing behavior
- Adding complex state tracking (let the graph tell us what's needed)

## Decisions

### Decision 1: Fail-fast on Editor installation failure

**Decision:** In the installation loop (`install_modules_with_installer()` at line 555), check if the failed component is the Editor (`module_id == "Unity"`). If so, immediately cleanup and return error instead of collecting and continuing.

**Rationale:**
- If "Unity" is being installed, it means it's required (not already installed)
- Topological order guarantees Editor installs first, so failing here prevents module installations
- No need to track additional state - the graph already tells us if Editor is needed
- Simple check: `if module_id == "Unity" && install_result.is_err()`

**Alternatives considered:**
- Add boolean flag to track "fresh install" → unnecessary, graph already knows
- Check in the caller → misses opportunity to skip module installations
- Continue and check at the end → too late, modules already installed

**Implementation approach:**
```rust
// In install_modules_with_installer, line ~546
fn install_modules_with_installer<'a, P: AsRef<Path>, I: ModuleInstaller>(
    graph: &'a InstallGraph<'a>,
    base_dir: P,
    modules: &mut Vec<Module>,
    installer: &I,
) -> Result<()> {  // Changed from Vec<InstallError>
    let base_dir = base_dir.as_ref();
    let mut errors = Vec::new();

    for node in graph.topo().iter(graph.context()) {
        if let Some(InstallStatus::Missing) = graph.install_status(node) {
            let component = graph.component(node).unwrap();
            let module_id = match component {
                UnityComponent::Editor(_) => "Unity".to_string(),
                UnityComponent::Module(m) => m.id().to_string(),
            };

            info!("install {}", module_id);
            let install_result = installer.install_module(&module_id, base_dir);

            match install_result {
                Err(err) if module_id == "Unity" => {
                    // Editor installation failed - cleanup and abort
                    log::error!("Editor installation failed, cleaning up");
                    if base_dir.exists() {
                        if let Err(cleanup_err) = std::fs::remove_dir_all(base_dir) {
                            log::warn!("Cleanup failed: {}", cleanup_err);
                        }
                    }
                    return Err(InstallError::EditorInstallationFailed(Box::new(err)));
                }
                Err(err) => {
                    // Module failure - collect and continue
                    log::warn!("Failed to install module {}: {}", module_id, err);
                    errors.push(err);
                }
                Ok(()) => {
                    // Success - mark installed
                    if let Some(m) = modules.iter_mut().find(|m| m.id() == module_id) {
                        m.is_installed = true;
                        trace!("module {} installed successfully", module_id);
                    }
                }
            }

            // Write modules.json after each attempt (unless we returned early)
            write_modules_json(base_dir, modules);
        }
    }

    // Return appropriate result
    if errors.is_empty() {
        Ok(())
    } else {
        Err(InstallError::ModuleInstallationsFailed(errors))
    }
}
```

### Decision 2: Perform cleanup inline in the installer loop

**Decision:** Cleanup the installation directory immediately when Editor fails, inside `install_modules_with_installer()`.

**Rationale:**
- Simpler control flow - single location for error handling
- Prevents any subsequent module installations from running
- No need to signal "cleanup needed" to caller
- Cleanup failure is non-fatal (best-effort)

**Alternatives considered:**
- Cleanup in caller → requires returning signal, more complex
- Create separate cleanup function → overengineered for `fs::remove_dir_all()`
- Don't cleanup on error → leaves broken partial installation (current problem)

**Error handling:** If cleanup fails, log warning but still return the Editor installation error. The cleanup failure is secondary to the primary error (Editor didn't install).

### Decision 3: Use Result<()> return type with semantic error variants

**Decision:** Change `install_modules_with_installer()` signature from returning `Vec<InstallError>` to returning `Result<()>`, using error variants to distinguish Editor vs. module failures.

**Rationale:**
- Error type encodes what happened, not the caller's responsibility to interpret
- `EditorInstallationFailed` = cleanup happened, Editor was required and failed
- `ModuleInstallationsFailed` = Editor succeeded (or not needed), one or more modules failed
- No vec scanning needed - the type tells you everything
- Idiomatic Rust: `Result<()>` for operations that succeed or fail

**Type change:**
```rust
// Before
fn install_modules_with_installer(...) -> Vec<InstallError>

// After
fn install_modules_with_installer(...) -> Result<()>
```

**Control flow at end of install loop:**
```rust
// After the loop completes
if errors.is_empty() {
    Ok(())
} else {
    Err(InstallError::ModuleInstallationsFailed(errors))
}
```

**Caller adjustment** (line 365):
```rust
// Before
let errors = install_module_and_dependencies(&graph, &base_dir, &mut modules);
if !errors.is_empty() {
    // ... check and return MultipleInstallFailures
}

// After
install_module_and_dependencies(&graph, &base_dir, &mut modules)?;
// Done - error handling is in the error type itself
```

### Decision 4: Add semantic error variants for Editor and module failures

**Decision:** Add two error variants to distinguish failure types:
- `InstallError::EditorInstallationFailed(Box<InstallError>)` - Editor failed, cleanup happened
- `InstallError::ModuleInstallationsFailed(Vec<InstallError>)` - One or more modules failed

**Rationale:**
- Error type itself communicates what failed and what action was taken
- Caller doesn't need to inspect error contents to know if cleanup happened
- Clear contract: EditorInstallationFailed means cleanup already done
- ModuleInstallationsFailed can contain multiple module errors (preserves current behavior)
- Replaces the current `MultipleInstallFailures` pattern with more semantic variants

**Implementation in `uvm_install/src/error.rs`:**
```rust
#[derive(Debug)]
pub enum InstallError {
    // ... existing variants
    EditorInstallationFailed(Box<InstallError>),
    ModuleInstallationsFailed(Vec<InstallError>),
}

impl Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // ... existing arms
            Self::EditorInstallationFailed(err) => {
                write!(f, "Unity Editor installation failed: {}", err)
            }
            Self::ModuleInstallationsFailed(errs) => {
                write!(f, "Module installation failed ({} errors)", errs.len())
            }
        }
    }
}
```

### Decision 5: Don't write modules.json after Editor failure

**Decision:** Only write `modules.json` after successful or module-only failures, not after Editor failure with cleanup.

**Rationale:**
- If we're removing the entire directory, no point writing modules.json
- Cleanup will delete it anyway
- Avoids momentary inconsistent state

**Implementation:** Move `write_modules_json()` call to after the match expression, so it only runs if we didn't return early.

```rust
match install_result {
    Err(err) if module_id == "Unity" => {
        // cleanup and return - DON'T write modules.json
        ...
        return Err(...);
    }
    Err(err) => { errors.push(err); }
    Ok(()) => { /* mark installed */ }
}

// Only reached if we didn't return early
write_modules_json(base_dir, modules);
```

## Risks / Trade-offs

**[Risk] Cleanup might fail due to file locks or permissions**
→ **Mitigation:** Best-effort cleanup - log warning if it fails, but still return the Editor error. User will need manual cleanup, but at least they'll know Editor failed (better than current state).

**[Trade-off] Breaking behavior change: installations now abort on Editor failure**
→ This is intentional and improves UX. Current behavior leaves broken partial installations.

**[Risk] Architecture mismatch reinstall path might be affected**
→ **Mitigation:** Architecture mismatch code (line 258-282) already deletes the directory and marks all as missing. If Editor reinstall fails, cleanup is appropriate behavior.

**[Risk] Cleanup removes base_dir even if user specified custom destination**
→ This is correct behavior - if user specified a destination and Editor failed, that destination should be cleaned up.

## Migration Plan

**Deployment:**
1. No database or external system changes required
2. Change is self-contained within `uvm_install` crate
3. Existing installations are unaffected (only impacts new installation attempts)
4. Error messages will be clearer (explicit "Editor installation failed")

**Rollback:**
- Revert to previous behavior where all errors are collected
- No data migration needed

**Testing approach:**
- Extend existing `MockModuleInstaller` tests to simulate Editor failure
- Test: Editor fails → cleanup triggered, modules not attempted
- Test: Module fails → continue with other modules, no cleanup
- Test: Cleanup failure doesn't mask Editor error
- Manual testing with network failures during Editor download
