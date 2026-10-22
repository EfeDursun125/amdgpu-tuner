#!/usr/bin/env python3
# Copyright (C) 2026 EfeDursun125
# This file is part of AMDGPU Tuner; see NOTICE and LICENSE.
# SPDX-License-Identifier: GPL-3.0-only
"""End-to-end checks using an isolated XDG_CONFIG_HOME; no desktop config is edited."""
import json
import os
import shlex
from pathlib import Path
import subprocess
import tempfile

binary = Path(os.environ.get("AMDGPU_TUNER_BINARY", Path(__file__).resolve().parents[1] / "target/debug/amdgpu-tuner")).absolute()
with tempfile.TemporaryDirectory(prefix="amdgpu-tuner-cli-") as directory:
    env = {k: v for k, v in os.environ.items() if not k.startswith(("RADV_", "ENABLE_LAYER_MESA", "DISABLE_LAYER_MESA"))}
    env.update(XDG_CONFIG_HOME=directory, ENABLE_LAYER_MESA_ANTI_LAG="1")
    initial_manifests = Path(directory) / "initial-layers"
    initial_manifests.mkdir()
    (initial_manifests / "anti-lag.json").write_text(json.dumps({"layer": {
        "name": "VK_LAYER_MESA_anti_lag", "library_path": "/fixture/not-a-runtime-library.so",
        "enable_environment": {"ENABLE_LAYER_MESA_ANTI_LAG": "1"},
        "disable_environment": {"DISABLE_LAYER_MESA_ANTI_LAG": "1"}}}))
    (initial_manifests / "korthos.json").write_text(json.dumps({"layer": {
        "name": "VK_LAYER_KORTHOS_low_latency", "library_path": str(binary),
        "enable_environment": {"LOW_LATENCY_LAYER": "1"},
        "disable_environment": {"DISABLE_LOW_LATENCY_LAYER": "1"}}}))
    env["VK_IMPLICIT_LAYER_PATH"] = str(initial_manifests)
    def run(*args):
        return subprocess.check_output([str(binary), *args], env=env, text=True)
    assert 'Copyright (C) 2026 EfeDursun125' in run('--help')
    source_root = Path(__file__).resolve().parents[1]
    assert run('--license') == (source_root / 'NOTICE').read_text() + '\n' + (source_root / 'LICENSE').read_text()
    assert "ENABLE_LAYER_MESA_ANTI_LAG: On" in run("--inspect")
    for value in ["1", "0", "", "unexpected"]:
        env["DISABLE_LAYER_MESA_ANTI_LAG"] = value
        env["DISABLE_LOW_LATENCY_LAYER"] = value
        env["LOW_LATENCY_LAYER"] = "1"
        assert "ENABLE_LAYER_MESA_ANTI_LAG: Off" in run("--inspect")
        assert "LOW_LATENCY_LAYER: Off" in run("--inspect")
    path = Path(directory) / "amdgpu-tuner/profile.json"
    path.parent.mkdir()
    overrides = {"anti-lag": True, "low-latency": True, "glthread": True, "vsync": False, "gpl": False, "sam": True}
    path.write_text(json.dumps({"overrides": overrides}))
    env["RADV_PERFTEST"] = "future,gpl"
    assert "ENABLE_LAYER_MESA_ANTI_LAG: On" in run("--inspect")
    probe = ["python3", "-c", "import os,json; print(json.dumps(dict(os.environ)))"]
    for value in ["1", "0", "", "unexpected"]:
        env["DISABLE_LAYER_MESA_ANTI_LAG"] = value
        env["DISABLE_LOW_LATENCY_LAYER"] = value
        line = shlex.split(run("--steam-line"))
        assert line[-1] == "%command%"
        launched = json.loads(run("--run", *probe))
        steam = json.loads(subprocess.check_output([*line[:-1], *probe], env=env, text=True))
        for actual in [launched, steam]:
            assert actual["ENABLE_LAYER_MESA_ANTI_LAG"] == "1"
            assert actual["LOW_LATENCY_LAYER"] == "1"
            assert "DISABLE_LAYER_MESA_ANTI_LAG" not in actual
            assert "DISABLE_LOW_LATENCY_LAYER" not in actual
            assert actual["mesa_glthread"] == "true"
            assert actual["vblank_mode"] == "0"
            assert actual["RADV_PERFTEST"] == "future,sam"
    session = Path(directory) / "environment.d/90-amdgpu-tuner.conf"
    rejected = subprocess.run([str(binary), "--write-session"], env=env, capture_output=True, text=True)
    assert rejected.returncode != 0 and "environment.d cannot remove" in rejected.stderr
    assert not session.exists()
    session.parent.mkdir(exist_ok=True)
    old = 'DISABLE_LAYER_MESA_ANTI_LAG="0"\n'
    session.write_text(old)
    assert subprocess.run([str(binary), "--write-session"], env=env, capture_output=True).returncode != 0
    assert session.read_text() == old
    # Turning layers off is representable in environment.d, and must still work.
    path.write_text(json.dumps({"overrides": {"anti-lag": False, "low-latency": False}}))
    disabled = json.loads(run("--run", *probe))
    assert disabled["DISABLE_LAYER_MESA_ANTI_LAG"] == "1"
    assert disabled["DISABLE_LOW_LATENCY_LAYER"] == "1"
    assert disabled["ENABLE_LAYER_MESA_ANTI_LAG"] == "0"
    assert disabled["LOW_LATENCY_LAYER"] == "0"
    run("--write-session")
    assert 'DISABLE_LAYER_MESA_ANTI_LAG="1"' in session.read_text()
    # Other settings retain session export support.
    path.write_text(json.dumps({"overrides": {k: v for k, v in overrides.items() if k not in ["anti-lag", "low-latency"]}}))
    run("--write-session")
    assert 'RADV_PERFTEST="future,sam"' in session.read_text()
    run("--remove-session")
    assert not session.exists()
    assert path.exists()  # Removing global settings never deletes the profile.
    # Reject expansion characters instead of silently changing inherited values.
    env["RADV_PERFTEST"] = "$NOT_LITERAL"
    result = subprocess.run([str(binary), "--write-session"], env=env, capture_output=True)
    assert result.returncode != 0
    assert not session.exists()
    # A stale environment value or saved override must not expose an uninstalled layer.
    manifests = Path(directory) / "isolated-layers"
    manifests.mkdir()
    env["VK_IMPLICIT_LAYER_PATH"] = str(manifests)
    env["LOW_LATENCY_LAYER"] = "1"
    path.write_text(json.dumps({"overrides": {"low-latency": True}}))
    assert "LOW_LATENCY_LAYER:" not in run("--inspect")
    assert run("--steam-line").strip() == "%command%"
    broken = {"layer": {"name": "VK_LAYER_KORTHOS_low_latency", "library_path": "/missing/korthos.so",
                       "enable_environment": {"LOW_LATENCY_LAYER": "1"}}}
    (manifests / "low_latency_layer.json").write_text(json.dumps(broken))
    assert "LOW_LATENCY_LAYER:" not in run("--inspect")
    assert run("--steam-line").strip() == "%command%"
print("CLI integration: presence-based disable precedence, real Launch/Steam unsets, layer off, session rejection/preservation, flag preservation, session write/remove, missing optional layer: PASS")
