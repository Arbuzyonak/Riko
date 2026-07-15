<script lang="ts">
  let { done, total }: { done: number; total: number | null } = $props();

  const percent = $derived(total ? Math.min(100, (done / total) * 100) : null);

  function fmtBytes(n: number): string {
    if (n > 1_000_000) return `${(n / 1_000_000).toFixed(1)} MB`;
    if (n > 1_000) return `${(n / 1_000).toFixed(0)} KB`;
    return `${n} B`;
  }
</script>

<div class="flex items-center gap-3">
  <div class="h-2 flex-1 overflow-hidden rounded-full bg-edge">
    {#if percent !== null}
      <div
        class="h-full rounded-full bg-accent transition-[width] duration-200"
        style="width: {percent}%"
      ></div>
    {:else}
      <div class="h-full w-1/3 animate-pulse rounded-full bg-accent"></div>
    {/if}
  </div>
  <span class="w-28 text-right text-xs tabular-nums text-zinc-500">
    {fmtBytes(done)}{total ? ` / ${fmtBytes(total)}` : ""}
  </span>
</div>
