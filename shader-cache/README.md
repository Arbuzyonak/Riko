# Community shader caches

Precompiled DXVK/VKD3D shader caches shared between users so the first minutes
of a game don't stutter. It's **opt-in** (Settings → "Download community shader
caches") and distributed exactly like the plugin marketplace: a static
`index.json` + zipped cache archives, verified by SHA-256 before use.

## How it works

Before launch (only when enabled), the launcher:

1. Fingerprints your GPU with `vulkaninfo` into a `gpu_key`
   (e.g. `nvidia-geforce-rtx-2070-super`).
2. Fetches `index.json` and looks for an entry matching `{game_id, gpu_key}`.
3. Downloads the archive, **verifies its SHA-256**, and extracts it into
   `~/.cache/vortex-shaders/<game_id>/` — the same directory the launcher points
   `VKD3D_SHADER_CACHE_PATH` / `DXVK_STATE_CACHE_PATH` at.

The index URL defaults to `shader_cache::DEFAULT_INDEX_URL` and can be
overridden in `config.toml`:

```toml
[shader_cache]
community = true
index_url = "https://raw.githubusercontent.com/OWNER/riko-shaders/main/index.json"
```

## index.json format

```json
{
  "entries": [
    {
      "game_id": 3,
      "gpu_key": "nvidia-geforce-rtx-2070-super",
      "label": "NVIDIA RTX 2070 SUPER · dxvk 2.4",
      "download_url": "https://github.com/OWNER/riko-shaders/releases/download/g3/game3-nvidia.zip",
      "sha256": "<hex digest of the zip>",
      "size_bytes": 12345678
    }
  ]
}
```

## Contributing your cache

After playing a game for a while with `shader_cache = true`, your compiled
cache lives in `~/.cache/vortex-shaders/<game_id>/`. To share it:

```sh
cd ~/.cache/vortex-shaders
zip -r game3-mygpu.zip 3/
sha256sum game3-mygpu.zip
```

Upload the zip (a GitHub Release asset works), then open a PR adding an entry
to `index.json` with your `gpu_key` (print it from the Doctor GPU line, slugged)
and the checksum. Review + merge is the trust gate.

> Caches are GPU- and driver-specific. Applying a cache from a very different
> driver version is harmless — the game just recompiles what doesn't match.
