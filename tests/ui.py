#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Requires a graphical Linux session. Uses disposable profiles and synthetic pointer events."""
import os
from pathlib import Path
import subprocess
import tempfile
import time
from PIL import Image

root = Path(__file__).resolve().parents[1]
binary = Path(os.environ.get("AMDGPU_TUNER_BINARY", root / "target/debug/amdgpu-tuner")).absolute()
shots = root / "screenshots"
shots.mkdir(exist_ok=True)
with tempfile.TemporaryDirectory(prefix="amdgpu-tuner-ui-") as directory:
    env = dict(os.environ, XDG_CURRENT_DESKTOP="KDE", XDG_CONFIG_HOME=directory, ENABLE_LAYER_MESA_ANTI_LAG="0", DISABLE_LAYER_MESA_ANTI_LAG="0")
    env.pop("vblank_mode", None)
    config = Path(directory) / "kdeglobals"
    source = Path(os.environ.get("XDG_CONFIG_HOME", str(Path.home() / ".config"))) / "kdeglobals"
    if source.exists():
        config.write_bytes(source.read_bytes())
    subprocess.run([str(binary), "--ui-smoke-test"], env=env, check=True, timeout=20)
    # The complete control/animation flow must also work with the optional row absent.
    isolated = Path(directory) / "empty-layers"
    isolated.mkdir()
    without_layer = dict(env, VK_IMPLICIT_LAYER_PATH=str(isolated))
    subprocess.run([str(binary), "--ui-smoke-test"], env=without_layer, check=True, timeout=20)
    subprocess.run([str(binary), "--screenshot", str(shots / "without-low-latency.png")], env=without_layer, check=True, timeout=20)
    # Explicitly inherit 1: the screenshot must render the real on state, with no manual profile.
    env["ENABLE_LAYER_MESA_ANTI_LAG"] = "1"
    subprocess.run([str(binary), "--screenshot", str(shots / "inherited-on.png")], env=env, check=True, timeout=20)
    # Change only the disposable palette while the window is running.
    dark = """[Colors:Window]
BackgroundNormal=#271d1c
BackgroundAlternate=#73342b
ForegroundNormal=#f1dfdc
ForegroundInactive=#d8c2be
[Colors:View]
BackgroundNormal=#1a1110
[Colors:Button]
BackgroundNormal=#322826
[Colors:Selection]
BackgroundNormal=#ffb4a8
ForegroundActive=#561e16
[General]
ColorScheme=TestDark
font=Sans Serif,11
"""
    light = """[Colors:Window]
BackgroundNormal=#eff0f6
BackgroundAlternate=#d8e2ff
ForegroundNormal=#191c20
ForegroundInactive=#44474e
[Colors:View]
BackgroundNormal=#f8f9ff
[Colors:Button]
BackgroundNormal=#e2e6ee
[Colors:Selection]
BackgroundNormal=#435e91
ForegroundActive=#ffffff
[General]
ColorScheme=TestLight
font=Sans Serif,11
"""
    config.write_text(dark)
    capture = shots / "light-theme.png"
    child = subprocess.Popen([str(binary), "--screenshot", str(capture)], env=env)
    time.sleep(1)
    config.write_text(light)
    assert child.wait(timeout=20) == 0
    with Image.open(capture) as img:
        assert img.getpixel((1110, 400))[:3] == (248, 249, 255), img.getpixel((1110, 400))
print("UI integration: Material switch, inherited 1, live KDE palette reload: PASS")
