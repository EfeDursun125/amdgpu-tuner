#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""End-to-end checks using an isolated XDG_CONFIG_HOME; no desktop config is edited."""
import json
import os
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
    env["VK_IMPLICIT_LAYER_PATH"] = str(initial_manifests)
    def run(*args):
        return subprocess.check_output([str(binary), *args], env=env, text=True)
    assert "ENABLE_LAYER_MESA_ANTI_LAG: On" in run("--inspect")
    env["DISABLE_LAYER_MESA_ANTI_LAG"] = "1"
    assert "ENABLE_LAYER_MESA_ANTI_LAG: Off" in run("--inspect")
    path = Path(directory) / "amdgpu-tuner/profile.json"
    path.parent.mkdir()
    path.write_text(json.dumps({"overrides": {"anti-lag": True, "glthread": True, "vsync": False, "gpl": False, "sam": True}}))
    env["RADV_PERFTEST"] = "future,gpl"
    assert "ENABLE_LAYER_MESA_ANTI_LAG: On" in run("--inspect")
    actual = json.loads(run("--run", "python3", "-c", "import os,json; print(json.dumps(dict(os.environ)))"))
    assert actual["ENABLE_LAYER_MESA_ANTI_LAG"] == "1"
    assert actual["DISABLE_LAYER_MESA_ANTI_LAG"] == "0"
    assert actual["mesa_glthread"] == "true"
    assert actual["vblank_mode"] == "0"
    assert actual["RADV_PERFTEST"] == "future,sam"
    assert "%command%" in run("--steam-line")
    assert "future,sam" in run("--steam-line")
    run("--write-session")
    session = Path(directory) / "environment.d/90-amdgpu-tuner.conf"
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
print("CLI integration: inherited values, disable precedence, launch, flag preservation, Steam, session write/remove, expansion rejection, missing optional layer: PASS")
