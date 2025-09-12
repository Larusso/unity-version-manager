unity-version-manager
=====================

A command-line application to manage unity versions.
This tool allows to install and manage multiple unity versions on a system from the command-line. This tool is compatible with Unity-Hub and will use the installation destination configured there by default.



Installation
------------

_install with cargo_

```bash
cargo install unity-version-manager
```

_install from source with cmake_

```bash
git clone git@github.com:Larusso/unity-version-manager.git
cd unity-version-manager
make install
```

_install from source with cargo_

```bash
git clone git@github.com:Larusso/unity-version-manager.git
cd unity-version-manager
cargo install --path ./uvm
```

Usage
-----

The _uvm_ (unity-version-manager) is a command-line tool for managing Unity installations and projects.

```bash
uvm [OPTIONS] <COMMAND>
```

### Core Commands

| Command | Description |
| ------- | ----------- |
| **install** | Install specified Unity version with optional modules |
| **uninstall** | Uninstall Unity version or specific modules |
| **list** | List installed Unity versions (from Hub, system, or all) |
| **launch** | Launch Unity with a project, optionally with specific build platform |

### Project & Version Management

| Command | Description |
| ------- | ----------- |
| **detect** | Find which Unity version was used to create a project |
| **modules** | List available modules for a specific Unity version |
| **version** | Unity version utilities (latest, matching version requirements) |

### Detailed Command Usage

#### Install Unity
```bash
# Install specific Unity version
uvm install 2023.1.4f1

# Install with additional modules
uvm install 2023.1.4f1 --module android --module ios

# Install to custom location
uvm install 2023.1.4f1 /path/to/install

# Install with sync modules (dependencies)
uvm install 2023.1.4f1 --module android --with-sync
```

#### List Unity Installations
```bash
# List Unity Hub installations (default)
uvm list

# List all Unity installations
uvm list --all

# List system installations only
uvm list --system

# Show path only
uvm list --path
```

#### Launch Unity Projects
```bash
# Launch Unity with current directory as project
uvm launch

# Launch specific project
uvm launch /path/to/project

# Launch with specific platform
uvm launch /path/to/project --platform android

# Auto-detect project and use its Unity version
uvm launch --force-project-version
```

#### Version Management
```bash
# Get latest LTS version
uvm version latest --stream lts

# Find versions matching requirement
uvm version matching ">=2023.1"

# List modules for specific version
uvm modules 2023.1.4f1

# List modules by category
uvm modules 2023.1.4f1 --category platforms
```

### Global Options

| Option | Description |
| ------ | ----------- |
| `-d, --debug` | Print debug output |
| `-v, --verbose` | Print more output (can be repeated) |
| `-c, --color` | Control color output: auto, always, never |
| `-h, --help` | Print help information |
| `-V, --version` | Print version information |

Development
===========

### Workspace Structure

This project uses a Cargo workspace with multiple crates:

| Crate | Description |
| ----- | ----------- |
| `uvm` | Main CLI application (produces `uvm` binary) |
| `unity-version` | Unity version parsing and management |
| `unity-hub` | Unity Hub integration |
| `unity-types` | Base Unity data types |
| `uvm_install` | Unity installation logic |
| `uvm_live_platform` | Unity release platform API |
| `uvm_install_graph` | Installation dependency graph |
| `uvm_move_dir` | Cross-platform directory operations |

### Building from Source

```bash
git clone git@github.com:Larusso/unity-version-manager.git
cd unity-version-manager
cargo build --workspace
```

### Running Tests

```bash
cargo test --workspace
```

### Running Development Version

```bash
# Run the uvm binary directly
cargo run --bin uvm -- --help

# Install locally for testing
cargo install --path ./uvm
```

License
=======

[Apache License 2.0](LICENSE)

[rvm]:      https://rvm.io/
[rustup]:   https://rustup.rs/
