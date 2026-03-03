## ADDED Requirements

### Requirement: Test helpers can create InstallGraph from minimal data

The test module SHALL provide helpers to create `InstallGraph` instances from minimal JSON fixtures without requiring real Unity API data.

#### Scenario: Create graph with single module

- **WHEN** a test creates a Release fixture with one module
- **THEN** an InstallGraph can be built from it
- **AND** the graph contains the expected module node

#### Scenario: Create graph with multiple modules

- **WHEN** a test creates a Release fixture with modules A, B, C
- **THEN** an InstallGraph can be built containing all three modules
- **AND** iteration yields modules in topological order

### Requirement: MockModuleInstaller controls per-module success/failure

The test module SHALL provide a `MockModuleInstaller` that implements `ModuleInstaller` trait and allows configuring which modules succeed or fail.

#### Scenario: Module not in fail set succeeds

- **WHEN** MockModuleInstaller has fail_modules = {"ios"}
- **AND** install_module is called for "android"
- **THEN** the call returns Ok(())

#### Scenario: Module in fail set fails

- **WHEN** MockModuleInstaller has fail_modules = {"ios"}
- **AND** install_module is called for "ios"
- **THEN** the call returns Err(InstallError::InstallFailed)

### Requirement: Tests can verify installation behavior via install_modules_with_installer

Tests SHALL call `install_modules_with_installer` directly with MockModuleInstaller to test the production loop logic.

#### Scenario: Single module fails, others continue

- **WHEN** MockModuleInstaller is configured to fail module "ios"
- **AND** install_modules_with_installer is called with graph containing [android, ios, webgl]
- **THEN** android succeeds and is marked installed in modules list
- **AND** ios fails and remains not installed
- **AND** webgl succeeds and is marked installed (installation continued past failure)
- **AND** the returned errors vector contains the ios failure

#### Scenario: Multiple modules fail

- **WHEN** MockModuleInstaller is configured to fail modules "ios" and "webgl"
- **AND** install_modules_with_installer is called
- **THEN** errors for both ios and webgl are returned
- **AND** only android is marked installed in modules list

### Requirement: Tests verify modules.json is written after each module

The test module SHALL verify that `modules.json` is written incrementally during installation.

#### Scenario: State persisted after each module

- **WHEN** install_modules_with_installer processes 3 modules
- **THEN** modules.json exists after the function returns
- **AND** the file reflects the final installation state
