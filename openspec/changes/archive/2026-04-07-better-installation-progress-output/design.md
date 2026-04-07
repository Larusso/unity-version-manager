## Context

The current installation flow in UVM has a `ProgressHandler` trait defined but is barely utilized. The `Loader` struct in `uvm_install/src/install/loader.rs` has optional progress handler support, but it's never actually wired up from the CLI layer. Users see only initial and final messages during what can be multi-gigabyte downloads and complex multi-component installations.

The existing architecture is already structured for progress reporting:
- `ProgressHandler` trait exists with `finish()`, `inc()`, `set_length()`, `set_position()` methods
- `Loader` has optional progress handler support
- `DownloadProgress` wrapper implements incremental progress updates during HTTP transfers

The `uvm` CLI binary already depends on `indicatif` 0.18.0, which provides cross-platform progress bars and multi-progress support for concurrent operations.

## Goals / Non-Goals

**Goals:**
- Provide real-time visual feedback during download operations with size/speed metrics
- Show clear phase transitions (fetching metadata → downloading → extracting → installing)
- Display component-level progress when installing multiple modules
- Summarize installation results (time taken, data downloaded, components installed)
- Maintain backward compatibility with existing error handling

**Non-Goals:**
- Progress reporting for non-installation operations (launch, list, detect)
- Machine-readable progress output (JSON/structured logging)
- Progress persistence across process restarts
- Parallel download optimization (only progress reporting, not concurrency changes)

## Decisions

### Decision 1: Use `indicatif` for progress rendering
**Rationale:** Already a dependency in `uvm` crate, mature library with excellent cross-platform support, handles multi-progress scenarios elegantly.

**Alternatives considered:**
- Custom progress implementation → Too much complexity for terminal handling, cursor control, etc.
- `pbr` crate → Less active maintenance, fewer features
- Simple print statements → Poor UX, no dynamic updates

### Decision 2: Implement progress at CLI layer, not library layer
**Rationale:** Keep `uvm_install` library agnostic of terminal UI concerns. The CLI creates and passes progress handlers to the library, maintaining separation of concerns.

**Alternatives considered:**
- Build `indicatif` into `uvm_install` → Creates unnecessary dependency for library users
- Create abstract progress types in library → Over-engineering for single use case

**Implementation approach:**
- Create concrete `ProgressHandler` implementation in `uvm/src/commands/install.rs`
- Pass it to `InstallOptions` via new builder method
- Wire through to `Loader` and installer implementations

### Decision 3: Multi-progress for component installations
**Rationale:** When installing editor + multiple modules, show overall progress plus individual component progress. Uses `indicatif::MultiProgress` to handle the hierarchy.

**Structure:**
```
Overall: [===========>        ] 3/7 components
├─ Editor 2022.3.0f1:     [================] 2.1 GB/2.1 GB
├─ Android Support:       [=====>          ] 512 MB/1.5 GB  
└─ iOS Support:           [                ] Waiting...
```

### Decision 4: Progress phases
**Phases to report:**
1. **Fetching metadata** - GraphQL API call (spinner, no progress bar)
2. **Resolving dependencies** - Graph construction (spinner)
3. **Downloading** - Per-component progress bars with size/speed
4. **Installing** - Per-component spinner/progress (platform-specific)
5. **Summary** - Final statistics

**Rationale:** Matches natural user mental model of installation flow.

### Decision 5: Extend `ProgressHandler` trait minimally
**Current trait is sufficient for downloads.** For installation phases (extraction), add optional method:
```rust
fn set_message(&self, msg: &str) {} // Default no-op
```

This allows spinners to show context ("Extracting pkg...", "Running installer...") without breaking existing interface.

### Decision 6: Integrate logging with indicatif
**Rationale:** UVM uses `flexi_logger` with `log` crate. When progress bars are active, log messages must be routed through indicatif to prevent corruption of progress display.

**Implementation approach:**
- Use `indicatif::MultiProgress::println()` or `ProgressBar::suspend()` for log output
- Create custom log writer that detects active progress bars and routes accordingly
- Check `std::io::IsTerminal` for stdout/stderr to detect piping
- In non-interactive mode, use normal logging without indicatif routing

**TTY/Pipe detection:**
```rust
use std::io::IsTerminal;

let is_interactive = std::io::stdout().is_terminal() 
                  && std::io::stderr().is_terminal()
                  && std::env::var("CI").is_err();
```

**Alternatives considered:**
- Ignore logging compatibility → Log messages would corrupt progress bars at verbose levels
- Disable logging when progress active → Loses important debug information
- Use separate terminal for logs → Not practical for CLI tool

## Risks / Trade-offs

**Risk:** Progress bars may flicker or display incorrectly on exotic terminals  
→ **Mitigation:** `indicatif` handles this well; fallback to simple logging if progress rendering fails

**Risk:** Installation phases without deterministic progress (platform installers) may show indefinite spinners  
→ **Mitigation:** Acceptable UX - spinner indicates activity, better than silence

**Risk:** Added overhead from progress updates in tight loops  
→ **Mitigation:** `indicatif` is optimized for this; updates are rate-limited automatically

**Trade-off:** Multi-progress output requires terminal height; may not fit on small terminals  
→ **Mitigation:** `indicatif` handles overflow by scrolling; users on tiny terminals are rare edge case

**Trade-off:** Dependency on `indicatif` already exists in CLI but not in `uvm_install`  
→ **Decision:** Keep it this way - library stays lean, CLI handles presentation

**Risk:** Logging integration with `flexi_logger` may be complex  
→ **Mitigation:** Use MultiProgress as global state accessible to custom log writer; fallback to standard output if setup fails

**Risk:** Piped output may still show progress artifacts  
→ **Mitigation:** Strict TTY detection on both stdout and stderr before enabling progress bars

## Migration Plan

**Deployment:**
1. Add progress handler implementation to CLI (`uvm/src/commands/install.rs`)
2. Extend `InstallOptions` builder with `with_progress_handler()` method
3. Wire progress through `Loader` and installer create functions
4. Add optional progress calls to platform-specific installers

**Rollback strategy:**
- Progress is additive only; no breaking changes
- If issues arise, can disable progress by not passing handler (defaults to `None`)
- Feature can be controlled via flag if needed: `--no-progress`

**Testing:**
- Manual testing on macOS, Linux, Windows
- Test with single component and multi-component installations
- Verify graceful degradation in non-TTY environments (CI, redirected output)

## Open Questions

None - design is straightforward enhancement to existing architecture.
