use env_logger;
use std::collections::HashSet;
use tempfile;
use test_env_log::test;
use uvm_core::unity::{Component, Installation, Version};

#[cfg(target_os = "macos")]
#[ignore]
#[test]
fn installs_editor_2019_3() {
    let version = Version::b(2019, 3, 0, 7);
    let destination = tempfile::tempdir().unwrap();
    let result = uvm_install2::install(
        &version,
        Option::<Vec<Component>>::None,
        false,
        Some(&destination),
    );
    assert!(result.is_ok());

    Installation::new(destination).expect("a unity installation");
}

#[cfg(target_os = "macos")]
#[ignore]
#[test]
fn installs_editor_and_modules_2019_3_with_android_and_sync_modules() {
    use Component::*;
    let version = Version::b(2019, 3, 0, 8);
    let destination = tempfile::tempdir().unwrap();
    let components: HashSet<Component> = [Android].into_iter().map(|c| *c).collect();
    let result = uvm_install2::install(&version, Some(&components), true, Some(&destination));

    assert!(result.is_ok());

    let installation = result.unwrap();
    let installed_components: HashSet<Component> = installation.installed_components().collect();
    let expected_components: HashSet<Component> = [
        Android,
        AndroidOpenJdk,
        AndroidNdk,
        AndroidSdkNdkTools,
        AndroidSdkBuildTools,
        AndroidSdkPlatformTools,
    ]
    .into_iter()
    .map(|c| *c)
    .collect();
    println!("{:?}", installed_components);
    assert!(installed_components.is_superset(&expected_components));
}

#[cfg(target_os = "macos")]
#[ignore]
#[test]
fn installs_editor_2018_4() {
    let version = Version::f(2018, 4, 12, 1);
    let destination = tempfile::tempdir().unwrap();
    let result = uvm_install2::install(
        &version,
        Option::<Vec<Component>>::None,
        false,
        Some(&destination),
    );
    assert!(result.is_ok());

    Installation::new(destination).expect("a unity installation");
}

#[cfg(target_os = "macos")]
#[ignore]
#[test]
fn installs_editor_and_modules_2018_4_with_ios_android_webgl() {
    let version = Version::f(2018, 4, 11, 1);
    let destination = tempfile::tempdir().unwrap();
    let components: HashSet<Component> = [Component::Ios, Component::Android, Component::WebGl]
        .into_iter()
        .map(|c| *c)
        .collect();
    let result = uvm_install2::install(&version, Some(&components), false, Some(&destination));
    assert!(result.is_ok());

    let installation = result.unwrap();
    let installed_components: HashSet<Component> = installation.installed_components().collect();
    assert!(installed_components.is_superset(&components));
}
