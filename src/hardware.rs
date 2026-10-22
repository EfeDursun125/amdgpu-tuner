// Copyright (C) 2026 EfeDursun125
// This file is part of AMDGPU Tuner; see NOTICE and LICENSE.
// SPDX-License-Identifier: GPL-3.0-only
use std::{collections::BTreeMap, fs, path::Path, process::Command};

fn pci_id(value: &str) -> Option<u32> {
    u32::from_str_radix(value.trim().trim_start_matches("0x"), 16).ok()
}

fn bracketed_id(value: &str) -> Option<u32> {
    pci_id(value.rsplit_once('[')?.1.strip_suffix(']')?)
}

fn parse_lspci(text: &str) -> BTreeMap<String, String> {
    let mut devices = BTreeMap::new();
    for line in text.lines() {
        let Ok(fields) = shell_words::split(line) else {
            continue;
        };
        if fields.len() < 4
            || bracketed_id(&fields[2]) != Some(0x1002)
            || bracketed_id(&fields[1]).map(|class| class >> 8) != Some(0x03)
        {
            continue;
        }
        let name = fields[3]
            .rsplit_once(" [")
            .filter(|(_, id)| id.strip_suffix(']').and_then(pci_id).is_some())
            .map(|(name, _)| name)
            .unwrap_or(&fields[3]);
        devices.insert(fields[0].clone(), name.to_string());
    }
    devices
}

fn sysfs_devices(root: &Path) -> Option<BTreeMap<String, String>> {
    let mut devices = BTreeMap::new();
    for entry in fs::read_dir(root).ok()?.flatten() {
        let path = entry.path();
        let read_id = |name| {
            fs::read_to_string(path.join(name))
                .ok()
                .and_then(|s| pci_id(&s))
        };
        let is_display = read_id("class").is_some_and(|class| class >> 16 == 0x03);
        let is_amdgpu = fs::read_link(path.join("driver"))
            .ok()
            .and_then(|driver| driver.file_name().map(|name| name == "amdgpu"))
            .unwrap_or(false);
        if read_id("vendor") != Some(0x1002) || !(is_display || is_amdgpu) {
            continue;
        }
        let name = read_id("device")
            .map(|id| format!("AMD GPU [1002:{id:04x}]"))
            .unwrap_or_else(|| "AMD GPU".into());
        devices.insert(entry.file_name().to_string_lossy().into_owned(), name);
    }
    Some(devices)
}

fn describe(devices: &BTreeMap<String, String>) -> String {
    if devices.len() == 1 {
        return devices.values().next().unwrap().clone();
    }
    devices
        .iter()
        .map(|(slot, name)| format!("{name} · {slot}"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn detect() -> String {
    let sysfs = sysfs_devices(Path::new("/sys/bus/pci/devices"));
    let pci = Command::new("lspci")
        .args(["-D", "-mm", "-nn", "-d", "1002:"])
        .output()
        .ok()
        .filter(|output| output.status.success());
    let mut devices = sysfs.clone().unwrap_or_default();
    if let Some(output) = &pci {
        // PCI addresses retain multiple physical cards even when their names match.
        devices.extend(parse_lspci(&String::from_utf8_lossy(&output.stdout)));
    }
    if devices.is_empty() {
        if sysfs.is_some() || pci.is_some() {
            "No AMD GPU detected".into()
        } else {
            "AMD GPU information unavailable".into()
        }
    } else {
        describe(&devices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_all_display_classes_without_audio_bridges_or_other_vendors() {
        let devices = parse_lspci(
            r#"0000:03:00.0 "VGA compatible controller [0300]" "AMD [1002]" "Discrete Radeon [747e]"
0000:0a:00.0 "VGA compatible controller [0300]" "AMD [1002]" "Integrated Radeon [13c0]"
0000:0b:00.0 "3D controller [0302]" "AMD [1002]" "Compute GPU [740f]"
0000:0c:00.0 "Display controller [0380]" "AMD [1002]" "Discrete Radeon [747e]"
0000:03:00.1 "Audio device [0403]" "AMD [1002]" "HDMI audio [ab30]"
0000:01:00.0 "PCI bridge [0604]" "AMD [1002]" "Bridge [1478]"
0000:0d:00.0 "VGA compatible controller [0300]" "Other [10de]" "Other GPU [1234]""#,
        );
        assert_eq!(devices.len(), 4);
        let summary = describe(&devices);
        assert_eq!(summary.lines().count(), 4);
        assert!(summary.contains("Integrated Radeon · 0000:0a:00.0"));
        assert!(summary.contains("Compute GPU · 0000:0b:00.0"));
        assert_eq!(summary.matches("Discrete Radeon").count(), 2);
    }

    #[test]
    fn sysfs_fallback_retains_integrated_discrete_and_compute_cards() {
        let root = std::env::temp_dir().join(format!("amdgpu-hardware-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        for (slot, vendor, class, device) in [
            ("0000:03:00.0", "0x1002", "0x030000", "0x747e"),
            ("0000:0a:00.0", "0x1002", "0x030200", "0x13c0"),
            ("0000:03:00.1", "0x1002", "0x040300", "0xab30"),
            ("0000:0d:00.0", "0x10de", "0x030000", "0x1234"),
        ] {
            let path = root.join(slot);
            fs::create_dir_all(&path).unwrap();
            for (key, value) in [("vendor", vendor), ("class", class), ("device", device)] {
                fs::write(path.join(key), value).unwrap();
            }
        }
        let devices = sysfs_devices(&root).unwrap();
        assert_eq!(devices.len(), 2);
        assert_eq!(devices["0000:03:00.0"], "AMD GPU [1002:747e]");
        assert_eq!(devices["0000:0a:00.0"], "AMD GPU [1002:13c0]");
        fs::remove_dir_all(root).unwrap();
    }
}
