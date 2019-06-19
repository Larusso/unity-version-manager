extern crate uvm_core;
use uvm_core::install;
use uvm_core::unity;

#[cfg(target_os = "macos")]
#[test]
fn downloads_editor_installer_for_version() {
    let variant = install::InstallVariant::Editor;
    let version = unity::Version::f(2018, 2, 6, 1);

    let installer_path = install::download_installer(variant, &version).expect("path to installer");
    assert!(installer_path.exists());
}
