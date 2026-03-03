## ADDED Requirements

### Requirement: System writes modules.json after each module installation

The system SHALL write `modules.json` to disk after each module completes installation, updating the `is_installed` flag for that module.

#### Scenario: Successful module installation updates state

- **WHEN** a module installation completes successfully
- **THEN** the system writes `modules.json` with that module's `is_installed` set to `true`

#### Scenario: Failed module installation updates state

- **WHEN** a module installation fails
- **THEN** the system writes `modules.json` with that module's `is_installed` remaining `false`
- **AND** the system continues installing remaining modules

### Requirement: Installation continues despite individual module failures

The system SHALL continue installing remaining modules when one module fails, collecting all errors for final reporting.

#### Scenario: Multiple modules with one failure

- **WHEN** the user installs modules A, B, and C
- **AND** module B fails during installation
- **THEN** the system installs module A successfully
- **AND** the system attempts module B and records the failure
- **AND** the system continues to install module C
- **AND** the system reports module B's failure at the end

#### Scenario: All errors reported at completion

- **WHEN** multiple modules fail during installation
- **THEN** the system reports all failures after attempting all modules
- **AND** the return value indicates installation had failures

### Requirement: modules.json reflects accurate installation state

The system SHALL ensure `modules.json` accurately reflects which modules are physically installed at all times during and after installation.

#### Scenario: Partial installation state is accurate

- **WHEN** installation is interrupted after module A succeeds but before module B completes
- **THEN** `modules.json` shows module A with `is_installed: true`
- **AND** `modules.json` shows module B with `is_installed: false`

#### Scenario: State is queryable immediately after write

- **WHEN** a module installation completes and `modules.json` is written
- **THEN** running `uvm list` or similar commands shows the updated state
