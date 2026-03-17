# Specification: Installation Progress

## Purpose

Define how the Unity Version Manager reports installation progress to the user, covering real-time download metrics, phase status messages, multi-component hierarchy, summary statistics, non-interactive output, logging integration, and error visibility.

## Requirements

### Requirement: Display download progress with metrics
The system SHALL display real-time download progress for each installer component including current size, total size, download speed, and estimated time remaining.

#### Scenario: Download progress updates during file transfer
- **WHEN** a Unity installer package is being downloaded
- **THEN** the system displays a progress bar showing bytes downloaded, total bytes, current speed in MB/s, and updates at least once per second

#### Scenario: Multiple components show individual progress
- **WHEN** installing Unity editor with multiple modules
- **THEN** each component displays its own progress bar with individual download metrics

#### Scenario: Download completes successfully
- **WHEN** a component download reaches 100%
- **THEN** the progress bar shows completion state and total time taken

### Requirement: Display installation phase status
The system SHALL clearly indicate the current installation phase with descriptive status messages for metadata fetching, dependency resolution, downloading, extracting, and installing.

#### Scenario: Metadata fetch phase
- **WHEN** installation begins
- **THEN** system displays a spinner with message "Fetching Unity version metadata..."

#### Scenario: Dependency resolution phase
- **WHEN** metadata is fetched and dependency graph is being built
- **THEN** system displays a spinner with message "Resolving component dependencies..."

#### Scenario: Download phase with component name
- **WHEN** downloading a specific component
- **THEN** system displays the component name and version being downloaded

#### Scenario: Extraction phase
- **WHEN** extracting an installer package
- **THEN** system displays a spinner with message indicating extraction is in progress

#### Scenario: Installation phase
- **WHEN** running platform-specific installer
- **THEN** system displays a spinner with message indicating installation is in progress

### Requirement: Show multi-component installation hierarchy
The system SHALL display overall installation progress when multiple components are being installed, showing total component count and individual component status.

#### Scenario: Overall progress with component count
- **WHEN** installing editor and 3 modules
- **THEN** system displays "Installing 4 components" with overall progress indicator

#### Scenario: Component status hierarchy
- **WHEN** multiple components are being installed
- **THEN** system displays a hierarchical view with overall progress at top and individual component progress below

#### Scenario: Pending components indication
- **WHEN** some components are waiting for dependencies
- **THEN** those components display "Waiting..." or "Pending..." status

### Requirement: Provide installation summary statistics
The system SHALL display summary statistics upon completion including total time elapsed, total data downloaded, number of components installed, and final installation path.

#### Scenario: Successful installation summary
- **WHEN** installation completes successfully
- **THEN** system displays total time, data downloaded in GB/MB, component count, and installation directory path

#### Scenario: Summary includes all downloaded data
- **WHEN** multiple components are downloaded
- **THEN** summary aggregates total bytes downloaded across all components

#### Scenario: Time formatting
- **WHEN** displaying elapsed time
- **THEN** system formats time appropriately (seconds for <60s, minutes:seconds for <1h, hours:minutes:seconds for >=1h)

### Requirement: Handle non-TTY and piped output gracefully
The system SHALL detect non-interactive terminal environments (CI, piped stdout/stderr) and disable progress bars entirely, falling back to simple text milestone messages.

#### Scenario: Non-TTY detection via stdout
- **WHEN** stdout is not a TTY (e.g., redirected to file)
- **THEN** system outputs simple text progress messages without progress bars or escape codes

#### Scenario: Piped output detection
- **WHEN** stdout or stderr is piped to another process
- **THEN** system disables progress bars and outputs only milestone messages

#### Scenario: Progress messages in non-interactive mode
- **WHEN** running in non-interactive mode
- **THEN** system outputs milestone messages like "Downloading component X...", "Downloaded X (Y MB)", "Installing X..."

#### Scenario: CI environment detection
- **WHEN** running in CI environment (CI=true env var or non-TTY)
- **THEN** system disables all progress bars and spinners

### Requirement: Integrate with logging system
The system SHALL route log messages through indicatif when progress bars are active to prevent log output from corrupting progress display, and SHALL respect log level settings.

#### Scenario: Log messages with active progress bars
- **WHEN** progress bars are displayed and log messages are emitted
- **THEN** system routes log output through indicatif's println/suspend methods to appear above progress bars

#### Scenario: Verbose logging compatibility
- **WHEN** user sets verbose log levels (debug, trace)
- **THEN** log messages appear correctly without interfering with progress bar rendering

#### Scenario: Log output in non-interactive mode
- **WHEN** running in non-interactive mode (no progress bars)
- **THEN** log messages output normally via standard logging without indicatif routing

#### Scenario: Logger initialization with progress support
- **WHEN** installing with progress bars enabled
- **THEN** system configures logging to use indicatif-compatible output methods

### Requirement: Preserve error reporting
The system SHALL maintain existing error messages and context when errors occur during any installation phase, ensuring errors are visible regardless of progress display state.

#### Scenario: Error during download
- **WHEN** a download fails with network error
- **THEN** system clears progress display and shows error message with context

#### Scenario: Error during installation
- **WHEN** platform installer fails
- **THEN** system clears progress display and shows error message with exit code and component name

#### Scenario: Error message visibility
- **WHEN** any error occurs during installation
- **THEN** error message is displayed prominently and not obscured by progress indicators
