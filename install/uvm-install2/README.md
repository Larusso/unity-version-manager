uvm-install2
============

This is a new installer which aims to have 100% compatibility with Unity Hub.

Usage
-----

*cli*:

```
USAGE:
    uvm-install2 [FLAGS] [OPTIONS] <version> [--] [destination]

FLAGS:
    -d, --debug
            print debug output
    -h, --help
            Prints help information
        --with-sync
            install also synced modules
    -V, --version
            Prints version information
    -v, --verbose
            print more output

OPTIONS:
        --color <color>
            Coloring [default: auto]  [possible values: auto, always, never]
    -m, --module <module>...
            A support module to install. You can list all awailable
            modules for a given version using `uvm-modules`
ARGS:
    <version>
            The unity version to install in the form of `2018.1.0f3`
    <destination>
            A directory to install the requested version to
```

*lib crate*

```rust
use uvm_install2::unity::{Component, Version};
let version = Version::b(2019, 3, 0, 8);
let components = [Component::Android, Component::Ios];
let install_sync_modules = true
uvm_install2::install(&version, Some(&components), install_sync_modules, Some("/install/path"))?;
```
