cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        use uvm_core::unity;

        #[test]
        fn downloads_editor_installer_for_version() {
            let variant = uvm_install_core::InstallVariant::Editor;
            let version = unity::Version::f(2018, 2, 6, 1);

            let installer_path = uvm_install_core::download_installer(variant, &version).expect("path to installer");
            assert!(installer_path.exists());
        }
    }
}
