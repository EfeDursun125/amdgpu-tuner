#!/usr/bin/env python3
# Copyright (C) 2026 EfeDursun125
# This file is part of AMDGPU Tuner; see NOTICE and LICENSE.
# SPDX-License-Identifier: GPL-3.0-only
"""Build/install for the current user, or uninstall only the installed application files."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def desktop_exec(path):
    # Exec quoting happens after Desktop Entry string unescaping.
    text = str(path).replace('%', '%%')
    text = ''.join('\\' + char if char in '\\"`$' else char for char in text)
    return '"' + text.replace('\\', '\\\\') + '"'


def destinations(prefix):
    return {
        'binary': prefix / 'bin/amdgpu-tuner',
        'desktop': prefix / 'share/applications/amdgpu-tuner.desktop',
        'icon': prefix / 'share/icons/hicolor/scalable/apps/amdgpu-tuner.svg',
        'license': prefix / 'share/licenses/amdgpu-tuner/LICENSE',
        'copyright': prefix / 'share/licenses/amdgpu-tuner/NOTICE',
        'material_license': prefix / 'share/licenses/amdgpu-tuner/Material-MIT.txt',
        'notices': prefix / 'share/licenses/amdgpu-tuner/THIRD_PARTY.md',
    }


def refresh_caches(prefix):
    for command, directory in [
        ('update-desktop-database', prefix / 'share/applications'),
        ('gtk-update-icon-cache', prefix / 'share/icons/hicolor'),
    ]:
        if shutil.which(command):
            args = [command, str(directory)]
            if command == 'gtk-update-icon-cache':
                args = [command, '--force', '--ignore-theme-index', str(directory)]
            subprocess.run(args, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--prefix', type=Path, default=Path.home() / '.local', help='Installation prefix (default: ~/.local)')
    parser.add_argument('--binary', type=Path, help='Install an existing executable instead of building')
    parser.add_argument('--uninstall', action='store_true', help='Remove application files; preserve profiles and session overrides')
    args = parser.parse_args()
    prefix = args.prefix.expanduser().absolute()
    if any(char in str(prefix) for char in '\r\n='):
        parser.error('Prefix must not contain a newline or = (Desktop Entry executable path restriction).')
    paths = destinations(prefix)
    if args.uninstall:
        for path in paths.values():
            path.unlink(missing_ok=True)
        refresh_caches(prefix)
        print('Application files removed. Profiles and environment.d settings were preserved.')
        return
    if args.binary:
        binary = args.binary.expanduser().absolute()
    else:
        subprocess.run(['cargo', 'build', '--release', '--locked', '--manifest-path', str(ROOT / 'Cargo.toml')], cwd=ROOT, check=True)
        # Ask Cargo instead of guessing target-dir, including .cargo/config overrides.
        import json
        metadata = subprocess.check_output(['cargo', 'metadata', '--locked', '--no-deps', '--format-version', '1', '--manifest-path', str(ROOT / 'Cargo.toml')], cwd=ROOT, text=True)
        binary = Path(json.loads(metadata)['target_directory']) / 'release/amdgpu-tuner'
    if not binary.is_file() or not os.access(binary, os.X_OK):
        parser.error(f'Executable not found or not executable: {binary}')
    sources = {
        'binary': binary,
        'icon': ROOT / 'packaging/amdgpu-tuner.svg',
        'license': ROOT / 'LICENSE',
        'copyright': ROOT / 'NOTICE',
        'material_license': ROOT / 'vendor/material-1.1.0/LICENSE.md',
        'notices': ROOT / 'THIRD_PARTY.md',
    }
    for key, source in sources.items():
        paths[key].parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, paths[key])
        paths[key].chmod(0o755 if key == 'binary' else 0o644)
    desktop = (ROOT / 'packaging/amdgpu-tuner.desktop').read_text()
    # A stable launcher executable also handles literal % in an installation path:
    # some desktop parsers check Exec's program before expanding %% field escapes.
    env_program = shutil.which('env', path=os.defpath)
    if not env_program:
        raise OSError('The system env executable is required for the desktop launcher.')
    command = desktop_exec(env_program) + ' -- ' + desktop_exec(paths['binary'])
    desktop = desktop.replace('Exec=amdgpu-tuner', 'Exec=' + command)
    paths['desktop'].parent.mkdir(parents=True, exist_ok=True)
    paths['desktop'].write_text(desktop)
    paths['desktop'].chmod(0o644)
    if shutil.which('desktop-file-validate'):
        subprocess.run(['desktop-file-validate', str(paths['desktop'])], check=True)
    refresh_caches(prefix)
    print(f'Installed: {paths["binary"]}')
    print(f'Launcher:  {paths["desktop"]}')
    print(f'Icon:      {paths["icon"]}')
    print('Start from your application menu, or run the executable above.')


if __name__ == '__main__':
    try:
        main()
    except (OSError, subprocess.CalledProcessError) as error:
        sys.exit(f'Installation failed: {error}')
