<script lang="ts">
  import {
    getConfig,
    installWineVersion,
    listAccounts,
    listWineVersions,
    logout,
    removeAccount,
    removeWineVersion,
    switchAccount,
    updateConfig,
    uninstallRiko,
    type AccountView,
    type ConfigPatch,
    type ConfigView,
    type WineVersions,
  } from "../lib/api";
  import { progressState, resetProgress } from "../lib/stores/progress.svelte";
  import ProgressBar from "../lib/components/ProgressBar.svelte";
  import { navigate } from "../lib/router.svelte";
  import { appState, refreshStatus } from "../lib/stores/app.svelte";
  import { toast } from "../lib/stores/toast.svelte";
  import Toggle from "../lib/components/Toggle.svelte";
  import Modal from "../lib/components/Modal.svelte";

  let cfg = $state<ConfigView | null>(null);
  let accounts = $state<AccountView[]>([]);
  let switching = $state<string | null>(null);
  let wineBinary = $state("");
  let envRows = $state<{ key: string; value: string }[]>([]);
  let confirmUninstall = $state(false);
  let uninstallBusy = $state(false);
  let wine = $state<WineVersions | null>(null);
  let wineBusy = $state<string | null>(null);
  let showAvailableWine = $state(false);

  const wineStage = $derived(progressState.stages["wine-install"]);

  $effect(() => {
    if (cfg === null) load();
  });

  async function load() {
    cfg = await getConfig();
    accounts = await listAccounts();
    wineBinary = cfg.wine_binary;
    envRows = Object.entries(cfg.wine_env).map(([key, value]) => ({ key, value }));
    listWineVersions()
      .then((w) => (wine = w))
      .catch(() => {});
  }

  async function installWine(url: string, name: string) {
    wineBusy = name;
    resetProgress();
    try {
      const installed = await installWineVersion(url);
      toast(`${installed.name} installed`, "success");
      wine = await listWineVersions();
    } catch (e) {
      toast(String(e), "error");
    } finally {
      wineBusy = null;
    }
  }

  async function useWine(binary: string) {
    await patch({ wine_binary: binary });
    wineBinary = binary;
    if (wine) wine.active_binary = binary;
    toast("Wine binary updated", "success");
  }

  async function removeWine(name: string) {
    try {
      await removeWineVersion(name);
      cfg = await getConfig();
      wineBinary = cfg.wine_binary;
      wine = await listWineVersions();
    } catch (e) {
      toast(String(e), "error");
    }
  }

  async function handleSwitch(username: string) {
    switching = username;
    try {
      accounts = await switchAccount(username);
      cfg = await getConfig();
      await refreshStatus();
      toast(`Switched to ${username}`, "success");
    } catch (e) {
      toast(String(e), "error");
    } finally {
      switching = null;
    }
  }

  async function handleRemove(username: string) {
    try {
      accounts = await removeAccount(username);
      cfg = await getConfig();
      await refreshStatus();
      if (!appState.status?.logged_in) navigate("/login");
    } catch (e) {
      toast(String(e), "error");
    }
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
    if (appState.status?.logged_in) {
      cfg = await getConfig();
      accounts = await listAccounts();
      toast(`Switched to ${appState.status.username}`, "success");
    } else {
      navigate("/login");
    }
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
      <div class="mb-2 flex items-center justify-between">
        <h2 class="text-sm font-semibold text-zinc-300 uppercase tracking-wide">
          Accounts
        </h2>
        <button
          class="rounded-lg border border-edge px-3 py-1.5 text-xs text-zinc-300 transition-colors hover:bg-panel-hover"
          onclick={() => navigate("/login")}
        >
          Add account
        </button>
      </div>
      {#each accounts as account (account.username)}
        <div class="flex items-center justify-between border-t border-edge/60 py-2.5 first:border-t-0">
          <div class="flex items-center gap-2.5">
            <span
              class="flex h-7 w-7 items-center justify-center rounded-full bg-accent/20 text-xs font-semibold text-accent uppercase"
            >
              {account.username.slice(0, 1)}
            </span>
            <div>
              <p class="text-sm text-zinc-200">{account.username}</p>
              {#if account.active}
                <p class="text-xs text-ok">Active</p>
              {/if}
            </div>
          </div>
          <div class="flex items-center gap-3">
            {#if account.active}
              <button
                class="rounded-lg border border-edge px-3 py-1.5 text-xs text-zinc-300 transition-colors hover:bg-panel-hover"
                onclick={handleLogout}
              >
                Log out
              </button>
            {:else}
              <button
                class="rounded-lg bg-accent px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
                disabled={switching !== null}
                onclick={() => handleSwitch(account.username)}
              >
                {switching === account.username ? "Switching…" : "Switch"}
              </button>
              <button
                class="text-xs text-zinc-500 transition-colors hover:text-danger"
                onclick={() => handleRemove(account.username)}
              >
                Remove
              </button>
            {/if}
          </div>
        </div>
      {:else}
        <p class="py-1 text-sm text-zinc-500">Not signed in</p>
      {/each}
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
      <Toggle
        checked={cfg.community_shaders}
        label="Download community shader caches"
        hint="Off by default. Before launch, fetch a checksum-verified precompiled shader cache matching your GPU so the first minutes don't stutter."
        onchange={(v) => patch({ community_shaders: v })}
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
      <Toggle
        checked={cfg.telemetry_enabled}
        label="Share anonymous usage & crash reports"
        hint="Off by default. Sends a random install ID, app version, and OS — plus crash messages if the app panics. Never your account, games, or logs."
        onchange={(v) => patch({ telemetry_enabled: v })}
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

    <section class="flex flex-col gap-3 rounded-xl border border-edge bg-panel px-5 py-4">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold text-zinc-300 uppercase tracking-wide">
          Wine builds
        </h2>
        <button
          class="rounded-lg border border-edge px-3 py-1.5 text-xs text-zinc-300 transition-colors hover:bg-panel-hover"
          onclick={() => (showAvailableWine = !showAvailableWine)}
        >
          {showAvailableWine ? "Hide catalog" : "Browse builds"}
        </button>
      </div>

      {#each wine?.installed ?? [] as build (build.name)}
        <div class="flex items-center justify-between border-t border-edge/60 py-2 first:border-t-0">
          <div>
            <p class="text-sm text-zinc-200">{build.name}</p>
            {#if wine?.active_binary === build.wine_binary}
              <p class="text-xs text-ok">Active</p>
            {/if}
          </div>
          <div class="flex items-center gap-3">
            {#if wine?.active_binary !== build.wine_binary}
              <button
                class="rounded-lg bg-accent px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-accent-hover"
                onclick={() => useWine(build.wine_binary)}
              >
                Use
              </button>
            {:else}
              <button
                class="rounded-lg border border-edge px-3 py-1.5 text-xs text-zinc-300 transition-colors hover:bg-panel-hover"
                onclick={() => useWine("wine")}
              >
                Use system wine
              </button>
            {/if}
            <button
              class="text-xs text-zinc-500 transition-colors hover:text-danger"
              onclick={() => removeWine(build.name)}
            >
              Remove
            </button>
          </div>
        </div>
      {:else}
        <p class="text-sm text-zinc-500">
          No downloaded builds; using <code class="text-zinc-400">{cfg.wine_binary}</code>.
        </p>
      {/each}

      {#if wineBusy && wineStage && !wineStage.finished}
        <ProgressBar done={wineStage.done} total={wineStage.total} />
      {/if}

      {#if showAvailableWine}
        <div class="flex flex-col divide-y divide-edge/60 rounded-lg border border-edge/60 px-3">
          {#each (wine?.available ?? []).filter((a) => !wine?.installed.some((b) => b.name === a.name)) as build (build.name)}
            <div class="flex items-center justify-between py-2">
              <div>
                <p class="text-sm text-zinc-200">{build.name}</p>
                <p class="text-xs text-zinc-500">{build.size_mb} MB</p>
              </div>
              <button
                class="rounded-lg bg-accent px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
                disabled={wineBusy !== null}
                onclick={() => installWine(build.download_url, build.name)}
              >
                {wineBusy === build.name ? "Installing…" : "Install"}
              </button>
            </div>
          {:else}
            <p class="py-2 text-sm text-zinc-500">Could not load the build catalog.</p>
          {/each}
        </div>
      {/if}
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
