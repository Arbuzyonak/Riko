<script lang="ts">
  import {
    getSetupPlan,
    runSetupStep,
    type PlannedStep,
    type SetupPlan,
  } from "../lib/api";
  import { navigate } from "../lib/router.svelte";
  import { refreshStatus } from "../lib/stores/app.svelte";
  import { progressState, resetProgress } from "../lib/stores/progress.svelte";
  import { toast } from "../lib/stores/toast.svelte";
  import ProgressBar from "../lib/components/ProgressBar.svelte";

  let plan = $state<SetupPlan | null>(null);
  let runningStep = $state<string | null>(null);
  let failedStep = $state<{ step: string; error: string } | null>(null);

  const activeStage = $derived(
    progressState.active ? progressState.stages[progressState.active] : null
  );
  const needed = $derived(plan?.steps.filter((s) => s.status === "needed") ?? []);
  const allDone = $derived(plan !== null && needed.length === 0);

  $effect(() => {
    if (plan === null) refresh();
  });

  async function refresh() {
    plan = await getSetupPlan();
  }

  async function runStep(step: PlannedStep) {
    if (runningStep) return;
    runningStep = step.step;
    failedStep = null;
    resetProgress();
    try {
      await runSetupStep(step.step);
      toast(`${step.label} finished`, "success");
    } catch (e) {
      failedStep = { step: step.step, error: String(e) };
    } finally {
      runningStep = null;
      await refresh();
    }
  }

  async function runAllNeeded() {
    for (const step of needed) {
      if (failedStep) break;
      await runStep(step);
    }
  }

  async function finish() {
    await refreshStatus();
    navigate("/");
  }
</script>

<div class="mx-auto flex max-w-3xl flex-col gap-6 p-8">
  <div>
    <h1 class="text-2xl font-semibold tracking-tight text-white">Set up Riko</h1>
    <p class="mt-1 text-sm text-zinc-500">
      Riko needs a few things in place before it can launch Vortex games.
    </p>
  </div>

  <div class="flex flex-col divide-y divide-edge rounded-xl border border-edge bg-panel">
    {#each plan?.steps ?? [] as step (step.step)}
      <div class="flex items-center gap-4 px-5 py-4">
        <div class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full">
          {#if runningStep === step.step}
            <div
              class="h-5 w-5 animate-spin rounded-full border-2 border-edge border-t-accent"
            ></div>
          {:else if step.status === "done"}
            <svg class="h-5 w-5 text-ok" fill="none" viewBox="0 0 24 24" stroke-width="2.5" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
            </svg>
          {:else if step.status === "optional"}
            <span class="h-2.5 w-2.5 rounded-full bg-zinc-600"></span>
          {:else}
            <span class="h-2.5 w-2.5 rounded-full bg-warn"></span>
          {/if}
        </div>
        <div class="min-w-0 flex-1">
          <p class="text-sm font-medium text-zinc-200">
            {step.label}
            {#if step.status === "optional"}
              <span class="ml-1.5 text-xs font-normal text-zinc-500">optional</span>
            {/if}
          </p>
          <p class="truncate text-xs text-zinc-500">{step.description}</p>
          {#if failedStep?.step === step.step}
            <p class="mt-1 text-xs text-danger">{failedStep.error}</p>
            {#if step.manual_command}
              <p class="mt-1 text-xs text-zinc-500">
                Run this in a terminal, then re-check:
              </p>
              <code
                class="mt-1 block w-fit rounded bg-black/40 px-2 py-1 font-mono text-xs text-zinc-300 select-text"
              >
                {step.manual_command}
              </code>
            {/if}
          {/if}
        </div>
        {#if step.status !== "done"}
          <button
            class="shrink-0 rounded-lg border border-edge px-3.5 py-1.5 text-xs font-medium text-zinc-200 transition-colors hover:bg-panel-hover disabled:opacity-40"
            disabled={runningStep !== null}
            onclick={() => runStep(step)}
          >
            Run
          </button>
        {/if}
      </div>
    {/each}
  </div>

  {#if runningStep && activeStage}
    <div class="flex flex-col gap-3 rounded-xl border border-edge bg-panel p-5">
      <p class="text-sm font-medium text-zinc-200">{activeStage.label}</p>
      {#if activeStage.total !== null || activeStage.done > 0}
        <ProgressBar done={activeStage.done} total={activeStage.total} />
      {/if}
      {#if activeStage.lines.length > 0}
        <div
          class="max-h-40 overflow-y-auto rounded-lg bg-black/40 p-3 font-mono text-xs leading-relaxed"
        >
          {#each activeStage.lines.slice(-60) as entry, i (i)}
            <div
              class={entry.level === "error"
                ? "text-danger"
                : entry.level === "warn"
                  ? "text-warn/80"
                  : "text-zinc-400"}
            >
              {entry.line}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  <div class="flex items-center justify-between">
    <button
      class="rounded-lg border border-edge px-4 py-2 text-sm text-zinc-300 transition-colors hover:bg-panel-hover disabled:opacity-40"
      disabled={runningStep !== null}
      onclick={refresh}
    >
      Re-check
    </button>
    <div class="flex gap-3">
      {#if needed.length > 0}
        <button
          class="rounded-lg bg-accent px-5 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-40"
          disabled={runningStep !== null}
          onclick={runAllNeeded}
        >
          Run all needed steps
        </button>
      {/if}
      <button
        class="rounded-lg px-5 py-2 text-sm font-medium transition-colors {allDone
          ? 'bg-accent text-white hover:bg-accent-hover'
          : 'border border-edge text-zinc-400 hover:bg-panel-hover'}"
        disabled={runningStep !== null}
        onclick={finish}
      >
        {allDone ? "Finish" : "Skip for now"}
      </button>
    </div>
  </div>
</div>
