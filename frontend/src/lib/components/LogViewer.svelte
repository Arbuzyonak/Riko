<script lang="ts">
  import type { LogEntry } from "../stores/session.svelte";

  let { entries }: { entries: LogEntry[] } = $props();

  let container: HTMLDivElement | undefined = $state();
  let autoScroll = $state(true);

  $effect(() => {
    void entries.length;
    if (autoScroll && container) {
      container.scrollTop = container.scrollHeight;
    }
  });

  function onScroll() {
    if (!container) return;
    autoScroll =
      container.scrollTop + container.clientHeight >= container.scrollHeight - 24;
  }
</script>

<div
  bind:this={container}
  onscroll={onScroll}
  class="h-64 overflow-y-auto rounded-lg border border-edge bg-black/40 p-3 font-mono text-xs leading-relaxed"
>
  {#if entries.length === 0}
    <p class="text-zinc-600">No output yet.</p>
  {:else}
    {#each entries as entry, i (i)}
      <div class={entry.isStderr ? "text-warn/80" : "text-zinc-400"}>
        {entry.line}
      </div>
    {/each}
  {/if}
</div>
