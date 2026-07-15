<script lang="ts">
  let { title }: { title: string } = $props();
</script>

<div class="flex flex-col gap-6 p-8">
  <h1 class="text-2xl font-semibold tracking-tight text-white">{title}</h1>
  <div
    class="flex flex-col items-center gap-2 rounded-xl border border-dashed border-edge py-20 text-center"
  >
    <p class="text-zinc-300">Coming soon</p>
    <p class="text-sm text-zinc-500">This page is being built.</p>
  </div>
</div>
