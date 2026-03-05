## ADDED Requirements

### Requirement: System detects whether Editor pre-exists before installation

The system SHALL detect whether a Unity Editor installation already exists at the target path before beginning the installation process.

#### Scenario: Fresh install detected when directory is empty

- **WHEN** the installation begins
- **AND** the target installation directory does not exist
- **THEN** the system records this as a "fresh install requiring Editor"

#### Scenario: Fresh install detected when directory exists but Editor is missing

- **WHEN** the installation begins
- **AND** the target installation directory exists but does not contain a valid Unity Editor (no `Unity.app/Contents/Info.plist` or equivalent)
- **THEN** the system records this as a "fresh install requiring Editor"

#### Scenario: Existing Editor detected

- **WHEN** the installation begins
- **AND** the target installation directory contains a valid Unity Editor installation
- **THEN** the system records this as "adding modules to existing Editor"

### Requirement: Editor failure triggers cleanup for fresh installs

The system SHALL remove the entire installation directory when Editor installation fails during a fresh install.

#### Scenario: Editor installation fails on fresh install

- **WHEN** the system detects a fresh install requiring Editor
- **AND** the Editor installation fails
- **THEN** the system removes the entire installation directory
- **AND** the system returns an error indicating Editor installation failed
- **AND** the system does NOT attempt to install any modules

#### Scenario: Editor failure leaves no partial state

- **WHEN** the Editor installation fails during a fresh install
- **AND** the installation directory is cleaned up
- **THEN** running `uvm list` does NOT show the failed installation
- **AND** no `modules.json` file exists at the target path

### Requirement: Editor failure does not affect module-only installations

The system SHALL NOT cleanup or abort when Editor already exists and only modules are being added, even if a module fails.

#### Scenario: Module fails when adding to existing Editor

- **WHEN** the system detects an existing Editor installation
- **AND** the user is installing additional modules
- **AND** one module installation fails
- **THEN** the system continues installing remaining modules
- **AND** the system does NOT remove the installation directory
- **AND** the system collects and reports the module failure

#### Scenario: All modules fail when adding to existing Editor

- **WHEN** the system detects an existing Editor installation
- **AND** all module installations fail
- **THEN** the installation directory remains intact
- **AND** the existing Editor remains functional
- **AND** the system reports all module failures

### Requirement: Cleanup removes all partial installation artifacts

The system SHALL remove all files and directories created during the failed installation when cleaning up.

#### Scenario: Cleanup removes base directory

- **WHEN** Editor installation fails and triggers cleanup
- **THEN** the system removes the entire base installation directory
- **AND** no subdirectories remain (e.g., no `PlaybackEngines/` fragments)

#### Scenario: Cleanup removes modules.json if written

- **WHEN** Editor installation fails after `modules.json` has been written
- **AND** cleanup is triggered
- **THEN** the system removes `modules.json` along with the installation directory
