# Contributing

Contributions are welcome under GPL-3.0-only. Keep third-party licenses and notices intact.

## Development

Install the dependencies listed in [README.md](README.md), then run:

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked
python3 tests/cli.py
python3 tests/packaging.py
python3 scripts/check-repository.py
```

With a graphical Linux session and Pillow installed:

```sh
python3 tests/ui.py
python3 tests/layout.py
```

To test the release binary, set `AMDGPU_TUNER_BINARY` to its absolute path when running Python tests.

## Behavior to preserve

- Never infer that an unset environment variable means a driver feature is disabled.
- Preserve unrelated values and flags, including those introduced by newer drivers.
- Missing optional layers must be hidden, and their stored overrides must remain inactive.
- Keep switches alive when model data changes so animation state is preserved.
- Keep the interface English and preserve system theme integration.
- Do not change the user's real profile or session settings in tests.
- Distinguish environment requests from proof that a driver feature is active in a game.

For new RADV controls, provide a primary upstream source, supported versions/hardware where known, accurate override semantics and appropriate validation.

## Pull requests

Describe the problem, resulting behavior and checks performed. For visual changes, include screenshots and check compact windows and fractional scaling. The [PR template](.github/pull_request_template.md) contains a short checklist.
