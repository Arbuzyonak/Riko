<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    open,
    onclose,
    children,
  }: {
    title: string;
    open: boolean;
    onclose: () => void;
    children: Snippet;
  } = $props();
</script>

{#if open}
  <div
    class="fixed inset-0 z-40 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    role="presentation"
    onclick={(e) => e.target === e.currentTarget && onclose()}
  >
    <div class="w-[28rem] rounded-2xl border border-edge bg-panel p-6 shadow-2xl">
      <h2 class="mb-4 text-lg font-semibold text-white">{title}</h2>
      {@render children()}
    </div>
  </div>
{/if}
