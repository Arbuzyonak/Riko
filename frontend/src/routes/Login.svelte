<script lang="ts">
  import { login } from "../lib/api";
  import { navigate } from "../lib/router.svelte";
  import { appState, refreshStatus } from "../lib/stores/app.svelte";
  import { toast } from "../lib/stores/toast.svelte";

  let username = $state("");
  let password = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!username || !password || busy) return;
    busy = true;
    error = null;
    try {
      const user = await login(username, password);
      await refreshStatus();
      toast(`Logged in as ${user}`, "success");
      navigate(appState.status?.setup_needed ? "/setup" : "/");
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="flex h-full items-center justify-center">
  <form
    class="flex w-96 flex-col gap-4 rounded-2xl border border-edge bg-panel p-8"
    onsubmit={submit}
  >
    <div class="mb-2 flex flex-col items-center gap-3">
      <div
        class="flex h-14 w-14 items-center justify-center rounded-2xl bg-accent text-2xl font-bold text-white"
      >
        R
      </div>
      <div class="text-center">
        <h1 class="text-xl font-semibold text-white">Sign in to Vortex</h1>
        <p class="mt-1 text-sm text-zinc-500">with your playvortex.io account</p>
      </div>
    </div>

    <label class="flex flex-col gap-1.5">
      <span class="text-xs font-medium text-zinc-400">Username</span>
      <input
        type="text"
        bind:value={username}
        autocomplete="username"
        class="rounded-lg border border-edge bg-surface px-3 py-2 text-sm text-white outline-none transition-colors focus:border-accent"
      />
    </label>

    <label class="flex flex-col gap-1.5">
      <span class="text-xs font-medium text-zinc-400">Password</span>
      <input
        type="password"
        bind:value={password}
        autocomplete="current-password"
        class="rounded-lg border border-edge bg-surface px-3 py-2 text-sm text-white outline-none transition-colors focus:border-accent"
      />
    </label>

    {#if error}
      <p class="rounded-lg bg-danger/10 px-3 py-2 text-sm text-danger">{error}</p>
    {/if}

    <button
      type="submit"
      disabled={busy || !username || !password}
      class="mt-2 rounded-lg bg-accent py-2.5 text-sm font-medium text-white transition-colors hover:bg-accent-hover disabled:cursor-not-allowed disabled:opacity-50"
    >
      {busy ? "Signing in…" : "Sign in"}
    </button>

    {#if appState.status?.logged_in}
      <button
        type="button"
        class="text-sm text-zinc-500 transition-colors hover:text-zinc-300"
        onclick={() => navigate("/settings")}
      >
        Cancel - stay as {appState.status.username}
      </button>
    {/if}
  </form>
</div>
