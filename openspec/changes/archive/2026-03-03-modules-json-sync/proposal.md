## Why

When a module installation fails mid-way, `modules.json` is never written because it only gets written once at the end of a successful installation. This leaves the installation in an inconsistent state where physically installed modules aren't reflected in `modules.json`, causing uvm to misreport installation status.

## What Changes

- Write `modules.json` incrementally after each module finishes installing (success or failure)
- Track partial installation state so users can see what actually installed
- Handle module installation errors gracefully - continue with remaining modules and update state accordingly

## Capabilities

### New Capabilities

- `incremental-modules-sync`: Write modules.json after each module installation step completes, ensuring installation state is always persisted to disk

### Modified Capabilities

None - this change modifies implementation details within the existing installation flow, not external requirements.

## Impact

- **Code**: `uvm_install/src/lib.rs` (installation flow), `unity-hub/src/unity/installation.rs` (write_modules)
- **Behavior**: modules.json will be written multiple times during installation instead of once at the end
- **Reliability**: Partial installations will have accurate state, improving recovery from failures
- **Performance**: Minimal - one small file write per module installed
