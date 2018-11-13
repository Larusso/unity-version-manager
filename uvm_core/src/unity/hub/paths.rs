use std::path::PathBuf;
use std::fs::File;

pub fn default_install_path() -> Option<PathBuf> {
    dirs::application_dir().map(|path| {
        path.join(["Unity","Hub","Editor"].iter().collect::<PathBuf>())
    })
}

pub fn install_path() -> Option<PathBuf> {
    secondary_install_path_config_path().and_then(|path|{
        File::open(path).and_then(|file| {
            let path:PathBuf = serde_json::from_reader(file)?;
            Ok(path)
        }).ok()
    })
    //filter out the default value `""` in secondaryInstallPath.json
    .filter(|p| p.as_os_str() != std::ffi::OsStr::new(""))
    .or_else(default_install_path)
}

pub fn config_path() -> Option<PathBuf> {
    dirs::data_dir().map(|path| path.join("UnityHub"))
}

pub fn editors_config_path() -> Option<PathBuf> {
    config_path().map(|path| path.join("editors.json"))
}

pub fn secondary_install_path_config_path() -> Option<PathBuf> {
    config_path().map(|path| path.join("secondaryInstallPath.json"))
}

pub fn default_editor_config_path() -> Option<PathBuf> {
    config_path().map(|path| path.join("defaultEditor.json"))
}

pub fn cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|path| path.join("Wooga").join("Unity Version Manager"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirs() {
        println!("default_editor_config_path:          {:?}", default_editor_config_path());
        println!("secondary_install_path_config_path:  {:?}", secondary_install_path_config_path());
        println!("editors_config_path:                 {:?}", editors_config_path());
        println!("config_path:                         {:?}", config_path());
        println!("install_path:                        {:?}", install_path());
        println!("default_install_path:                {:?}", default_install_path());
        println!("cache_dir:                           {:?}", cache_dir());
    }
}
