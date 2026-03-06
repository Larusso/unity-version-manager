## Context

The `uvm_install` crate writes `modules.json` as a side-effect of installation. During development and debugging of the installation pipeline, engineers need to inspect this file for a specific version/platform/architecture without running a full install. The `uvm_live_platform` crate already provides `FetchRelease` to query the Unity release API, and the platform and architecture types already carry a `clap` feature flag that exposes `ValueEnum`.

The `uvm` crate is the only production-facing binary. Adding a feature flag is the lowest-overhead path: no new crates, no new binaries, same build system, naturally excluded from production builds.

## Goals / Non-Goals

**Goals:**
- Add a `dev-commands` feature to the `uvm` crate that compiles in one or more debugging subcommands
- Implement `download-modules-json` as the first command under this feature
- Accept version (required), platform (optional, defaults to current), and architecture (optional, defaults to current) as CLI arguments
- Accept `--output <file>` to write the JSON to a file; default to stdout
- Produce output in the same format as `write_modules_json` in `uvm_install` (`serde_json::to_string_pretty` of a `&[Module]`)

**Non-Goals:**
- Changing any production code paths
- Supporting streaming or incremental output
- Adding caching behaviour (the existing `uvm_live_platform` cache applies transparently)
- Filtering or transforming the module list beyond what the API returns

## Decisions

### Feature flag name: `dev-commands`

Chosen over `debug` or `unstable` because it clearly signals intent — these are commands for developer use, not production distribution. The flag is off by default in `Cargo.toml` so `cargo build` / `cargo install` will never include it.

### Single module file, `#[cfg(feature = "dev-commands")]` throughout

The command lives in `uvm/src/commands/download_modules_json.rs`. The `mod` declaration in `commands/mod.rs` and the `Commands` enum variant in `main.rs` are both wrapped in `#[cfg(feature = "dev-commands")]`. This is consistent with how Rust handles optional features — no indirection or plugin system needed.

### Re-use `uvm_live_platform`'s `clap` feature for `ValueEnum`

`UnityReleaseDownloadPlatform` and `UnityReleaseDownloadArchitecture` already derive `clap::ValueEnum` behind the `clap` feature in `uvm_live_platform`. The `uvm` crate enables this feature in its dependency. Under `dev-commands`, we rely on the same types directly as clap `Args`, so no wrapper types are needed.

**Alternative considered**: Define our own string args and parse manually. Rejected — duplicates existing logic and loses the automatic `--help` value enumeration.

### `FetchRelease` builder with explicit platform/architecture filters

The command calls:
```
FetchRelease::builder(version)
    .with_extended_lts()
    .with_u7_alpha()
    .with_platform(platform)
    .with_architecture(architecture)
    .fetch()
```
This mirrors the pattern in the existing `modules.rs` command and `uvm_install`.

**Alternative considered**: Fetch all platforms/architectures and filter in memory. Rejected — unnecessary network payload; the API supports server-side filtering.

### Output: stdout default, `--output <file>` override

Writing to stdout makes the command composable (`uvm download-modules-json 2023.1.0f1 | jq …`). The `--output` flag covers the case where the file needs to persist without shell redirection. The implementation writes the serialized bytes to either a `BufWriter<File>` or `io::stdout()` via a shared `Write` trait object.

## Risks / Trade-offs

- **Feature flag discoverability**: Engineers must know to build with `--features dev-commands`. This is documented in the command's help text and in this spec. → Mitigation: add a note to `CLAUDE.md` or a contributing guide if this becomes a frequent source of confusion.
- **Format divergence**: If `write_modules_json` in `uvm_install` changes its serialization format, this command won't automatically follow. → Mitigation: the spec explicitly states both must use `serde_json::to_string_pretty(&[Module])`. A future refactor could extract a shared helper.
- **`uvm_live_platform` `clap` feature always on in `uvm`**: The `uvm` crate already depends on `uvm_live_platform` with the `clap` feature enabled (used by the existing `modules` command). No change needed, no risk.

## Migration Plan

No migration needed — additive, feature-gated change. Existing production builds are unaffected.
