<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { router, navigate } from "./lib/router.svelte";
  import { appState, refreshStatus } from "./lib/stores/app.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import Toasts from "./lib/components/Toasts.svelte";
  import Library from "./routes/Library.svelte";
  import Login from "./routes/Login.svelte";
  import GameDetail from "./routes/GameDetail.svelte";
  import SetupWizard from "./routes/SetupWizard.svelte";
  import Settings from "./routes/Settings.svelte";
  import Plugins from "./routes/Plugins.svelte";
  import Doctor from "./routes/Doctor.svelte";
  import Friends from "./routes/Friends.svelte";
  import Stats from "./routes/Stats.svelte";
  import { toast } from "./lib/stores/toast.svelte";
  import { initSessionEvents } from "./lib/stores/session.svelte";
  import { initProgressEvents } from "./lib/stores/progress.svelte";
  import { checkRikoUpdate, type UpdateInfo } from "./lib/api";

  let update = $state<UpdateInfo | null>(null);
  let updateDismissed = $state(false);

  onMount(async () => {
    await initSessionEvents();
    await initProgressEvents();
    await listen("vortex://updated", () =>
      toast("Vortex client updated to the latest version", "success")
    );
    await refreshStatus();
    if (appState.status?.migrated_from_tempest) {
      toast("Imported your existing tempest configuration", "success");
    }
    if (!appState.status?.logged_in) {
      navigate("/login");
    } else if (appState.status?.setup_needed) {
      navigate("/setup");
    }
    checkRikoUpdate()
      .then((info) => (update = info))
      .catch(() => {});
  });

  const showChrome = $derived(router.path !== "/login" && router.path !== "/setup");
</script>

<div class="flex h-screen overflow-hidden">
  {#if appState.loading}
    <div class="flex flex-1 items-center justify-center">
      <div
        class="h-8 w-8 animate-spin rounded-full border-2 border-edge border-t-accent"
      ></div>
    </div>
  {:else}
    {#if showChrome}
      <Sidebar />
    {/if}
    <main class="flex-1 overflow-y-auto">
      {#if update && !updateDismissed && showChrome}
        <div
          class="flex items-center gap-3 border-b border-edge bg-accent/10 px-6 py-2.5 text-sm"
        >
          <span class="text-zinc-200">
            Riko <b class="font-semibold text-white">{update.latest}</b> is available
            <span class="text-zinc-500">(you have {update.current})</span>
          </span>
          <a
            class="rounded-md bg-accent px-2.5 py-1 text-xs font-medium text-white transition-colors hover:bg-accent-hover"
            href={update.release_url}
            target="_blank"
            rel="noreferrer"
          >
            View release
          </a>
          <button
            class="ml-auto text-xs text-zinc-500 transition-colors hover:text-zinc-300"
            onclick={() => (updateDismissed = true)}
          >
            Dismiss
          </button>
        </div>
      {/if}
      {#if router.path === "/login"}
        <Login />
      {:else if router.path === "/setup"}
        <SetupWizard />
      {:else if router.path === "/plugins"}
        <Plugins />
      {:else if router.path === "/doctor"}
        <Doctor />
      {:else if router.path === "/friends"}
        <Friends />
      {:else if router.path === "/stats"}
        <Stats />
      {:else if router.path === "/settings"}
        <Settings />
      {:else if router.path.startsWith("/game/")}
        <GameDetail />
      {:else}
        <Library />
      {/if}
    </main>
  {/if}
</div>
<Toasts />
