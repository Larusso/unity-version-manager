use uvm_live_platform::*; // Adjust based on your actual crate name
use crate::{UnityReleaseDownloadArchitecture, UnityReleaseDownloadPlatform, UnityReleaseStream};

#[test]
fn test_fetch_release_basic() {
    let version = "2023.1.5f1".to_string(); // Use a real Unity version
    let result = FetchRelease::builder(version.clone())
        .platform(UnityReleaseDownloadPlatform::MacOs)
        .architecture(UnityReleaseDownloadArchitecture::Arm64)
        .stream(UnityReleaseStream::Lts)
        .fetch();

    assert!(result.is_ok(), "Fetching release failed");

    let release = result.unwrap();
    assert_eq!(release.version, version, "Incorrect version returned");
    println!("Fetched release: {:?}", release);
}

#[test]
fn test_fetch_release_invalid_version() {
    let version = "9999.9.9f9".to_string(); // Nonexistent version
    let result = FetchRelease::builder(version)
        .platform(UnityReleaseDownloadPlatform::Linux)
        .architecture(UnityReleaseDownloadArchitecture::X86_64)
        .stream(UnityReleaseStream::Lts)
        .fetch();

    assert!(result.is_err(), "Expected error for invalid Unity version, but got success");
    println!("Error received as expected: {:?}", result.err());
}

#[test]
fn test_fetch_release_different_platforms() {
    let version = "6000.0.35f1".to_string(); // Use a stable version

    let platforms = vec![
        UnityReleaseDownloadPlatform::MacOs,
        UnityReleaseDownloadPlatform::Windows,
        UnityReleaseDownloadPlatform::Linux,
    ];

    for platform in platforms {
        let result = FetchRelease::builder(version.clone())
            .platform(platform)
            .architecture(UnityReleaseDownloadArchitecture::X86_64)
            .stream(UnityReleaseStream::Lts)
            .fetch();

        assert!(result.is_ok(), "Failed to fetch release for platform {:?} error: {:?}", platform, result.unwrap_err());
        println!("Fetched release for {:?}: {:?}", platform, result.unwrap());
    }
}