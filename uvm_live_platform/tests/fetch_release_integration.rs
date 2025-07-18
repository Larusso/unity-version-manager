use uvm_live_platform::*; // Adjust based on your actual crate name
use crate::{UnityReleaseDownloadArchitecture, UnityReleaseDownloadPlatform, UnityReleaseStream};

#[test]
fn test_fetch_release_basic() {
    let version = "2022.3.33f1"; // Use a real Unity version
    let result = FetchRelease::try_builder(version).unwrap()
        .with_platform(UnityReleaseDownloadPlatform::MacOs)
        .with_architecture(UnityReleaseDownloadArchitecture::Arm64)
        .with_stream(UnityReleaseStream::Lts)
        .fetch();

    assert!(result.is_ok(), "Fetching release failed");

    let release = result.unwrap();
    assert_eq!(release.version, version, "Incorrect version returned");
    println!("Fetched release: {:?}", release);
}

#[test]
fn test_fetch_release_invalid_version() {
    let version = "9999.9.9f9".to_string(); // Nonexistent version
    let result = FetchRelease::try_builder(version).unwrap()
        .with_platform(UnityReleaseDownloadPlatform::Linux)
        .with_architecture(UnityReleaseDownloadArchitecture::X86_64)
        .with_stream(UnityReleaseStream::Lts)
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
        let result = FetchRelease::try_builder(version.clone()).unwrap()
            .with_platform(platform)
            .with_architecture(UnityReleaseDownloadArchitecture::X86_64)
            .with_stream(UnityReleaseStream::Lts)
            .fetch();

        assert!(result.is_ok(), "Failed to fetch release for platform {:?} error: {:?}", platform, result.unwrap_err());
        println!("Fetched release for {:?}: {:?}", platform, result.unwrap());
    }
}

#[test]
fn test_fetch_extended_lts_release() {
    let version = "2021.3.48f1".to_string(); // Use a stable version

    let platforms = vec![
        UnityReleaseDownloadPlatform::MacOs,
        UnityReleaseDownloadPlatform::Windows,
        UnityReleaseDownloadPlatform::Linux,
    ];

    for platform in platforms {
        let result = FetchRelease::try_builder(version.clone()).unwrap()
            .with_platform(platform)
            .with_system_architecture()
            .with_extended_lts()
            .with_stream(UnityReleaseStream::Lts)
            .fetch();

        assert!(result.is_ok(), "Failed to fetch release for platform {:?} error: {:?}", platform, result.unwrap_err());
        println!("Fetched release for {:?}: {:?}", platform, result.unwrap());
    }
}