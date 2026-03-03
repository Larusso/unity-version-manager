---
name: uvm-test-install
description: Install Unity to a temporary directory for testing. Use this to see real installation output and verify installer behavior.
license: MIT
metadata:
  author: larusso
  version: "1.0"
---

Use this skill to perform a real Unity installation to a temporary directory. This lets you observe actual installer output, verify fixes, and debug installation issues.

## Basic Usage

```bash
# Create a temp directory and install
TEMP_UNITY=$(mktemp -d)
cargo run --bin uvm -- install <version> "$TEMP_UNITY"

# Example: Install Unity 2022.3.0f1 to temp
TEMP_UNITY=$(mktemp -d)
cargo run --bin uvm -- install 2022.3.0f1 "$TEMP_UNITY"
```

## With Modules

```bash
# Install with specific modules
TEMP_UNITY=$(mktemp -d)
cargo run --bin uvm -- install <version> -m <module> "$TEMP_UNITY"

# Example: Install with Android support
TEMP_UNITY=$(mktemp -d)
cargo run --bin uvm -- install 2022.3.0f1 -m android "$TEMP_UNITY"
```

## With Verbose Logging

```bash
# Enable debug logging to see detailed installation steps
RUST_LOG=debug cargo run --bin uvm -- install <version> "$TEMP_UNITY"

# Or trace level for maximum detail
RUST_LOG=trace cargo run --bin uvm -- install <version> "$TEMP_UNITY"
```

## Cleanup

After testing, clean up the temporary installation:

```bash
rm -rf "$TEMP_UNITY"
```

## When to Use

- **Verifying a bug fix**: See if changes to installer code work correctly
- **Understanding error messages**: Observe real failure modes
- **Testing module installation**: Verify module dependencies resolve correctly
- **Debugging platform-specific issues**: See actual extraction/installation output

## Cautions

- **Downloads are large**: Unity editor is several GB; modules add more
- **Takes time**: Full installation can take 10+ minutes depending on network
- **Disk space**: Ensure temp directory has enough space (5-10 GB minimum)
- **Consider using small modules**: Language packs (.po files) are tiny and fast for quick tests

## Quick Test with Minimal Download

For a fast test of the installation flow, install just a language pack:

```bash
# First install editor (required base)
TEMP_UNITY=$(mktemp -d)
cargo run --bin uvm -- install 2022.3.0f1 "$TEMP_UNITY"

# Then add a small language module
cargo run --bin uvm -- install 2022.3.0f1 -m language-ja "$TEMP_UNITY"
```

## Architecture Selection (macOS)

```bash
# Force x86_64 architecture
cargo run --bin uvm -- install <version> --architecture x86-64 "$TEMP_UNITY"

# Force ARM64 architecture
cargo run --bin uvm -- install <version> --architecture arm64 "$TEMP_UNITY"
```
