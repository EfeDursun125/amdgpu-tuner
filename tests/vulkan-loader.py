#!/usr/bin/env python3
# Copyright (C) 2026 EfeDursun125
# This file is part of AMDGPU Tuner; see NOTICE and LICENSE.
# SPDX-License-Identifier: GPL-3.0-only
"""Optional live test: verify installed layers are inserted by the real Vulkan Loader."""
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import tempfile

binary = Path(os.environ.get('AMDGPU_TUNER_BINARY', Path(__file__).resolve().parents[1] / 'target/debug/amdgpu-tuner')).absolute()
if not shutil.which('vulkaninfo'):
    print('SKIP: vulkaninfo is required for the live loader test')
    raise SystemExit(0)
layers = {'anti-lag': 'VK_LAYER_MESA_anti_lag', 'low-latency': 'VK_LAYER_KORTHOS_low_latency'}
disable = ['DISABLE_LAYER_MESA_ANTI_LAG', 'DISABLE_LOW_LATENCY_LAYER']
probe = ['vulkaninfo', '--summary']


def active_layers(result):
    assert result.returncode == 0, result.stderr
    return {name for name in layers.values() if f'Insert instance layer "{name}"' in result.stderr}


with tempfile.TemporaryDirectory(prefix='amdgpu-loader-') as directory:
    env = {k: v for k, v in os.environ.items() if not k.startswith('RADV_')}
    for key in [*disable, 'VK_LOADER_LAYERS_ENABLE', 'VK_LOADER_LAYERS_DISABLE', 'VK_INSTANCE_LAYERS']:
        env.pop(key, None)
    env.update(XDG_CONFIG_HOME=directory, VK_LOADER_DEBUG='layer', ENABLE_LAYER_MESA_ANTI_LAG='1', LOW_LATENCY_LAYER='1')
    expected = active_layers(subprocess.run(probe, env=env, capture_output=True, text=True, timeout=30))
    if not expected:
        print('SKIP: neither target layer is active in the direct Vulkan baseline')
        raise SystemExit(0)
    path = Path(directory) / 'amdgpu-tuner/profile.json'
    path.parent.mkdir()
    for enabled in [True, False]:
        path.write_text(json.dumps({'overrides': {key: enabled for key, name in layers.items() if name in expected}}))
        for value in ['1', '0', '']:
            for key in disable:
                env[key] = value
            launched = subprocess.run([str(binary), '--run', *probe], env=env, capture_output=True, text=True, timeout=30)
            line = shlex.split(subprocess.check_output([str(binary), '--steam-line'], env=env, text=True))
            assert line.pop() == '%command%'
            steam = subprocess.run([*line, *probe], env=env, capture_output=True, text=True, timeout=30)
            for result in [launched, steam]:
                assert active_layers(result) == (expected if enabled else set()), (enabled, value, result.stderr)
    print('Live Vulkan Loader: Launch and Steam enable/disable, inherited 1/0/empty: PASS (' + ', '.join(sorted(expected)) + ')')
