mod helper;
use self::helper::*;
use uvm_move_dir::*;
use std::fs::DirBuilder;
use tempfile::TempDir;

#[test]
fn rename_directory_destination_does_not_exist() {
    let base_dir = TempDir::new().unwrap();

    let source = base_dir.path().join("source");
    let destination = base_dir.path().join("destination");

    DirBuilder::new().recursive(true).create(&source).unwrap();

    assert!(source.exists());
    assert!(!destination.exists());

    setup_directory_structure(&source).unwrap();
    move_dir(&source, &destination).expect("successful move operation");

    assert!(!source.exists());
    assert!(destination.exists());
    assert_moved_structure_at(&destination);
}

#[test]
fn rename_directory_destination_does_exist() {

    let base_dir = TempDir::new().unwrap();

    let source = base_dir.path().join("source");
    let destination = base_dir.path().join("destination");

    DirBuilder::new().recursive(true).create(&source).unwrap();
    DirBuilder::new().recursive(true).create(&destination).unwrap();

    assert!(source.exists());
    assert!(destination.exists());

    setup_directory_structure(&source).unwrap();
    move_dir(&source, &destination).unwrap();

    assert!(!source.exists());
    assert!(destination.exists());
    assert_moved_structure_at(&destination);
}

#[test]
fn rename_directory_destination_does_exist_not_empty() {
    let base_dir = TempDir::new().unwrap();

    let source = base_dir.path().join("source");
    let destination = base_dir.path().join("destination");

    DirBuilder::new().recursive(true).create(&source).unwrap();
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
