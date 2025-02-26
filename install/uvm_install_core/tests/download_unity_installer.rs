cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        use uvm_core::unity;

        #[test]
        fn downloads_editor_installer_for_version() {
            let component = unity::Component::Editor;
            let version = unity::Version::f(2018, 2, 6, 1);
            let manifest = unity::Manifest::load(&version).expect("a api manifest");

            let loader = uvm_install_core::Loader::new(component, &manifest);

            let installer_path = loader.download().expect("path to installer");
            assert!(installer_path.exists());
        }
    }
}
