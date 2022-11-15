use std::{
    collections::HashMap,
    fs::{File, self},
    io::{self, BufReader, BufRead},
    path::{Path, PathBuf}, str::FromStr,
};

use crate::Version;

fn get_project_version<P: AsRef<Path>>(base_dir: P) -> io::Result<PathBuf> {
    let project_version = base_dir
        .as_ref()
        .join("ProjectSettings")
        .join("ProjectVersion.txt");
    if project_version.exists() {
        Ok(project_version)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "directory {} is not a Unity project",
                base_dir.as_ref().display()
            ),
        ))
    }
}

pub fn detect_unity_project_dir(dir: &Path, recur: bool) -> io::Result<PathBuf> {
    let error = Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Unable to find a Unity project",
    ));

    if dir.is_dir() {
        if get_project_version(dir).is_ok() {
            return Ok(dir.to_path_buf());
        } else if !recur {
            return error;
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let f = detect_unity_project_dir(&path, true);
                if f.is_ok() {
                    return f;
                }
            }
        }
    }
    error
}

pub fn dectect_project_version(project_path: &Path, recur: Option<bool>) -> io::Result<Version> {
    let project_version = detect_unity_project_dir(project_path, recur.unwrap_or(false))
        .and_then(get_project_version)?;

    let file = File::open(project_version)?;
    let lines = BufReader::new(file).lines();

    let mut editor_versions: HashMap<&'static str, String> = HashMap::with_capacity(2);

    for line in lines {
        if let Ok(line) = line {
            if line.starts_with("m_EditorVersion: ") {
                editor_versions.insert("EditorVersion", line.to_owned());
            }

            if line.starts_with("m_EditorVersionWithRevision: ") {
                editor_versions.insert("EditorVersionWithRevision", line.to_owned());
            }
        }
    }

    let v = editor_versions
        .get("EditorVersionWithRevision")
        .or_else(|| editor_versions.get("EditorVersion"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Can't parse Unity version"))?;
    Version::from_str(&v)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Can't parse Unity version"))
}

#[cfg(test)]
mod tests {
    use std::fs::OpenOptions;

    use crate::unity::VersionType;

    use super::*;
    use indoc::indoc;
    use std::io::Write;
    use tempfile::tempdir;

    fn setup_fake_unity_project(path: &Path, project_version_content: &str) {
        let project_settings_dir = path.join("ProjectSettings");
        std::fs::create_dir(&project_settings_dir).expect("create ProjectSettings dir");
        let project_version_file_path = project_settings_dir.join("ProjectVersion.txt");

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(project_version_file_path)
            .expect("an opened ProjectVersion.txt");
        file.write(project_version_content.as_bytes()).unwrap();
    }

    #[test]
    fn dectect_project_version_prior_2019() {
        let dir = tempdir().expect("a temp directory");
        let content = indoc! {"
            m_EditorVersion: 2020.3.38f1
        "};

        setup_fake_unity_project(dir.path(), content);
        let version = dectect_project_version(dir.path(), Some(true)).unwrap();

        let expected_version = Version::new(2020, 3, 38, VersionType::Final, 1);
        assert_eq!(version, expected_version);
        assert!(!version.has_version_hash());
    }

    #[test]
    fn dectect_project_version_after_2019() {
        let dir = tempdir().expect("a temp directory");
        let content = indoc! {"
            m_EditorVersion: 2020.3.38f1
            m_EditorVersionWithRevision: 2020.3.38f1 (8f5fde82e2dc)
        "};

        setup_fake_unity_project(dir.path(), content);
        let version = dectect_project_version(dir.path(), Some(true)).unwrap();

        let mut expected_version = Version::new(2020, 3, 38, VersionType::Final, 1);
        expected_version.set_version_hash(Some("8f5fde82e2dc".to_string()));

        assert_eq!(version, expected_version);
        assert!(version.has_version_hash());
    }

    #[test]
    fn dectect_project_version_after_2019_with_altered_order() {
        let dir = tempdir().expect("a temp directory");
        let content = indoc! {"
            m_EditorVersionWithRevision: 2020.3.38f1 (8f5fde82e2dc)
            m_EditorVersion: 2020.3.38f1
        "};

        setup_fake_unity_project(dir.path(), content);
        let version = dectect_project_version(dir.path(), Some(true)).unwrap();

        let mut expected_version = Version::new(2020, 3, 38, VersionType::Final, 1);
        expected_version.set_version_hash(Some("8f5fde82e2dc".to_string()));

        assert_eq!(version, expected_version);
        assert!(version.has_version_hash());
    }
}
