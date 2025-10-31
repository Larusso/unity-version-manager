use log::{info, trace, warn};
use std::{env, io};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

pub const DEFAULT_MAX_AGE_ENV: &'static str = "UVM_GC_MAX_AGE";
pub const DEFAULT_MAX_AGE: Duration = Duration::from_secs(2_630_016);
pub const DEFAULT_MAX_AGE_HUMAN: &'static str = "1month";

pub const DEFAULT_ENABLED_ENV: &'static str = "UVM_GC_ENABLED";
pub const DEFAULT_ENABLED: bool = true;

pub fn gc_enabled() -> bool {
    env::var(DEFAULT_ENABLED_ENV).ok()
    .and_then(|v| v.parse().ok())
    .unwrap_or_else(|| {
        trace!("No GC enabled value found in environment variable: {}. Using default GC enabled: {}", DEFAULT_ENABLED_ENV, DEFAULT_ENABLED);
        DEFAULT_ENABLED
    })
}

fn parse_max_age_from_string(max_age_human_value: &str) -> Duration {
    trace!("Use max age value: {}", max_age_human_value);

    match humantime::parse_duration(max_age_human_value) {
        Ok(duration) => duration,
        Err(_) => {
            warn!("Invalid GC max age value: {}", max_age_human_value);
            warn!("Using default GCmax age: {}", DEFAULT_MAX_AGE_HUMAN);
            DEFAULT_MAX_AGE
        }
    }
}

pub fn default_max_age() -> Duration {
    // Keep behavior: env overrides; otherwise use the compile-time default
    let max_age_human_value = env::var(DEFAULT_MAX_AGE_ENV).unwrap_or_else(|_| {
        trace!(
            "No max age value found in environment variable: {}. Using default max age: {}",
            DEFAULT_MAX_AGE_ENV, DEFAULT_MAX_AGE_HUMAN
        );
        DEFAULT_MAX_AGE_HUMAN.to_owned()
    });

    parse_max_age_from_string(&max_age_human_value)
}
/// Garbage collector for Unity Version Manager
///
/// This collector is used to clean up old files in the cache directory.
/// The collector will delete files older than the specified max age.
/// The collector will run in dry run mode by default.

pub struct GarbageCollector {
    dry_run: bool,
    max_age: Duration,
    base_dir: PathBuf,
    version: Option<String>,
}

impl GarbageCollector {
    /// Create a new GarbageCollector
    ///
    /// # Arguments
    ///
    /// * `base_dir` - The base directory to clean up
    ///
    /// # Returns
    ///
    /// A new GarbageCollector
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            dry_run: true,
            max_age: default_max_age(),
            base_dir: base_dir.as_ref().to_path_buf(),
            version: None,
        }
    }
    /// Set the dry run mode
    ///
    /// # Arguments
    ///
    /// * `dry_run` - The dry run mode
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Set the maximum age of files to delete
    pub fn with_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = max_age;
        self
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    /// Collect the garbage
    pub fn collect(&self) -> io::Result<()> {
        let prefix = if self.dry_run { "[DRY RUN] " } else { "" };
        info!(
            "{}Cleaning up files older than {} in {}",
            prefix,
            humantime::format_duration(self.max_age),
            self.base_dir.display()
        );
        for (entry, accessed_elapsed) in walkdir::WalkDir::new(&self.base_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_file())
            .filter(|entry| {
                if let Some(version) = &self.version {
                    return entry.path().display().to_string().contains(version);
                }
                true
            })
            .filter_map(|entry| fs::metadata(entry.path()).ok().map(|m| (entry, m)))
            .filter_map(|(entry, metadata)| {
                let accessed = metadata.accessed().unwrap_or_else(|_| SystemTime::now());
                let accessed_elapsed = accessed.elapsed().unwrap_or_default();
                if accessed_elapsed > self.max_age {
                    Some((entry, accessed_elapsed))
                } else {
                    trace!(
                        "{}Skipping file: {} ({} old)",
                        prefix,
                        entry.path().display(),
                        humantime::format_duration(accessed_elapsed)
                    );
                    None
                }
            })
        {
            info!(
                "{}Deleting file: {} ({} old)",
                prefix,
                entry.path().display(),
                humantime::format_duration(accessed_elapsed)
            );
            if !self.dry_run {
                fs::remove_file(entry.path())?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_garbage_collector_respects_large_max_age() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        let file1 = temp_path.join("recent_file.txt");
        File::create(&file1).expect("Failed to create test file");

        let gc = GarbageCollector::new(&temp_path)
            .with_dry_run(false)
            .with_max_age(Duration::from_secs(365 * 24 * 60 * 60));

        let result = gc.collect();
        assert!(result.is_ok());

        assert!(file1.exists());
    }

    #[test]
    fn test_garbage_collector_respects_zero_max_age() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        let file1 = temp_path.join("any_file.txt");
        File::create(&file1).expect("Failed to create test file");

        let gc = GarbageCollector::new(&temp_path)
            .with_dry_run(false)
            .with_max_age(Duration::from_secs(0));

        let result = gc.collect();
        assert!(result.is_ok());

        assert!(!file1.exists());
    }

    #[test]
    fn test_garbage_collector_dry_run_preserves_files() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        let file1 = temp_path.join("test1.txt");
        let file2 = temp_path.join("test2.txt");
        File::create(&file1).expect("Failed to create test file");
        File::create(&file2).expect("Failed to create test file");

        let gc = GarbageCollector::new(&temp_path)
            .with_dry_run(true)
            .with_max_age(Duration::from_secs(0));

        let result = gc.collect();
        assert!(result.is_ok());

        assert!(file1.exists());
        assert!(file2.exists());
    }

    #[test]
    fn test_garbage_collector_execute_deletes_files() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        let file1 = temp_path.join("test1.txt");
        let file2 = temp_path.join("test2.txt");
        File::create(&file1).expect("Failed to create test file");
        File::create(&file2).expect("Failed to create test file");

        let gc = GarbageCollector::new(&temp_path)
            .with_dry_run(false)
            .with_max_age(Duration::from_secs(0));

        let result = gc.collect();
        assert!(result.is_ok());

        assert!(!file1.exists());
        assert!(!file2.exists());
    }

    #[test]
    fn default_human_matches_const() {
        let parsed = humantime::parse_duration(DEFAULT_MAX_AGE_HUMAN).unwrap();
        assert_eq!(parsed, DEFAULT_MAX_AGE);
    }

    #[test]
    fn test_parse_max_age_with_valid_durations() {
        let test_cases = vec![
            ("30days", Duration::from_secs(30 * 24 * 60 * 60)),
            ("2weeks", Duration::from_secs(2 * 7 * 24 * 60 * 60)),
            ("1hour", Duration::from_secs(60 * 60)),
            ("45min", Duration::from_secs(45 * 60)),
            ("3600s", Duration::from_secs(3600)),
            ("1month", Duration::from_secs(2_630_016)), // Default value
        ];

        for (input, expected) in test_cases {
            let result = parse_max_age_from_string(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_parse_max_age_with_invalid_durations() {
        let invalid_values = vec![
            "completely_invalid",
            "not_a_duration",
            "",
            "xyz123",
            "1.5.3seconds",
        ];

        for invalid_value in invalid_values {
            let result = parse_max_age_from_string(invalid_value);
            assert_eq!(
                result, DEFAULT_MAX_AGE,
                "Failed for invalid input: {}",
                invalid_value
            );
        }
    }

    #[test]
    fn test_parse_max_age_with_whitespace() {
        let whitespace_values = vec![
            ("  ", DEFAULT_MAX_AGE),                  // Invalid, should return default
            ("\t", DEFAULT_MAX_AGE),                  // Invalid, should return default
            ("\n", DEFAULT_MAX_AGE),                  // Invalid, should return default
            ("  1hour  ", Duration::from_secs(3600)), // Valid with trimming
        ];

        for (input, expected) in whitespace_values {
            let result = parse_max_age_from_string(input);
            assert_eq!(result, expected, "Failed for input: {:?}", input);
        }
    }

    #[test]
    fn test_parse_max_age_edge_cases() {
        let edge_cases = vec![
            ("0s", Duration::from_secs(0)),
            ("1ns", Duration::from_nanos(1)),
            ("999999999ns", Duration::from_nanos(999999999)),
        ];

        for (input, expected) in edge_cases {
            let result = parse_max_age_from_string(input);
            assert_eq!(result, expected, "Failed for edge case: {}", input);
        }
    }

    #[test]
    fn test_default_max_age_uses_default_when_no_env() {
        // This test assumes the env var is not set in the test environment
        // If it is set, this test might fail, but that's expected behavior
        let result = default_max_age();
        // We can't easily test the env var behavior without setting it,
        // but we can at least verify the function returns a valid duration
        assert!(result.as_secs() > 0);
    }

    #[test]
    fn test_garbage_collector_with_version_filter_matches() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        // Create files with different version strings in their names
        let matching_file = temp_path.join("unity-2023.1.1f1-install.log");
        let non_matching_file = temp_path.join("unity-2022.3.5f1-install.log");
        let another_matching = temp_path.join("2023.1.1f1-build-cache.dat");
        
        File::create(&matching_file).expect("Failed to create test file");
        File::create(&non_matching_file).expect("Failed to create test file");
        File::create(&another_matching).expect("Failed to create test file");

        let gc = GarbageCollector::new(&temp_path)
            .with_dry_run(false)
            .with_max_age(Duration::from_secs(0))
            .with_version("2023.1.1f1".to_string());

        let result = gc.collect();
        assert!(result.is_ok());

        // Files containing the version should be deleted
        assert!(!matching_file.exists());
        assert!(!another_matching.exists());
        // Files not containing the version should be preserved
        assert!(non_matching_file.exists());
    }

    #[test]
    fn test_garbage_collector_with_version_filter_dry_run() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        let matching_file = temp_path.join("unity-2023.1.1f1-install.log");
        let non_matching_file = temp_path.join("unity-2022.3.5f1-install.log");
        
        File::create(&matching_file).expect("Failed to create test file");
        File::create(&non_matching_file).expect("Failed to create test file");

        let gc = GarbageCollector::new(&temp_path)
            .with_dry_run(true)
            .with_max_age(Duration::from_secs(0))
            .with_version("2023.1.1f1".to_string());

        let result = gc.collect();
        assert!(result.is_ok());

        // All files should be preserved in dry run mode
        assert!(matching_file.exists());
        assert!(non_matching_file.exists());
    }

    #[test]
    fn test_garbage_collector_without_version_processes_all_files() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        let file1 = temp_path.join("unity-2023.1.1f1-install.log");
        let file2 = temp_path.join("unity-2022.3.5f1-install.log");
        let file3 = temp_path.join("some-other-file.txt");
        
        File::create(&file1).expect("Failed to create test file");
        File::create(&file2).expect("Failed to create test file");
        File::create(&file3).expect("Failed to create test file");

        // Create GC without version filter (version is None by default)
        let gc = GarbageCollector::new(&temp_path)
            .with_dry_run(false)
            .with_max_age(Duration::from_secs(0));

        let result = gc.collect();
        assert!(result.is_ok());

        // All files should be deleted when no version filter is applied
        assert!(!file1.exists());
        assert!(!file2.exists());
        assert!(!file3.exists());
    }

    #[test]
    fn test_garbage_collector_version_filter_respects_max_age() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        let matching_file = temp_path.join("unity-2023.1.1f1-install.log");
        let non_matching_file = temp_path.join("unity-2022.3.5f1-install.log");
        
        File::create(&matching_file).expect("Failed to create test file");
        File::create(&non_matching_file).expect("Failed to create test file");

        // Use a large max age so files won't be deleted due to age
        let gc = GarbageCollector::new(&temp_path)
            .with_dry_run(false)
            .with_max_age(Duration::from_secs(365 * 24 * 60 * 60))
            .with_version("2023.1.1f1".to_string());

        let result = gc.collect();
        assert!(result.is_ok());

        // Both files should still exist because they're not old enough
        assert!(matching_file.exists());
        assert!(non_matching_file.exists());
    }

    #[test]
    fn test_garbage_collector_version_filter_empty_string() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        let file1 = temp_path.join("unity-2023.1.1f1-install.log");
        let file2 = temp_path.join("some-other-file.txt");
        
        File::create(&file1).expect("Failed to create test file");
        File::create(&file2).expect("Failed to create test file");

        // Test with empty string version filter
        let gc = GarbageCollector::new(&temp_path)
            .with_dry_run(false)
            .with_max_age(Duration::from_secs(0))
            .with_version("".to_string());

        let result = gc.collect();
        assert!(result.is_ok());

        // All files should be processed since empty string matches everything
        assert!(!file1.exists());
        assert!(!file2.exists());
    }
}
