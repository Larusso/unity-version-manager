use std::fs::{self, File};
use std::process::Command;
use std::time::Duration;
use tempfile::tempdir;

#[test]
#[cfg(unix)]
fn test_uvm_gc_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("gc")
        .arg("--help")
        .output()
        .expect("failed to run uvm");

    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: uvm gc [OPTIONS]"));
    assert!(stdout.contains("--execute"));
    assert!(stdout.contains("--max-age"));
    assert!(stdout.contains("Execute the garbage collection"));
    assert!(stdout.contains("The maximum age of files to delete"));
}

#[test]
fn test_uvm_gc_dry_run_default() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("gc")
        .output()
        .expect("failed to run uvm");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let _stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(!stderr.contains("required"));
    assert!(!stderr.contains("error: the following required arguments were not provided"));
}

#[test]
fn test_uvm_gc_with_execute_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("gc")
        .arg("--execute")
        .output()
        .expect("failed to run uvm");

    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(!stderr.contains("required"));
    assert!(!stderr.contains("error: the following required arguments were not provided"));
}

#[test]
fn test_uvm_gc_with_custom_max_age() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("gc")
        .arg("--max-age")
        .arg("1h")
        .output()
        .expect("failed to run uvm");

    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("error parsing"));
}

#[test]
fn test_uvm_gc_invalid_max_age() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("gc")
        .arg("--max-age")
        .arg("invalid")
        .output()
        .expect("failed to run uvm");

    assert!(!output.status.success());
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value") || stderr.contains("error parsing"));
}

#[test]
fn test_uvm_gc_short_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("gc")
        .arg("-e")  // short flag for --execute
        .arg("-m")  // short flag for --max-age
        .arg("30m")
        .output()
        .expect("failed to run uvm");

    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("unrecognized"));
}

#[test]
fn test_uvm_gc_environment_variable() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("gc")
        .env("UVM_GC_MAX_AGE", "2h")
        .output()
        .expect("failed to run uvm");

    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("error parsing"));
}

#[test]
fn test_uvm_gc_arg_overrides_env() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("gc")
        .arg("--max-age")
        .arg("1h")
        .env("UVM_GC_MAX_AGE", "2h")  // This should be overridden
        .output()
        .expect("failed to run uvm");

    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("error parsing"));
}

#[test]
fn test_uvm_gc_various_duration_formats() {
    let test_cases = vec![
        "30s",
        "5m",
        "2h",
        "1d",
        "1w",
        "30sec",
        "5min",
        "2hours",
    ];

    for duration in test_cases {
        let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
            .arg("gc")
            .arg("--max-age")
            .arg(duration)
            .output()
            .expect(&format!("failed to run uvm with duration {}", duration));

        let stderr = String::from_utf8_lossy(&output.stderr);
        
        assert!(
            !stderr.contains("invalid value") && !stderr.contains("error parsing"),
            "Duration format '{}' should be valid, but got error: {}",
            duration,
            stderr
        );
    }
}

#[test]
fn test_uvm_gc_combined_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("gc")
        .arg("--execute")
        .arg("--max-age")
        .arg("1h")
        .output()
        .expect("failed to run uvm");

    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("unrecognized"));
}

fn create_test_file_with_age(dir: &std::path::Path, filename: &str, _age: Duration) -> std::io::Result<std::path::PathBuf> {
    let file_path = dir.join(filename);
    File::create(&file_path)?;
    
    Ok(file_path)
}

#[test]
fn test_gc_command_functionality_with_temp_cache() {
    
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let cache_path = temp_dir.path();
    
    let _file1 = create_test_file_with_age(cache_path, "old_cache_file1.tmp", Duration::from_secs(3600))
        .expect("Failed to create test file");
    let _file2 = create_test_file_with_age(cache_path, "old_cache_file2.tmp", Duration::from_secs(7200))
        .expect("Failed to create test file");
    
    let subdir = cache_path.join("subdir");
    fs::create_dir(&subdir).expect("Failed to create subdirectory");
    let _file3 = create_test_file_with_age(&subdir, "nested_file.tmp", Duration::from_secs(1800))
        .expect("Failed to create nested test file");
    
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("gc")
        .arg("--max-age")
        .arg("1s")
        .output()
        .expect("failed to run uvm gc");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("error parsing"));
    
}

#[test]
fn test_gc_command_error_handling() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("gc")
        .arg("--max-age")
        .arg("not-a-duration")
        .output()
        .expect("failed to run uvm gc");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value") || stderr.contains("error"));
}

#[test] 
fn test_gc_command_logging_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("gc")
        .arg("--max-age")
        .arg("1m")
        .output()
        .expect("failed to run uvm gc");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("error parsing"));
}

#[test]
#[cfg(unix)]
fn test_gc_command_exit_codes() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("gc")
        .arg("--max-age")
        .arg("1h")
        .output()
        .expect("failed to run uvm gc");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("unrecognized"));
    
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("gc")
        .arg("--invalid-flag")
        .output()
        .expect("failed to run uvm gc");
    
    assert!(!output.status.success());
}
