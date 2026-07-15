<script lang="ts">
  import { applyFix, runDoctor, type CheckResult, type FixAction } from "../lib/api";
  import { navigate } from "../lib/router.svelte";
  import { progressState, resetProgress } from "../lib/stores/progress.svelte";
  import { toast } from "../lib/stores/toast.svelte";
  import ProgressBar from "../lib/components/ProgressBar.svelte";

  let checks = $state<CheckResult[] | null>(null);
  let running = $state(false);
  let fixing = $state<string | null>(null);
  let copiedId = $state<string | null>(null);

  const failCount = $derived((checks ?? []).filter((c) => !c.passed).length);
  const downloadStage = $derived(progressState.stages["download-vortex"]);

  $effect(() => {
    if (checks === null && !running) refresh();
  });

  async function refresh() {
    running = true;
    try {
      checks = await runDoctor();
    } catch (e) {
      toast(String(e), "error");
    } finally {
      running = false;
    }
  }

  async function copyCommand(check: CheckResult, shell: string) {
    try {
      await navigator.clipboard.writeText(shell);
      copiedId = check.id;
      setTimeout(() => {
        if (copiedId === check.id) copiedId = null;
      }, 2000);
    } catch {
      toast("Could not access the clipboard; copy the command manually", "error");
    }
  }

  async function handleFix(check: CheckResult, fix: FixAction) {
    if (fix.kind === "command") {
      await copyCommand(check, fix.shell);
      return;
    }
    if (fix.kind === "run_setup") {
      navigate("/setup");
      return;
    }
    if (fix.kind === "run_login") {
      navigate("/login");
      return;
    }
    fixing = check.id;
    if (fix.kind === "run_update") resetProgress();
    try {
      await applyFix(fix.kind === "run_update" ? { kind: "run_update" } : { kind: "register_uri" });
      toast(`${check.name}: fix applied`, "success");
    } catch (e) {
      toast(String(e), "error");
    } finally {
      fixing = null;
      await refresh();
    }
  }
</script>

<div class="flex flex-col gap-6 p-8">
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-semibold tracking-tight text-white">Doctor</h1>
      <p class="mt-1 text-sm text-zinc-500">
        {#if checks === null}
          Checking your system…
        {:else if failCount === 0}
          All {checks.length} checks passed.
        {:else}
          {failCount} of {checks.length} checks need attention.
        {/if}
      </p>
    </div>
    <button
      class="shrink-0 rounded-lg border border-edge bg-panel px-3.5 py-2 text-sm text-zinc-300 transition-colors hover:bg-panel-hover disabled:opacity-50"
      disabled={running}
      onclick={refresh}
    >
      {running ? "Checking…" : "Re-check"}
    </button>
  </div>

  <div class="flex flex-col divide-y divide-edge rounded-xl border border-edge bg-panel px-5">
    {#each checks ?? [] as check (check.id)}
      <div class="flex flex-col py-3.5">
        <div class="flex items-center justify-between gap-4">
          <div class="flex min-w-0 items-center gap-3">
            {#if check.passed}
              <span
                class="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-ok/15 text-ok"
              >
                <svg class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke-width="3" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
                </svg>
              </span>
            {:else}
              <span
                class="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-danger/15 text-danger"
              >
                <svg class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke-width="3" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </span>
            {/if}
            <div class="min-w-0">
              <span class="text-sm text-zinc-200">{check.name}</span>
              <p class="truncate text-xs text-zinc-500">{check.detail}</p>
            </div>
          </div>
          {#if !check.passed && check.fix}
            {#if fixing === check.id}
              <div
                class="h-5 w-5 shrink-0 animate-spin rounded-full border-2 border-edge border-t-accent"
              ></div>
            {:else}
              <button
                class="shrink-0 rounded-lg bg-accent px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
                disabled={fixing !== null}
                onclick={() => check.fix && handleFix(check, check.fix)}
              >
                {copiedId === check.id ? "Copied!" : check.fix.label}
              </button>
            {/if}
          {/if}
        </div>
        {#if !check.passed && check.fix?.kind === "command"}
          <code
            class="mt-2 ml-8 block w-fit rounded-lg bg-black/40 px-3 py-2 font-mono text-xs break-all text-zinc-300 select-text"
          >
            {check.fix.shell}
          </code>
        {/if}
        {#if fixing === check.id && check.fix?.kind === "run_update" && downloadStage && !downloadStage.finished}
          <div class="mt-3 ml-8">
            <ProgressBar done={downloadStage.done} total={downloadStage.total} />
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>
