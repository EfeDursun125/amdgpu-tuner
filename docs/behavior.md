# Values, defaults and profile behavior

## Values and defaults

- Reads the actual **inherited process environment**. For example, `ENABLE_LAYER_MESA_ANTI_LAG=1` appears enabled unless the layer's disable environment takes precedence.
- Reads Vulkan `enable_environment` and `disable_environment` definitions from installed JSON manifests. A manifest is evidence of configuration, not proof that the library loads or a game uses the extension.
- A defined `disable_environment` variable disables its layer, including values `0` and an empty string. Enabling a layer removes these variables from launched children with `env_remove()`; Steam options use `env -u`. No manual override leaves the inherited environment intact.
- **Low Latency Layer is optional.** Its control is shown only when a `VK_LAYER_KORTHOS_low_latency` manifest and the shared library it references are present. A stale `LOW_LATENCY_LAYER=1` value or a dangling manifest does not expose the setting. Saved overrides for a missing layer are retained but excluded from generated launch options and session files. Refresh rescans installations. See the [Korthos project](https://github.com/Korthos-Software/low_latency_layer).
- Respects `VK_IMPLICIT_LAYER_PATH` and `VK_ADD_IMPLICIT_LAYER_PATH` during discovery. Loader filters and other applications' environments may still affect runtime layer selection.
- Uses **Automatic / unknown** when the driver's internal default cannot be determined. An unset value is never presented as proof that a feature is disabled. Click `?` to request enabling it; then use the switch to disable the request.
- **Use system** removes a manual override. **System & sources** shows inherited values and manifest paths.
- Other processes, including Steam, may inherit different values. systemd's environment and shell configuration files are not merged into the running process and represented as active. Refresh cannot change this process's startup environment.
- Disabling an item in `RADV_PERFTEST`, `RADV_DEBUG` or `RADV_EXPERIMENTAL` removes only that flag. It does not force a driver's default optimization off. Unrelated flags are preserved; unsupported flags may be ignored by the driver.
- `vblank_mode`: on is `3`, off is `0`. Existing `1` / `2` values are labeled accurately. This affects OpenGL, not Vulkan presentation.
- `RADV_FORCE_VRS`: on is `2x2`, off is `1x1`. Existing `2x1` / `1x2` values remain visible. A `RADV_FORCE_VRS_CONFIG_FILE` can take precedence.

## Applying a profile

1. **Save profile** writes `$XDG_CONFIG_HOME/amdgpu-tuner/profile.json` only.
2. **Launch & apply → Launch** starts a command immediately with the current overrides. Arguments are parsed without executing a shell.
3. **Steam launch options** exports only manual overrides. Paste into a game's launch options.
4. **Apply to session** writes `$XDG_CONFIG_HOME/environment.d/90-amdgpu-tuner.conf` for the next systemd user session. `environment.d` cannot unset inherited variables, so profiles that enable a Vulkan layer are rejected without modifying the existing file. Use Launch or Steam options for those profiles. Disabling layers and exporting other settings remain supported. Desktop propagation depends on the session manager. Running applications remain unchanged.
5. **Remove session file** removes only the application's session file. Log in again to clear values already inherited by processes. Restoring every override and applying to the session also removes this file.

`XDG_CONFIG_HOME` defaults to `~/.config`. An entirely inherited profile exports `%command%` for Steam. Saving the profile and writing the session file are separate actions.

If version 0.1.0 wrote `DISABLE_...="0"` into the application's session file, use **Remove session file** (or `--remove-session`) and log in again to clear those inherited values. Existing games and Steam keep their startup environment; corrected Launch and Steam options explicitly remove the disabling variables for the launched game. Other disabling assignments in your own shell/session configuration must be removed at their source if you want them absent globally.

```sh
cargo run -- --inspect
cargo run -- --steam-line
cargo run -- --run vkcube
cargo run -- --write-session
cargo run -- --remove-session
```

For potential future controls, see [additional RADV settings](RADV-options.md). These suggestions are not added or applied automatically.

