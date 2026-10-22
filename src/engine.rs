// Copyright (C) 2026 EfeDursun125
// This file is part of AMDGPU Tuner; see NOTICE and LICENSE.
// SPDX-License-Identifier: GPL-3.0-only
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

pub type Environment = BTreeMap<String, String>;
#[derive(Default)]
pub struct EnvironmentChanges {
    pub set: Environment,
    pub unset: BTreeSet<String>,
}
impl EnvironmentChanges {
    fn is_empty(&self) -> bool {
        self.set.is_empty() && self.unset.is_empty()
    }
    fn apply(&self, command: &mut Command) {
        for key in &self.unset {
            command.env_remove(key);
        }
        command.envs(&self.set);
    }
}
pub const LOW_LATENCY_LAYER: &str = "VK_LAYER_KORTHOS_low_latency";
#[derive(Clone, Copy)]
pub enum Kind {
    Layer(&'static str),
    Bool,
    Vblank,
    Vrs,
    Flag(&'static str),
}
pub struct Setting {
    pub id: &'static str,
    pub title: &'static str,
    pub variable: &'static str,
    pub kind: Kind,
    pub group: i32,
    pub description: &'static str,
}
pub static SETTINGS: &[Setting] = &[
    Setting {
        id: "anti-lag",
        title: "Mesa Anti-Lag",
        variable: "ENABLE_LAYER_MESA_ANTI_LAG",
        kind: Kind::Layer("VK_LAYER_MESA_anti_lag"),
        group: 0,
        description: "Vulkan Anti-Lag layer. The application must use the extension.",
    },
    Setting {
        id: "low-latency",
        title: "Low Latency Layer",
        variable: "LOW_LATENCY_LAYER",
        kind: Kind::Layer("VK_LAYER_KORTHOS_low_latency"),
        group: 0,
        description: "Enable the installed Korthos Vulkan low-latency layer.",
    },
    Setting {
        id: "vsync",
        title: "OpenGL vertical sync",
        variable: "vblank_mode",
        kind: Kind::Vblank,
        group: 0,
        description: "On: 3 (force sync), off: 0. Does not affect Vulkan presentation.",
    },
    Setting {
        id: "glthread",
        title: "OpenGL multithreading",
        variable: "mesa_glthread",
        kind: Kind::Bool,
        group: 0,
        description: "Threaded OpenGL command processing. Defaults depend on driver and game profiles.",
    },
    Setting {
        id: "nomemfault",
        title: "Skip memory fault checks",
        variable: "RADV_DEBUG",
        kind: Kind::Flag("nomemfault"),
        group: 1,
        description: "Request nomemfault. A debug flag with driver-version-dependent support.",
    },
    Setting {
        id: "vrs",
        title: "Variable rate shading · 2×2",
        variable: "RADV_FORCE_VRS",
        kind: Kind::Vrs,
        group: 1,
        description: "On: 2x2, off: 1x1. Requires compatible hardware and pipelines.",
    },
    Setting {
        id: "dccmsaa",
        title: "DCC · MSAA",
        variable: "RADV_PERFTEST",
        kind: Kind::Flag("dccmsaa"),
        group: 1,
        description: "Request dccmsaa. An absent flag does not mean the feature is disabled.",
    },
    Setting {
        id: "gpl",
        title: "Graphics Pipeline Library",
        variable: "RADV_PERFTEST",
        kind: Kind::Flag("gpl"),
        group: 1,
        description: "Request gpl. Modern RADV versions may enable this automatically.",
    },
    Setting {
        id: "sam",
        title: "Smart Access Memory",
        variable: "RADV_PERFTEST",
        kind: Kind::Flag("sam"),
        group: 1,
        description: "Request sam. Does not change hardware ReBAR / SAM configuration.",
    },
    Setting {
        id: "nircache",
        title: "NIR shader cache",
        variable: "RADV_PERFTEST",
        kind: Kind::Flag("nircache"),
        group: 1,
        description: "Request nircache. Support and driver defaults depend on the installed version.",
    },
    Setting {
        id: "nggc",
        title: "NGG culling",
        variable: "RADV_PERFTEST",
        kind: Kind::Flag("nggc"),
        group: 1,
        description: "Request nggc. Removing this flag may leave automatic optimizations enabled.",
    },
    Setting {
        id: "transfer",
        title: "Transfer queue",
        variable: "RADV_EXPERIMENTAL",
        kind: Kind::Flag("transfer_queue"),
        group: 2,
        description: "Request experimental transfer_queue. Unsupported versions may ignore it.",
    },
];

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub overrides: BTreeMap<String, bool>,
}
#[derive(Clone, Debug)]
pub struct Layer {
    pub name: String,
    pub path: String,
    pub library_present: bool,
    pub enable: Environment,
    pub disable: Environment,
}
#[derive(Clone, Debug, Default)]
pub struct System {
    pub environment: Environment,
    pub sources: BTreeMap<String, String>,
    pub layers: Vec<Layer>,
    pub notes: Vec<String>,
}
#[derive(Clone, Debug)]
pub struct Reading {
    pub checked: Option<bool>,
    pub label: String,
    pub source: String,
}

pub fn config_dir() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".config"))
}
pub fn profile_path() -> PathBuf {
    config_dir().join("amdgpu-tuner/profile.json")
}
pub fn session_path() -> PathBuf {
    config_dir().join("environment.d/90-amdgpu-tuner.conf")
}

impl Profile {
    pub fn load(path: &Path) -> Result<Self, String> {
        match fs::read_to_string(path) {
            Ok(s) => serde_json::from_str(&s).map_err(|e| format!("Could not read profile: {e}")),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.to_string()),
        }
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        atomic_write(
            path,
            &serde_json::to_string_pretty(self).map_err(|e| e.to_string())?,
        )
    }
}
pub fn atomic_write(path: &Path, text: &str) -> Result<(), String> {
    let parent = path.parent().ok_or("Invalid file path")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temp = parent.join(format!(".amdgpu-tuner-{}.tmp", std::process::id()));
    fs::write(&temp, text).map_err(|e| e.to_string())?;
    fs::rename(&temp, path).map_err(|e| e.to_string())
}

impl System {
    pub fn detect() -> Self {
        let mut this = Self {
            environment: env::vars().collect(),
            ..Self::default()
        };
        // The process environment is the effective inherited environment for child applications.
        // systemd's environment is deliberately NOT merged: it can differ from this desktop process.
        for key in this.environment.keys() {
            this.sources
                .insert(key.clone(), "Inherited process environment".into());
        }
        let mut dirs = vec![config_dir().join("vulkan/implicit_layer.d")];
        let data_home = env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".local/share")
            });
        dirs.push(data_home.join("vulkan/implicit_layer.d"));
        for base in env::var("XDG_CONFIG_DIRS")
            .unwrap_or_else(|_| "/etc/xdg".into())
            .split(':')
        {
            dirs.push(PathBuf::from(base).join("vulkan/implicit_layer.d"));
        }
        dirs.push(PathBuf::from("/etc/vulkan/implicit_layer.d"));
        for base in env::var("XDG_DATA_DIRS")
            .unwrap_or_else(|_| "/usr/local/share:/usr/share".into())
            .split(':')
        {
            dirs.push(PathBuf::from(base).join("vulkan/implicit_layer.d"));
        }
        if let Some(paths) = this.environment.get("VK_IMPLICIT_LAYER_PATH") {
            dirs = env::split_paths(paths).collect();
        } else if let Some(paths) = this.environment.get("VK_ADD_IMPLICIT_LAYER_PATH") {
            dirs.splice(0..0, env::split_paths(paths));
        }
        let mut library_dirs: Vec<PathBuf> = env::var_os("LD_LIBRARY_PATH")
            .map(|paths| env::split_paths(&paths).collect())
            .unwrap_or_default();
        library_dirs.extend(
            [
                "/usr/lib",
                "/usr/lib64",
                "/usr/local/lib",
                "/lib",
                "/lib64",
                "/usr/lib/x86_64-linux-gnu",
                "/lib/x86_64-linux-gnu",
                "/usr/lib/aarch64-linux-gnu",
                "/lib/aarch64-linux-gnu",
            ]
            .map(PathBuf::from),
        );
        // Loader filters may still affect whether an installed layer is actually loaded.
        for key in [
            "VK_IMPLICIT_LAYER_PATH",
            "VK_ADD_IMPLICIT_LAYER_PATH",
            "VK_LOADER_LAYERS_DISABLE",
            "VK_LOADER_LAYERS_ENABLE",
            "VK_INSTANCE_LAYERS",
        ] {
            if let Some(value) = this.environment.get(key) {
                this.notes.push(format!(
                    "{key}={value}: Vulkan loader overrides may affect layer selection."
                ));
            }
        }
        for dir in dirs {
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };
            let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
            paths.sort();
            for path in paths {
                if path.extension().is_none_or(|e| e != "json") {
                    continue;
                }
                let Ok(bytes) = fs::read(&path) else { continue };
                let Ok(json) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
                    continue;
                };
                let layers: Vec<_> = json
                    .get("layer")
                    .into_iter()
                    .chain(
                        json.get("layers")
                            .and_then(|l| l.as_array())
                            .into_iter()
                            .flatten(),
                    )
                    .collect();
                for layer in layers {
                    let Some(name) = layer.get("name").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let map = |key: &str| -> Environment {
                        layer
                            .get(key)
                            .and_then(|v| v.as_object())
                            .into_iter()
                            .flatten()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.into())))
                            .collect()
                    };
                    let candidate = Layer {
                        name: name.into(),
                        path: path.display().to_string(),
                        library_present: layer
                            .get("library_path")
                            .and_then(|v| v.as_str())
                            .is_some_and(|library| {
                                layer_library_present(&path, library, &library_dirs)
                            }),
                        enable: map("enable_environment"),
                        disable: map("disable_environment"),
                    };
                    if let Some(index) = this.layers.iter().position(|l| l.name == name) {
                        if !this.layers[index].library_present && candidate.library_present {
                            this.layers[index] = candidate;
                        }
                    } else {
                        this.layers.push(candidate);
                    }
                }
            }
        }
        this
    }
    pub fn setting_available(&self, setting: &Setting) -> bool {
        setting.id != "low-latency"
            || self
                .layers
                .iter()
                .any(|layer| layer.name == LOW_LATENCY_LAYER && layer.library_present)
    }
    pub fn read(&self, setting: &Setting, profile: &Profile) -> Reading {
        if !self.setting_available(setting) {
            return Reading {
                checked: None,
                label: "Not installed · setting hidden".into(),
                source: "Requires the Korthos low_latency_layer manifest and shared library".into(),
            };
        }
        let computed = self.compose(profile);
        let overridden = profile.overrides.contains_key(setting.id);
        let value = computed.get(setting.variable);
        let base_source = if overridden {
            "Manual override · profile".into()
        } else {
            self.sources
                .get(setting.variable)
                .cloned()
                .unwrap_or_else(|| "Not set in the environment".into())
        };
        match setting.kind {
            Kind::Layer(name) => {
                let Some(layer) = self.layers.iter().find(|l| l.name == name) else {
                    return Reading {
                        checked: profile.overrides.get(setting.id).copied(),
                        label: "Layer not found · profile request only".into(),
                        source: format!("{} · an override can be requested", setting.variable),
                    };
                };
                let disabled = layer
                    .disable
                    .iter()
                    .find(|(k, _)| computed.contains_key(*k));
                let enabled = layer.enable.is_empty()
                    || layer.enable.iter().any(|(k, v)| computed.get(k) == Some(v));
                let active = disabled.is_none() && enabled;
                Reading {
                    checked: Some(active),
                    label: if disabled.is_some() {
                        "Off · disabled by environment".into()
                    } else if active {
                        "On · layer requested".into()
                    } else {
                        "Off · layer not requested".into()
                    },
                    source: format!(
                        "{} · {}{}",
                        base_source,
                        layer.path,
                        if self.notes.is_empty() {
                            ""
                        } else {
                            " · loader overrides detected"
                        }
                    ),
                }
            }
            Kind::Bool => {
                let state = value.and_then(|v| match v.to_ascii_lowercase().as_str() {
                    "1" | "true" | "yes" => Some(true),
                    "0" | "false" | "no" => Some(false),
                    _ => None,
                });
                Reading {
                    checked: state,
                    label: state
                        .map(|s| {
                            if s {
                                "On · environment"
                            } else {
                                "Off · environment"
                            }
                            .into()
                        })
                        .unwrap_or_else(|| "Automatic / unknown".into()),
                    source: format!(
                        "{base_source} · {}",
                        value
                            .map(String::as_str)
                            .unwrap_or("Determined by driver / game profile")
                    ),
                }
            }
            Kind::Vblank => {
                let (state, label) = match value.map(String::as_str) {
                    Some("0") => (Some(false), "Off · forced"),
                    Some("1") => (Some(false), "Default off · application may override"),
                    Some("2") => (Some(true), "Default on · application may override"),
                    Some("3") => (Some(true), "On · forced"),
                    _ => (None, "Automatic / unknown"),
                };
                Reading {
                    checked: state,
                    label: label.into(),
                    source: format!(
                        "{base_source} · vblank_mode={}",
                        value.map(String::as_str).unwrap_or("unset")
                    ),
                }
            }
            Kind::Vrs => {
                let state = value.and_then(|v| match v.as_str() {
                    "2x2" | "2x1" | "1x2" => Some(true),
                    "1x1" => Some(false),
                    _ => None,
                });
                Reading {
                    checked: state,
                    label: value
                        .map(|v| format!("VRS · {v}"))
                        .unwrap_or_else(|| "Automatic / unknown".into()),
                    source: format!(
                        "{base_source} · RADV_FORCE_VRS_CONFIG_FILE takes precedence when set"
                    ),
                }
            }
            Kind::Flag(flag) => {
                let requested = value.is_some_and(|v| tokens(v).iter().any(|t| t == flag));
                let state = if overridden {
                    profile.overrides.get(setting.id).copied()
                } else if requested {
                    Some(true)
                } else {
                    None
                };
                Reading {
                    checked: state,
                    label: if requested {
                        "Flag requested · support not verified".into()
                    } else if overridden {
                        "Flag removed · feature may still be automatic".into()
                    } else {
                        "Automatic / unknown".into()
                    },
                    source: format!(
                        "{base_source} · {}={}",
                        setting.variable,
                        value.map(String::as_str).unwrap_or("unset")
                    ),
                }
            }
        }
    }
    /// Only explicitly edited keys are emitted. Unrelated and unknown flags are retained.
    pub fn changes(&self, profile: &Profile) -> EnvironmentChanges {
        let mut output = EnvironmentChanges::default();
        for setting in SETTINGS {
            if !self.setting_available(setting) {
                continue;
            }
            let Some(&on) = profile.overrides.get(setting.id) else {
                continue;
            };
            let value = match setting.kind {
                Kind::Layer(name) => {
                    if let Some(layer) = self.layers.iter().find(|l| l.name == name) {
                        for key in layer.disable.keys() {
                            // Vulkan Loader checks presence, not the manifest's value.
                            // Even an empty string can disable the layer.
                            if on {
                                output.unset.insert(key.clone());
                            } else {
                                output.set.insert(key.clone(), "1".into());
                            }
                        }
                    }
                    if on { "1" } else { "0" }.into()
                }
                Kind::Bool => if on { "true" } else { "false" }.into(),
                Kind::Vblank => if on { "3" } else { "0" }.into(),
                Kind::Vrs => if on { "2x2" } else { "1x1" }.into(),
                Kind::Flag(flag) => {
                    let mut flags = tokens(
                        output
                            .set
                            .get(setting.variable)
                            .or_else(|| self.environment.get(setting.variable))
                            .map(String::as_str)
                            .unwrap_or(""),
                    );
                    flags.retain(|f| f != flag);
                    if on {
                        flags.push(flag.into());
                    }
                    flags.join(",")
                }
            };
            output.set.insert(setting.variable.into(), value);
        }
        output
    }
    pub fn compose(&self, profile: &Profile) -> Environment {
        let mut env = self.environment.clone();
        let changes = self.changes(profile);
        for key in changes.unset {
            env.remove(&key);
        }
        env.extend(changes.set);
        env
    }
    pub fn steam_line(&self, profile: &Profile) -> String {
        let changes = self.changes(profile);
        if changes.is_empty() {
            return "%command%".into();
        }
        let mut arguments = vec!["env".to_string()];
        for key in &changes.unset {
            arguments.push("-u".into());
            arguments.push(shell_words::quote(key).into_owned());
        }
        arguments.extend(
            changes
                .set
                .iter()
                .map(|(k, v)| shell_words::quote(&format!("{k}={v}")).into_owned()),
        );
        arguments.push("%command%".into());
        arguments.join(" ")
    }
    pub fn write_session(&self, profile: &Profile, path: &Path) -> Result<(), String> {
        let changes = self.changes(profile);
        if !changes.unset.is_empty() {
            return Err(format!(
                "environment.d cannot remove {}. Use Launch or Steam options. Session file unchanged.",
                changes.unset.iter().cloned().collect::<Vec<_>>().join(", ")
            ));
        }
        if changes.is_empty() {
            return match fs::remove_file(path) {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(e.to_string()),
            };
        }
        let mut content = "# Managed by amdgpu-tuner. Applies to future systemd user sessions.\n# Restore with: amdgpu-tuner --remove-session\n".to_string();
        for (key, value) in changes.set {
            // environment.d expands $, so refuse values that cannot be preserved literally.
            if value.contains(['$', '\n', '\r', '\0']) {
                return Err(format!(
                    "{key}: cannot preserve this value safely in environment.d; use the application launcher instead."
                ));
            }
            content.push_str(&format!(
                "{key}=\"{}\"\n",
                value.replace('\\', "\\\\").replace('"', "\\\"")
            ));
        }
        atomic_write(path, &content)
    }
    pub fn launch(
        &self,
        profile: &Profile,
        argv: &[String],
    ) -> Result<std::process::Child, String> {
        let (program, args) = argv.split_first().ok_or("Enter a command to launch")?;
        let mut command = Command::new(program);
        command.args(args);
        self.changes(profile).apply(&mut command);
        command.spawn().map_err(|e| format!("{program}: {e}"))
    }
}
fn layer_library_present(manifest: &Path, library: &str, search: &[PathBuf]) -> bool {
    if library.is_empty() {
        return false;
    }
    let path = Path::new(library);
    if path.is_absolute() {
        return path.is_file();
    }
    if manifest
        .parent()
        .is_some_and(|parent| parent.join(path).is_file())
    {
        return true;
    }
    // A relative path with directories is relative to the manifest, not the library search path.
    !library.contains('/') && search.iter().any(|dir| dir.join(path).is_file())
}

fn tokens(value: &str) -> Vec<String> {
    value
        .split([',', ';', ' '])
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn system() -> System {
        System {
            layers: vec![Layer {
                name: "VK_LAYER_MESA_anti_lag".into(),
                path: "test.json".into(),
                library_present: true,
                enable: Environment::from([("ENABLE_LAYER_MESA_ANTI_LAG".into(), "1".into())]),
                disable: Environment::from([("DISABLE_LAYER_MESA_ANTI_LAG".into(), "1".into())]),
            }],
            ..Default::default()
        }
    }
    #[test]
    fn optional_layer_requires_manifest_and_library() {
        let mut s = system();
        let p = Profile {
            overrides: BTreeMap::from([("low-latency".into(), true)]),
        };
        assert!(!s.setting_available(&SETTINGS[1]));
        assert!(!s.changes(&p).set.contains_key("LOW_LATENCY_LAYER"));
        s.layers.push(Layer {
            name: LOW_LATENCY_LAYER.into(),
            path: "test.json".into(),
            library_present: false,
            enable: Environment::from([("LOW_LATENCY_LAYER".into(), "1".into())]),
            disable: Environment::from([("DISABLE_LOW_LATENCY_LAYER".into(), "1".into())]),
        });
        assert!(!s.setting_available(&SETTINGS[1]));
        assert!(!s.changes(&p).set.contains_key("LOW_LATENCY_LAYER"));
        s.layers.last_mut().unwrap().library_present = true;
        assert!(s.setting_available(&SETTINGS[1]));
        assert_eq!(s.changes(&p).set["LOW_LATENCY_LAYER"], "1");
    }
    #[test]
    fn manifest_library_paths_resolve_without_fallback_for_broken_absolute_paths() {
        let executable = std::env::current_exe().unwrap();
        let parent = executable.parent().unwrap();
        let manifest = parent.join("test.json");
        let filename = executable.file_name().unwrap().to_str().unwrap();
        assert!(layer_library_present(
            &manifest,
            executable.to_str().unwrap(),
            &[]
        ));
        assert!(layer_library_present(&manifest, filename, &[]));
        assert!(layer_library_present(
            Path::new("/no-such-manifest/test.json"),
            filename,
            &[parent.into()]
        ));
        assert!(!layer_library_present(
            &manifest,
            "/no-such-directory/missing.so",
            &[parent.into()]
        ));
        assert!(!layer_library_present(&manifest, "", &[parent.into()]));
    }
    #[test]
    fn inherited_one_is_on_and_disable_wins() {
        let mut s = system();
        s.environment
            .insert("ENABLE_LAYER_MESA_ANTI_LAG".into(), "1".into());
        assert_eq!(
            s.read(&SETTINGS[0], &Profile::default()).checked,
            Some(true)
        );
        let p = Profile {
            overrides: BTreeMap::from([("anti-lag".into(), true)]),
        };
        for value in ["1", "0", "", "unexpected"] {
            s.environment
                .insert("DISABLE_LAYER_MESA_ANTI_LAG".into(), value.into());
            assert_eq!(
                s.read(&SETTINGS[0], &Profile::default()).checked,
                Some(false),
                "A defined disable variable must win, including {value:?}"
            );
            assert_eq!(s.read(&SETTINGS[0], &p).checked, Some(true));
            assert!(!s.compose(&p).contains_key("DISABLE_LAYER_MESA_ANTI_LAG"));
        }
    }
    #[test]
    fn unknown_is_not_false() {
        let s = system();
        assert_eq!(s.read(&SETTINGS[3], &Profile::default()).checked, None);
        assert_eq!(s.read(&SETTINGS[7], &Profile::default()).checked, None);
        assert_eq!(
            s.read(&SETTINGS[0], &Profile::default()).checked,
            Some(false)
        );
    }
    #[test]
    fn flags_preserve_unknowns_and_other_overrides() {
        let mut s = system();
        s.environment
            .insert("RADV_PERFTEST".into(), "future,gpl,sam".into());
        let p = Profile {
            overrides: BTreeMap::from([
                ("gpl".into(), false),
                ("dccmsaa".into(), true),
                ("nircache".into(), true),
            ]),
        };
        assert_eq!(
            s.changes(&p).set["RADV_PERFTEST"],
            "future,sam,dccmsaa,nircache"
        );
        assert!(!s.changes(&p).set.contains_key("mesa_glthread"));
    }
    #[test]
    fn reset_leaves_no_overrides() {
        let s = system();
        assert!(s.changes(&Profile::default()).is_empty());
        assert_eq!(s.steam_line(&Profile::default()), "%command%");
    }
    #[test]
    fn launch_really_overrides_inherited_environment() {
        let s = system();
        let p = Profile {
            overrides: BTreeMap::from([("anti-lag".into(), true), ("vsync".into(), false)]),
        };
        for value in ["1", "0", ""] {
            let mut command = Command::new("sh");
            command
                .args(["-c", "printf '%s:%s:%s' \"$ENABLE_LAYER_MESA_ANTI_LAG\" \"${DISABLE_LAYER_MESA_ANTI_LAG+present}\" \"$vblank_mode\""])
                .env("DISABLE_LAYER_MESA_ANTI_LAG", value);
            s.changes(&p).apply(&mut command);
            let output = command.output().unwrap();
            assert!(output.status.success());
            assert_eq!(String::from_utf8_lossy(&output.stdout), "1::0");
        }
        assert!(
            s.steam_line(&p)
                .starts_with("env -u DISABLE_LAYER_MESA_ANTI_LAG ")
        );
    }
    #[test]
    fn session_rejects_unsets_without_touching_existing_file() {
        let directory = env::temp_dir().join(format!("amdgpu-unset-test-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("90.conf");
        let s = system();
        let p = Profile {
            overrides: BTreeMap::from([("anti-lag".into(), true), ("glthread".into(), true)]),
        };
        assert!(
            s.write_session(&p, &path)
                .unwrap_err()
                .contains("cannot remove")
        );
        assert!(!path.exists());
        let old = "DISABLE_LAYER_MESA_ANTI_LAG=\"0\"\n";
        fs::write(&path, old).unwrap();
        assert!(s.write_session(&p, &path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), old);
        let off = Profile {
            overrides: BTreeMap::from([("anti-lag".into(), false)]),
        };
        s.write_session(&off, &path).unwrap();
        assert!(
            fs::read_to_string(&path)
                .unwrap()
                .contains("DISABLE_LAYER_MESA_ANTI_LAG=\"1\"")
        );
        let _ = fs::remove_dir_all(directory);
    }
    #[test]
    fn session_file_round_trip_and_remove() {
        let path =
            env::temp_dir().join(format!("amdgpu-tuner-test-{}/90.conf", std::process::id()));
        let s = system();
        let p = Profile {
            overrides: BTreeMap::from([("glthread".into(), true)]),
        };
        s.write_session(&p, &path).unwrap();
        assert!(
            fs::read_to_string(&path)
                .unwrap()
                .contains("mesa_glthread=\"true\"")
        );
        s.write_session(&Profile::default(), &path).unwrap();
        assert!(!path.exists());
        let _ = fs::remove_dir(path.parent().unwrap());
    }
}
