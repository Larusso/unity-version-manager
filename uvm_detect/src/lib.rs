use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
    str::FromStr,
};

use unity_version::Version;


/// Configuration options for Unity project detection.
///
/// This struct provides a builder-style API similar to `std::fs::OpenOptions` for configuring
/// how Unity project detection should behave. Use the builder methods to customize the search
/// behavior, then call the detection methods to perform the actual search.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use uvm_detect::DetectOptions;
///
/// // Basic usage with default options
/// let project_path = DetectOptions::new()
///     .detect_unity_project_dir(Path::new("."))?;
///
/// // Recursive search with custom depth limit
/// let project_path = DetectOptions::new()
///     .recursive(true)
///     .max_depth(3)
///     .detect_unity_project_dir(Path::new("./projects"))?;
///
/// // Get version with custom options
/// let version = DetectOptions::new()
///     .recursive(true)
///     .max_depth(5)
///     .detect_project_version(Path::new("."))?;
/// # Ok::<(), std::io::Error>(())
/// ```
#[derive(Default)]
pub struct DetectOptions {
    /// Whether to search recursively through subdirectories
    pub recursive: bool,
    /// Maximum depth to search when recursive is enabled (u32::MAX = unlimited)
    pub max_depth: u32,
    /// Whether to use case-sensitive path matching
    pub case_sensitive: bool,
}

impl DetectOptions {
    /// Creates a new `DetectOptions` with default settings.
    ///
    /// Default settings:
    /// - `recursive`: false (search only in specified directory)
    /// - `max_depth`: u32::MAX (unlimited depth when recursive)
    /// - `case_sensitive`: true (use case-sensitive path matching)
    pub fn new() -> Self {
        Self {
            recursive: false,
            max_depth: u32::MAX,
            case_sensitive: true,
        }
    }

    /// Sets whether to search recursively through subdirectories.
    ///
    /// When `true`, the search will traverse into subdirectories looking for Unity projects.
    /// When `false`, only the specified directory is searched.
    ///
    /// # Arguments
    ///
    /// * `recursive` - Whether to enable recursive search
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uvm_detect::DetectOptions;
    /// use std::path::Path;
    ///
    /// // Search recursively
    /// let result = DetectOptions::new()
    ///     .recursive(true)
    ///     .detect_unity_project_dir(Path::new("."))?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.recursive = recursive;
        self
    }

    /// Sets the maximum depth to search when recursive search is enabled.
    ///
    /// This limits how deep the recursive search will go. A depth of 1 means only
    /// immediate subdirectories are searched. Use `u32::MAX` for unlimited depth.
    ///
    /// # Arguments
    ///
    /// * `max_depth` - Maximum search depth, or `u32::MAX` for unlimited
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use uvm_detect::DetectOptions;
    /// use std::path::Path;
    ///
    /// // Search recursively but only 2 levels deep
    /// let result = DetectOptions::new()
    ///     .recursive(true)
    ///     .max_depth(2)
    ///     .detect_unity_project_dir(Path::new("."))?;
    ///
    /// // Search with unlimited depth
    /// let result = DetectOptions::new()
    ///     .recursive(true)
    ///     .max_depth(u32::MAX)
    ///     .detect_unity_project_dir(Path::new("."))?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn max_depth(&mut self, max_depth: u32) -> &mut Self {
        self.max_depth = max_depth;
        self
    }


    /// Sets whether to use case-sensitive path matching.
    ///
    /// This affects how file and directory names are compared during search.
    ///
    /// # Arguments
    ///
    /// * `case_sensitive` - Whether to use case-sensitive matching
    ///
    /// # Note
    ///
    /// This option is reserved for future functionality and does not currently affect behavior.
    pub fn case_sensitive(&mut self, case_sensitive: bool) -> &mut Self {
        self.case_sensitive = case_sensitive;
        self
    }

    /// Detects and parses the Unity version from a Unity project.
    ///
    /// This function locates a Unity project directory using the configured options and reads 
    /// the ProjectVersion.txt file to extract Unity editor version information. It prioritizes 
    /// the version with revision information if available, falling back to the basic editor version.
    ///
    /// # Arguments
    ///
    /// * `dir` - The directory path to start searching from
    ///
    /// # Returns
    ///
    /// * `Ok(Version)` - The parsed Unity version if found and valid
    /// * `Err(io::Error)` - An error if the project is not found, the version file cannot be read,
    ///   or the version string cannot be parsed
    ///
    /// # File Format
    ///
    /// The function reads ProjectSettings/ProjectVersion.txt and looks for lines like:
    /// - `m_EditorVersion: 2021.3.16f1`
    /// - `m_EditorVersionWithRevision: 2021.3.16f1 (4016570cf34f)`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use uvm_detect::DetectOptions;
    ///
    /// // Basic version detection
    /// let version = DetectOptions::new()
    ///     .detect_project_version(Path::new("./my-unity-project"))?;
    /// println!("Unity version: {}", version);
    ///
    /// // Recursive search with depth limit
    /// let version = DetectOptions::new()
    ///     .recursive(true)
    ///     .max_depth(3)
    ///     .detect_project_version(Path::new("./projects"))?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn detect_project_version(&self, dir: &Path) -> io::Result<Version> {
        let project_version = self.detect_unity_project_dir(dir).and_then(|p| {
            self.try_get_project_version(p).ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "ProjectVersion.txt not found")
            })
        })?;

        let file = File::open(project_version)?;
        let lines = BufReader::new(file).lines();

        let mut editor_versions: HashMap<&'static str, String> = HashMap::with_capacity(2);

        for line in lines {
            if let Ok(line) = line {
                if line.starts_with("m_EditorVersion: ") {
                    let value = line.replace("m_EditorVersion: ", "");
                    editor_versions.insert("EditorVersion", value.to_owned());
                }

                if line.starts_with("m_EditorVersionWithRevision: ") {
                    let value = line.replace("m_EditorVersionWithRevision: ", "");
                    editor_versions.insert("EditorVersionWithRevision", value.to_owned());
                }
            }
        }

        let v = editor_versions
            .get("EditorVersionWithRevision")
            .or_else(|| editor_versions.get("EditorVersion"))
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Can't parse Unity version")
            })?;
        Version::from_str(&v)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Can't parse Unity version"))
    }

    /// Detects a Unity project directory by searching for Unity project markers.
    ///
    /// This function searches for Unity project directories using the configured options. It looks 
    /// for the presence of a valid Unity project structure (specifically, a ProjectSettings/ProjectVersion.txt file).
    /// The search behavior (recursive vs. current directory only, max depth) is determined by the 
    /// configuration set via the builder methods.
    ///
    /// # Arguments
    ///
    /// * `dir` - The directory path to start searching from
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - The path to the Unity project directory if found
    /// * `Err(io::Error)` - An error if no Unity project is found or if there are I/O issues
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use uvm_detect::DetectOptions;
    ///
    /// // Search for Unity project in current directory only
    /// if let Ok(project_path) = DetectOptions::new().detect_unity_project_dir(Path::new(".")) {
    ///     println!("Found Unity project at: {}", project_path.display());
    /// }
    ///
    /// // Search recursively through subdirectories with depth limit
    /// match DetectOptions::new()
    ///     .recursive(true)
    ///     .max_depth(5)
    ///     .detect_unity_project_dir(Path::new(".")) {
    ///     Ok(project_path) => println!("Found Unity project at: {}", project_path.display()),
    ///     Err(e) => println!("No Unity project found: {}", e),
    /// }
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn detect_unity_project_dir(&self, dir: &Path) -> io::Result<PathBuf> {
        self.detect_unity_project_dir_with_depth(dir, 0)
    }

    fn detect_unity_project_dir_with_depth(&self, dir: &Path, current_depth: u32) -> io::Result<PathBuf> {
        let error = Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Unable to find a Unity project",
        ));

        // Check if the path is a directory
        if !dir.is_dir() {
            return error;
        }

        // Check if this directory contains a Unity project
        if self.try_get_project_version(dir).is_some() {
            return Ok(dir.to_path_buf());
        }

        // If not recursive, or we've hit max depth, stop here
        if !self.recursive || current_depth >= self.max_depth {
            return error;
        }

        // Search subdirectories
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Only recurse into directories
            if path.is_dir() {
                let result = self.detect_unity_project_dir_with_depth(&path, current_depth + 1);
                if result.is_ok() {
                    return result;
                }
            }
        }

        error
    }

    /// Attempts to get the path to the Unity ProjectVersion.txt file if it exists.
    ///
    /// This function constructs the expected path to Unity's ProjectVersion.txt file
    /// within a Unity project structure and checks if the file exists.
    ///
    /// # Arguments
    ///
    /// * `base_dir` - The base directory to check for Unity project structure
    ///
    /// # Returns
    ///
    /// * `Some(PathBuf)` - The path to ProjectSettings/ProjectVersion.txt if it exists
    /// * `None` - If the file doesn't exist (indicating the directory is not a Unity project)
    ///
    /// # Unity Project Structure
    ///
    /// A valid Unity project should have the following structure:
    /// ```text
    /// ProjectRoot/
    /// ├── ProjectSettings/
    /// │   └── ProjectVersion.txt  ← This file is checked
    /// ├── Assets/
    /// └── ...
    /// ```
    fn try_get_project_version<P: AsRef<Path>>(&self, base_dir: P) -> Option<PathBuf> {
        let project_version = base_dir
            .as_ref()
            .join("ProjectSettings")
            .join("ProjectVersion.txt");
        if project_version.exists() {
            Some(project_version)
        } else {
            None
        }
    }
}

/// Detects a Unity project directory using default options.
///
/// This is a convenience function that uses default detection options (non-recursive search).
/// For more control over the search behavior, use `DetectOptions`.
///
/// # Arguments
///
/// * `dir` - The directory path to search in
///
/// # Returns
///
/// * `Ok(PathBuf)` - The path to the Unity project directory if found
/// * `Err(io::Error)` - An error if no Unity project is found
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use uvm_detect::{detect_unity_project_dir, DetectOptions};
///
/// // Simple search in current directory
/// let project_path = detect_unity_project_dir(Path::new("."))?;
///
/// // For recursive search, use DetectOptions
/// let project_path = DetectOptions::new()
///     .recursive(true)
///     .detect_unity_project_dir(Path::new("."))?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn detect_unity_project_dir(dir: &Path) -> io::Result<PathBuf> {
    DetectOptions::new().detect_unity_project_dir(dir)
}

/// Detects and parses the Unity version from a Unity project using default options.
///
/// This is a convenience function that uses default detection options (non-recursive search).
/// For more control over the search behavior, use `DetectOptions`.
///
/// This function locates a Unity project directory and reads the ProjectVersion.txt file
/// to extract the Unity editor version information. It prioritizes the version with
/// revision information if available, falling back to the basic editor version.
///
/// # Arguments
///
/// * `project_path` - The path to start searching for a Unity project
///
/// # Returns
///
/// * `Ok(Version)` - The parsed Unity version if found and valid
/// * `Err(io::Error)` - An error if the project is not found, the version file cannot be read,
///   or the version string cannot be parsed
///
/// # File Format
///
/// The function reads ProjectSettings/ProjectVersion.txt and looks for lines like:
/// - `m_EditorVersion: 2021.3.16f1`
/// - `m_EditorVersionWithRevision: 2021.3.16f1 (4016570cf34f)`
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use uvm_detect::{detect_project_version, DetectOptions};
///
/// // Simple version detection
/// let version = detect_project_version(Path::new("./my-unity-project"))?;
/// println!("Unity version: {}", version);
///
/// // For recursive search with custom options
/// let version = DetectOptions::new()
///     .recursive(true)
///     .max_depth(3)
///     .detect_project_version(Path::new("./projects"))?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn detect_project_version(project_path: &Path) -> io::Result<Version> {
    DetectOptions::new().detect_project_version(project_path)
}

/// Attempts to get the path to the Unity ProjectVersion.txt file if it exists.
/// 
/// Convenience function using default detection options.
pub fn try_get_project_version<P: AsRef<Path>>(base_dir: P) -> Option<PathBuf> {
    DetectOptions::new().try_get_project_version(base_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn create_unity_project(base_dir: &Path, version_content: &str) -> std::io::Result<()> {
        let project_settings = base_dir.join("ProjectSettings");
        fs::create_dir_all(&project_settings)?;

        let version_file = project_settings.join("ProjectVersion.txt");
        fs::write(version_file, version_content)?;

        Ok(())
    }

    #[test]
    fn test_try_get_project_version_valid_project() {
        let temp_dir = TempDir::new().unwrap();
        create_unity_project(temp_dir.path(), "m_EditorVersion: 2021.3.16f1").unwrap();

        let result = try_get_project_version(temp_dir.path());
        assert!(result.is_some());

        let path = result.unwrap();
        assert!(path.ends_with("ProjectSettings/ProjectVersion.txt"));
        assert!(path.exists());
    }

    #[test]
    fn test_try_get_project_version_no_project() {
        let temp_dir = TempDir::new().unwrap();
        // Don't create Unity project structure

        let result = try_get_project_version(temp_dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_try_get_project_version_missing_project_settings() {
        let temp_dir = TempDir::new().unwrap();
        // Create directory but no ProjectSettings
        fs::create_dir(temp_dir.path().join("Assets")).unwrap();

        let result = try_get_project_version(temp_dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_unity_project_dir_current_directory() {
        let temp_dir = TempDir::new().unwrap();
        create_unity_project(temp_dir.path(), "m_EditorVersion: 2021.3.16f1").unwrap();

        let result = detect_unity_project_dir(temp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path());
    }

    #[test]
    fn test_detect_unity_project_dir_not_found_no_recursion() {
        let temp_dir = TempDir::new().unwrap();
        // Create a subdirectory with Unity project, but don't search recursively
        let subdir = temp_dir.path().join("subproject");
        fs::create_dir(&subdir).unwrap();
        create_unity_project(&subdir, "m_EditorVersion: 2021.3.16f1").unwrap();

        let result = detect_unity_project_dir(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_unity_project_dir_recursive_search() {
        let temp_dir = TempDir::new().unwrap();
        // Create a subdirectory with Unity project and search recursively
        let subdir = temp_dir.path().join("subproject");
        fs::create_dir(&subdir).unwrap();
        create_unity_project(&subdir, "m_EditorVersion: 2021.3.16f1").unwrap();

        let result = DetectOptions::new().recursive(true).detect_unity_project_dir(temp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), subdir);
    }

    #[test]
    fn test_detect_unity_project_dir_nested_recursive() {
        let temp_dir = TempDir::new().unwrap();
        // Create nested directories with Unity project deep inside
        let nested_path = temp_dir
            .path()
            .join("level1")
            .join("level2")
            .join("unity_project");
        fs::create_dir_all(&nested_path).unwrap();
        create_unity_project(&nested_path, "m_EditorVersion: 2021.3.16f1").unwrap();

        let result = DetectOptions::new().recursive(true).detect_unity_project_dir(temp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), nested_path);
    }

    #[test]
    fn test_detect_project_version_with_editor_version() {
        let temp_dir = TempDir::new().unwrap();
        let version_content =
            "m_EditorVersion: 2021.3.16f1\nm_EditorVersionWithRevision: 2021.3.16f1 (4016570cf34f)";
        create_unity_project(temp_dir.path(), version_content).unwrap();

        let result = detect_project_version(temp_dir.path());
        assert!(result.is_ok());

        let version = result.unwrap();
        assert_eq!(version.to_string(), "2021.3.16f1");
    }

    #[test]
    fn test_detect_project_version_with_revision() {
        let temp_dir = TempDir::new().unwrap();
        let version_content =
            "m_EditorVersion: 2020.3.1f1\nm_EditorVersionWithRevision: 2021.3.16f1 (4016570cf34f)";
        create_unity_project(temp_dir.path(), version_content).unwrap();

        let result = detect_project_version(temp_dir.path());
        assert!(result.is_ok());

        let version = result.unwrap();
        // Should prefer the version with revision
        assert_eq!(version.to_string(), "2021.3.16f1");
    }

    #[test]
    fn test_detect_project_version_only_editor_version() {
        let temp_dir = TempDir::new().unwrap();
        let version_content = "m_EditorVersion: 2019.4.31f1\nSomeOtherField: value";
        create_unity_project(temp_dir.path(), version_content).unwrap();

        let result = detect_project_version(temp_dir.path());
        assert!(result.is_ok());

        let version = result.unwrap();
        assert_eq!(version.to_string(), "2019.4.31f1");
    }

    #[test]
    fn test_detect_project_version_no_version_info() {
        let temp_dir = TempDir::new().unwrap();
        let version_content = "SomeField: value\nAnotherField: another_value";
        create_unity_project(temp_dir.path(), version_content).unwrap();

        let result = detect_project_version(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_project_version_malformed_version() {
        let temp_dir = TempDir::new().unwrap();
        let version_content = "m_EditorVersion: not_a_valid_version";
        create_unity_project(temp_dir.path(), version_content).unwrap();

        let result = detect_project_version(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_project_version_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("my_project");
        fs::create_dir(&subdir).unwrap();
        create_unity_project(&subdir, "m_EditorVersion: 2022.1.5f1").unwrap();

        let result = DetectOptions::new().recursive(true).detect_project_version(temp_dir.path());
        assert!(result.is_ok());

        let version = result.unwrap();
        assert_eq!(version.to_string(), "2022.1.5f1");
    }

    #[test]
    fn test_detect_project_version_no_project_found() {
        let temp_dir = TempDir::new().unwrap();
        // Empty directory with no Unity project

        let result = DetectOptions::new().recursive(true).detect_project_version(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_options_builder_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("nested").join("project");
        fs::create_dir_all(&subdir).unwrap();
        create_unity_project(&subdir, "m_EditorVersion: 2023.1.0f1").unwrap();

        // Test builder pattern chaining
        let result = DetectOptions::new()
            .recursive(true)
            .max_depth(5)
            .case_sensitive(true)
            .detect_unity_project_dir(temp_dir.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), subdir);
    }

    #[test]
    fn test_detect_options_convenience_functions() {
        let temp_dir = TempDir::new().unwrap();
        create_unity_project(temp_dir.path(), "m_EditorVersion: 2022.3.5f1").unwrap();

        // Test convenience function
        let result = detect_unity_project_dir(temp_dir.path());
        assert!(result.is_ok());

        // Test convenience function for version detection
        let version_result = detect_project_version(temp_dir.path());
        assert!(result.is_ok());
        assert_eq!(version_result.unwrap().to_string(), "2022.3.5f1");
    }

    #[test] 
    fn test_detect_options_default_vs_custom() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("deep").join("project");
        fs::create_dir_all(&subdir).unwrap();
        create_unity_project(&subdir, "m_EditorVersion: 2021.2.8f1").unwrap();

        // Default options should not find nested project
        let default_result = DetectOptions::new().detect_unity_project_dir(temp_dir.path());
        assert!(default_result.is_err());

        // Custom options with recursion should find it
        let recursive_result = DetectOptions::new()
            .recursive(true)
            .detect_unity_project_dir(temp_dir.path());
        assert!(recursive_result.is_ok());
        assert_eq!(recursive_result.unwrap(), subdir);
    }

    #[test]
    fn test_max_depth_limiting() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a deep nested structure: temp/level1/level2/level3/project
        let deep_project = temp_dir.path()
            .join("level1")
            .join("level2") 
            .join("level3")
            .join("project");
        fs::create_dir_all(&deep_project).unwrap();
        create_unity_project(&deep_project, "m_EditorVersion: 2023.2.1f1").unwrap();

        // With max_depth = 2, should not find the project (it's at depth 4)
        let limited_result = DetectOptions::new()
            .recursive(true)
            .max_depth(2)
            .detect_unity_project_dir(temp_dir.path());
        assert!(limited_result.is_err());

        // With max_depth = 5, should find the project
        let deep_result = DetectOptions::new()
            .recursive(true)
            .max_depth(5)
            .detect_unity_project_dir(temp_dir.path());
        assert!(deep_result.is_ok());
        assert_eq!(deep_result.unwrap(), deep_project);

        // With no max_depth limit, should also find it
        let unlimited_result = DetectOptions::new()
            .recursive(true)
            .detect_unity_project_dir(temp_dir.path());
        assert!(unlimited_result.is_ok());
        assert_eq!(unlimited_result.unwrap(), deep_project);
    }

    #[test]
    fn test_max_depth_zero_means_current_only() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create Unity project in subdirectory
        let subdir = temp_dir.path().join("subproject");
        fs::create_dir(&subdir).unwrap();
        create_unity_project(&subdir, "m_EditorVersion: 2022.1.0f1").unwrap();

        // With max_depth = 0 and recursive = true, should behave like recursive = false
        let depth_zero_result = DetectOptions::new()
            .recursive(true)
            .max_depth(0)
            .detect_unity_project_dir(temp_dir.path());
        assert!(depth_zero_result.is_err());

        // But should still find project in the current directory
        let current_dir_result = DetectOptions::new()
            .recursive(true)
            .max_depth(0)
            .detect_unity_project_dir(&subdir);
        assert!(current_dir_result.is_ok());
        assert_eq!(current_dir_result.unwrap(), subdir);
    }

    #[test]
    fn test_combined_options() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a structure: temp/level1/level2/project
        let project_path = temp_dir.path().join("level1").join("level2").join("project");
        fs::create_dir_all(&project_path).unwrap();
        create_unity_project(&project_path, "m_EditorVersion: 2023.1.15f1").unwrap();

        // Test combining recursive, max_depth, and other options
        let result = DetectOptions::new()
            .recursive(true)
            .max_depth(3)
            .case_sensitive(true)
            .detect_unity_project_dir(temp_dir.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), project_path);
    }
}
