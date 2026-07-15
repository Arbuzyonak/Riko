<script lang="ts">
  let {
    checked,
    label,
    hint,
    onchange,
  }: {
    checked: boolean;
    label: string;
    hint?: string;
    onchange: (value: boolean) => void;
  } = $props();
</script>

<label class="flex cursor-pointer items-center justify-between gap-4 py-2.5">
  <span class="flex flex-col">
    <span class="text-sm text-zinc-200">{label}</span>
    {#if hint}
      <span class="text-xs text-zinc-500">{hint}</span>
    {/if}
  </span>
  <button
    role="switch"
    aria-checked={checked}
    aria-label={label}
    class="relative h-6 w-11 shrink-0 rounded-full transition-colors {checked
      ? 'bg-accent'
      : 'bg-edge'}"
    onclick={() => onchange(!checked)}
  >
    <span
      class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform {checked
        ? 'translate-x-5'
        : ''}"
    ></span>
  </button>
</label>
