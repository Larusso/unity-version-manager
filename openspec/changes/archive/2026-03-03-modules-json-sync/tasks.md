## 1. Refactor Installation Setup

- [x] 1.1 Move UnityInstallation creation before the installation loop in `Installer::install()`
- [x] 1.2 Initialize modules list before installation loop (load existing or create from release)
- [x] 1.3 Ensure base directory exists before creating UnityInstallation handle

## 2. Implement Incremental State Updates

- [x] 2.1 Refactor `install_module_and_dependencies` to accept mutable modules list and installation handle
- [x] 2.2 Update module's `is_installed` flag after each successful installation
- [x] 2.3 Call `write_modules()` after each module completes (success or failure)

## 3. Implement Error Collection

- [x] 3.1 Change `install_module_and_dependencies` to collect errors instead of returning early
- [x] 3.2 Return collected errors after all modules are attempted
- [x] 3.3 Update `Installer::install()` to handle multiple errors and report them

## 4. Testing

- [x] 4.1 Add test for incremental modules.json updates during installation
- [x] 4.2 Add test for partial failure scenario (some modules fail, others succeed)
- [x] 4.3 Verify modules.json state matches physical installation after partial failure

## 5. Multi-Error Reporting

- [x] 5.1 Add `MultipleInstallFailures` variant to `InstallError` that holds a `Vec<InstallError>`
- [x] 5.2 Update `Installer::install()` to return `MultipleInstallFailures` when multiple modules fail
- [x] 5.3 Implement `Display` for the new variant to show all failures
