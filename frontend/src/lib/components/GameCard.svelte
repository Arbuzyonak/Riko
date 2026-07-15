<script lang="ts">
  import type { Game } from "../api";
  import { launchGame } from "../api";
  import { navigate } from "../router.svelte";
  import { isRunning } from "../stores/session.svelte";
  import { toast } from "../stores/toast.svelte";

  let { game }: { game: Game } = $props();

  const hue = $derived((game.id * 137) % 360);
  const running = $derived(isRunning(game.id));

  async function play(event: MouseEvent) {
    event.stopPropagation();
    try {
      await launchGame(game.id);
      navigate(`/game/${game.id}`);
    } catch (e) {
      toast(String(e), "error");
    }
  }
</script>

<div
  class="group relative flex cursor-pointer flex-col overflow-hidden rounded-xl border border-edge bg-panel text-left transition-all hover:-translate-y-0.5 hover:border-accent/60"
  role="button"
  tabindex="0"
  onclick={() => navigate(`/game/${game.id}`)}
  onkeydown={(e) => e.key === "Enter" && navigate(`/game/${game.id}`)}
>
  <div
    class="relative flex aspect-video w-full items-center justify-center overflow-hidden"
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
    {#if !running}
      <button
        class="absolute right-3 bottom-3 flex h-10 w-10 items-center justify-center rounded-full bg-accent text-white opacity-0 shadow-lg transition-all group-hover:opacity-100 hover:bg-accent-hover"
        onclick={play}
        aria-label="Play {game.name}"
      >
        <svg class="ml-0.5 h-4 w-4" fill="currentColor" viewBox="0 0 24 24">
          <path d="M8 5.14v14l11-7-11-7z" />
        </svg>
      </button>
    {/if}
  </div>
  {#if running}
    <span
      class="absolute top-3 left-3 flex items-center gap-1.5 rounded-full bg-black/60 px-2.5 py-1 text-xs font-medium text-ok backdrop-blur"
    >
      <span class="h-1.5 w-1.5 animate-pulse rounded-full bg-ok"></span>
      Running
    </span>
  {/if}
  <div class="flex flex-col gap-0.5 px-4 py-3">
    <span class="truncate font-medium text-white">{game.name}</span>
    <span class="truncate text-xs text-zinc-500">
      {game.creator ?? `Game #${game.id}`}
    </span>
  </div>
</div>
