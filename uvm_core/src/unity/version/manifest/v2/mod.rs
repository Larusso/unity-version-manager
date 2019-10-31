use super::ini::IniManifest;
use crate::error::*;
use crate::unity::urls::DownloadURL;
use crate::unity::{Component, Module, Modules, ModulesMap, Version};
use reqwest::Url;
use std::collections::hash_map::Iter;
use std::fs::File;
use std::io::Read;
use std::ops::{Deref, DerefMut};
use std::path::Path;

#[derive(Debug)]
pub struct Manifest<'a> {
    version: &'a Version,
    base_url: DownloadURL,
    modules: ModulesMap,
    editor: Module,
}

impl<'a> Manifest<'a> {
    pub fn load(version: &'a Version) -> Result<Manifest<'a>> {
        let mut components = IniManifest::load(version)?;
        let editor = components
            .remove(&Component::Editor)
            .ok_or(UvmErrorKind::ManifestReadError)?;
        let editor = Module::from(((Component::Editor, editor), version));

        let modules: ModulesMap = Self::get_modules(components, version);
        let base_url = DownloadURL::new(&version)?;

        Ok(Manifest {
            version,
            base_url,
            modules,
            editor,
        })
    }

    pub fn read_manifest_version_from_path<P: AsRef<Path>>(manifest_path: P) -> Result<Version> {
        IniManifest::read_manifest_version_from_path(manifest_path)
    }

    pub fn read_manifest_version<R: Read>(reader: R) -> Result<Version> {
        IniManifest::read_manifest_version(reader)
    }

    pub fn from_reader<R: Read>(version: &'a Version, manifest: R) -> Result<Manifest<'a>> {
        let base_url = DownloadURL::new(&version)?;
        let mut components = IniManifest::from_reader(version, manifest)?;
        let editor = components
            .remove(&Component::Editor)
            .ok_or(UvmErrorKind::ManifestReadError)?;
        let editor = Module::from(((Component::Editor, editor), version));
        let modules: ModulesMap = Self::get_modules(components, version);

        Ok(Manifest {
            version,
            base_url,
            modules,
            editor,
        })
    }

    pub fn new<P: AsRef<Path>>(version: &'a Version, manifest_path: P) -> Result<Manifest<'a>> {
        let manifest = File::open(manifest_path)?;
        Self::from_reader(version, manifest)
    }

    fn get_modules(components: IniManifest, version: &Version) -> ModulesMap {
        use crate::unity::version::module::ModuleBuilder;

        let modules = ModuleBuilder::from(components, version);
        modules.into_iter().collect()
    }

    pub fn url(&self, component: Component) -> Option<Url> {
        self.get(&component)
            .and_then(|m| Url::parse(&m.download_url).ok())
    }

    pub fn size(&self, component: Component) -> Option<u64> {
        self.get(&component).map(|m| m.download_size)
    }

    pub fn version(&self) -> &Version {
        self.version
    }

    pub fn iter(&self) -> Iter<'_, Component, Module> {
        self.modules.iter()
    }

    pub fn get(&self, component: &Component) -> Option<&Module> {
        match component {
            Component::Editor => Some(&self.editor),
            _ => self.modules.get(component),
        }
    }

    pub fn get_mut(&mut self, component: &Component) -> Option<&mut Module> {
        match component {
            Component::Editor => Some(&mut self.editor),
            _ => self.modules.get_mut(component),
        }
    }

    pub fn mark_installed_modules<C>(&mut self, components: C)
    where
        C: IntoIterator<Item = Component>,
    {
        for component in components {
            if let Some(m) = self.get_mut(&component) {
                m.selected = true;
            }
        }
    }

    pub fn modules_map(&self) -> &ModulesMap {
        &self.modules
    }

    /// Takes ownership of the `Manifest` and returns the internal `Modules` map.
    pub fn into_modules_map(self) -> ModulesMap {
        self.modules
    }

    pub fn into_modules(self) -> Modules {
        self.modules.into()
    }
}

impl Deref for Manifest<'_> {
    type Target = ModulesMap;

    fn deref(&self) -> &Self::Target {
        self.modules_map()
    }
}

impl DerefMut for Manifest<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.modules
    }
}
