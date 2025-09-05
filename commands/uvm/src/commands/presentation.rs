use console::Style;
use std::io::{self, Write};
use unity_hub::unity::{UnityInstallation, Installation};
use unity_hub::unity::hub::module::Module as HubModule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleView {
    pub id: String,
    pub description: String,
}

impl View for ModuleView {
    fn render(&self, w: &mut dyn Write, opts: &RenderOptions) -> io::Result<()> {
        let out_style = Style::new().cyan();

        if opts.verbose {
            writeln!(w, "  * {} - {}", out_style.apply_to(&self.id), out_style.apply_to(&self.description))?;
        } else {
            writeln!(w, "  * {}", out_style.apply_to(&self.id))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationView {
    pub version: String,
    pub path: String,
    pub modules: Vec<ModuleView>,
}

impl View for InstallationView {
    fn render(&self, w: &mut dyn Write, opts: &RenderOptions) -> io::Result<()> {
        let out_style = Style::new().cyan();
        let path_style = Style::new().italic().green();

        if !opts.path_only {
            write!(w, "{}", out_style.apply_to(&self.version))?;
        }
        if opts.verbose {
            write!(w, " - ")?;
        }
        if opts.verbose || opts.path_only {
            write!(w, "{}", path_style.apply_to(&self.path))?;
        }
        writeln!(w)?;

        if opts.list_modules {
            for m in self.modules.iter() {
                m.render(w, opts)?;
            }
        }
        Ok(())
    }
}

// impl View for dyn IntoIterator {
//     type Item = dynView;

//     fn render<W: Write>(&self, w: W, opts: &RenderOptions) -> io::Result<()> {
//         for item in self {
//             item.render(&mut w, opts)?;
//         }
//         Ok(())
//     }
// }

pub trait View {
    fn render(&self, w: &mut dyn Write, opts: &RenderOptions) -> io::Result<()>;
}

impl<T: View + ?Sized> View for &T {
    fn render(&self, w: &mut dyn Write, opts: &RenderOptions) -> io::Result<()> {
        (**self).render(w, opts)
    }
}

pub struct ViewIter<I>(pub I);

impl<I, T> View for ViewIter<I>
where
    I: Clone + IntoIterator<Item = T>,
    T: View,
{
    fn render(&self, w: &mut dyn Write, opts: &RenderOptions) -> io::Result<()> {
        for v in self.0.clone().into_iter() {
            v.render(w, opts)?;
        }
        Ok(())
    }
}

pub fn as_view_iter<I, T>(iter: I) -> ViewIter<I>
where
    I: IntoIterator<Item = T>,
    T: View,
{
    ViewIter(iter)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RenderOptions {
    pub path_only: bool,
    pub verbose: bool,
    pub list_modules: bool,
}

pub struct TextRenderer {
    pub opts: RenderOptions,
}

impl TextRenderer {
    pub fn new(opts: RenderOptions) -> Self {
        Self { opts }
    }

    pub fn render_view(&self, v: &dyn View) -> String {
        let mut buf = Vec::new();
        let _ = v.render(&mut buf, &self.opts);
        String::from_utf8(buf).unwrap_or_default()
    }

    pub fn render_to_string(&self, items: impl IntoIterator<Item = InstallationView>) -> String {
        let items: Vec<_> = items.into_iter().collect();
        self.render_view(&as_view_iter(items))
    }
}

// Zero-allocation View impls for domain types
impl View for HubModule {
    fn render(&self, w: &mut dyn Write, opts: &RenderOptions) -> io::Result<()> {
        let out_style = Style::new().cyan();
        if opts.verbose {
            writeln!(w, "  * {} - {}", out_style.apply_to(self.id()), out_style.apply_to(self.base.description()))
        } else {
            writeln!(w, "  * {}", out_style.apply_to(self.id()))
        }
    }
}

impl View for UnityInstallation {
    fn render(&self, w: &mut dyn Write, opts: &RenderOptions) -> io::Result<()> {
        let out_style = Style::new().cyan();
        let path_style = Style::new().italic().green();

        if !opts.path_only {
            write!(w, "{}", out_style.apply_to(self.version().to_string()))?;
        }
        if opts.verbose {
            write!(w, " - ")?;
        }
        if opts.verbose || opts.path_only {
            write!(w, "{}", path_style.apply_to(self.path().display()))?;
        }
        writeln!(w)?;

        if opts.list_modules {
            if let Ok(mods) = self.installed_modules() {
                for m in mods {
                    m.render(w, opts)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(v: &str, p: &str) -> InstallationView {
        InstallationView { version: v.to_string(), path: p.to_string(), modules: vec![] }
    }

    #[test]
    fn render_default_shows_version_only() {
        let items = vec![item("2021.3.1f1", "/path/Editor"), item("6000.1.8f1", "/p2")];
        let s = TextRenderer::new(RenderOptions::default()).render_to_string(items);
        assert!(s.contains("2021.3.1f1"));
        assert!(s.contains("6000.1.8f1"));
        assert!(!s.contains("/path/Editor"));
    }

    #[test]
    fn render_path_only_shows_paths() {
        let items = vec![item("2021.3.1f1", "/path/Editor")];
        let s = TextRenderer::new(RenderOptions { path_only: true, ..Default::default() }).render_to_string(items);
        assert!(s.contains("/path/Editor"));
        assert!(!s.contains("2021.3.1f1"));
    }

    #[test]
    fn render_verbose_shows_version_and_path() {
        let items = vec![item("2021.3.1f1", "/path/Editor")];
        let s = TextRenderer::new(RenderOptions { verbose: true, ..Default::default() }).render_to_string(items);
        assert!(s.contains("2021.3.1f1"));
        assert!(s.contains("/path/Editor"));
        assert!(s.contains(" - "));
    }

    #[test]
    fn render_modules_compact() {
        let mut it = item("2021.3.1f1", "/path");
        it.modules = vec![ModuleView { id: "android".into(), description: "Android Build Support".into() }];
        let s = TextRenderer::new(RenderOptions { list_modules: true, ..Default::default() }).render_to_string(vec![it]);
        assert!(s.contains("2021.3.1f1"));
        assert!(s.contains("  * android"));
        assert!(!s.contains("Android Build Support"));
    }

    #[test]
    fn render_modules_verbose_includes_description() {
        let mut it = item("2021.3.1f1", "/path");
        it.modules = vec![ModuleView { id: "ios".into(), description: "iOS Build Support".into() }];
        let s = TextRenderer::new(RenderOptions { list_modules: true, verbose: true, ..Default::default() }).render_to_string(vec![it]);
        assert!(s.contains("  * ios - iOS Build Support"));
    }
}


