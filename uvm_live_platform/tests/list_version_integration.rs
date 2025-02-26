use uvm_live_platform::*;
// Adjust the crate name based on your project
use crate::{UnityReleaseDownloadArchitecture, UnityReleaseDownloadPlatform, UnityReleaseStream};

#[test]
fn test_list_versions_basic() {
    let result = ListVersions::builder()
        .platform(UnityReleaseDownloadPlatform::Linux)
        .architecture(UnityReleaseDownloadArchitecture::X86_64)
        .stream(UnityReleaseStream::Lts)
        .limit(10)
        .list();

    assert!(result.is_ok(), "Failed to fetch Unity versions");

    let versions = result.unwrap();
    let versions_vec: Vec<String> = versions.collect();

    assert!(!versions_vec.is_empty(), "No versions returned");
    println!("Fetched versions: {:?}", versions_vec);
}

#[test]
fn test_list_versions_pagination() {
    let result = ListVersions::builder()
        .platform(UnityReleaseDownloadPlatform::Windows)
        .architecture(UnityReleaseDownloadArchitecture::X86_64)
        .stream(UnityReleaseStream::Lts)
        .limit(5)
        .autopage(true)
        .list();

    assert!(result.is_ok(), "Pagination test failed");

    let versions = result.unwrap();
    let versions_vec: Vec<String> = versions.collect();

    assert!(versions_vec.len() > 5, "Pagination did not fetch multiple pages");
    println!("Fetched paginated versions: {:?}", versions_vec);
}

#[test]
fn test_list_versions_with_revision() {
    let result = ListVersions::builder()
        .platform(UnityReleaseDownloadPlatform::MacOs)
        .architecture(UnityReleaseDownloadArchitecture::Arm64)
        .stream(UnityReleaseStream::Beta)
        .limit(3)
        .include_revision(true)
        .list();

    assert!(result.is_ok(), "Fetching versions with revision failed");

    let versions = result.unwrap();
    let versions_vec: Vec<String> = versions.collect();

    assert!(!versions_vec.is_empty(), "No versions returned");
    assert!(
        versions_vec.iter().all(|v| v.contains('(') && v.contains(')')),
        "Versions do not contain revision hashes"
    );
    println!("Fetched versions with revision: {:?}", versions_vec);
}
