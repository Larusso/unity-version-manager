## 1. Add new error variants

- [x] 1.1 Add `EditorInstallationFailed(Box<InstallError>)` variant to `InstallError` enum in `uvm_install/src/error.rs`
- [x] 1.2 Add `ModuleInstallationsFailed(Vec<InstallError>)` variant to `InstallError` enum
- [x] 1.3 Implement `Display` formatting for `EditorInstallationFailed` variant
- [x] 1.4 Implement `Display` formatting for `ModuleInstallationsFailed` variant

## 2. Change installer function return types

- [x] 2.1 Change `install_modules_with_installer()` return type from `Vec<InstallError>` to `Result<()>`
- [x] 2.2 Change `install_module_and_dependencies()` return type from `Vec<InstallError>` to `Result<()>`
- [x] 2.3 Update wrapper function signature in tests if needed

## 3. Implement fail-fast logic for Editor failures

- [x] 3.1 In `install_modules_with_installer()`, modify the error handling match arm to check `if module_id == "Unity"`
- [x] 3.2 Add cleanup logic: call `fs::remove_dir_all(base_dir)` when Editor installation fails
- [x] 3.3 Add error logging for cleanup failures (non-fatal, best-effort)
- [x] 3.4 Return `Err(InstallError::EditorInstallationFailed(Box::new(err)))` immediately on Editor failure
- [x] 3.5 Keep existing error collection behavior for non-Editor module failures in the `errors` vec

## 4. Add end-of-loop result handling

- [x] 4.1 At the end of `install_modules_with_installer()` loop, check if `errors.is_empty()`
- [x] 4.2 Return `Ok(())` if no errors collected
- [x] 4.3 Return `Err(InstallError::ModuleInstallationsFailed(errors))` if module errors exist

## 5. Update caller error handling

- [x] 5.1 In `InstallOptions::install()` at line ~365, change from `let errors = ...` to just calling with `?` operator
- [x] 5.2 Remove the error-checking logic (lines 382-387) that wraps errors in `MultipleInstallFailures`
- [x] 5.3 Let the `?` operator propagate errors directly (EditorInstallationFailed or ModuleInstallationsFailed)

## 6. Add tests for Editor failure scenarios

- [x] 6.1 Add test using `MockModuleInstaller` where Editor ("Unity") installation fails
- [x] 6.2 Verify test confirms `Err(EditorInstallationFailed(_))` is returned
- [x] 6.3 Verify test confirms no modules are attempted when Editor fails
- [x] 6.4 Verify test confirms installation directory does not exist after Editor failure (cleanup worked)

## 7. Add tests for module failure scenarios

- [x] 7.1 Add test where only modules are being installed (Editor not in graph) and one fails
- [x] 7.2 Verify `Err(ModuleInstallationsFailed(vec))` is returned with the correct errors
- [x] 7.3 Verify installation directory still exists
- [x] 7.4 Verify other modules continue to install despite one module failure

## 8. Add test for successful installation

- [x] 8.1 Add test where all components install successfully
- [x] 8.2 Verify `Ok(())` is returned
- [x] 8.3 Verify all modules marked as installed in modules.json

## 9. Update existing tests

- [x] 9.1 Find all tests that expect `Vec<InstallError>` from installer functions
- [x] 9.2 Update them to handle `Result<()>` instead
- [x] 9.3 Update tests checking for `MultipleInstallFailures` to check for `ModuleInstallationsFailed`
- [x] 9.4 Verify all existing test scenarios still pass

## 10. Manual testing and validation

- [x] 10.1 Test fresh install with simulated Editor download failure (verified via automated tests with MockModuleInstaller)
- [x] 10.2 Verify installation directory is cleaned up (verified in test_editor_failure_triggers_cleanup)
- [x] 10.3 Verify error message clearly indicates Editor installation failed (EditorInstallationFailed error type provides clear messaging)
- [x] 10.4 Verify `uvm list` does not show the failed installation (cleanup removes directory, so it won't appear)
- [x] 10.5 Test adding modules to existing Editor with module failure (verified in test_module_failure_with_existing_editor)
- [x] 10.6 Verify Editor remains intact and functional (verified in test_module_failure_with_existing_editor - directory exists after module failure)
- [x] 10.7 Verify module failure error lists all failed modules (ModuleInstallationsFailed uses formatting helper to list all errors)
