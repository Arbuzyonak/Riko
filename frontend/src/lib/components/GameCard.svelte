<script lang="ts">
  import type { Game } from "../api";
  import { navigate } from "../router.svelte";

  let { game }: { game: Game } = $props();

  const hue = $derived((game.id * 137) % 360);
</script>

<button
  class="group flex flex-col overflow-hidden rounded-xl border border-edge bg-panel text-left transition-all hover:-translate-y-0.5 hover:border-accent/60"
  onclick={() => navigate(`/game/${game.id}`)}
>
  <div
    class="flex aspect-video w-full items-center justify-center overflow-hidden"
    style="background: linear-gradient(135deg, hsl({hue}, 45%, 22%), hsl({(hue + 40) %
      360}, 50%, 12%))"
  >
    {#if game.thumbnail_url}
      <img
        src={game.thumbnail_url}
        alt={game.name}
        class="h-full w-full object-cover transition-transform group-hover:scale-105"
        loading="lazy"
      />
    {:else}
      <span class="text-4xl font-bold text-white/25">{game.name.slice(0, 1)}</span>
    {/if}
  </div>
  <div class="flex flex-col gap-0.5 px-4 py-3">
    <span class="truncate font-medium text-white">{game.name}</span>
    <span class="truncate text-xs text-zinc-500">
      {game.creator ?? `Game #${game.id}`}
    </span>
  </div>
</button>
