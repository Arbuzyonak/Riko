<script lang="ts">
  import {
    getGamePluginOverrides,
    launchGame,
    listPlugins,
    setPluginEnabled,
    stopGame,
    type PerGamePlugins,
    type PluginInfo,
  } from "../lib/api";
  import { navigate, routeParam } from "../lib/router.svelte";
  import { gamesState, loadGames } from "../lib/stores/games.svelte";
  import { formatPlaytime, loadPlaytime, playtimeState } from "../lib/stores/playtime.svelte";
  import { formatVisits, loadStats, statsState } from "../lib/stores/stats.svelte";
  import { sessionState, isRunning } from "../lib/stores/session.svelte";
  import { toast } from "../lib/stores/toast.svelte";
  import LogViewer from "../lib/components/LogViewer.svelte";

  const gameId = $derived(Number(routeParam("/game/") ?? 0));
  const game = $derived(gamesState.games.find((g) => g.id === gameId));
  const running = $derived(isRunning(gameId));
  const logs = $derived(sessionState.logs[gameId] ?? []);
  const hue = $derived((gameId * 137) % 360);
  const playtime = $derived(playtimeState.entries[gameId]);
  const stats = $derived(statsState.entries[gameId]);

  let busy = $state(false);
  let showLogs = $state(false);
  let heroBroken = $state(false);
  let plugins = $state<PluginInfo[]>([]);
  let overrides = $state<PerGamePlugins>({ enabled: [], disabled: [] });
  let pluginsLoadedFor = $state(0);

  const usablePlugins = $derived(
    plugins.filter((p) => p.installed && p.built && p.supported)
  );

  type PluginMode = "inherit" | "on" | "off";

  function modeOf(name: string): PluginMode {
    if (overrides.enabled.includes(name)) return "on";
    if (overrides.disabled.includes(name)) return "off";
    return "inherit";
  }

  async function loadPlugins() {
    try {
      plugins = await listPlugins();
      overrides = await getGamePluginOverrides(gameId);
    } catch (e) {
      toast(String(e), "error");
    }
  }

  async function setMode(name: string, mode: PluginMode) {
    const enabled = mode === "on" ? true : mode === "off" ? false : null;
    try {
      await setPluginEnabled(name, gameId, enabled);
      overrides = await getGamePluginOverrides(gameId);
    } catch (e) {
      toast(String(e), "error");
    }
  }

  $effect(() => {
    if (!gamesState.loaded && !gamesState.loading) {
      loadGames(false);
    }
  });

  $effect(() => {
    if (gameId && pluginsLoadedFor !== gameId) {
      pluginsLoadedFor = gameId;
      loadPlugins();
      loadStats();
    }
  });

  $effect(() => {
    if (running) showLogs = true;
    else loadPlaytime(true);
  });

  async function play() {
    busy = true;
    try {
      await launchGame(gameId);
    } catch (e) {
      toast(String(e), "error");
    } finally {
      busy = false;
    }
  }

  async function stop() {
    busy = true;
    try {
      await stopGame(gameId);
    } catch (e) {
      toast(String(e), "error");
    } finally {
      busy = false;
    }
  }
</script>

<div class="flex flex-col">
  <div
    class="relative flex h-56 items-end overflow-hidden px-8 pb-6"
    style="background: linear-gradient(160deg, hsl({hue}, 45%, 24%), hsl({(hue + 40) %
      360}, 50%, 10%) 70%)"
  >
    {#if game?.thumbnail_url && !heroBroken}
      <img
        src={game.thumbnail_url}
        alt=""
        class="absolute inset-0 h-full w-full object-cover opacity-30 blur-sm"
        onerror={() => (heroBroken = true)}
      />
    {/if}
    <button
      class="absolute top-5 left-6 flex items-center gap-1.5 text-sm text-zinc-300 transition-colors hover:text-white"
      onclick={() => navigate("/")}
    >
      <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 19.5L8.25 12l7.5-7.5" />
      </svg>
      Library
    </button>
    <div class="relative flex w-full items-end justify-between gap-6">
      <div>
        <h1 class="text-3xl font-bold tracking-tight text-white">
          {game?.name ?? `Game #${gameId}`}
        </h1>
        <p class="mt-1 text-sm text-zinc-300">
          {game?.creator ? `by ${game.creator}` : `Game #${gameId}`}
          {#if stats}
            <span class="ml-2 text-zinc-400 tabular-nums">
              {formatVisits(stats.visits)} {stats.visits === 1 ? "visit" : "visits"}
            </span>
            {#if stats.active > 0}
              <span class="ml-2 inline-flex items-center gap-1.5 text-ok">
                <span class="h-1.5 w-1.5 rounded-full bg-ok"></span>
                {stats.active} playing
              </span>
            {/if}
          {/if}
          {#if playtime && playtime.total_secs > 0}
            <span class="ml-2 text-zinc-400">
              {formatPlaytime(playtime.total_secs)} played · {playtime.launches}
              {playtime.launches === 1 ? "launch" : "launches"}
            </span>
          {/if}
          {#if running}
            <span class="ml-2 inline-flex items-center gap-1.5 text-ok">
              <span class="h-2 w-2 animate-pulse rounded-full bg-ok"></span>
              Playing
            </span>
          {/if}
        </p>
      </div>
      {#if running}
        <div class="flex items-center gap-3">
          <span
            class="flex items-center gap-2 rounded-lg bg-ok/15 px-6 py-2.5 text-sm font-semibold text-ok"
          >
            <span class="h-2 w-2 animate-pulse rounded-full bg-ok"></span>
            Playing
          </span>
          <button
            class="rounded-lg bg-danger/90 px-6 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-danger disabled:opacity-50"
            disabled={busy}
            onclick={stop}
          >
            Stop
          </button>
        </div>
      {:else}
        <button
          class="flex items-center gap-2 rounded-lg bg-accent px-6 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
          disabled={busy}
          onclick={play}
        >
          <svg class="h-4 w-4" fill="currentColor" viewBox="0 0 24 24">
            <path d="M8 5.14v14l11-7-11-7z" />
          </svg>
          {busy ? "Launching…" : "Play"}
        </button>
      {/if}
    </div>
  </div>

  <div class="flex flex-col gap-6 p-8">
    {#if game?.description}
      <p class="max-w-2xl text-sm leading-relaxed text-zinc-400">{game.description}</p>
    {/if}

    {#if usablePlugins.length > 0}
      <div class="flex max-w-2xl flex-col gap-3">
        <h2 class="text-sm font-medium text-zinc-200">Plugins for this game</h2>
        <div class="flex flex-col divide-y divide-edge rounded-xl border border-edge bg-panel px-5">
          {#each usablePlugins as plugin (plugin.name)}
            {@const mode = modeOf(plugin.name)}
            <div class="flex items-center justify-between gap-4 py-3">
              <div class="min-w-0">
                <span class="text-sm text-zinc-200">{plugin.name}</span>
                <p class="text-xs text-zinc-500">
                  {mode === "inherit"
                    ? `Following global setting (${plugin.enabled ? "on" : "off"})`
                    : mode === "on"
                      ? "Forced on for this game"
                      : "Forced off for this game"}
                </p>
              </div>
              <div class="flex shrink-0 overflow-hidden rounded-lg border border-edge text-xs">
                {#each ["inherit", "on", "off"] as const as option (option)}
                  <button
                    class="px-3 py-1.5 capitalize transition-colors {mode === option
                      ? 'bg-accent text-white'
                      : 'text-zinc-400 hover:bg-panel-hover'}"
                    onclick={() => setMode(plugin.name, option)}
                  >
                    {option}
                  </button>
                {/each}
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <div class="flex flex-col gap-2">
      <button
        class="flex w-fit items-center gap-2 text-sm text-zinc-400 transition-colors hover:text-zinc-200"
        onclick={() => (showLogs = !showLogs)}
      >
        <svg
          class="h-4 w-4 transition-transform {showLogs ? 'rotate-90' : ''}"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="2"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
        </svg>
        Game output
      </button>
      {#if showLogs}
        <LogViewer entries={logs} />
      {/if}
    </div>
  </div>
</div>
