<script lang="ts">
  import { toastState } from "../stores/toast.svelte";

  const colors: Record<string, string> = {
    info: "border-edge",
    success: "border-ok",
    error: "border-danger",
  };
</script>

<div class="pointer-events-none fixed right-4 bottom-4 z-50 flex w-80 flex-col gap-2">
  {#each toastState.items as item (item.id)}
    <div
      class="pointer-events-auto rounded-lg border-l-4 bg-panel px-4 py-3 text-sm shadow-lg {colors[
        item.kind
      ]}"
    >
      {item.message}
    </div>
  {/each}
</div>
