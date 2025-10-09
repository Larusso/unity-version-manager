use crate::commands::presentation::{
    as_view_iter, CategoryView, ModuleView, RenderOptions, TextRenderer,
};
use anyhow::Result;
use clap::Args;
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    io,
};
use unity_version::Version;
use uvm_install::InstallArchitecture;
use uvm_live_platform::{FetchRelease, Module, UnityReleaseDownloadPlatform};

#[derive(Args, Debug)]
pub struct ModulesCommand {
    /// filter by category
    #[arg(long, value_delimiter = ',')]
    category: Option<Vec<String>>,

    /// list also sync modules
    #[arg(long = "show-sync-modules", short)]
    show_sync_modules: bool,

    /// The api version to list modules for in the form of `2018.1.0f3`
    version: Version,

    /// list also invisible modules
    #[arg(short, long)]
    all: bool,

    /// list only modules for the current platform and architecture
    #[arg(short, long)]
    current: bool,

    /// print more output
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

struct UnityModule {
    module: Module,
    platform: UnityReleaseDownloadPlatform,
    architecture: InstallArchitecture,
}

fn load_modules<V: AsRef<Version>>(
    version: V,
    show_sync_modules: bool,
) -> Result<Vec<UnityModule>> {
    let version = version.as_ref();
    let release = FetchRelease::builder(version.clone())
        .with_extended_lts()
        .with_u7_alpha()
        .fetch()
        .map_err(|_e| io::Error::new(io::ErrorKind::NotFound, "failed to load release"))?;

    let downloads = release.downloads.iter();

    let modules = if show_sync_modules {
        downloads
            .flat_map(|download| {
                download.iter_modules().map(move |m| UnityModule {
                    module: m.clone(),
                    platform: download.platform,
                    architecture: download.architecture.into(),
                })
            })
            .collect()
    } else {
        downloads
            .flat_map(|download| {
                download.modules.iter().map(move |m| UnityModule {
                    module: m.clone(),
                    platform: download.platform,
                    architecture: download.architecture.into(),
                })
            })
            .collect()
    };

    Ok(modules)
}

impl ModulesCommand {
    pub fn execute(self) -> io::Result<i32> {
        match self.list() {
            Ok(_) => Ok(0),
            Err(e) => {
                eprintln!("Error: {}", e);
                Ok(1)
            }
        }
    }

    fn list(&self) -> Result<()> {
        let modules = load_modules(&self.version, self.show_sync_modules)?;
        let modules = modules
            .iter()
            .filter(|m| self.all || !m.module.hidden())
            .filter(|m| {
                if let Some(categories) = &self.category {
                    categories.contains(&m.module.category().to_string())
                } else {
                    true
                }
            })
            .fold(HashMap::new(), |mut acc, m| {
                if !acc.contains_key(&m.module.id().to_string()) {
                    let module_view = ModuleView {
                        id: m.module.id().to_string(),
                        description: m.module.description().to_string(),
                        platform_architectures: vec![format!("{}/{}", m.platform, m.architecture)],
                    };
                    acc.insert(
                        m.module.id().to_string(),
                        (m.module.category().to_string(), module_view),
                    );
                } else {
                    let (_, module_view) = acc.get_mut(&m.module.id().to_string()).unwrap();
                    module_view
                        .platform_architectures
                        .push(format!("{}/{}", m.platform, m.architecture));
                }
                acc
            })
            .into_values()
            .sorted_by(|m_1, m_2| match Ord::cmp(&m_1.0, &m_2.0) {
                std::cmp::Ordering::Equal => Ord::cmp(&m_1.1.id, &m_2.1.id),
                x => x,
            })
            .filter(|m| {
                if self.current {
                    let platform_string = format!(
                        "{}/{}",
                        UnityReleaseDownloadPlatform::default(),
                        InstallArchitecture::default()
                    );
                    m.1.platform_architectures.contains(&platform_string)
                } else {
                    true
                }
            });

        // Group modules by category
        let mut categories: std::collections::BTreeMap<String, Vec<ModuleView>> =
            std::collections::BTreeMap::new();
        for (category, module) in modules {
            categories.entry(category).or_default().push(module);
        }

        let category_views: Vec<CategoryView> = categories
            .into_iter()
            .map(|(category, modules)| CategoryView { category, modules })
            .collect();

        if category_views.is_empty() {
            if self.current {
                eprintln!("No modules found for the current platform and architecture");
            } else {
                eprintln!("No modules found");
            }
            return Ok(());
        }

        let renderer = TextRenderer::new(RenderOptions {
            path_only: false,
            verbose: self.verbose > 0,
            list_modules: self.show_sync_modules,
            no_color: false,
        });

        let rendered = renderer.render_view(&as_view_iter(category_views));
        print!("{}", rendered);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::presentation::{CategoryView, RenderOptions, View};

    fn create_test_module_view(id: &str, description: &str, _category: &str) -> ModuleView {
        ModuleView {
            id: id.to_string(),
            description: description.to_string(),
            platform_architectures: vec!["PLATFORM".to_string()],
        }
    }

    fn create_test_category_view(category: &str, modules: Vec<ModuleView>) -> CategoryView {
        CategoryView {
            category: category.to_string(),
            modules,
        }
    }

    #[test]
    fn module_view_renders_compact() {
        let module = create_test_module_view("android", "Android Build Support", "PLATFORM");
        let mut output = Vec::new();
        let opts = RenderOptions {
            no_color: true,
            ..Default::default()
        };

        module.render(&mut output, &opts).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("  * android"));
        assert!(!output_str.contains("Android Build Support"));
    }

    #[test]
    fn module_view_renders_verbose() {
        let module = create_test_module_view("ios", "iOS Build Support", "PLATFORM");
        let mut output = Vec::new();
        let opts = RenderOptions {
            verbose: true,
            no_color: true,
            ..Default::default()
        };

        module.render(&mut output, &opts).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("  * ios - iOS Build Support"));
    }

    #[test]
    fn category_view_renders_with_modules() {
        let modules = vec![
            create_test_module_view("android", "Android Build Support", "PLATFORM"),
            create_test_module_view("ios", "iOS Build Support", "PLATFORM"),
        ];
        let category = create_test_category_view("PLATFORM", modules);
        let mut output = Vec::new();
        let opts = RenderOptions {
            no_color: true,
            ..Default::default()
        };

        category.render(&mut output, &opts).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("PLATFORM:"));
        assert!(output_str.contains("  * android"));
        assert!(output_str.contains("  * ios"));
    }

    #[test]
    fn category_view_renders_verbose() {
        let modules = vec![create_test_module_view(
            "android",
            "Android Build Support",
            "PLATFORM",
        )];
        let category = create_test_category_view("PLATFORM", modules);
        let mut output = Vec::new();
        let opts = RenderOptions {
            verbose: true,
            no_color: true,
            ..Default::default()
        };

        category.render(&mut output, &opts).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("PLATFORM:"));
        assert!(output_str.contains("  * android - Android Build Support"));
    }

    // Child module tests removed since ModuleView from presentation.rs doesn't support children
}
