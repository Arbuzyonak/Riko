<script lang="ts">
  import {
    getConfig,
    logout,
    updateConfig,
    uninstallRiko,
    type ConfigPatch,
    type ConfigView,
  } from "../lib/api";
  import { navigate } from "../lib/router.svelte";
  import { refreshStatus } from "../lib/stores/app.svelte";
  import { toast } from "../lib/stores/toast.svelte";
  import Toggle from "../lib/components/Toggle.svelte";
  import Modal from "../lib/components/Modal.svelte";

  let cfg = $state<ConfigView | null>(null);
  let wineBinary = $state("");
  let envRows = $state<{ key: string; value: string }[]>([]);
  let confirmUninstall = $state(false);
  let uninstallBusy = $state(false);

  $effect(() => {
    if (cfg === null) load();
  });

  async function load() {
    cfg = await getConfig();
    wineBinary = cfg.wine_binary;
    envRows = Object.entries(cfg.wine_env).map(([key, value]) => ({ key, value }));
  }

  async function patch(p: ConfigPatch) {
    try {
      cfg = await updateConfig(p);
    } catch (e) {
      toast(String(e), "error");
      await load();
    }
  }

  async function saveWineBinary() {
    if (cfg && wineBinary !== cfg.wine_binary) {
      await patch({ wine_binary: wineBinary });
      toast("Wine binary updated", "success");
    }
  }

  async function saveEnv() {
    const env: Record<string, string> = {};
    for (const row of envRows) {
      if (row.key.trim()) env[row.key.trim()] = row.value;
    }
    await patch({ wine_env: env });
    toast("Environment variables saved", "success");
  }

  async function handleLogout() {
    await logout();
    await refreshStatus();
    navigate("/login");
  }

  async function handleUninstall() {
    uninstallBusy = true;
    try {
      await uninstallRiko();
    } catch (e) {
      toast(String(e), "error");
      uninstallBusy = false;
    }
  }
</script>

<div class="mx-auto flex max-w-3xl flex-col gap-8 p-8">
  <h1 class="text-2xl font-semibold tracking-tight text-white">Settings</h1>

  {#if cfg}
    <section class="flex flex-col gap-1 rounded-xl border border-edge bg-panel px-5 py-4">
      <h2 class="mb-2 text-sm font-semibold text-zinc-300 uppercase tracking-wide">
        Account
      </h2>
      <div class="flex items-center justify-between py-1">
        <div>
          <p class="text-sm text-zinc-200">{cfg.username ?? "Not signed in"}</p>
          <p class="text-xs text-zinc-500">
            {cfg.has_session ? "Session active" : "No active session"}
          </p>
        </div>
        <button
          class="rounded-lg border border-edge px-3.5 py-1.5 text-xs text-zinc-300 transition-colors hover:bg-panel-hover"
          onclick={handleLogout}
        >
          Log out
        </button>
      </div>
    </section>

    <section class="flex flex-col rounded-xl border border-edge bg-panel px-5 py-4">
      <h2 class="mb-2 text-sm font-semibold text-zinc-300 uppercase tracking-wide">
        Performance
      </h2>
      <Toggle
        checked={cfg.use_fsync}
        label="fsync"
        hint="Fast kernel-based synchronisation (preferred over esync)"
        onchange={(v) => patch({ use_fsync: v })}
      />
      <Toggle
        checked={cfg.use_esync}
        label="esync"
        hint="Eventfd-based synchronisation fallback"
        onchange={(v) => patch({ use_esync: v })}
      />
      <Toggle
        checked={cfg.use_gamemode}
        label="GameMode"
        hint="Switch CPU governor to performance while playing"
        onchange={(v) => patch({ use_gamemode: v })}
      />
      <Toggle
        checked={cfg.shader_cache}
        label="Shader cache"
        hint="Persist compiled shaders between sessions"
        onchange={(v) => patch({ shader_cache: v })}
      />
    </section>

    <section class="flex flex-col rounded-xl border border-edge bg-panel px-5 py-4">
      <h2 class="mb-2 text-sm font-semibold text-zinc-300 uppercase tracking-wide">
        Launcher
      </h2>
      <Toggle
        checked={cfg.filter_wine_noise}
        label="Filter Wine noise"
        hint="Hide known-harmless Wine warnings from game output"
        onchange={(v) => patch({ filter_wine_noise: v })}
      />
      <Toggle
        checked={cfg.auto_update}
        label="Auto-update Vortex"
        hint="Keep the Vortex client up to date"
        onchange={(v) => patch({ auto_update: v })}
      />
      <Toggle
        checked={cfg.minimize_while_playing}
        label="Minimize while playing"
        hint="Hide the launcher during a game and bring it back when the game closes"
        onchange={(v) => patch({ minimize_while_playing: v })}
      />
      <Toggle
        checked={cfg.presence_enabled}
        label="Discord Rich Presence"
        hint="Show what you're playing on Discord"
        onchange={(v) => patch({ presence_enabled: v })}
      />
    </section>

    <section class="flex flex-col gap-3 rounded-xl border border-edge bg-panel px-5 py-4">
      <h2 class="text-sm font-semibold text-zinc-300 uppercase tracking-wide">Wine</h2>
      <label class="flex flex-col gap-1.5">
        <span class="text-xs text-zinc-500">Wine binary</span>
        <div class="flex gap-2">
          <input
            type="text"
            bind:value={wineBinary}
            class="flex-1 rounded-lg border border-edge bg-surface px-3 py-2 font-mono text-sm text-white outline-none focus:border-accent"
          />
          <button
            class="rounded-lg border border-edge px-3.5 text-xs text-zinc-300 transition-colors hover:bg-panel-hover"
            onclick={saveWineBinary}
          >
            Save
          </button>
        </div>
      </label>

      <div class="flex flex-col gap-2">
        <span class="text-xs text-zinc-500">Extra environment variables</span>
        {#each envRows as row, i (i)}
          <div class="flex gap-2">
            <input
              type="text"
              placeholder="KEY"
              bind:value={row.key}
              class="w-48 rounded-lg border border-edge bg-surface px-3 py-1.5 font-mono text-xs text-white outline-none focus:border-accent"
            />
            <input
              type="text"
              placeholder="value"
              bind:value={row.value}
              class="flex-1 rounded-lg border border-edge bg-surface px-3 py-1.5 font-mono text-xs text-white outline-none focus:border-accent"
            />
            <button
              class="px-2 text-zinc-500 transition-colors hover:text-danger"
              aria-label="Remove variable"
              onclick={() => envRows.splice(i, 1)}
            >
              ✕
            </button>
          </div>
        {/each}
        <div class="flex gap-2">
          <button
            class="w-fit rounded-lg border border-dashed border-edge px-3 py-1.5 text-xs text-zinc-400 transition-colors hover:bg-panel-hover"
            onclick={() => envRows.push({ key: "", value: "" })}
          >
            + Add variable
          </button>
          <button
            class="w-fit rounded-lg border border-edge px-3 py-1.5 text-xs text-zinc-300 transition-colors hover:bg-panel-hover"
            onclick={saveEnv}
          >
            Save variables
          </button>
        </div>
      </div>
    </section>

    <section class="flex flex-col gap-1 rounded-xl border border-edge bg-panel px-5 py-4">
      <h2 class="mb-2 text-sm font-semibold text-zinc-300 uppercase tracking-wide">
        Paths
      </h2>
      {#each [["Wine prefix", cfg.wine_prefix], ["Vortex client", cfg.vortex_exe], ["Log file", cfg.log_file]] as [label, value] (label)}
        <div class="flex items-baseline justify-between gap-4 py-1">
          <span class="shrink-0 text-xs text-zinc-500">{label}</span>
          <span class="truncate font-mono text-xs text-zinc-400 select-text">{value}</span>
        </div>
      {/each}
    </section>

    <section
      class="flex items-center justify-between rounded-xl border border-danger/30 bg-danger/5 px-5 py-4"
    >
      <div>
        <h2 class="text-sm font-semibold text-danger">Uninstall Riko</h2>
        <p class="text-xs text-zinc-500">
          Removes the Wine prefix, downloaded client, settings and URI handler.
        </p>
      </div>
      <button
        class="rounded-lg border border-danger/50 px-3.5 py-1.5 text-xs text-danger transition-colors hover:bg-danger/10"
        onclick={() => (confirmUninstall = true)}
      >
        Uninstall…
      </button>
    </section>
  {/if}
</div>

<Modal
  title="Uninstall Riko?"
  open={confirmUninstall}
  onclose={() => (confirmUninstall = false)}
>
  <p class="text-sm leading-relaxed text-zinc-400">
    This permanently deletes the Wine prefix, the downloaded Vortex client, your
    settings and the saved session. The launcher will close afterwards.
  </p>
  <div class="mt-5 flex justify-end gap-3">
    <button
      class="rounded-lg border border-edge px-4 py-2 text-sm text-zinc-300 transition-colors hover:bg-panel-hover"
      onclick={() => (confirmUninstall = false)}
    >
      Cancel
    </button>
    <button
      class="rounded-lg bg-danger px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-danger/80 disabled:opacity-50"
      disabled={uninstallBusy}
      onclick={handleUninstall}
    >
      {uninstallBusy ? "Removing…" : "Uninstall everything"}
    </button>
  </div>
</Modal>
