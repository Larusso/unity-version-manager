# Unity Version Manager - Installer Logic Research

## Executive Summary

The UVM installer is a sophisticated cross-platform system for installing Unity Editor versions and their modules. It fetches release metadata from Unity's GraphQL API, constructs a dependency graph, downloads installer packages, and extracts them using platform-specific strategies.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                          CLI Layer (uvm)                            │
│                    uvm/src/commands/install.rs                      │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Installation Orchestration                        │
│                      uvm_install/src/lib.rs                         │
│  ┌──────────────┐  ┌────────────────┐  ┌──────────────────────┐    │
│  │InstallOptions│  │InstallGraph   │  │install_module_and_   │    │
│  │  (Builder)   │──│ (DAG Walker)  │──│   dependencies()     │    │
│  └──────────────┘  └────────────────┘  └──────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
         │                    │                      │
         ▼                    ▼                      ▼
┌─────────────────┐ ┌─────────────────┐ ┌──────────────────────────────┐
│ Live Platform   │ │  Install Graph  │ │     Platform Installers      │
│ (API Client)    │ │ (Dependency Res)│ │     uvm_install/src/sys/     │
│                 │ │                 │ │  ┌───────┬────────┬───────┐  │
│ ┌─────────────┐ │ │ ┌─────────────┐ │ │  │macOS  │ Linux  │Windows│  │
│ │FetchRelease │ │ │ │  DAG with   │ │ │  ├───────┼────────┼───────┤  │
│ │(GraphQL API)│ │ │ │  Install    │ │ │  │.pkg   │.xz     │.exe   │  │
│ └─────────────┘ │ │ │  Status     │ │ │  │.dmg   │.zip    │.msi   │  │
│                 │ │ └─────────────┘ │ │  │.zip   │.pkg    │.zip   │  │
└─────────────────┘ └─────────────────┘ │  │.po    │.po     │.po    │  │
                                        │  └───────┴────────┴───────┘  │
                                        └──────────────────────────────┘
```

---

## Core Components

### 1. InstallOptions (Entry Point)

**Location**: `uvm_install/src/lib.rs:127-376`

The `InstallOptions` struct serves as a builder for installation configuration:

```rust
pub struct InstallOptions {
    version: Version,
    requested_modules: HashSet<String>,
    install_sync: bool,           // Include synced/optional dependencies
    destination: Option<PathBuf>, // Custom install location
    architecture: Option<InstallArchitecture>,  // x86_64 or arm64
}
```

**Execution Flow** (`install()` method):

1. **Acquire Lock**: Uses file-based locking (`locks_dir().join("<version>.lock")`) to prevent concurrent installations of the same version
2. **Fetch Release**: Calls Unity's GraphQL API to get release metadata
3. **Build Dependency Graph**: Creates `InstallGraph` from the release
4. **Detect Existing Installation**: Checks if Unity is already installed at destination
5. **Architecture Check** (macOS ARM64 only): Verifies binary architecture matches system
6. **Resolve Dependencies**: Collects all required modules including transitive dependencies
7. **Filter Graph**: Removes unrequested modules from the graph
8. **Install Components**: Walks the graph in topological order, installing each component
9. **Update Module Manifest**: Writes installed modules to Unity's module list

### 2. Release Fetching (Live Platform API)

**Location**: `uvm_live_platform/src/api/fetch_release.rs`

Uses Unity's GraphQL API at `https://live-platform-api.prd.ld.unity3d.com/graphql`:

```
┌────────────────────────────────────────────────────────────────┐
│                    FetchReleaseBuilder                          │
├────────────────────────────────────────────────────────────────┤
│  .with_current_platform()  → MAC_OS / LINUX / WINDOWS          │
│  .with_architecture()      → X86_64 / ARM64                    │
│  .with_extended_lts()      → Include XLTS versions             │
│  .with_u7_alpha()          → Include Unity 7 alpha             │
│  .fetch()                  → Execute GraphQL query             │
└────────────────────────────────────────────────────────────────┘
```

**Response Structure** (`Release`):
- `version`: Version string (e.g., "2022.3.0f1")
- `short_revision`: Git revision hash
- `downloads[]`: Platform-specific download info
  - `release_file`: URL and integrity hash
  - `modules[]`: Available add-on modules (recursive structure)

**Caching**: 7-day cache by default, controlled via `UVM_LIVE_PLATFORM_FETCH_RELEASE_CACHE_*` env vars.

### 3. Dependency Resolution (Install Graph)

**Location**: `uvm_install_graph/src/lib.rs`

Uses `daggy` (DAG library built on `petgraph`) to model the module dependency tree:

```
                    ┌──────────┐
                    │  Editor  │
                    └────┬─────┘
           ┌─────────────┼─────────────┐
           ▼             ▼             ▼
      ┌────────┐   ┌──────────┐   ┌────────┐
      │Android │   │   iOS    │   │ WebGL  │
      └────┬───┘   └──────────┘   └────────┘
           │
    ┌──────┴──────┐
    ▼             ▼
┌───────┐   ┌──────────┐
│SDK/NDK│   │OpenJDK   │
└───────┘   └──────────┘
```

**Key Operations**:
- `from(release)`: Builds DAG from release metadata
- `mark_installed(components)`: Marks already-installed components
- `keep(requested_modules)`: Filters graph to only requested components
- `topo()`: Returns topological traversal for ordered installation
- `get_dependend_modules()`: Walks parent chain (dependencies)
- `get_sub_modules()`: Walks child chain (optional sub-dependencies)

**Install Status**:
- `Unknown`: Initial state
- `Missing`: Needs to be installed
- `Installed`: Already present

### 4. Download/Load System

**Location**: `uvm_install/src/install/loader.rs`

The `Loader` handles downloading installer packages:

```
┌─────────────────────────────────────────────────────────────┐
│                      Download Flow                           │
├─────────────────────────────────────────────────────────────┤
│  1. Calculate cache paths:                                   │
│     - installer_dir: ~/.cache/uvm/installer/<ver>-<rev>/    │
│     - temp_dir: ~/.cache/uvm/tmp/<ver>-<rev>/               │
│                                                              │
│  2. Check for cached installer:                              │
│     - If exists & checksum matches → return cached path      │
│     - If checksum mismatch → delete and re-download          │
│                                                              │
│  3. Download with resume support:                            │
│     - Uses HTTP Range headers for partial downloads          │
│     - Saves as .part file, renames on completion             │
│                                                              │
│  4. Verify integrity (ssri/SRI hash)                        │
└─────────────────────────────────────────────────────────────┘
```

**Features**:
- **Resume Support**: Checks existing `.part` file size, requests with `Range: bytes=N-`
- **Integrity Verification**: Uses SRI (Subresource Integrity) hashes
- **Process Locking**: Per-file locks prevent concurrent downloads of same package

---

## Platform-Specific Installers

### InstallHandler Trait

**Location**: `uvm_install/src/install/mod.rs:15-45`

```rust
pub trait InstallHandler {
    fn install_handler(&self) -> Result<(), InstallerError>;  // Core logic
    fn installer(&self) -> &Path;                              // Source file
    fn error_handler(&self) {}                                 // Cleanup on error
    fn before_install(&self) -> Result<()> { Ok(()) }         // Pre-install hook
    fn after_install(&self) -> Result<()> { Ok(()) }          // Post-install hook

    fn install(&self) -> Result<()> {
        self.before_install()?;
        self.install_handler().map_err(|e| { self.error_handler(); e })?;
        self.after_install()?;
        Ok(())
    }
}
```

---

### macOS Installers

**Location**: `uvm_install/src/sys/mac/`

#### Supported Formats

| Format | Editor | Module | Handler |
|--------|--------|--------|---------|
| `.pkg` | ✓ | ✓ | `EditorPkgInstaller`, `ModulePkgInstaller`, `ModulePkgNativeInstaller` |
| `.dmg` | ✗ | ✓ | `ModuleDmgInstaller`, `ModuleDmgWithDestinationInstaller` |
| `.zip` | ✗ | ✓ | `ModuleZipInstaller` |
| `.po`  | ✗ | ✓ | `ModulePoInstaller` (language files) |

#### PKG Installation Flow (mac/pkg.rs)

```
┌────────────────────────────────────────────────────────────────────┐
│                    PKG Installation (Editor)                        │
├────────────────────────────────────────────────────────────────────┤
│  1. before_install():                                               │
│     - Remove existing destination directory                         │
│                                                                      │
│  2. install_handler():                                               │
│     ┌────────────────┐                                              │
│     │ xar -x -f pkg  │  Extract PKG using macOS xar                 │
│     │ -C tmp_dest    │                                              │
│     └───────┬────────┘                                              │
│             ▼                                                        │
│     ┌────────────────┐                                              │
│     │ find_payload() │  Locate *.pkg.tmp/Payload or Payload~        │
│     └───────┬────────┘                                              │
│             ▼                                                        │
│     ┌────────────────┐                                              │
│     │ tar -zmxf      │  Extract gzipped tar payload                 │
│     └───────┬────────┘                                              │
│             ▼                                                        │
│     ┌────────────────┐                                              │
│     │ cleanup_editor │  Move Unity/ contents up, remove temp        │
│     └────────────────┘                                              │
└────────────────────────────────────────────────────────────────────┘
```

**Module PKG Variants**:
- `ModulePkgInstaller`: Extracts to specific destination using xar/tar
- `ModulePkgNativeInstaller`: Uses `sudo installer -package <pkg> -target /` for system-level packages

#### DMG Installation Flow (mac/dmg.rs)

```
┌────────────────────────────────────────────────────────────────────┐
│                    DMG Installation                                 │
├────────────────────────────────────────────────────────────────────┤
│  1. Mount DMG using `dmg` crate (Attach::new().with())              │
│                                                                      │
│  2. Find .app bundle in mounted volume                              │
│                                                                      │
│  3. Copy .app to destination using `cp -a`                          │
│                                                                      │
│  4. DMG auto-unmounts when Attach drops                             │
└────────────────────────────────────────────────────────────────────┘
```

#### Architecture Check (mac/arch.rs)

**Only on macOS ARM64 systems**, when `UVM_ARCHITECTURE_CHECK_ENABLED=true`:

```
┌────────────────────────────────────────────────────────────────────┐
│               Architecture Verification (ARM64 Mac)                 │
├────────────────────────────────────────────────────────────────────┤
│  For Unity >= 2021.2.0f1:                                           │
│                                                                      │
│  1. Read Mach-O binary from Unity.app/Contents/MacOS/Unity         │
│                                                                      │
│  2. Parse architectures (handles Fat binaries with multiple archs)  │
│                                                                      │
│  3. Compare against system arch (sysctl hw.machine)                │
│                                                                      │
│  4. If mismatch: reinstall with correct architecture               │
│     - Preserves installed modules list                              │
│     - Deletes existing installation & installer cache              │
│     - Downloads ARM64 version                                       │
└────────────────────────────────────────────────────────────────────┘
```

---

### Linux Installers

**Location**: `uvm_install/src/sys/linux/`

#### Supported Formats

| Format | Editor | Module | Handler |
|--------|--------|--------|---------|
| `.xz`  | ✓ | ✓ | `EditorXzInstaller`, `ModuleXzInstaller` |
| `.zip` | ✓ | ✓ | `EditorZipInstaller`, `ModuleZipInstaller` |
| `.pkg` | ✗ | ✓ | `ModulePkgInstaller` |
| `.po`  | ✗ | ✓ | `ModulePoInstaller` |

#### XZ Installation Flow (linux/xz.rs)

```
┌────────────────────────────────────────────────────────────────────┐
│                    XZ/Tar Installation                              │
├────────────────────────────────────────────────────────────────────┤
│  tar -C <destination> -amxf <installer.tar.xz>                     │
│                                                                      │
│  Flags:                                                              │
│    -a : Auto-detect compression (xz in this case)                  │
│    -m : Don't extract modification time                            │
│    -x : Extract                                                     │
│    -f : File to extract                                             │
├────────────────────────────────────────────────────────────────────┤
│  QUIRK: PlaybackEngines destination adjustment                      │
│                                                                      │
│  If destination ends with "Editor/Data/PlaybackEngines":           │
│    → Navigate 3 levels up to find correct root                     │
│    → Tar extracts with full internal path structure                │
│                                                                      │
│  If destination ends with "Editor/Data/PlaybackEngines/iOSSupport":│
│    → Strip iOSSupport suffix first, then apply above logic         │
└────────────────────────────────────────────────────────────────────┘
```

#### PKG on Linux (linux/pkg.rs)

Linux can install macOS-style `.pkg` files using 7z + gzip + cpio:

```
┌────────────────────────────────────────────────────────────────────┐
│                    PKG on Linux (via 7z)                           │
├────────────────────────────────────────────────────────────────────┤
│  1. Extract PKG:  7z x -y -o<dest> <installer.pkg>                 │
│                                                                      │
│  2. Find payload: Payload or Payload~ file                          │
│                                                                      │
│  3. Extract payload:                                                 │
│     ┌─────────────────────────────────────────────────┐            │
│     │ If Payload~:                                     │            │
│     │   cat Payload~ | cpio -iu                       │            │
│     │                                                  │            │
│     │ If Payload (gzipped):                           │            │
│     │   gzip -dc Payload | cpio -iu                   │            │
│     └─────────────────────────────────────────────────┘            │
└────────────────────────────────────────────────────────────────────┘
```

**External Dependency**: Requires `7z` command (p7zip package).

---

### Windows Installers

**Location**: `uvm_install/src/sys/win/`

#### Supported Formats

| Format | Editor | Module | Handler |
|--------|--------|--------|---------|
| `.exe` | ✓ | ✓ | `EditorExeInstaller`, `ModuleExeInstaller`, `ModuleExeTargetInstaller` |
| `.msi` | ✗ | ✓ | `ModuleMsiInstaller` |
| `.zip` | ✗ | ✓ | `ModuleZipInstaller` |
| `.po`  | ✗ | ✓ | `ModulePoInstaller` |

#### EXE Installation Flow (win/exe.rs)

```
┌────────────────────────────────────────────────────────────────────┐
│                    EXE Installation (Windows)                       │
├────────────────────────────────────────────────────────────────────┤
│  Creates temporary .cmd script:                                     │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ ECHO OFF                                                     │   │
│  │ CALL "<installer.exe>" /S /D=<destination>                  │   │
│  │                                                              │   │
│  │ Or with custom UI flag for editor:                          │   │
│  │ CALL "<installer.exe>" -UI=reduced /D=<destination>         │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  Executes script and waits for completion                          │
├────────────────────────────────────────────────────────────────────┤
│  QUIRK: Editor gets "-UI=reduced" for minimal UI during install    │
│  QUIRK: Modules without destination use just /S (silent)           │
└────────────────────────────────────────────────────────────────────┘
```

#### MSI Installation Flow (win/msi.rs)

```
┌────────────────────────────────────────────────────────────────────┐
│                    MSI Installation                                 │
├────────────────────────────────────────────────────────────────────┤
│  Uses command string from module metadata:                         │
│                                                                      │
│  Original: msiexec /i <placeholder>                                 │
│  Modified: msiexec /i "<installer.msi>"                            │
│                                                                      │
│  Wrapped in .cmd script and executed                               │
└────────────────────────────────────────────────────────────────────┘
```

#### Windows Path Handling (utils.rs)

```
┌────────────────────────────────────────────────────────────────────┐
│               Long Path Support (Windows-specific)                  │
├────────────────────────────────────────────────────────────────────┤
│  prepend_long_path_support():                                       │
│                                                                      │
│  Input:  c:/path/to/file.txt                                        │
│  Output: \\?\c:\path\to\file.txt                                   │
│                                                                      │
│  - Handles paths > 260 characters                                   │
│  - Converts forward slashes to backslashes                         │
│  - Skips already-prefixed verbatim paths                           │
└────────────────────────────────────────────────────────────────────┘
```

#### Windows Move Directory (uvm_move_dir)

Uses Win32 `MoveFileW` API instead of Rust's `fs::rename` for cross-volume moves:

```rust
// win_move_file.rs
winbase::MoveFileW(from.as_ptr(), to.as_ptr())
```

---

## Shared Installers (Cross-Platform)

### ZIP Installer (installer/zip.rs)

Uses the `zip` crate for pure-Rust extraction:

```
┌────────────────────────────────────────────────────────────────────┐
│                    ZIP Extraction                                   │
├────────────────────────────────────────────────────────────────────┤
│  Features:                                                          │
│  - In-memory extraction using zip crate                            │
│  - Preserves Unix permissions (chmod) on Unix systems              │
│  - Supports rename mapping (from → to path prefix replacement)     │
│  - Creates directories as needed                                    │
└────────────────────────────────────────────────────────────────────┘
```

### PO Installer (installer/po.rs)

Simple file copy for language pack `.po` files:

```
┌────────────────────────────────────────────────────────────────────┐
│                    PO File Installation                            │
├────────────────────────────────────────────────────────────────────┤
│  1. Extract filename from source path                              │
│  2. Create destination directory if needed                         │
│  3. Copy file to destination/<filename>                            │
└────────────────────────────────────────────────────────────────────┘
```

---

## Module Rename/Destination Logic

**Location**: `uvm_install/src/lib.rs:437-472`

Unity modules can specify:
- **destination**: Where to install (relative to Unity base, uses `{UNITY_PATH}` placeholder)
- **extracted_path_rename**: Rename files after extraction

```rust
fn strip_unity_base_url<P: AsRef<Path>, Q: AsRef<Path>>(path: P, base_dir: Q) -> PathBuf {
    // Replaces "{UNITY_PATH}" with actual base directory
    base_dir.join(path.strip_prefix("{UNITY_PATH}").unwrap_or(path))
}
```

**Special Case - iOS Module**:
```rust
// In Module::destination()
if &self.id == "ios" {
    self.destination.map(|d| format!("{}/iOSSupport", d))
}
```

---

## Process Locking

**Location**: `uvm_install/src/install/utils.rs:12-40`

Uses `cluFlock` for file-based process locking:

```rust
macro_rules! lock_process {
    ($lock_path:expr) => {
        let lock_file = fs::File::create($lock_path)?;
        let _lock = utils::lock_process_or_wait(&lock_file)?;
    };
}
```

**Lock Types**:
1. **Version Lock**: `~/.unity/locks/<version>.lock` - Prevents concurrent installs of same version
2. **Download Lock**: `~/.cache/uvm/tmp/<ver>/<file>.lock` - Prevents concurrent downloads of same file

**Behavior**: If lock is held, waits for it to be released rather than failing.

---

## Error Handling

**Error Types** (`uvm_install/src/error.rs`):

| Error | Description |
|-------|-------------|
| `ReleaseLoadFailure` | Failed to fetch from Unity API |
| `LockProcessFailure` | Can't acquire installation lock |
| `UnsupportedModule` | Requested module not available for version |
| `LoadingInstallerFailed` | Download or checksum failure |
| `InstallerCreatedFailed` | Can't create appropriate installer for format |
| `InstallFailed` | Installation execution failed |

**Cleanup Strategy**:
- `error_handler()` is called on install failure
- Most installers delete the destination directory on failure
- Temp directories are always cleaned up

---

## Platform Quirks Summary

### macOS
- Uses `xar` for PKG extraction (system command)
- Uses `tar -zmxf` for payload extraction
- Uses `dmg` crate for DMG mounting
- Some modules require `sudo installer` for system-level packages
- ARM64 architecture verification for Unity >= 2021.2.0f1

### Linux
- Uses `7z` for PKG extraction (external dependency)
- Uses `gzip` + `cpio` pipeline for payload extraction
- PlaybackEngines path adjustment for correct extraction location
- Always defaults to x86_64 architecture regardless of system

### Windows
- Uses temp `.cmd` scripts to invoke installers
- Editor installs use `-UI=reduced` flag for minimal UI
- Uses Win32 `MoveFileW` API for reliable directory moves
- Supports `\\?\` long path prefix for > 260 char paths
- MSI modules require command string from module metadata

### All Platforms
- HTTP Range headers for download resumption
- SRI hash verification of downloaded packages
- File-based process locking for concurrent operation safety
- Modules may have `extracted_path_rename` for post-install reorganization

---

## Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           User Request                                   │
│                    uvm install 2022.3.0f1 -m android                    │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          InstallOptions                                  │
│   version: 2022.3.0f1                                                   │
│   modules: ["android"]                                                  │
│   architecture: x86_64                                                  │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                     FetchRelease (GraphQL API)                          │
│   POST https://live-platform-api.prd.ld.unity3d.com/graphql            │
│   Returns: Release with downloads[] and modules[]                       │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          InstallGraph                                    │
│   ┌─────────┐                                                           │
│   │ Editor  │ ◄── Already installed? mark_installed()                  │
│   └────┬────┘                                                           │
│        │                                                                 │
│   ┌────▼────┐                                                           │
│   │ Android │ ◄── Requested, mark as needed                            │
│   └────┬────┘                                                           │
│        │                                                                 │
│   ┌────▼─────┐                                                          │
│   │ SDK/NDK  │ ◄── Dependency (if --with-sync)                         │
│   └──────────┘                                                          │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    For each MISSING component (topo order):            │
│                                                                         │
│   ┌─────────────┐     ┌──────────────────┐     ┌──────────────────┐   │
│   │   Loader    │ ──► │ Platform-specific│ ──► │  InstallHandler  │   │
│   │ (download)  │     │ create_installer │     │    .install()    │   │
│   └─────────────┘     └──────────────────┘     └──────────────────┘   │
│                                                                         │
│   Cache dir: ~/.cache/uvm/installer/<version>-<rev>/                   │
│   Install dir: /Applications/Unity/<version>/ (or custom)              │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      Write Module Manifest                              │
│   Updates modules.json in Unity installation                           │
│   Registers with Unity Hub if custom destination                       │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Configuration & Environment Variables

| Variable | Purpose |
|----------|---------|
| `UVM_ARCHITECTURE_CHECK_ENABLED` | Enable ARM64 binary verification (macOS) |
| `UVM_LIVE_PLATFORM_CACHE_*` | Control API response caching |
| `UVM_LIVE_PLATFORM_FETCH_RELEASE_CACHE_*` | Release-specific cache settings |

---

## External Dependencies

| Platform | Dependency | Used For |
|----------|------------|----------|
| macOS | `xar` | PKG extraction |
| macOS | `tar` | Payload extraction |
| Linux | `7z` (p7zip) | PKG extraction |
| Linux | `gzip` | Payload decompression |
| Linux | `cpio` | Payload extraction |
| Linux | `tar` | XZ extraction |

---

## Future Considerations

1. **Parallel Downloads**: Currently downloads are sequential; could parallelize
2. **Better Progress Reporting**: `ProgressHandler` trait exists but implementation is partial
3. **Checksum Caching**: Currently reads entire file for verification; could use incremental hash
4. **Linux ARM64**: Currently forces x86_64; could support native ARM64 when Unity provides it
