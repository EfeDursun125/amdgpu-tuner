# AppImage releases

The x86_64 AppImage includes the application, embedded Slint/Material UI, icon and license notices. It uses the desktop's Fontconfig/FreeType, Wayland or X11 and OpenGL libraries. Mesa, Vulkan drivers, optional layers and fonts come from the host. Rust, Cargo and Python are not needed to run it.

## Run

```sh
chmod +x amdgpu-tuner-0.1.0-x86_64.AppImage
./amdgpu-tuner-0.1.0-x86_64.AppImage
```

All CLI options also work, for example `./amdgpu-tuner-0.1.0-x86_64.AppImage --inspect`. Profiles use the same XDG paths as a normal installation. The AppImage does not install a menu shortcut.

When FUSE mounting is unavailable, use:

```sh
./amdgpu-tuner-0.1.0-x86_64.AppImage --appimage-extract-and-run
```

**Compatibility:** the initial locally built release requires **glibc 2.43 or newer** because it was compiled on CachyOS with glibc 2.44. It is not compatible with Ubuntu 24.04's older glibc. Consult the adjacent `.build-info.txt` for each artifact; AppImage packaging does not lower a binary's glibc requirements. To support older distributions, build on the oldest supported desktop baseline and run the packaged artifact there before release. This repository does not claim such a baseline test has happened.

The launcher preserves the working directory and does not inject a private library search path into games launched through the application.

If a separately installed AppImageLauncher intercepts execution, its integration dialog may move the file into `~/Applications`. For direct CLI testing, prefix the command with `APPIMAGELAUNCHER_DISABLE=1` to bypass that external integration.

## Build

Use Linux x86_64 with the normal project build dependencies, Python 3.11+, `binutils` (`readelf`, `ldd`) and network access for the first tool download:

```sh
python3 scripts/package-appimage.py
python3 scripts/package-source.py --with-dependencies
```

For an already built release executable, pass `--binary target/release/amdgpu-tuner`. The script verifies the SHA-256 of both the official AppImage packaging tool and runtime. Tools are cached under `target/appimage-tools`. If the upstream continuous download changes, it deliberately stops until the new download is reviewed and its pinned digest updated.

Output goes to `dist/`:

- `amdgpu-tuner-0.1.0-x86_64.AppImage`
- `amdgpu-tuner-0.1.0-x86_64.AppImage.build-info.txt`
- `amdgpu-tuner-0.1.0-source-with-dependencies.zip`
- `SHA256SUMS`

The source-with-dependencies ZIP contains the exact project snapshot, Cargo.lock, vendored Rust sources with upstream notices, Material sources and a `.cargo/config.toml` for offline dependency resolution. Build from the extracted root using `cargo build --release --locked --offline` with the documented system development libraries and Rust toolchain. System desktop/graphics libraries are not redistributed inside the AppImage.

Upload the AppImage, build-info, matching source-with-dependencies ZIP and checksums together to the same GitHub Release. Keep the matching sources available under GPLv3. The smaller regular source ZIP alone does not include Cargo dependency sources.

## Verification

```sh
cd dist
sha256sum -c SHA256SUMS
./amdgpu-tuner-0.1.0-x86_64.AppImage --help
```

From the repository root, run the integration checks against the packaged application:

```sh
AMDGPU_TUNER_BINARY="$PWD/dist/amdgpu-tuner-0.1.0-x86_64.AppImage" python3 tests/cli.py
```

In a graphical session, `--screenshot /tmp/amdgpu-appimage.png 0 1120x1000` opens the actual packaged UI, captures it and exits. Use a temporary `XDG_CONFIG_HOME` when testing profile edits.

AppImage format references: [AppDir layout](https://docs.appimage.org/reference/appdir.html), [manual packaging](https://docs.appimage.org/packaging-guide/manual.html), [packaging tool](https://github.com/AppImage/appimagetool), [runtime](https://github.com/AppImage/type2-runtime).
