<script lang="ts">
  import { getPlaytime, getSessions, type PlaytimeEntry, type SessionRecord } from "../lib/api";
  import { gamesState, loadGames } from "../lib/stores/games.svelte";
  import { formatPlaytime } from "../lib/stores/playtime.svelte";
  import { navigate } from "../lib/router.svelte";

  const DAYS_SHOWN = 14;

  let entries = $state<Record<number, PlaytimeEntry>>({});
  let sessions = $state<SessionRecord[]>([]);
  let loaded = $state(false);

  const gameName = (id: number) =>
    gamesState.games.find((g) => g.id === id)?.name ?? `Game #${id}`;

  const totalSecs = $derived(
    Object.values(entries).reduce((sum, e) => sum + e.total_secs, 0)
  );
  const totalLaunches = $derived(
    Object.values(entries).reduce((sum, e) => sum + e.launches, 0)
  );

  const topGames = $derived(
    Object.entries(entries)
      .map(([id, e]) => ({ id: Number(id), secs: e.total_secs }))
      .filter((g) => g.secs > 0)
      .sort((a, b) => b.secs - a.secs)
      .slice(0, 8)
  );
  const maxGameSecs = $derived(topGames[0]?.secs ?? 1);

  const days = $derived.by(() => {
    const buckets: { label: string; secs: number }[] = [];
    const today = new Date();
    for (let i = DAYS_SHOWN - 1; i >= 0; i--) {
      const day = new Date(today);
      day.setDate(today.getDate() - i);
      const key = day.toDateString();
      const secs = sessions
        .filter((s) => new Date(s.started_at).toDateString() === key)
        .reduce((sum, s) => sum + s.duration_secs, 0);
      buckets.push({
        label: day.toLocaleDateString(undefined, { weekday: "short" }).slice(0, 2),
        secs,
      });
    }
    return buckets;
  });
  const maxDaySecs = $derived(Math.max(...days.map((d) => d.secs), 1));

  const recentSessions = $derived([...sessions].reverse().slice(0, 12));

  $effect(() => {
    if (!gamesState.loaded && !gamesState.loading) loadGames(false);
    if (!loaded) {
      loaded = true;
      Promise.all([getPlaytime(), getSessions()]).then(([p, s]) => {
        entries = p;
        sessions = s;
      });
    }
  });

  function formatWhen(iso: string): string {
    const date = new Date(iso);
    const days = Math.floor((Date.now() - date.getTime()) / 86_400_000);
    if (days === 0) return date.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
    if (days === 1) return "Yesterday";
    return date.toLocaleDateString(undefined, { month: "short", day: "numeric" });
  }
</script>

<div class="flex flex-col gap-6 p-8">
  <div>
    <h1 class="text-2xl font-semibold tracking-tight text-white">Stats</h1>
    <p class="mt-1 text-sm text-zinc-500">Your playtime across all games on this machine.</p>
  </div>

  <div class="grid grid-cols-3 gap-4">
    <div class="rounded-xl border border-edge bg-panel px-5 py-4">
      <p class="text-xs text-zinc-500 uppercase tracking-wide">Total playtime</p>
      <p class="mt-1 text-2xl font-semibold text-white tabular-nums">
        {totalSecs > 0 ? formatPlaytime(totalSecs) : "-"}
      </p>
    </div>
    <div class="rounded-xl border border-edge bg-panel px-5 py-4">
      <p class="text-xs text-zinc-500 uppercase tracking-wide">Launches</p>
      <p class="mt-1 text-2xl font-semibold text-white tabular-nums">{totalLaunches}</p>
    </div>
    <div class="rounded-xl border border-edge bg-panel px-5 py-4">
      <p class="text-xs text-zinc-500 uppercase tracking-wide">Games played</p>
      <p class="mt-1 text-2xl font-semibold text-white tabular-nums">{topGames.length}</p>
    </div>
  </div>

  <div class="rounded-xl border border-edge bg-panel px-5 py-4">
    <h2 class="mb-3 text-sm font-semibold text-zinc-300 uppercase tracking-wide">
      Last {DAYS_SHOWN} days
    </h2>
    <div class="flex h-28 items-end gap-1.5">
      {#each days as day, i (i)}
        <div class="group flex flex-1 flex-col items-center gap-1.5" title={formatPlaytime(day.secs)}>
          <div
            class="w-full rounded-t {day.secs > 0 ? 'bg-accent/70 group-hover:bg-accent' : 'bg-edge'}"
            style="height: {day.secs > 0 ? Math.max((day.secs / maxDaySecs) * 88, 4) : 2}px"
          ></div>
          <span class="text-[10px] text-zinc-600">{day.label}</span>
        </div>
      {/each}
    </div>
  </div>

  <div class="grid grid-cols-2 gap-4">
    <div class="rounded-xl border border-edge bg-panel px-5 py-4">
      <h2 class="mb-3 text-sm font-semibold text-zinc-300 uppercase tracking-wide">Top games</h2>
      {#each topGames as game (game.id)}
        <button
          class="group flex w-full flex-col gap-1 py-1.5 text-left"
          onclick={() => navigate(`/game/${game.id}`)}
        >
          <div class="flex items-center justify-between text-sm">
            <span class="truncate text-zinc-300 group-hover:text-white">{gameName(game.id)}</span>
            <span class="ml-3 shrink-0 text-xs text-zinc-500 tabular-nums">
              {formatPlaytime(game.secs)}
            </span>
          </div>
          <div class="h-1.5 w-full overflow-hidden rounded-full bg-edge">
            <div
              class="h-full rounded-full bg-accent/70 group-hover:bg-accent"
              style="width: {(game.secs / maxGameSecs) * 100}%"
            ></div>
          </div>
        </button>
      {:else}
        <p class="py-2 text-sm text-zinc-500">Play something and it'll show up here.</p>
      {/each}
    </div>

    <div class="rounded-xl border border-edge bg-panel px-5 py-4">
      <h2 class="mb-3 text-sm font-semibold text-zinc-300 uppercase tracking-wide">
        Recent sessions
      </h2>
      <div class="flex flex-col divide-y divide-edge/60">
        {#each recentSessions as session, i (i)}
          <div class="flex items-center justify-between py-2 text-sm">
            <span class="truncate text-zinc-300">{gameName(session.game_id)}</span>
            <span class="ml-3 flex shrink-0 gap-3 text-xs text-zinc-500 tabular-nums">
              <span>{formatPlaytime(session.duration_secs)}</span>
              <span class="w-16 text-right">{formatWhen(session.started_at)}</span>
            </span>
          </div>
        {:else}
          <p class="py-2 text-sm text-zinc-500">No sessions recorded yet.</p>
        {/each}
      </div>
    </div>
  </div>
</div>
