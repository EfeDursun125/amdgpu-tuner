#!/usr/bin/env python3
# Copyright (C) 2026 EfeDursun125
# This file is part of AMDGPU Tuner; see NOTICE and LICENSE.
# SPDX-License-Identifier: GPL-3.0-only
"""Build an x86_64 AppImage; use host desktop/graphics libraries, never bundled drivers."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import tomllib
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
TOOLS = {
    'appimagetool': ('https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage', '95cbe7cce9717fce90c484e34052ee7c7f1d7635b33c12525b4776826a7d29b6'),
    'runtime': ('https://github.com/AppImage/type2-runtime/releases/download/continuous/runtime-x86_64', '617aa77314d57c8b8e4fae9c3f0b35af7b30d5529cb69e98f64f92510ed98a46'),
}


def checked_tool(name, cache):
    url, expected = TOOLS[name]
    path = cache / name
    if not path.exists():
        print(f'Downloading {url}', flush=True)
        with urllib.request.urlopen(url, timeout=90) as response:
            data = response.read()
        if hashlib.sha256(data).hexdigest() != expected:
            raise SystemExit(f'{name}: upstream download changed; review and update the pinned digest.')
        path.write_bytes(data)
    if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
        raise SystemExit(f'{path}: checksum mismatch')
    path.chmod(0o755)
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, help='Package an existing release binary instead of building')
    args = parser.parse_args()
    if os.uname().machine != 'x86_64':
        raise SystemExit('This packaging configuration currently supports x86_64 Linux only.')
    if not args.binary:
        subprocess.run(['cargo', 'build', '--release', '--locked'], cwd=ROOT, check=True)
    metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--format-version', '1'], cwd=ROOT))
    binary = (args.binary or Path(metadata['target_directory']) / 'release/amdgpu-tuner').resolve(strict=True)
    version = tomllib.loads((ROOT / 'Cargo.toml').read_text())['package']['version']
    output = ROOT / 'dist'
    output.mkdir(exist_ok=True)
    cache = Path(metadata['target_directory']) / 'appimage-tools'
    cache.mkdir(parents=True, exist_ok=True)
    tool = checked_tool('appimagetool', cache)
    runtime = checked_tool('runtime', cache)
    versions = re.findall(r'GLIBC_(\d+\.\d+)', subprocess.check_output(['readelf', '--version-info', str(binary)], text=True))
    glibc = max(versions, key=lambda value: tuple(map(int, value.split('.'))))
    libraries = subprocess.check_output(['ldd', str(binary)], text=True)
    if 'not found' in libraries:
        raise SystemExit(f'Unresolved host libraries:\n{libraries}')
    artifact = output / f'amdgpu-tuner-{version}-x86_64.AppImage'
    with tempfile.TemporaryDirectory(prefix='amdgpu-appimage-') as directory:
        appdir = Path(directory) / 'AMDGPU-Tuner.AppDir'
        (appdir / 'usr/bin').mkdir(parents=True)
        notices = appdir / 'usr/share/licenses/amdgpu-tuner'
        notices.mkdir(parents=True)
        shutil.copy2(binary, appdir / 'usr/bin/amdgpu-tuner')
        for name in ['LICENSE', 'NOTICE', 'THIRD_PARTY.md']:
            shutil.copy2(ROOT / name, notices / name)
        shutil.copy2(ROOT / 'vendor/material-1.1.0/LICENSE.md', notices / 'Material-MIT.txt')
        shutil.copy2(ROOT / 'packaging/AppImage-runtime-LICENSE', notices / 'AppImage-runtime-LICENSE')
        # Keep original upstream notices for every Cargo package in the locked graph.
        packages = []
        for package in metadata['packages']:
            packages.append({key: package.get(key) for key in ['name', 'version', 'license', 'repository', 'source']})
            if package['source'] is None:
                continue
            source = Path(package['manifest_path']).parent
            destination = notices / 'cargo' / f"{package['name']}-{package['version']}"
            destination.mkdir(parents=True, exist_ok=True)
            for path in source.iterdir():
                if path.is_file() and path.name.lower().startswith(('license', 'licence', 'copying', 'notice', 'copyright')):
                    shutil.copy2(path, destination / path.name)
        (notices / 'cargo-packages.json').write_text(json.dumps(packages, indent=2) + '\n')
        shutil.copy2(ROOT / 'packaging/amdgpu-tuner.desktop', appdir / 'amdgpu-tuner.desktop')
        shutil.copy2(ROOT / 'packaging/amdgpu-tuner.svg', appdir / 'amdgpu-tuner.svg')
        (appdir / '.DirIcon').symlink_to('amdgpu-tuner.svg')
        shutil.copy2(ROOT / 'packaging/AppRun', appdir / 'AppRun')
        (appdir / 'AppRun').chmod(0o755)
        (appdir / 'BUILD-INFO.txt').write_text(
            f'AMDGPU Tuner {version}\nArchitecture: x86_64\nRequired GLIBC symbol version: {glibc}\n'
            'Uses host desktop and graphics libraries; no GPU driver is bundled.\n'
            + subprocess.check_output(['rustc', '--version'], text=True)
            + '\nHost library resolution at packaging time:\n' + libraries
            + '\nPackaging tool digests:\n' + json.dumps(TOOLS, indent=2) + '\n')
        subprocess.run([str(tool), '--appimage-extract-and-run', '--no-appstream', '--comp', 'zstd', '--runtime-file', str(runtime), str(appdir), str(artifact)],
                       env={**os.environ, 'ARCH': 'x86_64', 'SOURCE_DATE_EPOCH': '1767225600'}, check=True)
    artifact.chmod(0o755)
    # Keep a standalone record alongside the release asset.
    (output / f'{artifact.name}.build-info.txt').write_text(
        f'AMDGPU Tuner {version}\nx86_64 Linux; GLIBC {glibc} or newer.\n'
        'Requires host Fontconfig/FreeType and Wayland or X11/OpenGL libraries.\n'
        'No bundled GPU drivers. See docs/appimage.md for support and source distribution.\n'
        + json.dumps(TOOLS, indent=2) + '\n')
    checksums = output / 'SHA256SUMS'
    lines = checksums.read_text().splitlines() if checksums.exists() else []
    lines = [line for line in lines if not line.endswith('  ' + artifact.name)]
    lines.append(f'{hashlib.sha256(artifact.read_bytes()).hexdigest()}  {artifact.name}')
    info = output / f'{artifact.name}.build-info.txt'
    lines = [line for line in lines if not line.endswith('  ' + info.name)]
    lines.append(f'{hashlib.sha256(info.read_bytes()).hexdigest()}  {info.name}')
    checksums.write_text('\n'.join(lines) + '\n')
    print(f'Created {artifact}\nRequired GLIBC symbol version: {glibc}', flush=True)


if __name__ == '__main__':
    main()
