## ADDED Requirements

### Requirement: Feature flag gates the command
The `uvm` crate SHALL expose `download-modules-json` as a subcommand only when compiled with the `dev-commands` feature flag. Production builds (without the flag) SHALL NOT include this subcommand or any of its dependencies.

#### Scenario: Command absent in default build
- **WHEN** `uvm` is built without `--features dev-commands`
- **THEN** `uvm --help` does not list `download-modules-json`

#### Scenario: Command present in dev build
- **WHEN** `uvm` is built with `--features dev-commands`
- **THEN** `uvm --help` lists `download-modules-json`

---

### Requirement: Version argument is required
The command SHALL accept a single positional `<version>` argument in Unity version format (e.g. `2023.1.0f1`). The command SHALL fail with a usage error if the version is omitted or unparseable.

#### Scenario: Valid version provided
- **WHEN** the user runs `uvm download-modules-json 2023.1.0f1`
- **THEN** the command proceeds to fetch release data for that version

#### Scenario: Version omitted
- **WHEN** the user runs `uvm download-modules-json` with no arguments
- **THEN** the command exits with a non-zero code and prints a usage error to stderr

---

### Requirement: Platform defaults to current host platform
The command SHALL accept an optional `--platform <platform>` argument accepting `macos`, `linux`, or `windows` (case-insensitive, via `clap::ValueEnum`). When omitted, the platform SHALL default to the host platform.

#### Scenario: Platform omitted on macOS host
- **WHEN** the user runs `uvm download-modules-json 2023.1.0f1` on a macOS machine
- **THEN** the command fetches modules for the `macos` platform

#### Scenario: Platform explicitly specified
- **WHEN** the user runs `uvm download-modules-json 2023.1.0f1 --platform linux`
- **THEN** the command fetches modules for the `linux` platform regardless of host OS

---

### Requirement: Architecture defaults to current host architecture
The command SHALL accept an optional `--architecture <arch>` argument accepting `x86_64` or `arm64` (case-insensitive, via `clap::ValueEnum`). When omitted, the architecture SHALL default to the host architecture (`x86_64` on Linux regardless of actual arch, per existing platform logic).

#### Scenario: Architecture omitted
- **WHEN** the user runs `uvm download-modules-json 2023.1.0f1` on an arm64 macOS machine
- **THEN** the command fetches modules for the `arm64` architecture

#### Scenario: Architecture explicitly overridden
- **WHEN** the user runs `uvm download-modules-json 2023.1.0f1 --architecture x86_64`
- **THEN** the command fetches modules for the `x86_64` architecture

---

### Requirement: Output defaults to stdout
When `--output` is not specified, the command SHALL write the serialized JSON to stdout.

#### Scenario: No output flag
- **WHEN** the user runs `uvm download-modules-json 2023.1.0f1`
- **THEN** the JSON content is written to stdout
- **THEN** the process exits with code 0

---

### Requirement: Output can be written to a file
The command SHALL accept an optional `--output <path>` argument. When provided, the command SHALL write the JSON to the specified file path, creating the file if it does not exist and overwriting it if it does. If the parent directory does not exist, the command SHALL create it (including all intermediate directories) before writing the file.

#### Scenario: Valid output path provided
- **WHEN** the user runs `uvm download-modules-json 2023.1.0f1 --output /tmp/modules.json`
- **THEN** the file `/tmp/modules.json` is created (or overwritten) with the JSON content
- **THEN** nothing is written to stdout
- **THEN** the process exits with code 0

#### Scenario: Output path in non-existent directory
- **WHEN** the user runs `uvm download-modules-json 2023.1.0f1 --output /some/new/dir/modules.json`
- **THEN** the directory `/some/new/dir/` is created recursively
- **THEN** the file `/some/new/dir/modules.json` is written with the JSON content
- **THEN** the process exits with code 0

---

### Requirement: JSON format matches modules.json produced during installation
The output SHALL be the pretty-printed JSON serialization (`serde_json::to_string_pretty`) of the `Vec<Module>` slice returned by `FetchRelease` for the given version, platform, and architecture — identical to the format written by `write_modules_json` in `uvm_install`.

#### Scenario: Output is valid JSON array
- **WHEN** the command succeeds
- **THEN** stdout (or the output file) contains a valid JSON array
- **THEN** each element is a serialized `Module` object matching the structure written during installation

---

### Requirement: Version not found exits with error
When the Unity release API returns no result for the requested version, the command SHALL exit with a non-zero code and print a human-readable error to stderr.

#### Scenario: Unknown version
- **WHEN** the user runs `uvm download-modules-json 9999.9.9f9`
- **THEN** the command exits with code 1
- **THEN** stderr contains a message indicating the version was not found
