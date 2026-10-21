# Additional RADV settings to consider

Reviewed against the [Mesa environment-variable documentation](https://docs.mesa3d.org/envvars.html) on 2026-10-21. These are candidates, not implemented controls or universal performance recommendations. Support must be checked against the installed Mesa build and GPU.

| Setting | Purpose | Suggested control |
| --- | --- | --- |
| `RADV_TEX_ANISO` | Force anisotropic filtering up to 16×. | Inherit / 2× / 4× / 8× / 16× selector. |
| `RADV_PERFTEST=cswave32` | Request wave32 compute shaders on GFX10+. | Inherit / request / remove flag. |
| `RADV_PERFTEST=pswave32` | Request wave32 pixel shaders on GFX10+. | Inherit / request / remove flag. |
| `RADV_PERFTEST=gewave32` | Request wave32 vertex, tessellation and geometry shaders on GFX10+. | Inherit / request / remove flag. |
| `RADV_PERFTEST=dmashaders` | Upload shaders to invisible VRAM; documented as potentially useful on systems without resizable BAR. | Advanced flag control. |
| `RADV_PERFTEST=rtwave64` | Request wave64 ray-tracing shaders on GFX10+. | Advanced, per-game RT tuning. |
| `RADV_PERFTEST=rtcps` | Use CPS lowering instead of function calls for ray tracing. | Advanced, per-game RT tuning. |
| `RADV_DEBUG=nogpl` | Explicitly disable graphics pipeline library support. | Troubleshooting control; different from removing the old `gpl` request. |
| `RADV_DEBUG=nodcc` | Disable Delta Color Compression on images. | Troubleshooting control. |

Shader wave choices can help or hurt depending on the workload. Removing a request does not force the driver's automatic behavior off. Avoid contradictory positive/negative flags in a future UI, preserve unrelated flags, and show the source and support status of each setting.

Related Mesa-wide Vulkan controls, separate from RADV-specific flags:

- `MESA_VK_WSI_PRESENT_MODE`: `fifo`, `relaxed`, `mailbox`, `immediate`. A presentation-mode selector is more appropriate than an on/off switch.
- `MESA_SHADER_CACHE_DISABLE`: controls the on-disk shader cache. A future “Shader cache” switch needs inverse mapping to this variable.
- `MESA_SHADER_CACHE_MAX_SIZE`: a size field rather than a switch.

Experimental features such as emulated ray tracing or transfer queues belong in an advanced section with version and hardware constraints, rather than a generic “performance” preset.
