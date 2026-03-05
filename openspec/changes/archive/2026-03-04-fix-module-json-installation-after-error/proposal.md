## Why

When the Unity Editor installation fails during a fresh install, the current implementation continues installing modules and writes `modules.json`, leaving a broken partial installation on disk. This partial state causes version detection to fail (can't read `Info.plist` or find Unity executable), breaks `uvm list`, and leaves orphaned module files without a working Editor. Users must manually clean up these broken installations.

## What Changes

- Add detection to distinguish "fresh install" (Editor required) from "adding modules" (Editor already exists)
- When Editor installation fails during a fresh install, immediately cleanup the installation directory and abort
- Preserve current behavior for module-only installations: continue past module failures and collect errors
- Ensure no partial installations are left on disk when Editor installation fails

## Capabilities

### New Capabilities
- `editor-failure-cleanup`: Handle Editor installation failures differently from module failures, with cleanup for fresh installs

### Modified Capabilities
- `incremental-modules-sync`: Modify the installation flow to check Editor existence before installing and cleanup on Editor failure

## Impact

**Affected code:**
- `uvm_install/src/lib.rs`: Main installation flow, needs to track whether Editor pre-existed
- `uvm_install/src/lib.rs` (`install_module_and_dependencies`, `install_modules_with_installer`): Add Editor failure detection and cleanup logic
- `uvm_install/src/error.rs`: May need new error variant for "Editor installation required but failed"

**Behavior changes:**
- Fresh installations will abort and cleanup if Editor fails (breaking change in behavior, but better UX)
- Module-only installations continue with current resilient behavior
- No more broken partial installations left on disk
