use std::process::Command;

#[test]
#[cfg(unix)]
fn test_uvm_modules_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("modules")
        .arg("--help")
        .output()
        .expect("failed to run uvm");

    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: uvm modules [OPTIONS] <VERSION>"));
    assert!(stdout.contains("--category <CATEGORY>"));
    assert!(stdout.contains("--show-sync-modules"));
    assert!(stdout.contains("--all"));
    assert!(stdout.contains("-v, --verbose"));
}

#[test]
fn test_uvm_modules_with_valid_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("modules")
        .arg("2022.3.0f1")
        .output()
        .expect("failed to run uvm");

    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Check that we get some expected categories and modules
    assert!(stdout.contains("PLATFORM:"));
    assert!(stdout.contains("android"));
    assert!(stdout.contains("ios"));
    assert!(stdout.contains("webgl"));
}

#[test]
#[cfg(unix)]
fn test_uvm_modules_verbose_mode() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("modules")
        .arg("-v")
        .arg("2022.3.0f1")
        .output()
        .expect("failed to run uvm");

    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // In verbose mode, we should see descriptions
    // Use more flexible matching to handle Windows console differences
    assert!(stdout.contains("android") && stdout.contains("Android"));
    assert!(stdout.contains("ios") && stdout.contains("iOS"));
}

#[test]
fn test_uvm_modules_filter_by_category() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("modules")
        .arg("--category")
        .arg("PLATFORM")
        .arg("2022.3.0f1")
        .output()
        .expect("failed to run uvm");

    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should only show PLATFORM category
    assert!(stdout.contains("PLATFORM:"));
    assert!(stdout.contains("android"));
    assert!(stdout.contains("ios"));
    // Should not show other categories
    assert!(!stdout.contains("DOCUMENTATION:"));
    assert!(!stdout.contains("LANGUAGE_PACK:"));
}

#[test]
fn test_uvm_modules_filter_by_multiple_categories() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("modules")
        .arg("--category")
        .arg("PLATFORM,DOCUMENTATION")
        .arg("2022.3.0f1")
        .output()
        .expect("failed to run uvm");

    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show both categories
    assert!(stdout.contains("PLATFORM:"));
    assert!(stdout.contains("DOCUMENTATION:"));
    assert!(stdout.contains("android"));
    assert!(stdout.contains("documentation"));
    // Should not show other categories
    assert!(!stdout.contains("LANGUAGE_PACK:"));
}

#[test]
fn test_uvm_modules_with_invalid_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("modules")
        .arg("9999.9.9f9")
        .output()
        .expect("failed to run uvm");

    // Should fail for invalid version
    assert!(!output.status.success());
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error:"));
}

#[test]
fn test_uvm_modules_show_all_modules() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("modules")
        .arg("--all")
        .arg("2022.3.0f1")
        .output()
        .expect("failed to run uvm");

    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show modules (the exact count may vary, but we should have some)
    assert!(stdout.contains("PLATFORM:"));
    assert!(stdout.contains("android"));
}

#[test]
fn test_uvm_modules_show_sync_modules() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("modules")
        .arg("--show-sync-modules")
        .arg("2022.3.0f1")
        .output()
        .expect("failed to run uvm");

    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show modules (sync modules functionality is implemented but may not have children in current data)
    assert!(stdout.contains("PLATFORM:"));
    assert!(stdout.contains("android"));
}
