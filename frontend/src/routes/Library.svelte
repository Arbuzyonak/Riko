<script lang="ts">
  import { gamesState, loadGames } from "../lib/stores/games.svelte";
  import { loadPlaytime } from "../lib/stores/playtime.svelte";
  import { statsState, loadStats } from "../lib/stores/stats.svelte";
  import GameCard from "../lib/components/GameCard.svelte";

  const sortedGames = $derived(
    [...gamesState.games].sort(
      (a, b) =>
        (statsState.entries[b.id]?.visits ?? 0) - (statsState.entries[a.id]?.visits ?? 0)
    )
  );

  $effect(() => {
    if (!gamesState.loaded && !gamesState.loading) {
      loadGames(false);
    }
  });

  $effect(() => {
    loadPlaytime(true);
    loadStats();
  });
</script>

<div class="flex flex-col gap-6 p-8">
  <div class="flex items-center justify-between">
    <h1 class="text-2xl font-semibold tracking-tight text-white">Library</h1>
    <button
      class="flex items-center gap-2 rounded-lg border border-edge bg-panel px-3.5 py-2 text-sm text-zinc-300 transition-colors hover:bg-panel-hover disabled:opacity-50"
      disabled={gamesState.loading}
      onclick={() => loadGames(true)}
    >
      <svg
        class="h-4 w-4 {gamesState.loading ? 'animate-spin' : ''}"
        fill="none"
        viewBox="0 0 24 24"
        stroke-width="1.8"
        stroke="currentColor"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182m0-4.991v4.99"
        />
      </svg>
      Refresh
    </button>
  </div>

  {#if gamesState.error}
    <div class="rounded-xl border border-danger/40 bg-danger/10 px-5 py-4 text-sm">
      <p class="font-medium text-danger">Could not load games</p>
      <p class="mt-1 text-zinc-400">{gamesState.error}</p>
    </div>
  {:else if gamesState.loading && gamesState.games.length === 0}
    <div class="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-4">
      {#each Array(8) as _, i (i)}
        <div class="aspect-[4/3.4] animate-pulse rounded-xl bg-panel"></div>
      {/each}
    </div>
  {:else if gamesState.games.length === 0}
    <div
      class="flex flex-col items-center gap-2 rounded-xl border border-dashed border-edge py-20 text-center"
    >
      <p class="text-zinc-300">No games found for your account</p>
      <p class="text-sm text-zinc-500">Games you can play will show up here.</p>
    </div>
  {:else}
    <div class="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-4">
      {#each sortedGames as game (game.id)}
        <GameCard {game} />
      {/each}
    </div>
  {/if}
</div>
