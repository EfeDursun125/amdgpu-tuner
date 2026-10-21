#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Validate install/launch metadata/uninstall in temporary prefixes, including special characters."""
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET

root = Path(__file__).resolve().parents[1]
binary = Path(os.environ.get('AMDGPU_TUNER_BINARY', root / 'target/debug/amdgpu-tuner')).absolute()
ET.parse(root / 'packaging/amdgpu-tuner.svg')
with tempfile.TemporaryDirectory(prefix='amdgpu-packaging-') as directory:
    config = Path(directory) / 'config'
    config.mkdir()
    profile = config / 'amdgpu-tuner/profile.json'
    profile.parent.mkdir()
    profile.write_text('{"overrides": {}}')
    session = config / 'environment.d/90-amdgpu-tuner.conf'
    session.parent.mkdir()
    session.write_text('# preserved by uninstall\n')
    env = dict(os.environ, XDG_CONFIG_HOME=str(config))
    for name in ['normal', 'space $money %percent `tick` "quote" \\slash']:
        prefix = Path(directory) / name
        subprocess.run([sys.executable, str(root / 'scripts/install.py'), '--prefix', str(prefix), '--binary', str(binary)], env=env, check=True)
        installed = prefix / 'bin/amdgpu-tuner'
        assert installed.read_bytes() == binary.read_bytes()
        assert 'AMDGPU Tuner' in subprocess.check_output([str(installed), '--help'], env=env, text=True)
        desktop = prefix / 'share/applications/amdgpu-tuner.desktop'
        icon = prefix / 'share/icons/hicolor/scalable/apps/amdgpu-tuner.svg'
        assert icon.read_bytes() == (root / 'packaging/amdgpu-tuner.svg').read_bytes()
        licenses = prefix / 'share/licenses/amdgpu-tuner'
        assert (licenses / 'LICENSE').read_bytes() == (root / 'LICENSE').read_bytes()
        assert (licenses / 'Material-MIT.txt').read_bytes() == (root / 'vendor/material-1.1.0/LICENSE.md').read_bytes()
        assert (licenses / 'THIRD_PARTY.md').read_bytes() == (root / 'THIRD_PARTY.md').read_bytes()
        if shutil.which('desktop-file-validate'):
            subprocess.run(['desktop-file-validate', str(desktop)], check=True)
        try:
            from gi.repository import Gio, GLib
        except ImportError:
            Gio = None
        if Gio is not None:
            info = Gio.DesktopAppInfo.new_from_filename(str(desktop))
            assert info is not None, desktop
            # Desktop Entry string escapes are decoded by Gio before command-line parsing.
            valid, argv = GLib.shell_parse_argv(info.get_commandline())
            assert valid and argv[1] == '--' and argv[2].replace('%%', '%') == str(installed), argv
        if shutil.which('gio'):
            help_desktop = desktop.with_name('amdgpu-tuner-help-test.desktop')
            lines = desktop.read_text().splitlines()
            help_desktop.write_text('\n'.join(line + ' --help' if line.startswith('Exec=') else line for line in lines) + '\n')
            output = subprocess.check_output(['gio', 'launch', str(help_desktop)], env=env, text=True)
            assert 'AMDGPU Tuner' in output, output
            help_desktop.unlink()
        unrelated = prefix / 'share/unrelated.txt'
        unrelated.parent.mkdir(exist_ok=True, parents=True)
        unrelated.write_text('keep')
        subprocess.run([sys.executable, str(root / 'scripts/install.py'), '--prefix', str(prefix), '--uninstall'], env=env, check=True)
        assert not installed.exists() and not desktop.exists() and not icon.exists()
        assert unrelated.read_text() == 'keep'
        assert profile.read_text() == '{"overrides": {}}'
        assert session.read_text() == '# preserved by uninstall\n'
print('Packaging: SVG, executable, desktop quoting, icon, licenses and non-destructive uninstall: PASS')
