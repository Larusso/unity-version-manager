mod helper;
use self::helper::*;
use uvm_move_dir::*;
use std::fs::DirBuilder;
use tempfile::TempDir;

#[test]
fn move_dir_one_level_up() {
    let base_dir = TempDir::new().unwrap();

    let destination = base_dir.path().join("dir1/dir2");
    let source = &destination.join("dir3");

    DirBuilder::new().recursive(true).create(&source).expect("the source dir");

    assert!(source.exists());
    assert!(destination.exists());

    setup_directory_structure(&source).expect("directory setup");
    move_dir(&source, &destination).expect("successful move operation");
    assert!(!source.exists());
    assert_moved_structure_at(&destination)
}

#[test]
fn move_dir_multiple_empty_level_up() {
    let base_dir = TempDir::new().expect("a temp dir");

    let destination = base_dir.path().join("dir1");
    let middle = destination.join("dir2/dir3");
    let source = middle.join("dir4");

    DirBuilder::new().recursive(true).create(&source).expect("the source dir");

    assert!(source.exists());
    assert!(middle.exists());
    assert!(destination.exists());

    setup_directory_structure(&source).expect("directory setup");
    move_dir(&source, &destination).expect("successful move operation");
    assert!(!source.exists());
    assert!(!middle.exists());
    assert_moved_structure_at(&destination)
}

#[test]
fn move_dir_multiple_non_empty_level_up() {
    let base_dir = TempDir::new().unwrap();

    let destination = base_dir.path().join("dir1");
    let middle = destination.join("dir2/dir3");
    let source = middle.join("dir4");

    DirBuilder::new().recursive(true).create(&source).expect("the source dir");

    assert!(source.exists());
    assert!(middle.exists());
    assert!(destination.exists());

    setup_directory_structure(&source).expect("directory setup");
    setup_directory_structure(&middle).expect("directory setup 2");

    let result = move_dir(&source, &destination);

    assert!(result.is_err());
    //assert!(source.exists());
}
