mod helper;
use self::helper::*;
use uvm_move_dir::*;
use std::fs::DirBuilder;
use tempfile::TempDir;

#[test]
fn rename_directory_when_destination_does_not_exist() {
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
fn rename_directory_when_destination_exists_and_is_empty() {

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
fn fails_rename_when_from_is_not_a_directory() {
    let base_dir = TempDir::new().unwrap();

    let source = base_dir.path().join("source");
    let destination = base_dir.path().join("destination");

    std::fs::File::create(&source).expect("a source as file");
    DirBuilder::new().recursive(true).create(&destination).unwrap();

    assert!(source.exists());
    assert!(destination.exists());

    let result = move_dir(&source, &destination);

    assert!(result.is_err());
    assert!(source.exists());
    assert!(destination.exists());
}

#[test]
fn fails_rename_directory_when_destination_exists_and_is_file() {
    let base_dir = TempDir::new().unwrap();

    let source = base_dir.path().join("source");
    let destination = base_dir.path().join("destination");

    DirBuilder::new().recursive(true).create(&source).unwrap();
    std::fs::File::create(&destination).expect("a destination as file");

    assert!(source.exists());
    assert!(destination.exists());
    assert!(destination.is_file());

    setup_directory_structure(&source).unwrap();
    let result = move_dir(&source, &destination);

    assert!(result.is_err());
    assert!(source.exists());
    assert!(destination.exists());
}
