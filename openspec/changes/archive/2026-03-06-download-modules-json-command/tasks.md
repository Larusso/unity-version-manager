## 1. Feature Flag Setup

- [x] 1.1 Add `[features]` section to `uvm/Cargo.toml` with a `dev-commands` feature (no extra dependencies required — all needed crates are already in `[dependencies]`)

## 2. Command Implementation

- [x] 2.1 Create `uvm/src/commands/download_modules_json.rs` with a `DownloadModulesJsonCommand` struct deriving `clap::Args`, containing:
  - positional `version: Version`
  - `--platform` (`UnityReleaseDownloadPlatform`, default = current)
  - `--architecture` (`UnityReleaseDownloadArchitecture`, default = current)
  - `--output` (`Option<PathBuf>`)
- [x] 2.2 Implement `execute()` on `DownloadModulesJsonCommand`:
  - Call `FetchRelease::builder(version).with_extended_lts().with_u7_alpha().with_platform(platform).with_architecture(architecture).fetch()`
  - Collect modules from `release.downloads` (all modules via `iter_modules()`)
  - Serialize with `serde_json::to_string_pretty(&modules)`
  - If `--output` is set: create parent directories with `fs::create_dir_all`, then write to file
  - Otherwise: write to stdout
  - Return exit code 0 on success, 1 on error (print error to stderr)

## 3. Wire Into CLI

- [x] 3.1 Add `#[cfg(feature = "dev-commands")] pub mod download_modules_json;` to `uvm/src/commands/mod.rs`
- [x] 3.2 Add `#[cfg(feature = "dev-commands")] DownloadModulesJson(DownloadModulesJsonCommand)` variant to the `Commands` enum in `uvm/src/main.rs`
- [x] 3.3 Add the matching arm to `Commands::exec()`: `#[cfg(feature = "dev-commands")] Commands::DownloadModulesJson(cmd) => cmd.execute()`

## 4. Verification

- [x] 4.1 Build without feature flag and confirm `download-modules-json` does not appear in `uvm --help`
- [x] 4.2 Build with `--features dev-commands` and confirm `download-modules-json` appears in `uvm --help`
- [x] 4.3 Run `uvm download-modules-json <real-version>` and verify valid JSON is printed to stdout
- [x] 4.4 Run with `--output /tmp/test/modules.json` (non-existent dir) and verify the directory is created and file is written
- [x] 4.5 Run with `--platform linux --architecture x86_64` on a non-Linux host and verify the output reflects Linux/x86_64 modules
