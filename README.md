unity-version-manager
=====================

A command-line application to manage unity versions.
This tool allows to install and manage multiple unity versions on a system from the command-line. This tool is compatible with Unity-Hub and will use the installation destination configured there by default.

[![Build Status](https://img.shields.io/travis/Larusso/unity-version-manager/master.svg?logo=travis)](https://travis-ci.org/Larusso/unity-version-manager)
[![Build status](https://ci.appveyor.com/api/projects/status/ev6ms6wgo8jmeym0/branch/master?svg=true)](https://ci.appveyor.com/project/Larusso/unity-version-manager/branch/master)
[![License](https://img.shields.io/github/license/Larusso/unity-version-manager.svg)](https://github.com/Larusso/unity-version-manager/blob/master/LICENSE)
![](https://img.shields.io/github/issues/Larusso/unity-version-manager.svg)
[![Latest Release](https://img.shields.io/github/release/Larusso/unity-version-manager.svg)](https://github.com/Larusso/unity-version-manager/releases/latest)
![macOS-supported](https://img.shields.io/badge/macOS-supported-brightgreen.svg)
![windows-experimental](https://img.shields.io/badge/windows-experimental-blue.svg)
![linux-experimental](https://img.shields.io/badge/linux-experimental-blue.svg)


Installation
------------

_install with brew_

```bash
brew tap wooga/tools
brew install wooga/api-version-manager
```

To build from source a recent version of rust is needed `> 1.30`. You should use [rustup].

_install from source with cmake_

```bash
git clone git@github.com:Larusso/api-version-manager.git
cd api-version-manager
make install
```

_install from source with cargo_

```bash
git clone git@github.com:Larusso/api-version-manager.git
cd api-version-manager
cargo build --release
#symlink or move binaries in target/release
```

Usage
-----

The _uvm_ (unity-version-manager) is a collection of small command-line tools. Each command can be invoked through the main tool `uvm`.

### Version handling

The main purpose of uvm was the management of multiple unity installations on macOS. The idea was to have a similar interface as [rvm] to activate and deactivate different unity installations. This is done by creating a symlink at the default unity installation location (`/Applications/Unity` on macOS).

| command        | description |
| -------------- | ----------- |
| use            | Use specific version of unity. |
| clear          | Remove the link so you can install a new version without overwriting. |
| current        | Prints current activated version of unity. |
| list           | List installed unity versions |

### Version installation

These commands allow the installation and deinstallation of Unity versions with additional components.

| command        | description |
| -------------- | ----------- |
| install        | Install specified unity version. |
| uninstall      | Uninstall specified unity version |
| versions       | List available Unity versions to install. |

### Miscellaneous commands

| command        | description |
| -------------- | ----------- |
| detect         | Find which version of unity was used to generate a project |
| launch         | Launch the current active version of unity. |
| help           | Prints help page for command. |

License
=======

[Apache License 2.0](LICENSE)

[rvm]:      https://rvm.io/
[rustup]:   https://rustup.rs/
