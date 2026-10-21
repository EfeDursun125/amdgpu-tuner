#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Capture a real Slint window with a disposable example profile for the README."""
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
binary = Path(os.environ.get('AMDGPU_TUNER_BINARY', ROOT / 'target/debug/amdgpu-tuner')).absolute()
output = ROOT / 'docs/images/amdgpu-tuner.png'
output.parent.mkdir(parents=True, exist_ok=True)
with tempfile.TemporaryDirectory(prefix='amdgpu-readme-') as directory:
    config = Path(directory)
    original = Path(os.environ.get('XDG_CONFIG_HOME', str(Path.home() / '.config'))) / 'kdeglobals'
    if original.exists():
        (config / 'kdeglobals').write_bytes(original.read_bytes())
    profile = config / 'amdgpu-tuner/profile.json'
    profile.parent.mkdir()
    profile.write_text(json.dumps({'overrides': {'anti-lag': True, 'glthread': True}}))
    env = dict(os.environ, XDG_CONFIG_HOME=str(config))
    subprocess.run([str(binary), '--screenshot', str(output), '0', '1120x1000'], env=env, check=True, timeout=30)
print(output)
