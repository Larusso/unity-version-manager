use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    #[serde(flatten)]
    pub base: uvm_live_platform::Module,
    #[serde(default)]
    pub is_installed: bool,
    #[serde(flatten)]
    module_backwards_compatible: ModuleBackwardsCompatible,
}

impl Module {
    pub fn id(&self) -> &str {
        &self.base.id()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleBackwardsCompatible {
    rename_from: String,
    rename_to: String,
    sync: String,
    parent: String,
    visible: bool,
    preselected: bool,
    eula_url_1: String,
    eula_label_1: String,
    eula_message: String,
}

impl From<&uvm_live_platform::Module> for ModuleBackwardsCompatible {
    fn from(value: &uvm_live_platform::Module) -> Self {
        let (rename_from, rename_to) = value
            .extracted_path_rename().as_ref()
            .map(|e| {
                (
                    e.from.to_path_buf().display().to_string(),
                    e.to.to_path_buf().display().to_string(),
                )
            })
            .unwrap_or(("".to_string(), "".to_string()));
        let visible = !value.hidden();
        let preselected = value.pre_selected();

        let (eula_url_1, eula_label_1, eula_message) = value.eula().first().as_ref().map(|eula| {
            (eula.release_file.url.to_string(), eula.label.to_string(), eula.message.to_string())
        }).unwrap_or(("".to_string(), "".to_string(), "".to_string()));


        Self {
            rename_from,
            rename_to,
            visible,
            preselected,
            sync: "".to_string(),
            parent: "".to_string(),
            eula_url_1,
            eula_label_1,
            eula_message,
        }
    }
}

impl From<uvm_live_platform::Module> for Module {
    fn from(mut value: uvm_live_platform::Module) -> Self {
        value.download_size.as_bytes_representation();
        value.installed_size.as_bytes_representation();
        let module_backwards_compatible = ModuleBackwardsCompatible::from(&value);
        Self {
            base: value,
            is_installed: false,
            module_backwards_compatible
        }
    }
}
