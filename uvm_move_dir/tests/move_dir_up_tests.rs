mod helper;
use self::helper::*;
use uvm_move_dir::*;
use std::fs::DirBuilder;
use tempfile::TempDir;

#[test]
fn move_dir_one_level_down() {
    let base_dir = TempDir::new().unwrap();

    let source = base_dir.path().join("source");
    let destination = base_dir.path().join("middle/destination");

    DirBuilder::new().recursive(true).create(&source).expect("the source dir");

    assert!(source.exists());
    assert!(!destination.exists());

    setup_directory_structure(&source).expect("directory setup");

    move_dir(&source, &destination).expect("successful move operation");
    assert!(!source.exists());
    assert_moved_structure_at(&destination)
}

#[test]
fn move_dir_multiple_level_down() {
    let base_dir = TempDir::new().unwrap();

    let source = base_dir.path().join("source");
    let destination = base_dir.path().join("middle/middle2/destination");

    DirBuilder::new().recursive(true).create(&source).expect("the source dir");

    assert!(source.exists());
    assert!(!destination.exists());

    setup_directory_structure(&source).expect("directory setup");

    move_dir(&source, &destination).expect("successful move operation");
    assert!(!source.exists());
    assert_moved_structure_at(&destination)
}

#[test]
fn fails_move_dir_one_level_down_when_destination_exists() {
    let base_dir = TempDir::new().unwrap();

    let source = base_dir.path().join("source");
    let destination = base_dir.path().join("middle/destination");

    DirBuilder::new().recursive(true).create(&source).expect("the source dir");
    DirBuilder::new().recursive(true).create(&destination).unwrap();

    assert!(source.exists());
    assert!(destination.exists());

    setup_directory_structure(&source).unwrap();
    setup_directory_structure(&destination).unwrap();
    let result = move_dir(&source, &destination);

    assert!(result.is_err());
    assert!(source.exists());
    assert!(destination.exists());
}
