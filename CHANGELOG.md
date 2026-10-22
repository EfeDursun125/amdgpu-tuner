# Changelog

## 0.1.1 — 2026-10-22

- Remove layer-disabling environment variables when enabling Anti-Lag or Korthos, including inherited `0` and empty values.
- Match Vulkan Loader's presence-based disable precedence in the UI, and use `env -u` in Steam options.
- Reject session exports requiring variable removal, preserving existing files and explaining the supported Launch/Steam alternatives.
- Add real child-environment, Steam execution and optional live Vulkan Loader regression checks.
- List every AMD GPU, including integrated, discrete and 3D controllers, with sysfs fallback and refresh support.
- Update the tuning headline and detected-theme label.

## 0.1.0 — 2026-10-21

- English Rust + Slint interface using Material 3 components and KDE palette integration.
- Inherited environment values, explicit unknown defaults and manual per-setting overrides.
- Optional Korthos Low Latency Layer discovery and conditional visibility.
- Profile persistence, command launching, Steam options and user-session export.
- Animated switches, content-sized buttons and consistent card geometry.
- Original SVG icon, per-user installer, documentation screenshots and source packaging.
- Unit, command-line, packaging, animation and rendered layout checks.
