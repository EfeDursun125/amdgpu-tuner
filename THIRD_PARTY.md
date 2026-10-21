# Third-party components

AMDGPU Tuner's own code and artwork are licensed under **GPL-3.0-only**. The full license is in [LICENSE](LICENSE). Files under `vendor/` retain their original notices and licenses.

## Slint Material Components 1.1.0

- Upstream: <https://material.slint.dev/getting-started/>
- Source archive: <https://material.slint.dev/zip/material-1.1.0.zip>
- Archive SHA-256: `83c57b6f1b8799ddacd94b7e8b140f3f2112c36407b6df31883da0ba4e2dd425`
- Location: `vendor/material-1.1.0/`
- License: MIT; see `vendor/material-1.1.0/LICENSE.md`.
- Copyright: SixtyFPS GmbH, as specified in the upstream files.
- Modifications: none. Application-specific controls live in `ui/controls.slint`.

## Slint and Rust dependencies

Slint 1.18.1 is used under its **GPL-3.0-only** option for this GPLv3 application. The Material Components library has its separate MIT license above. Versions of all Cargo dependencies are recorded in `Cargo.lock`; each dependency retains its upstream license.

Cargo downloads Rust dependencies from their declared sources. The regular source ZIP contains this application's sources and Material Components. The **source-with-dependencies ZIP**, generated with `scripts/package-source.py --with-dependencies`, additionally includes the locked Cargo sources, upstream license files and offline Cargo configuration. Distribute that matching archive alongside the AppImage. The AppImage includes original Cargo package notices and a package/license inventory under `usr/share/licenses/amdgpu-tuner/`. Host desktop libraries and graphics drivers are not bundled.

## AppImage runtime

The AppImage embeds the official [AppImage/type2-runtime](https://github.com/AppImage/type2-runtime) runtime, distributed under MIT; see `packaging/AppImage-runtime-LICENSE`. The x86_64 runtime used for the initial artifact is from commit `fedfad51acc2860b2916a1eef32ba4a500ff31ef`, SHA-256 `617aa77314d57c8b8e4fae9c3f0b35af7b30d5529cb69e98f64f92510ed98a46`. Packaging tool/runtime downloads are hash-pinned in `scripts/package-appimage.py`.

## Application artwork

`packaging/amdgpu-tuner.svg` and `ui/icons/*.svg` are original project artwork, licensed under GPL-3.0-only. The icon represents a GPU with tuning sliders and does not incorporate a vendor logo.
