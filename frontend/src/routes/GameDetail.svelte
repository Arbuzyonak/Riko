<script lang="ts">
  import {
    createShortcut,
    getGamePluginOverrides,
    getLaunchOverrides,
    launchGame,
    listAccounts,
    listPlugins,
    setLaunchOverrides,
    setPluginEnabled,
    stopGame,
    type AccountView,
    type LaunchOverrides,
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
  let accounts = $state<AccountView[]>([]);
  let showAccountMenu = $state(false);
  let showLaunchOptions = $state(false);
  let overridesDraft = $state<LaunchOverrides>({
    wine_binary: null,
    use_esync: null,
    use_fsync: null,
    use_gamemode: null,
    env: {},
  });
  let envRows = $state<{ key: string; value: string }[]>([]);
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
      loadOverrides();
      listAccounts()
        .then((a) => (accounts = a))
        .catch(() => {});
    }
  });

  $effect(() => {
    if (running) showLogs = true;
    else loadPlaytime(true);
  });

  async function play(username?: string) {
    showAccountMenu = false;
    busy = true;
    try {
      await launchGame(gameId, username);
      if (username) toast(`Launched as ${username}`, "success");
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

  type ToggleKey = "use_esync" | "use_fsync" | "use_gamemode";
  const toggleRows: { key: ToggleKey; label: string }[] = [
    { key: "use_esync", label: "esync" },
    { key: "use_fsync", label: "fsync" },
    { key: "use_gamemode", label: "GameMode" },
  ];

  async function loadOverrides() {
    try {
      overridesDraft = await getLaunchOverrides(gameId);
      envRows = Object.entries(overridesDraft.env).map(([key, value]) => ({ key, value }));
    } catch (e) {
      toast(String(e), "error");
    }
  }

  async function saveOverrides() {
    const env: Record<string, string> = {};
    for (const row of envRows) {
      if (row.key.trim()) env[row.key.trim()] = row.value;
    }
    try {
      await setLaunchOverrides(gameId, { ...overridesDraft, env });
      toast("Launch options saved", "success");
    } catch (e) {
      toast(String(e), "error");
    }
  }

  async function addShortcut() {
    try {
      await createShortcut(gameId);
      toast("Shortcut added to your app menu", "success");
    } catch (e) {
      toast(String(e), "error");
    }
  }

  async function copyInvite() {
    const link = `riko://join?game=${gameId}`;
    try {
      await navigator.clipboard.writeText(link);
      toast("Invite link copied - anyone with Riko can open it to join", "success");
    } catch {
      toast(link, "info");
    }
  }
</script>

<div class="flex flex-col">
  <div
    class="relative flex h-56 items-end px-8 pb-6"
    style="background: linear-gradient(160deg, hsl({hue}, 45%, 24%), hsl({(hue + 40) %
      360}, 50%, 10%) 70%)"
  >
    {#if game?.thumbnail_url && !heroBroken}
      <div class="absolute inset-0 overflow-hidden">
        <img
          src={game.thumbnail_url}
          alt=""
          class="h-full w-full object-cover opacity-30 blur-sm"
          onerror={() => (heroBroken = true)}
        />
      </div>
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
        <div class="relative flex">
          <button
            class="flex items-center gap-2 rounded-lg bg-accent px-6 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-accent-hover disabled:opacity-50 {accounts.length >
            1
              ? 'rounded-r-none'
              : ''}"
            disabled={busy}
            onclick={() => play()}
          >
            <svg class="h-4 w-4" fill="currentColor" viewBox="0 0 24 24">
              <path d="M8 5.14v14l11-7-11-7z" />
            </svg>
            {busy ? "Launching…" : "Play"}
          </button>
          {#if accounts.length > 1}
            <button
              class="flex items-center rounded-r-lg border-l border-white/20 bg-accent px-2 text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
              disabled={busy}
              aria-label="Play as another account"
              onclick={() => (showAccountMenu = !showAccountMenu)}
            >
              <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 8.25l-7.5 7.5-7.5-7.5" />
              </svg>
            </button>
            {#if showAccountMenu}
              <div
                class="absolute top-full right-0 z-30 mt-1.5 flex w-48 flex-col overflow-hidden rounded-lg border border-edge bg-panel py-1 shadow-xl"
              >
                {#each accounts as account (account.username)}
                  <button
                    class="px-4 py-2 text-left text-sm text-zinc-300 transition-colors hover:bg-panel-hover"
                    onclick={() => play(account.username)}
                  >
                    Play as {account.username}
                    {#if account.active}
                      <span class="text-xs text-zinc-500">(current)</span>
                    {/if}
                  </button>
                {/each}
              </div>
            {/if}
          {/if}
        </div>
      {/if}
    </div>
  </div>

  <div class="flex flex-col gap-6 p-8">
    {#if game?.description}
      <p class="max-w-2xl text-sm leading-relaxed text-zinc-400">{game.description}</p>
    {/if}

    <div class="flex flex-wrap items-center gap-5">
      <button
        class="flex w-fit items-center gap-2 text-sm text-zinc-400 transition-colors hover:text-zinc-200"
        onclick={addShortcut}
      >
        <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke-width="1.8" stroke="currentColor">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M13.19 8.688a4.5 4.5 0 011.242 7.244l-4.5 4.5a4.5 4.5 0 01-6.364-6.364l1.757-1.757m13.35-.622l1.757-1.757a4.5 4.5 0 00-6.364-6.364l-4.5 4.5a4.5 4.5 0 001.242 7.244"
          />
        </svg>
        Create desktop shortcut
      </button>
      <button
        class="flex w-fit items-center gap-2 text-sm text-zinc-400 transition-colors hover:text-zinc-200"
        onclick={copyInvite}
      >
        <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke-width="1.8" stroke="currentColor">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M7.217 10.907a2.25 2.25 0 100 2.186m0-2.186c.18.324.283.696.283 1.093s-.103.77-.283 1.093m0-2.186l9.566-5.314m-9.566 7.5l9.566 5.314m0 0a2.25 2.25 0 103.935 2.186 2.25 2.25 0 00-3.935-2.186zm0-12.814a2.25 2.25 0 103.933-2.185 2.25 2.25 0 00-3.933 2.185z"
          />
        </svg>
        Copy invite link
      </button>
    </div>

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
        onclick={() => (showLaunchOptions = !showLaunchOptions)}
      >
        <svg
          class="h-4 w-4 transition-transform {showLaunchOptions ? 'rotate-90' : ''}"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="2"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
        </svg>
        Launch options
      </button>
      {#if showLaunchOptions}
        <div class="flex max-w-2xl flex-col gap-4 rounded-xl border border-edge bg-panel px-5 py-4">
          <label class="flex flex-col gap-1.5">
            <span class="text-xs font-medium text-zinc-400">Wine binary for this game</span>
            <input
              type="text"
              placeholder="global default"
              value={overridesDraft.wine_binary ?? ""}
              oninput={(e) =>
                (overridesDraft.wine_binary = e.currentTarget.value.trim() || null)}
              class="rounded-lg border border-edge bg-surface px-3 py-2 text-sm text-white outline-none transition-colors focus:border-accent"
            />
          </label>

          {#each toggleRows as row (row.key)}
            <div class="flex items-center justify-between">
              <span class="text-sm text-zinc-200">{row.label}</span>
              <div class="flex overflow-hidden rounded-lg border border-edge text-xs">
                {#each [null, true, false] as option, i (i)}
                  <button
                    class="px-3 py-1.5 transition-colors {overridesDraft[row.key] === option
                      ? 'bg-accent text-white'
                      : 'text-zinc-400 hover:bg-panel-hover'}"
                    onclick={() => (overridesDraft[row.key] = option)}
                  >
                    {option === null ? "Inherit" : option ? "On" : "Off"}
                  </button>
                {/each}
              </div>
            </div>
          {/each}

          <div class="flex flex-col gap-2">
            <span class="text-xs font-medium text-zinc-400">Extra environment variables</span>
            {#each envRows as row, i (i)}
              <div class="flex gap-2">
                <input
                  type="text"
                  placeholder="KEY"
                  bind:value={row.key}
                  class="w-40 rounded-lg border border-edge bg-surface px-3 py-1.5 font-mono text-xs text-white outline-none focus:border-accent"
                />
                <input
                  type="text"
                  placeholder="value"
                  bind:value={row.value}
                  class="flex-1 rounded-lg border border-edge bg-surface px-3 py-1.5 font-mono text-xs text-white outline-none focus:border-accent"
                />
                <button
                  class="text-xs text-zinc-500 hover:text-danger"
                  onclick={() => (envRows = envRows.filter((_, idx) => idx !== i))}
                >
                  ✕
                </button>
              </div>
            {/each}
            <button
              class="w-fit text-xs text-zinc-500 transition-colors hover:text-zinc-300"
              onclick={() => (envRows = [...envRows, { key: "", value: "" }])}
            >
              + Add variable
            </button>
          </div>

          <button
            class="w-fit rounded-lg bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover"
            onclick={saveOverrides}
          >
            Save launch options
          </button>
        </div>
      {/if}
    </div>

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
