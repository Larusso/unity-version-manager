# uvm_detect

A Rust library for detecting Unity projects and extracting Unity version information from project directories.

## Overview

`uvm_detect` provides utilities to:
- 🔍 **Detect Unity projects** by searching for Unity project structure markers
- 📋 **Extract Unity version information** from ProjectVersion.txt files
- 🔄 **Support configurable recursive searching** with depth control
- ✅ **Builder pattern API** similar to `std::fs::OpenOptions` for clean configuration
- 🎯 **Follow Rust conventions** with proper error handling and idiomatic patterns

## Features

- **Builder Pattern API**: Clean, chainable configuration similar to `std::fs::OpenOptions`
- **Project Detection**: Automatically locate Unity projects in directories
- **Version Parsing**: Extract Unity editor versions from project files
- **Flexible Search**: Support both current-directory and recursive searching
- **Depth Control**: Configure maximum search depth to prevent excessive traversal
- **Robust Error Handling**: Distinguish between different failure modes
- **Cross-Platform**: Works consistently on Windows, macOS, and Linux
- **Minimal Dependencies**: Only `unity-version` for version parsing

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
uvm_detect = "0.1.0"
```

## Usage

### Basic Project Detection

```rust
use std::path::Path;
use uvm_detect::DetectOptions;

// Search for Unity project in current directory only
match DetectOptions::new().detect_unity_project_dir(Path::new(".")) {
    Ok(project_path) => {
        println!("Found Unity project at: {}", project_path.display());
    }
    Err(_) => {
        println!("No Unity project found in current directory");
    }
}
```

### Recursive Project Search

```rust
use std::path::Path;
use uvm_detect::DetectOptions;

// Search recursively through subdirectories
match DetectOptions::new()
    .recursive(true)
    .detect_unity_project_dir(Path::new("./projects")) {
    Ok(project_path) => {
        println!("Found Unity project at: {}", project_path.display());
    }
    Err(_) => {
        println!("No Unity project found in projects directory or subdirectories");
    }
}
```

### Recursive Search with Depth Limit

```rust
use std::path::Path;
use uvm_detect::DetectOptions;

// Search recursively but only 3 levels deep
match DetectOptions::new()
    .recursive(true)
    .max_depth(3)
    .detect_unity_project_dir(Path::new("./projects")) {
    Ok(project_path) => {
        println!("Found Unity project at: {}", project_path.display());
    }
    Err(_) => {
        println!("No Unity project found within 3 directory levels");
    }
}
```

### Extract Unity Version

```rust
use std::path::Path;
use uvm_detect::DetectOptions;

// Get Unity version from a project directory
match DetectOptions::new()
    .recursive(true)
    .max_depth(5)
    .detect_project_version(Path::new("./my-unity-project")) {
    Ok(version) => {
        println!("Unity version: {}", version);
    }
    Err(e) => {
        println!("Failed to detect Unity version: {}", e);
    }
}
```

### Convenience Functions

For simple use cases, convenience functions are available:

```rust
use std::path::Path;
use uvm_detect::{detect_unity_project_dir, detect_project_version, try_get_project_version};

// Simple project detection (non-recursive)
if let Ok(project_path) = detect_unity_project_dir(Path::new(".")) {
    println!("Found Unity project at: {}", project_path.display());
}

// Simple version detection (non-recursive)
if let Ok(version) = detect_project_version(Path::new("./my-unity-project")) {
    println!("Unity version: {}", version);
}

// Check if a directory contains Unity project structure
if let Some(version_file_path) = try_get_project_version(Path::new("./suspected-unity-project")) {
    println!("Found Unity project! ProjectVersion.txt at: {}", version_file_path.display());
} else {
    println!("Not a Unity project directory");
}
```

## Unity Project Structure

This library detects Unity projects by looking for the standard Unity project structure:

```
UnityProject/
├── ProjectSettings/
│   └── ProjectVersion.txt    ← This file is the detection marker
├── Assets/
├── Library/
└── ...
```

The `ProjectVersion.txt` file contains Unity editor version information in formats like:

```
m_EditorVersion: 2021.3.16f1
m_EditorVersionWithRevision: 2021.3.16f1 (4016570cf34f)
```

## API Reference

### DetectOptions

The main configuration struct that provides a builder-style API for configuring Unity project detection.

#### Methods

##### `DetectOptions::new() -> Self`

Creates a new `DetectOptions` with default settings:
- `recursive`: false (search only in specified directory)
- `max_depth`: u32::MAX (unlimited depth when recursive)
- `case_sensitive`: true (for future use)

##### `recursive(&mut self, recursive: bool) -> &mut Self`

Sets whether to search recursively through subdirectories.

##### `max_depth(&mut self, max_depth: u32) -> &mut Self`

Sets the maximum depth to search when recursive search is enabled. Use `u32::MAX` for unlimited depth.

##### `case_sensitive(&mut self, case_sensitive: bool) -> &mut Self`

Sets whether to use case-sensitive path matching (reserved for future functionality).

##### `detect_unity_project_dir(&self, dir: &Path) -> io::Result<PathBuf>`

Detects a Unity project directory using the configured options.

##### `detect_project_version(&self, dir: &Path) -> io::Result<Version>`

Detects and parses the Unity version from a Unity project using the configured options.

### Convenience Functions

#### `detect_unity_project_dir(dir: &Path) -> io::Result<PathBuf>`

Detects a Unity project directory using default options (non-recursive search).

#### `detect_project_version(project_path: &Path) -> io::Result<Version>`

Detects and parses the Unity version from a Unity project using default options.

#### `try_get_project_version<P: AsRef<Path>>(base_dir: P) -> Option<PathBuf>`

Attempts to get the path to the Unity ProjectVersion.txt file if it exists.

## Configuration Examples

### Complex Configuration

```rust
use std::path::Path;
use uvm_detect::DetectOptions;

let result = DetectOptions::new()
    .recursive(true)                // Enable recursive search
    .max_depth(10)                  // Limit to 10 directory levels
    .case_sensitive(true)           // Use case-sensitive matching (future)
    .detect_unity_project_dir(Path::new("./workspace"));

match result {
    Ok(project_path) => {
        println!("Found Unity project at: {}", project_path.display());
        
        // Now get the version from the found project
        let version = DetectOptions::new()
            .detect_project_version(&project_path)?;
        println!("Unity version: {}", version);
    }
    Err(e) => println!("No Unity project found: {}", e),
}
```

### Performance Considerations

```rust
// For large directory trees, limit depth to improve performance
let result = DetectOptions::new()
    .recursive(true)
    .max_depth(5)  // Only search 5 levels deep
    .detect_unity_project_dir(Path::new("./large-workspace"));
```

## Error Handling

The library uses Rust's standard error handling patterns:

- **`io::Result<T>`** for operations that can fail due to I/O errors or missing projects
- **`Option<T>`** for simple existence checks where absence isn't an error
- **Specific error messages** that help identify the type of failure

Common error scenarios:
- **NotFound**: No Unity project found in the specified directory (or within max_depth if recursive)
- **InvalidInput**: ProjectVersion.txt found but contains invalid version information
- **I/O errors**: Permission denied, file not accessible, etc.

## Testing

Run the test suite:

```bash
cargo test
```

The library includes comprehensive tests covering:
- ✅ Valid Unity project detection
- ❌ Non-Unity directory handling
- 🔄 Recursive search functionality
- 📏 Max depth limiting
- 📋 Version parsing with different formats
- 🚫 Malformed version handling
- 📁 Nested project structures
- 🛠️ Builder pattern chaining
- 🎯 Convenience functions

## Performance

- **Efficient traversal**: Stops at first Unity project found
- **Depth limiting**: Prevents excessive directory traversal
- **Early termination**: Returns immediately when project is found
- **Minimal allocations**: Uses efficient path handling

## Dependencies

- `unity-version` - For parsing Unity version strings
- `tempfile` (dev-dependency) - For creating temporary test directories

## License

This project is part of the Unity Version Manager (UVM) toolkit.

## Contributing

Contributions are welcome! Please ensure that:
- All tests pass (`cargo test`)
- Documentation tests compile (`cargo test --doc`)
- Code follows Rust conventions
- New features include appropriate tests
- Documentation is updated for API changes

## Related Projects

This crate is part of the larger Unity Version Manager ecosystem:
- `unity-version` - Unity version parsing and manipulation
- `uvm_install` - Unity installation management  
- `uvm` - Main CLI application

## Changelog

### v0.1.0
- Initial release with builder pattern API
- Support for recursive search with depth limiting
- Cross-platform compatibility
- Comprehensive test coverage
- Clean separation of concerns between detection and version parsing