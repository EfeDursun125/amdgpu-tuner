#!/usr/bin/env python3
# Copyright (C) 2026 EfeDursun125
# This file is part of AMDGPU Tuner; see NOTICE and LICENSE.
# SPDX-License-Identifier: GPL-3.0-only
"""Create a curated source ZIP without builds, private profiles or temporary screenshots."""
import hashlib
import argparse
from pathlib import Path
import subprocess
import tempfile
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--with-dependencies', action='store_true', help='Include locked Cargo sources for a binary release')
args = parser.parse_args()
version = tomllib.loads((ROOT / 'Cargo.toml').read_text())['package']['version']
name = f'amdgpu-tuner-{version}-' + ('source-with-dependencies' if args.with_dependencies else 'source')
output = ROOT / 'dist'
output.mkdir(exist_ok=True)
files = [ROOT / file for file in ['.gitignore', '.gitattributes', 'Cargo.toml', 'Cargo.lock', 'build.rs', 'LICENSE', 'NOTICE', 'README.md', 'THIRD_PARTY.md', 'CONTRIBUTING.md', 'CHANGELOG.md']]
for directory in ['.github', 'docs', 'packaging', 'scripts', 'src', 'tests', 'ui', 'vendor']:
    files.extend(path for path in (ROOT / directory).rglob('*') if path.is_file() and '__pycache__' not in path.parts and path.suffix != '.pyc')
archive = output / f'{name}.zip'
with tempfile.TemporaryDirectory(prefix='amdgpu-source-') as temporary, zipfile.ZipFile(archive, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=6, strict_timestamps=False) as bundle:
    for path in sorted(set(files)):
        if path.is_symlink():
            raise SystemExit(f'Refusing symlink in source bundle: {path}')
        relative = path.relative_to(ROOT)
        info = zipfile.ZipInfo(f'{name}/{relative.as_posix()}', date_time=(2026, 1, 1, 0, 0, 0))
        info.compress_type = zipfile.ZIP_DEFLATED
        info.external_attr = 0o100644 << 16
        bundle.writestr(info, path.read_bytes())
    if args.with_dependencies:
        vendored = Path(temporary) / 'rust-vendor'
        subprocess.run(['cargo', 'vendor', '--locked', '--versioned-dirs', str(vendored)], cwd=ROOT, check=True, stdout=subprocess.DEVNULL)
        config = '[source.crates-io]\nreplace-with = "vendored-sources"\n\n[source.vendored-sources]\ndirectory = "rust-vendor"\n'
        bundle.writestr(f'{name}/.cargo/config.toml', config)
        for path in sorted(vendored.rglob('*')):
            if path.is_file():
                bundle.write(path, f'{name}/rust-vendor/{path.relative_to(vendored).as_posix()}')
digest = hashlib.sha256(archive.read_bytes()).hexdigest()
checksums = output / 'SHA256SUMS'
lines = checksums.read_text().splitlines() if checksums.exists() else []
lines = [line for line in lines if not line.endswith('  ' + archive.name)]
lines.append(f'{digest}  {archive.name}')
checksums.write_text('\n'.join(lines) + '\n')
print(archive)
print(f'SHA256: {digest}')
