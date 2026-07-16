# In-game overlay (foundation)

**Status: the data pipeline ships; the rendering layer does not yet exist.**

A real in-game overlay for a Vulkan game is a native Vulkan implicit layer that
hooks `vkQueuePresentKHR` and draws UI over each frame. That layer is a
substantial standalone effort and is **not implemented here**. What *is* built
is the launcher-side half it needs: a live state file the overlay reads.

## The data pipeline (implemented)

While a game is running, the launcher writes `overlay-state.json` into its data
dir (`~/.local/share/riko/overlay-state.json`) and deletes it when the game
exits. See `crates/riko-core/src/overlay.rs`.

```json
{
  "game_id": 3,
  "game_name": "Snowy Peak",
  "started_at_unix": 1700000000,
  "friends_online": 2
}
```

This is a stable IPC surface. Anything can consume it — the overlay layer below,
a MangoHud custom-text feed, a Conky widget, a stream deck, or a second monitor
companion window.

## The rendering layer (to build)

The intended consumer is a Vulkan implicit layer, packaged as a `vulkan-layer`
plugin (same format as the built-in `fps-unlocker`, which already injects a
layer via `VK_ADD_IMPLICIT_LAYER_PATH`). It would:

1. Load on game start via the layer manifest the launcher already wires up.
2. Poll `overlay-state.json` (or watch it via inotify).
3. Render a small HUD on present — session timer, friends online, and toast
   notifications — using Dear ImGui or a minimal vertex/texture pass.

Recommended path: fork an existing MIT-licensed present-hook overlay (vkBasalt
and MangoHud are both good references for the layer plumbing) and swap the draw
step for the state above. Ship it through the plugin marketplace so users opt in.

Until that layer exists, the launcher still produces everything it needs, so the
remaining work is isolated to the native layer — no launcher changes required.
