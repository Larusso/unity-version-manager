use anyhow::Result;
use clap::Args;
use itertools::Itertools;
use std::io;
use uvm_live_platform::{fetch_release, Module as LiveModule};
use unity_version::Version;
use crate::commands::presentation::{RenderOptions, TextRenderer, View, as_view_iter};

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

    /// print more output
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleView {
    pub id: String,
    pub description: String,
    pub category: String,
    pub children: Vec<ModuleView>,
}

impl View for ModuleView {
    fn render(&self, w: &mut dyn io::Write, opts: &RenderOptions) -> io::Result<()> {
        use console::Style;
        let out_style = Style::new().cyan();
        let path_style = Style::new().italic().green();

        if opts.verbose {
            writeln!(w, "  * {} - {}", out_style.apply_to(&self.id), path_style.apply_to(&self.description))?;
        } else {
            writeln!(w, "  * {}", out_style.apply_to(&self.id))?;
        }

        if opts.list_modules && !self.children.is_empty() {
            for child in &self.children {
                child.render(w, opts)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryView {
    pub category: String,
    pub modules: Vec<ModuleView>,
}

impl View for CategoryView {
    fn render(&self, w: &mut dyn io::Write, opts: &RenderOptions) -> io::Result<()> {
        use console::Style;
        let category_style = Style::new().white().bold();
        
        writeln!(w, "{}:", category_style.apply_to(&self.category))?;
        
        for module in &self.modules {
            module.render(w, opts)?;
        }
        
        Ok(())
    }
}

pub fn load_modules<V: AsRef<Version>>(version: V) -> Result<Vec<LiveModule>> {
    let version = version.as_ref();
    let release = fetch_release(version.clone())
        .map_err(|_e| io::Error::new(io::ErrorKind::NotFound, "failed to load release"))?;
    
    // Get modules from the first download (assuming single platform)
    let modules = release.downloads
        .first()
        .map(|download| download.modules.clone())
        .unwrap_or_default();
    
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
        let modules = load_modules(&self.version)?;

        let modules = modules
            .iter()
            .filter(|m| self.all || !m.hidden())
            .filter(|m| {
                if let Some(categories) = &self.category {
                    categories.contains(&m.category().to_string())
                } else {
                    true
                }
            })
            .sorted_by(|m_1, m_2| match Ord::cmp(&m_1.category(), &m_2.category()) {
                std::cmp::Ordering::Equal => Ord::cmp(&m_1.id().to_string(), &m_2.id().to_string()),
                x => x,
            })
            .map(|m| ModuleView {
                id: m.id().to_string(),
                description: m.description().to_string(),
                category: m.category().to_string(),
                children: Vec::new(), // For now, we'll keep children empty
            })
            .filter(|m| m.children.is_empty()); // Only show top-level modules for now

        // Group modules by category
        let mut categories: std::collections::BTreeMap<String, Vec<ModuleView>> = std::collections::BTreeMap::new();
        for module in modules {
            categories.entry(module.category.clone()).or_default().push(module);
        }

        let category_views: Vec<CategoryView> = categories
            .into_iter()
            .map(|(category, modules)| CategoryView { category, modules })
            .collect();

        let renderer = TextRenderer::new(RenderOptions { 
            path_only: false, 
            verbose: self.verbose > 0, 
            list_modules: self.show_sync_modules 
        });
        
        let rendered = renderer.render_view(&as_view_iter(category_views));
        print!("{}", rendered);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::presentation::RenderOptions;

    fn create_test_module_view(id: &str, description: &str, category: &str) -> ModuleView {
        ModuleView {
            id: id.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            children: Vec::new(),
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
        let opts = RenderOptions::default();
        
        module.render(&mut output, &opts).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        
        assert!(output_str.contains("  * android"));
        assert!(!output_str.contains("Android Build Support"));
    }

    #[test]
    fn module_view_renders_verbose() {
        let module = create_test_module_view("ios", "iOS Build Support", "PLATFORM");
        let mut output = Vec::new();
        let opts = RenderOptions { verbose: true, ..Default::default() };
        
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
        let opts = RenderOptions::default();
        
        category.render(&mut output, &opts).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        
        assert!(output_str.contains("PLATFORM:"));
        assert!(output_str.contains("  * android"));
        assert!(output_str.contains("  * ios"));
    }

    #[test]
    fn category_view_renders_verbose() {
        let modules = vec![
            create_test_module_view("android", "Android Build Support", "PLATFORM"),
        ];
        let category = create_test_category_view("PLATFORM", modules);
        let mut output = Vec::new();
        let opts = RenderOptions { verbose: true, ..Default::default() };
        
        category.render(&mut output, &opts).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        
        assert!(output_str.contains("PLATFORM:"));
        assert!(output_str.contains("  * android - Android Build Support"));
    }

    #[test]
    fn module_view_with_children_renders_children_when_list_modules_enabled() {
        let child = create_test_module_view("android-sdk", "Android SDK", "PLATFORM");
        let mut parent = create_test_module_view("android", "Android Build Support", "PLATFORM");
        parent.children = vec![child];
        
        let mut output = Vec::new();
        let opts = RenderOptions { list_modules: true, ..Default::default() };
        
        parent.render(&mut output, &opts).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        
        assert!(output_str.contains("  * android"));
        assert!(output_str.contains("  * android-sdk"));
    }

    #[test]
    fn module_view_with_children_does_not_render_children_when_list_modules_disabled() {
        let child = create_test_module_view("android-sdk", "Android SDK", "PLATFORM");
        let mut parent = create_test_module_view("android", "Android Build Support", "PLATFORM");
        parent.children = vec![child];
        
        let mut output = Vec::new();
        let opts = RenderOptions { list_modules: false, ..Default::default() };
        
        parent.render(&mut output, &opts).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        
        assert!(output_str.contains("  * android"));
        assert!(!output_str.contains("android-sdk"));
    }
}
