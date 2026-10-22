#!/usr/bin/env python3
# Copyright (C) 2026 EfeDursun125
# This file is part of AMDGPU Tuner; see NOTICE and LICENSE.
# SPDX-License-Identifier: GPL-3.0-only
"""Capture default and host-system themes using identical disposable example profiles."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
binary = Path(os.environ.get('AMDGPU_TUNER_BINARY', ROOT / 'target/debug/amdgpu-tuner')).absolute()
images = ROOT / 'docs/images'
images.mkdir(parents=True, exist_ok=True)
original = Path(os.environ.get('XDG_CONFIG_HOME', str(Path.home() / '.config'))) / 'kdeglobals'
for system_theme, filename in [(False, 'amdgpu-tuner-blue.png'), (True, 'amdgpu-tuner.png')]:
    with tempfile.TemporaryDirectory(prefix='amdgpu-readme-') as directory:
        config = Path(directory)
        if system_theme and original.exists():
            (config / 'kdeglobals').write_bytes(original.read_bytes())
        profile = config / 'amdgpu-tuner/profile.json'
        profile.parent.mkdir()
        profile.write_text(json.dumps({'overrides': {'anti-lag': True, 'glthread': True}}))
        env = dict(os.environ, XDG_CONFIG_HOME=str(config))
        if not system_theme:
            # Bypass KDE palette/font import; retain Material's system light/dark choice.
            env['XDG_CURRENT_DESKTOP'] = ''
        output = images / filename
        subprocess.run([str(binary), '--screenshot', str(output), '0', '1120x1080'], env=env, check=True, timeout=30)
        print(output)
