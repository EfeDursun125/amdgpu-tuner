#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Validate source-distribution metadata, licenses, local README media and vector assets."""
from pathlib import Path
import re
import tomllib
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
required = ['LICENSE', 'Cargo.lock', 'README.md', 'THIRD_PARTY.md', 'CONTRIBUTING.md', 'CHANGELOG.md', '.gitattributes', '.github/workflows/ci.yml', 'docs/images/amdgpu-tuner.png', 'packaging/amdgpu-tuner.desktop', 'scripts/install.py']
for name in required:
    assert (ROOT / name).is_file(), f'Missing distribution file: {name}'
package = tomllib.loads((ROOT / 'Cargo.toml').read_text())['package']
assert package['license'] == 'GPL-3.0-only'
assert package['rust-version'] == '1.92'
license_text = (ROOT / 'LICENSE').read_text()
assert 'GNU GENERAL PUBLIC LICENSE' in license_text and 'Version 3, 29 June 2007' in license_text
assert 'MIT License' in (ROOT / 'vendor/material-1.1.0/LICENSE.md').read_text()
readme = (ROOT / 'README.md').read_text()
references = re.findall(r'!\[[^\]]*\]\(([^\s)]+)\)', readme) + re.findall(r'<img[^>]+src="([^"]+)"', readme)
assert references, 'README must include local project images'
for reference in references:
    if not reference.startswith(('http://', 'https://')):
        assert (ROOT / reference).is_file(), f'Broken README media reference: {reference}'
for path in [ROOT / 'packaging/amdgpu-tuner.svg', *ROOT.glob('ui/icons/*.svg')]:
    svg = ET.parse(path).getroot()
    assert svg.tag.endswith('svg') and 'viewBox' in svg.attrib, path
    assert not any(element.tag.endswith('script') for element in svg.iter()), path
png = ROOT / 'docs/images/amdgpu-tuner.png'
assert png.read_bytes().startswith(b'\x89PNG\r\n\x1a\n'), 'Screenshot must be an actual PNG'
# Check only source/documentation roots, never local build caches or user configuration.
secret = re.compile(r'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----|ghp_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{40,}|AKIA[A-Z0-9]{16}')
for directory in ['src', 'ui', 'docs', 'packaging', '.github']:
    for path in (ROOT / directory).rglob('*'):
        if path.is_file() and path.suffix not in ['.png']:
            assert not secret.search(path.read_text()), f'Potential private credential: {path}'
print('Repository: GPLv3 metadata, third-party notices, README images, SVG assets and source hygiene: PASS')
