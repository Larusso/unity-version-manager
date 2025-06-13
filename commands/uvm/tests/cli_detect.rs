use std::fs;
use std::process::Command;
use tempfile::tempdir;


#[test]
#[cfg(unix)]
fn test_uvm_detect_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("detect")
        .arg("--help")
        .output()
        .expect("failed to run uvm");

    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: uvm detect [OPTIONS] [PROJECT_PATH]"));
}

#[test]
fn detects_project_version_file() {
    let temp = tempdir().unwrap();
    let project_settings = temp.path().join("ProjectSettings");
    fs::create_dir(&project_settings).unwrap();
    let version_file = project_settings.join("ProjectVersion.txt");

    let version_content = "m_EditorVersion: 2021.3.2f1";
    fs::write(&version_file, version_content).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("detect")
        .arg("-r")
        .arg(temp.path())
        .output()
        .expect("failed to run uvm");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(stdout.trim(), "2021.3.2f1");
}

#[test]
fn detects_nested_project_when_recursive() {
    let temp = tempdir().unwrap();
    let nested = temp.path().join("nested/project");
    let project_settings = nested.join("ProjectSettings");
    fs::create_dir_all(&project_settings).unwrap();
    let version_file = project_settings.join("ProjectVersion.txt");
    fs::write(&version_file, "m_EditorVersion: 2020.1.5f1").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("detect")
        .arg("-r")
        .arg(temp.path())
        .output()
        .expect("failed to run uvm");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(stdout.trim(), "2020.1.5f1");
}

#[test]
fn fails_on_missing_project_version() {
    let temp = tempdir().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("detect")
        .arg("-r")
        .arg(temp.path())
        .output()
        .expect("failed to run uvm");

    assert!(!output.status.success());
}