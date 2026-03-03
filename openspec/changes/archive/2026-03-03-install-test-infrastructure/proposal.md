## Why

The `install_modules_with_installer` function in `uvm_install` cannot be properly unit tested. It requires an `InstallGraph` which is built from Unity release data, making it difficult to test error handling, incremental state updates, and partial failure scenarios without real installations.

## What Changes

- Add test fixture helpers to create mock `Release` and `Module` data via JSON deserialization
- Add a test helper module in `uvm_install` for constructing test `InstallGraph` instances
- Write integration tests for `install_modules_with_installer` using `MockModuleInstaller`
- Test the actual production code paths: iteration, error collection, state updates, and incremental writes

## Capabilities

### New Capabilities

- `install-graph-test-helpers`: Test utilities for creating mock `InstallGraph` instances from JSON fixtures, enabling unit tests for installation logic

### Modified Capabilities

None

## Impact

- **Code**: `uvm_install/src/lib.rs` (test module), possibly `uvm_install_graph` if we need to expose internals
- **Dependencies**: None - uses existing `serde_json` for fixture deserialization
- **Tests**: Adds meaningful tests for `install_modules_with_installer` covering success, failure, and partial failure scenarios
