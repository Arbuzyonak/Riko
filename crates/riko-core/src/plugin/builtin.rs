use super::manifest::{self, PluginManifest};
use crate::RikoError;
use std::path::PathBuf;

pub struct BuiltinPlugin {
    pub name: &'static str,
    pub files: &'static [(&'static str, &'static [u8])],
}

pub const BUILTINS: &[BuiltinPlugin] = &[
    BuiltinPlugin {
        name: "fps-unlocker",
        files: &[
            (
                "plugin.toml",
                include_bytes!("../../plugins/fps-unlocker/plugin.toml"),
            ),
            (
                "present_mode_layer.c",
                include_bytes!("../../plugins/fps-unlocker/present_mode_layer.c"),
            ),
            (
                "VkLayer_vortstrap_present_mode.json",
                include_bytes!("../../plugins/fps-unlocker/VkLayer_vortstrap_present_mode.json"),
            ),
        ],
    },
    BuiltinPlugin {
        name: "vortex-optim",
        files: &[
            (
                "plugin.toml",
                include_bytes!("../../plugins/vortex-optim/plugin.toml"),
            ),
            (
                "optimizer.c",
                include_bytes!("../../plugins/vortex-optim/optimizer.c"),
            ),
        ],
    },
    BuiltinPlugin {
        name: "mangohud",
        files: &[(
            "plugin.toml",
            include_bytes!("../../plugins/mangohud/plugin.toml"),
        )],
    },
    BuiltinPlugin {
        name: "fsr-upscale",
        files: &[(
            "plugin.toml",
            include_bytes!("../../plugins/fsr-upscale/plugin.toml"),
        )],
    },
    BuiltinPlugin {
        name: "low-spec-mode",
        files: &[(
            "plugin.toml",
            include_bytes!("../../plugins/low-spec-mode/plugin.toml"),
        )],
    },
    BuiltinPlugin {
        name: "vkbasalt",
        files: &[
            (
                "plugin.toml",
                include_bytes!("../../plugins/vkbasalt/plugin.toml"),
            ),
            (
                "vkBasalt.conf",
                include_bytes!("../../plugins/vkbasalt/vkBasalt.conf"),
            ),
        ],
    },
    BuiltinPlugin {
        name: "replay-buffer",
        files: &[
            (
                "plugin.toml",
                include_bytes!("../../plugins/replay-buffer/plugin.toml"),
            ),
            (
                "replay.sh",
                include_bytes!("../../plugins/replay-buffer/replay.sh"),
            ),
        ],
    },
    BuiltinPlugin {
        name: "ping-logger",
        files: &[
            (
                "plugin.toml",
                include_bytes!("../../plugins/ping-logger/plugin.toml"),
            ),
            (
                "ping-logger.sh",
                include_bytes!("../../plugins/ping-logger/ping-logger.sh"),
            ),
        ],
    },
];

pub fn get(name: &str) -> Option<&'static BuiltinPlugin> {
    BUILTINS.iter().find(|b| b.name == name)
}

pub fn manifest_for(name: &str) -> Option<PluginManifest> {
    let builtin = get(name)?;
    let contents = builtin
        .files
        .iter()
        .find(|(file, _)| *file == "plugin.toml")?;
    manifest::parse(std::str::from_utf8(contents.1).ok()?).ok()
}

pub fn install_files(name: &str) -> Result<PathBuf, RikoError> {
    let builtin =
        get(name).ok_or_else(|| RikoError::Plugin(format!("unknown built-in plugin '{name}'")))?;
    let dir = super::plugins_dir().join(name);
    std::fs::create_dir_all(&dir)?;
    for (file, contents) in builtin.files {
        let path = dir.join(file);
        std::fs::write(&path, contents)?;
        #[cfg(unix)]
        if file.ends_with(".sh") {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
        }
    }
    Ok(dir)
}
