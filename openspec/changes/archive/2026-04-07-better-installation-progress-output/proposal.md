## Why

The current Unity installation process provides minimal progress feedback, making it difficult for users to understand what's happening during downloads and installations. Users only see initial request messages and final success/failure messages, leaving them in the dark during the lengthy download and installation phases. This poor user experience makes the tool feel unresponsive and creates uncertainty about whether the installation is proceeding correctly.

## What Changes

- Add visual progress indicators for download operations showing current progress, total size, and download speed
- Display clear status messages for each installation phase (downloading, extracting, installing, verifying)
- Show component-level progress when installing modules with dependencies
- Provide summary statistics at completion (total time, data downloaded, components installed)
- Maintain current error handling while improving error context presentation

## Capabilities

### New Capabilities
- `installation-progress`: Visual progress reporting for downloads, extraction, and installation phases with component tracking and summary statistics

### Modified Capabilities
<!-- Existing capabilities whose REQUIREMENTS are changing (not just implementation).
     Only list here if spec-level behavior changes. Each needs a delta spec file.
     Use existing spec names from openspec/specs/. Leave empty if no requirement changes. -->

## Impact

- **CLI**: `uvm/src/commands/install.rs` - integrate progress reporting into install command
- **Installer Core**: `uvm_install/src/lib.rs` - wire up progress handlers during installation
- **Download Logic**: `uvm_install/src/install/loader.rs` - enhance ProgressHandler implementation with download metrics
- **Platform Installers**: `uvm_install/src/sys/*/` - add progress hooks to extraction/installation phases
- **Dependencies**: Add progress bar library (e.g., `indicatif`) for cross-platform terminal progress display
