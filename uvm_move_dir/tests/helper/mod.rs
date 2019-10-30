use std::path::Path;
use std::io;
use std::fs::DirBuilder;

pub fn setup_directory_structure<P: AsRef<Path>>(target:P) -> io::Result<()> {
    let target = target.as_ref();
    let subfile0 = target.join("file0.txt");
    let subdir1 = target.join("subdir1");
    let subfile1 = subdir1.join("file1.txt");
    let subdir2 = target.join("subdir2");
    let subfile2 = subdir2.join("file2.txt");

    DirBuilder::new().recursive(true).create(&subdir1).unwrap();
    DirBuilder::new().recursive(true).create(&subdir2).unwrap();

    std::fs::File::create(subfile0).unwrap();
    std::fs::File::create(subfile1).unwrap();
    std::fs::File::create(subfile2).unwrap();
    Ok(())
}

pub fn assert_moved_structure_at<P: AsRef<Path>>(target:P) {
    let target = target.as_ref();
    let subfile0 = target.join("file0.txt");
    let subdir1 = target.join("subdir1");
    let subfile1 = subdir1.join("file1.txt");
    let subdir2 = target.join("subdir2");
    let subfile2 = subdir2.join("file2.txt");

    assert!(subfile0.exists(), format!("{} should exist", subfile0.display()));
    assert!(subdir1.exists(), format!("{} should exist", subdir1.display()));
    assert!(subfile1.exists(), format!("{} should exist", subfile1.display()));
    assert!(subdir2.exists(), format!("{} should exist", subdir2.display()));
    assert!(subfile2.exists(), format!("{} should exist", subfile2.display()));
}
