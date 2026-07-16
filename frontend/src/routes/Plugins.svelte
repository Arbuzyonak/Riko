<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    importPlugin,
    installPlugin,
    listPlugins,
    removePlugin,
    setPluginEnabled,
    listMarketplace,
    installMarketplacePlugin,
    type PluginInfo,
    type MarketplaceEntry,
  } from "../lib/api";
  import { progressState, resetProgress } from "../lib/stores/progress.svelte";
  import { toast } from "../lib/stores/toast.svelte";
  import Modal from "../lib/components/Modal.svelte";
  import Toggle from "../lib/components/Toggle.svelte";

  let plugins = $state<PluginInfo[] | null>(null);
  let busyPlugin = $state<string | null>(null);
  let confirmInstall = $state<PluginInfo | null>(null);
  let confirmRemove = $state<PluginInfo | null>(null);

  let market = $state<MarketplaceEntry[] | null>(null);
  let marketError = $state<string | null>(null);
  let marketLoading = $state(false);
  let confirmMarket = $state<MarketplaceEntry | null>(null);
  let activeTab = $state<"installed" | "marketplace">("installed");

  function selectTab(tab: "installed" | "marketplace") {
    activeTab = tab;
    if (tab === "marketplace" && market === null && !marketLoading) loadMarket();
  }

  async function loadMarket() {
    marketLoading = true;
    marketError = null;
    try {
      market = await listMarketplace();
    } catch (e) {
      marketError = String(e);
      market = [];
    } finally {
      marketLoading = false;
    }
  }

  async function doMarketInstall(entry: MarketplaceEntry) {
    confirmMarket = null;
    busyPlugin = entry.name;
    resetProgress();
    try {
      await installMarketplacePlugin(entry.name);
      toast(`${entry.name} installed from the marketplace`, "success");
    } catch (e) {
      toast(String(e), "error");
    } finally {
      busyPlugin = null;
      await refresh();
      await loadMarket();
    }
  }

  const buildStage = $derived(progressState.stages["plugin"]);

  $effect(() => {
    if (plugins === null) refresh();
  });

  async function refresh() {
    plugins = await listPlugins();
  }

  async function doInstall(plugin: PluginInfo) {
    confirmInstall = null;
    busyPlugin = plugin.name;
    resetProgress();
    try {
      await installPlugin(plugin.name);
      toast(`${plugin.name} installed`, "success");
    } catch (e) {
      toast(String(e), "error");
    } finally {
      busyPlugin = null;
      await refresh();
    }
  }

  async function doRemove(plugin: PluginInfo) {
    confirmRemove = null;
    try {
      await removePlugin(plugin.name);
      toast(`${plugin.name} removed`, "success");
    } catch (e) {
      toast(String(e), "error");
    }
    await refresh();
  }

  async function toggleEnabled(plugin: PluginInfo, enabled: boolean) {
    try {
      plugins = await setPluginEnabled(plugin.name, null, enabled);
    } catch (e) {
      toast(String(e), "error");
    }
  }

  async function importFolder() {
    const path = await open({ directory: true, title: "Select a plugin folder" });
    if (typeof path !== "string") return;
    try {
      const info = await importPlugin(path);
      toast(`Imported ${info.name}; review the build command before installing`, "success");
    } catch (e) {
      toast(String(e), "error");
    }
    await refresh();
  }
</script>

<div class="flex flex-col gap-6 p-8">
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-semibold tracking-tight text-white">Plugins</h1>
      <p class="mt-1 text-sm text-zinc-500">
        Enhancements injected when a game launches. Only install plugins you trust —
        building runs their commands on your machine.
      </p>
    </div>
    {#if activeTab === "installed"}
      <button
        class="shrink-0 rounded-lg border border-edge bg-panel px-3.5 py-2 text-sm text-zinc-300 transition-colors hover:bg-panel-hover"
        onclick={importFolder}
      >
        Import folder…
      </button>
    {:else}
      <button
        class="shrink-0 rounded-lg border border-edge bg-panel px-3.5 py-2 text-sm text-zinc-300 transition-colors hover:bg-panel-hover disabled:opacity-40"
        onclick={loadMarket}
        disabled={marketLoading}
      >
        Refresh
      </button>
    {/if}
  </div>

  <div class="flex gap-1 border-b border-edge">
    <button
      class="-mb-px border-b-2 px-3 py-2 text-sm font-medium transition-colors {activeTab ===
      'installed'
        ? 'border-accent text-white'
        : 'border-transparent text-zinc-500 hover:text-zinc-300'}"
      onclick={() => selectTab("installed")}
    >
      Installed
    </button>
    <button
      class="-mb-px border-b-2 px-3 py-2 text-sm font-medium transition-colors {activeTab ===
      'marketplace'
        ? 'border-accent text-white'
        : 'border-transparent text-zinc-500 hover:text-zinc-300'}"
      onclick={() => selectTab("marketplace")}
    >
      Marketplace
    </button>
  </div>

  {#if activeTab === "installed"}
  <div class="flex flex-col gap-4">
    {#each plugins ?? [] as plugin (plugin.name)}
      <div class="flex flex-col rounded-xl border border-edge bg-panel px-5 py-4">
        <div class="flex items-start justify-between gap-4">
          <div class="min-w-0">
            <div class="flex items-center gap-2">
              <h2 class="font-medium text-white">{plugin.name}</h2>
              <span class="text-xs text-zinc-500">v{plugin.version}</span>
              <span class="rounded bg-panel-hover px-1.5 py-0.5 text-[10px] text-zinc-400 uppercase">
                {plugin.kind}
              </span>
              {#if plugin.builtin}
                <span class="rounded bg-accent/15 px-1.5 py-0.5 text-[10px] text-accent uppercase">
                  built-in
                </span>
              {/if}
              {#if !plugin.supported}
                <span class="rounded bg-warn/15 px-1.5 py-0.5 text-[10px] text-warn uppercase">
                  not for this OS
                </span>
              {/if}
            </div>
            <p class="mt-1 text-sm text-zinc-400">{plugin.description}</p>
            {#if plugin.installed && !plugin.built}
              <p class="mt-1 text-xs text-warn">Imported but not built yet.</p>
            {/if}
            {#if plugin.missing_requirement}
              <div class="mt-1.5 flex items-center gap-2 text-xs text-warn">
                <span>Won't take effect — missing system package. Install it with:</span>
                <code
                  class="rounded bg-black/40 px-2 py-0.5 font-mono text-zinc-300 select-text"
                >
                  {plugin.missing_requirement}
                </code>
              </div>
            {/if}
          </div>
          <div class="flex shrink-0 items-center gap-3">
            {#if plugin.installed && plugin.built}
              <Toggle
                checked={plugin.enabled}
                label=""
                onchange={(v) => toggleEnabled(plugin, v)}
              />
              <button
                class="text-xs text-zinc-500 transition-colors hover:text-danger"
                onclick={() => (confirmRemove = plugin)}
              >
                Remove
              </button>
            {:else if busyPlugin === plugin.name}
              <div
                class="h-5 w-5 animate-spin rounded-full border-2 border-edge border-t-accent"
              ></div>
            {:else}
              <button
                class="rounded-lg bg-accent px-3.5 py-1.5 text-xs font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-40"
                disabled={!plugin.supported || busyPlugin !== null}
                onclick={() => (confirmInstall = plugin)}
              >
                {plugin.installed ? "Build" : "Install"}
              </button>
              {#if plugin.installed && !plugin.builtin}
                <button
                  class="text-xs text-zinc-500 transition-colors hover:text-danger"
                  onclick={() => (confirmRemove = plugin)}
                >
                  Remove
                </button>
              {/if}
            {/if}
          </div>
        </div>

        {#if busyPlugin === plugin.name && buildStage && buildStage.lines.length > 0}
          <div
            class="mt-3 max-h-32 overflow-y-auto rounded-lg bg-black/40 p-3 font-mono text-xs leading-relaxed"
          >
            {#each buildStage.lines.slice(-40) as entry, i (i)}
              <div class={entry.level === "warn" ? "text-warn/80" : "text-zinc-400"}>
                {entry.line}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </div>
  {:else}
  <p class="max-w-2xl text-sm text-zinc-500">
    Community plugins from the shared catalog. Each download is verified against a
    checksum before it's installed — but installing still runs its build command, so
    only add plugins you trust.
  </p>

  {#if marketLoading}
    <div class="flex justify-center py-6">
      <div class="h-5 w-5 animate-spin rounded-full border-2 border-edge border-t-accent"></div>
    </div>
  {:else if marketError}
    <p class="text-sm text-warn">Couldn't load the catalog: {marketError}</p>
  {:else if market !== null}
    {#if market.length === 0}
      <p class="text-sm text-zinc-500">The catalog is empty right now.</p>
    {:else}
      <div class="flex flex-col gap-4">
        {#each market as entry (entry.name)}
          <div class="flex items-start justify-between gap-4 rounded-xl border border-edge bg-panel px-5 py-4">
            <div class="min-w-0">
              <div class="flex items-center gap-2">
                <h3 class="font-medium text-white">{entry.name}</h3>
                <span class="text-xs text-zinc-500">v{entry.version}</span>
                <span class="rounded bg-panel-hover px-1.5 py-0.5 text-[10px] text-zinc-400 uppercase">
                  {entry.kind}
                </span>
                {#if entry.author}
                  <span class="text-xs text-zinc-500">by {entry.author}</span>
                {/if}
              </div>
              <p class="mt-1 text-sm text-zinc-400">{entry.description}</p>
              <p class="mt-1 text-xs text-zinc-600">
                {(entry.size_bytes / 1024).toFixed(0)} KB · sha256 verified on install
              </p>
            </div>
            <div class="flex shrink-0 items-center gap-3">
              {#if entry.installed}
                <span class="text-xs text-zinc-500">Installed</span>
              {:else if busyPlugin === entry.name}
                <div class="h-5 w-5 animate-spin rounded-full border-2 border-edge border-t-accent"></div>
              {:else}
                <button
                  class="rounded-lg bg-accent px-3.5 py-1.5 text-xs font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-40"
                  disabled={busyPlugin !== null}
                  onclick={() => (confirmMarket = entry)}
                >
                  Install
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
  {/if}
</div>

<Modal
  title="Install {confirmInstall?.name}?"
  open={confirmInstall !== null}
  onclose={() => (confirmInstall = null)}
>
  <p class="text-sm leading-relaxed text-zinc-400">
    Installing will run this build command on your machine:
  </p>
  <code
    class="mt-3 block rounded-lg bg-black/40 p-3 font-mono text-xs leading-relaxed break-all text-zinc-300 select-text"
  >
    {confirmInstall?.build_command ?? "(no build step)"}
  </code>
  <p class="mt-3 text-xs text-zinc-500">
    Only continue if you trust this plugin's source.
  </p>
  <div class="mt-5 flex justify-end gap-3">
    <button
      class="rounded-lg border border-edge px-4 py-2 text-sm text-zinc-300 transition-colors hover:bg-panel-hover"
      onclick={() => (confirmInstall = null)}
    >
      Cancel
    </button>
    <button
      class="rounded-lg bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover"
      onclick={() => confirmInstall && doInstall(confirmInstall)}
    >
      Install
    </button>
  </div>
</Modal>

<Modal
  title="Remove {confirmRemove?.name}?"
  open={confirmRemove !== null}
  onclose={() => (confirmRemove = null)}
>
  <p class="text-sm leading-relaxed text-zinc-400">
    This deletes the plugin's folder{confirmRemove?.builtin
      ? "; you can reinstall it from the catalog at any time"
      : ""}.
  </p>
  <div class="mt-5 flex justify-end gap-3">
    <button
      class="rounded-lg border border-edge px-4 py-2 text-sm text-zinc-300 transition-colors hover:bg-panel-hover"
      onclick={() => (confirmRemove = null)}
    >
      Cancel
    </button>
    <button
      class="rounded-lg bg-danger px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-danger/80"
      onclick={() => confirmRemove && doRemove(confirmRemove)}
    >
      Remove
    </button>
  </div>
</Modal>

<Modal
  title="Install {confirmMarket?.name} from the marketplace?"
  open={confirmMarket !== null}
  onclose={() => (confirmMarket = null)}
>
  <p class="text-sm leading-relaxed text-zinc-400">
    Riko will download this plugin, verify its checksum, then run its build command on
    your machine. Source:
  </p>
  <code
    class="mt-3 block rounded-lg bg-black/40 p-3 font-mono text-xs leading-relaxed break-all text-zinc-300 select-text"
  >
    {confirmMarket?.download_url}
  </code>
  <p class="mt-3 text-xs text-zinc-500">
    Only continue if you trust this plugin's author.
  </p>
  <div class="mt-5 flex justify-end gap-3">
    <button
      class="rounded-lg border border-edge px-4 py-2 text-sm text-zinc-300 transition-colors hover:bg-panel-hover"
      onclick={() => (confirmMarket = null)}
    >
      Cancel
    </button>
    <button
      class="rounded-lg bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover"
      onclick={() => confirmMarket && doMarketInstall(confirmMarket)}
    >
      Install
    </button>
  </div>
</Modal>
