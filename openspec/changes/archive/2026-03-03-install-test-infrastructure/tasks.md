## 1. Create Test Fixtures

- [x] 1.1 Create helper function to build minimal `Release` JSON for testing
- [x] 1.2 Create helper function to build `Module` JSON with configurable id and properties
- [x] 1.3 Verify fixtures can be deserialized into valid `Release` instances

## 2. Wire Up InstallGraph for Testing

- [x] 2.1 Create helper to build `InstallGraph` from test `Release`
- [x] 2.2 Mark all modules as `Missing` status for installation testing
- [x] 2.3 Verify graph iteration yields expected modules

## 3. MockModuleInstaller for Controlled Success/Failure

- [x] 3.1 Implement `MockModuleInstaller` with `fail_modules: HashSet<String>` to control which modules fail
- [x] 3.2 MockModuleInstaller returns `Ok(())` for modules not in fail set
- [x] 3.3 MockModuleInstaller returns `Err(InstallError::InstallFailed)` for modules in fail set
- [x] 3.4 Add tracking to record which modules were attempted (install order)

## 4. Test `install_modules_with_installer` Directly

Tests call `install_modules_with_installer` directly with MockModuleInstaller to bypass RealModuleInstaller.

- [x] 4.1 Test: all modules succeed - MockModuleInstaller with empty fail set, verify all marked installed
- [x] 4.2 Test: single module fails - fail set contains "ios", verify android/webgl succeed, ios fails
- [x] 4.3 Test: multiple modules fail - fail set contains "ios" and "webgl", verify errors collected
- [x] 4.4 Test: verify modules.json reflects correct state (installed modules true, failed modules false)
- [x] 4.5 Test: verify install continues after failure (all modules attempted, not just up to first failure)
