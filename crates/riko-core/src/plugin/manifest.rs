use crate::RikoError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginMeta,
    #[serde(default)]
    pub build: Option<BuildSpec>,
    #[serde(default)]
    pub env: HashMap<String, EnvValue>,
    #[serde(default)]
    pub vulkan_layer: Option<VulkanLayerSpec>,
    #[serde(default)]
    pub binary: Option<BinarySpec>,
    #[serde(default)]
    pub env_append: HashMap<String, EnvAppend>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(rename = "type")]
    pub kind: PluginKind,
    #[serde(default = "default_platforms")]
    pub platforms: Vec<String>,
}

fn default_platforms() -> Vec<String> {
    vec!["linux".to_string()]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginKind {
    VulkanLayer,
    Binary,
    EnvOnly,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildSpec {
    pub command: String,
    pub artifacts: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EnvValue {
    Fixed(String),
    Overridable { default: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VulkanLayerSpec {
    pub manifest: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BinarySpec {
    pub entrypoint: String,
    #[serde(default)]
    pub run_after_launch: bool,
    #[serde(default)]
    pub delay_secs: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvAppend {
    pub separator: String,
    pub values: Vec<String>,
}

pub fn parse(contents: &str) -> Result<PluginManifest, RikoError> {
    let manifest: PluginManifest =
        toml::from_str(contents).map_err(|e| RikoError::Plugin(format!("invalid plugin.toml: {e}")))?;
    validate(&manifest)?;
    Ok(manifest)
}

pub fn load(dir: &Path) -> Result<PluginManifest, RikoError> {
    let contents = std::fs::read_to_string(dir.join("plugin.toml"))
        .map_err(|e| RikoError::Plugin(format!("cannot read plugin.toml: {e}")))?;
    parse(&contents)
}

fn validate(manifest: &PluginManifest) -> Result<(), RikoError> {
    let name = &manifest.plugin.name;
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(RikoError::Plugin(format!(
            "plugin name '{name}' must be lowercase alphanumeric with dashes"
        )));
    }

    for platform in &manifest.plugin.platforms {
        if platform != "linux" && platform != "windows" {
            return Err(RikoError::Plugin(format!(
                "unknown platform '{platform}' (expected linux or windows)"
            )));
        }
    }

    let mut relative_paths: Vec<&str> = vec![];
    if let Some(build) = &manifest.build {
        if build.command.trim().is_empty() {
            return Err(RikoError::Plugin("build.command is empty".to_string()));
        }
        relative_paths.extend(build.artifacts.iter().map(String::as_str));
    }
    if let Some(layer) = &manifest.vulkan_layer {
        relative_paths.push(&layer.manifest);
    }
    if let Some(binary) = &manifest.binary {
        relative_paths.push(&binary.entrypoint);
    }
    for path in relative_paths {
        if path.is_empty() || path.starts_with('/') || path.contains("..") || path.contains('\\') {
            return Err(RikoError::Plugin(format!(
                "path '{path}' must be relative and inside the plugin directory"
            )));
        }
    }

    match manifest.plugin.kind {
        PluginKind::VulkanLayer if manifest.vulkan_layer.is_none() => Err(RikoError::Plugin(
            "type is vulkan-layer but [vulkan_layer] section is missing".to_string(),
        )),
        PluginKind::Binary if manifest.binary.is_none() => Err(RikoError::Plugin(
            "type is binary but [binary] section is missing".to_string(),
        )),
        _ => Ok(()),
    }
}

pub fn supported_on_current_platform(manifest: &PluginManifest) -> bool {
    let current = if cfg!(target_os = "windows") { "windows" } else { "linux" };
    manifest.plugin.platforms.iter().any(|p| p == current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_builtin_manifests() {
        for builtin in crate::plugin::builtin::BUILTINS {
            let manifest = crate::plugin::builtin::manifest_for(builtin.name)
                .unwrap_or_else(|| panic!("built-in '{}' has an invalid manifest", builtin.name));
            assert_eq!(manifest.plugin.name, builtin.name);
            if matches!(manifest.plugin.kind, PluginKind::Binary) {
                let entrypoint = &manifest.binary.as_ref().unwrap().entrypoint;
                let shipped = builtin.files.iter().any(|(file, _)| file == entrypoint);
                let built = manifest
                    .build
                    .as_ref()
                    .is_some_and(|b| b.artifacts.iter().any(|a| a == entrypoint));
                assert!(
                    shipped || built,
                    "built-in '{}' neither ships nor builds its entrypoint '{entrypoint}'",
                    builtin.name
                );
            }
        }
    }

    #[test]
    fn rejects_bad_name() {
        let toml = r#"
[plugin]
name = "Bad Name!"
version = "1.0"
description = "x"
type = "env-only"
"#;
        assert!(parse(toml).is_err());
    }

    #[test]
    fn rejects_escaping_paths() {
        let toml = r#"
[plugin]
name = "sneaky"
version = "1.0"
description = "x"
type = "binary"

[binary]
entrypoint = "../../../usr/bin/true"
"#;
        assert!(parse(toml).is_err());
    }

    #[test]
    fn rejects_missing_section_for_kind() {
        let toml = r#"
[plugin]
name = "layerless"
version = "1.0"
description = "x"
type = "vulkan-layer"
"#;
        assert!(parse(toml).is_err());
    }

    #[test]
    fn env_value_forms() {
        let toml = r#"
[plugin]
name = "envy"
version = "1.0"
description = "x"
type = "env-only"

[env]
FIXED = "1"
FLEX = { default = "0" }
"#;
        let manifest = parse(toml).unwrap();
        assert!(matches!(manifest.env["FIXED"], EnvValue::Fixed(ref v) if v == "1"));
        assert!(matches!(manifest.env["FLEX"], EnvValue::Overridable { ref default } if default == "0"));
    }
}
