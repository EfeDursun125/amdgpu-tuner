# Validation record

## 0.1.1 layer-disable fix — 2026-10-22

On the local CachyOS/KDE/Wayland system, formatting, Clippy, 12 unit tests, locked debug/release builds, CLI regression tests, UI/animation tests and all 13 layout renders passed. Launch and the generated Steam command were executed with both disable variables inherited as `1`, `0`, empty and an unrelated string; enabled child environments contained neither variable. Session export rejection preserves existing files and does not create partial files; layer-off and non-layer exports remain supported.

Hardware fixtures cover integrated/discrete GPUs, 3D and other display classes, identical model names at different PCI addresses, audio/bridge exclusion and sysfs fallback. Real UI captures at 1120×1000 and 850×650 show both the local Navi 32 Radeon RX 7700 XT / 7800 XT and Granite Ridge Radeon 610M. The controls/animation smoke test uses a fixed hardware caption so host GPU count cannot invalidate pointer coordinates.

`tests/vulkan-loader.py` additionally ran against the installed Mesa Anti-Lag and Korthos libraries using `vulkaninfo` and `VK_LOADER_DEBUG=layer`. For both Launch and Steam, both layers were inserted when enabled and absent when disabled, including inherited disable values `1`, `0` and empty. This verifies actual Loader insertion, not game extension use or latency improvement. Without `vulkaninfo` or either target layer, that optional test explicitly reports a skip.

The original 0.1.0 environment-value tests did not check Loader insertion and incorrectly accepted `DISABLE_...=0`; 0.1.1 replaces that expectation with actual variable absence and executable Steam regressions.

## Initial release checks

Checked locally on **2026-10-21**, CachyOS Linux / KDE / Wayland, with **Rust 1.99.1** and Slint 1.18.1. The declared Rust floor is 1.92; this record does not claim a separate build with that compiler.

| Check | Local result |
| --- | --- |
| `cargo fmt --check` | Passed |
| `cargo clippy --locked --all-targets -- -D warnings` | Passed |
| `cargo test --locked` | 9 tests passed |
| Debug and locked release builds | Passed |
| CLI integration | Passed, including inherited enabled values, override precedence and absent-layer metadata |
| Installer / uninstaller | Passed in temporary prefixes; actual Gio launch tested with spaces, dollar signs, percent signs, backticks, quotes and backslashes in the destination path |
| Profiles and session files during uninstall | Preserved |
| Release-binary UI checks | Passed: click/toggle/restore, both animation directions, missing optional layer and live KDE palette reload |
| Rendered layout | 13 captures passed: all pages at 850 / 1120 / 1440 widths, hover, 100% / 125% / 150% scale |
| SVG icon | Parsed and rendered at 32px and 256px; embedded in the Slint executable and installed as a scalable desktop icon |
| Material Components provenance | All 78 bundled files match the official 1.1.0 archive byte-for-byte |
| Repository metadata and media | GPLv3 metadata, local README media, SVGs and source hygiene checks passed |
| GitHub workflow | YAML parsed and `actionlint` passed |
| Source ZIP | Integrity and content exclusions checked; a fresh extraction completed a locked release build and CLI startup |
| x86_64 AppImage | Native FUSE startup, extraction-and-run CLI, full CLI integration, real UI screenshot and UI smoke/animation tests passed on this host; requires glibc 2.43+ |

The README screenshot was captured from the real application using a disposable example profile. User configuration was not overwritten. This record does not establish game performance, latency improvements, every distribution's package installation, or successful execution on GitHub-hosted runners. The first remote CI run happens after the repository is uploaded.
