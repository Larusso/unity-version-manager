## Why

During development and debugging of the installation pipeline, engineers need to inspect the raw `modules.json` payload that would be generated for a given Unity version, platform, and architecture — without performing a full installation. No such standalone tool exists today; the only way to obtain a `modules.json` is to run a real install and extract the side-effect file.

## What Changes

- Add a `dev-commands` feature flag to the `uvm` crate that gates developer/debugging subcommands out of production builds
- Add a `download-modules-json` subcommand (behind the feature flag) that fetches release data via `uvm_live_platform` and outputs the serialized `modules.json` content for a given version, platform, and architecture — to stdout by default, or to a file via `--output <file>`

## Capabilities

### New Capabilities
- `download-modules-json-command`: A CLI command that accepts a Unity version, target platform, and target architecture, fetches the corresponding release via `FetchRelease`, and writes the `modules.json` payload to stdout (default) or a file path supplied via `--output`

### Modified Capabilities
<!-- none -->

## Impact

- **`uvm` crate**: new `dev-commands` feature flag; command registered in the CLI only when the feature is enabled
- **`uvm/src/commands/`**: new `download_modules_json.rs` module compiled only under `#[cfg(feature = "dev-commands")]`
- **`uvm_live_platform`**: consumed read-only via `FetchRelease` — no changes needed
- **`uvm_install`**: the `modules.json` serialization format (pretty-printed JSON of `Module` slice) is reused; no source changes required
- **Production builds**: feature is off by default; the subcommand does not appear in release binaries
