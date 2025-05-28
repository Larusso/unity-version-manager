use std::{env, fs};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use tempfile::TempDir;

#[test]
#[cfg(unix)]
fn test_invocation_of_external_command_in_path() {
    let temp_dir = TempDir::new().unwrap();
    let mock_command = if cfg!(windows) {
        temp_dir.path().join("uvm-test.exe")
    } else {
        temp_dir.path().join("uvm-test")
    };

    let script_content = if cfg!(windows) {
        r#"@echo off
if "%1"=="--help" (
    echo Usage: uvm test [OPTIONS]
) else (
    echo Dummy output
)"#

    } else {
        r#"#!/bin/bash
if [ "$1" = "--help" ]; then
    echo "Usage: uvm test [OPTIONS]"
else
    echo "Dummy output"
fi"#

    };

    fs::write(&mock_command, script_content).unwrap();

    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&mock_command)
            .unwrap()
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&mock_command, permissions).unwrap();
    }


    let old_path = env::var("PATH").unwrap_or_default();
    let path_delimiter = if cfg!(windows) { ";" } else { ":" };
    env::set_var(
        "PATH",
        format!(
            "{}{}{}",
            temp_dir.path().display(),
            path_delimiter,
            old_path
        ),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_uvm"))
        .arg("test")
        .arg("--help")
        .output()
        .expect("failed to run uvm");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: uvm test [OPTIONS]"));

    env::set_var("PATH", old_path);
}