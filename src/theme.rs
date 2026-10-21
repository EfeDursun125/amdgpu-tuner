// SPDX-License-Identifier: GPL-3.0-only
use crate::{AppWindow, MaterialPalette};
use slint::{Color, ComponentHandle};
use std::{collections::BTreeMap, fs};

fn parse_ini(text: &str) -> BTreeMap<(String, String), String> {
    let mut result = BTreeMap::new();
    let mut group = String::new();
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') && line.ends_with(']') {
            group = line[1..line.len() - 1].into();
        } else if let Some((key, value)) = line.split_once('=') {
            result.insert((group.clone(), key.trim().into()), value.trim().into());
        }
    }
    result
}
fn color(value: &str) -> Option<Color> {
    if let Some(hex) = value.strip_prefix('#') {
        return u32::from_str_radix(hex, 16)
            .ok()
            .map(|v| Color::from_rgb_u8((v >> 16) as u8, (v >> 8) as u8, v as u8));
    }
    let rgb: Vec<_> = value
        .split(',')
        .filter_map(|v| v.trim().parse::<u8>().ok())
        .collect();
    if rgb.len() >= 3 {
        Some(Color::from_rgb_u8(rgb[0], rgb[1], rgb[2]))
    } else {
        None
    }
}
pub fn apply(ui: &AppWindow) {
    if !std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .split(':')
        .any(|v| v.eq_ignore_ascii_case("KDE"))
    {
        ui.set_theme_name("System light / dark theme".into());
        return;
    }
    let mut paths: Vec<_> = std::env::var("XDG_CONFIG_DIRS")
        .unwrap_or_else(|_| "/etc/xdg".into())
        .split(':')
        .map(|v| std::path::PathBuf::from(v).join("kdeglobals"))
        .collect();
    paths.reverse();
    paths.push(crate::engine::config_dir().join("kdeglobals"));
    let mut ini = BTreeMap::new();
    for path in paths {
        if let Ok(s) = fs::read_to_string(path) {
            ini.extend(parse_ini(&s));
        }
    }
    let lookup = |group: &str, key: &str| ini.get(&(group.into(), key.into()));
    let c = |group: &str, key: &str, fallback: Color| {
        lookup(group, key)
            .and_then(|v| color(v))
            .unwrap_or(fallback)
    };
    let Some(background) = lookup("Colors:Window", "BackgroundNormal").and_then(|v| color(v))
    else {
        ui.set_theme_name("System light / dark theme".into());
        return;
    };
    let palette = ui.global::<MaterialPalette>();
    let mut schemes = palette.get_schemes();
    let dark = (background.red() as f32 * 0.2126
        + background.green() as f32 * 0.7152
        + background.blue() as f32 * 0.0722)
        < 128.;
    let mut scheme = if dark {
        schemes.dark.clone()
    } else {
        schemes.light.clone()
    };
    let fg = c("Colors:Window", "ForegroundNormal", scheme.onSurface);
    let view = c("Colors:View", "BackgroundNormal", background);
    let button = c("Colors:Button", "BackgroundNormal", background);
    let primary = c("Colors:Selection", "BackgroundNormal", scheme.primary);
    let primary_fg = c(
        "Colors:Selection",
        "ForegroundActive",
        c("Colors:Selection", "ForegroundNormal", scheme.onPrimary),
    );
    let secondary_fg = c("Colors:Window", "ForegroundInactive", fg);
    let container = c("Colors:Window", "BackgroundAlternate", button);
    scheme.background = view;
    scheme.surface = view;
    scheme.onBackground = fg;
    scheme.onSurface = fg;
    scheme.primary = primary;
    scheme.surfaceTint = primary;
    scheme.onPrimary = primary_fg;
    scheme.primaryContainer = container;
    scheme.onPrimaryContainer = fg;
    scheme.secondary = primary;
    scheme.onSecondary = primary_fg;
    scheme.secondaryContainer = button;
    scheme.onSecondaryContainer = fg;
    scheme.tertiary = primary;
    scheme.onTertiary = primary_fg;
    scheme.tertiaryContainer = container;
    scheme.onTertiaryContainer = fg;
    scheme.surfaceVariant = button;
    scheme.onSurfaceVariant = secondary_fg;
    scheme.outline = c("Colors:Window", "ForegroundInactive", scheme.outline);
    scheme.outlineVariant = button;
    scheme.surfaceDim = view;
    scheme.surfaceBright = button;
    scheme.surfaceContainerLowest = view;
    scheme.surfaceContainerLow = background;
    scheme.surfaceContainer = background;
    scheme.surfaceContainerHigh = button;
    scheme.surfaceContainerHighest = button;
    scheme.error = c("Colors:Window", "ForegroundNegative", scheme.error);
    // Both branches use the active KDE palette: Slint's portal color scheme can briefly lag KDE.
    schemes.light = scheme.clone();
    schemes.dark = scheme;
    if palette.get_schemes() != schemes {
        palette.set_schemes(schemes);
    }
    let name = lookup("General", "ColorScheme")
        .map(String::as_str)
        .unwrap_or("KDE");
    ui.set_theme_name(format!("KDE · {name} · {}", if dark { "Dark" } else { "Light" }).into());
    if let Some(font) = lookup("General", "font").and_then(|s| s.split(',').next()) {
        ui.set_system_font(font.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kde_colors_accept_hex_and_rgb() {
        assert_eq!(color("#ffb4a8"), color("255,180,168"));
        assert!(color("bad").is_none());
    }
}
