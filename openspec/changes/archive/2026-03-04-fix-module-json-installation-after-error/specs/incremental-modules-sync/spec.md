## MODIFIED Requirements

### Requirement: Installation continues despite individual module failures

The system SHALL continue installing remaining modules when one module fails, collecting all errors for final reporting, UNLESS the failed component is the Editor during a fresh install requiring the Editor.

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

#### Scenario: Editor failure stops installation during fresh install

- **WHEN** the system is performing a fresh install requiring the Editor
- **AND** the Editor installation fails
- **THEN** the system does NOT attempt to install any modules
- **AND** the system triggers cleanup of the installation directory
- **AND** the system returns an error indicating Editor installation failed

#### Scenario: Editor failure does not stop module installation for existing Editors

- **WHEN** the system is adding modules to an existing Editor installation
- **AND** the Editor installation was attempted (e.g., architecture mismatch reinstall) and fails
- **THEN** the system continues installing modules as normal
- **AND** the system reports the Editor failure along with any module failures
