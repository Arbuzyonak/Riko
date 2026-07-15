<script lang="ts">
  import { onMount } from "svelte";
  import { router, navigate } from "./lib/router.svelte";
  import { appState, refreshStatus } from "./lib/stores/app.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import Toasts from "./lib/components/Toasts.svelte";
  import Library from "./routes/Library.svelte";
  import Login from "./routes/Login.svelte";
  import GameDetail from "./routes/GameDetail.svelte";
  import SetupWizard from "./routes/SetupWizard.svelte";
  import Settings from "./routes/Settings.svelte";
  import Placeholder from "./routes/Placeholder.svelte";
  import { toast } from "./lib/stores/toast.svelte";
  import { initSessionEvents } from "./lib/stores/session.svelte";
  import { initProgressEvents } from "./lib/stores/progress.svelte";

  onMount(async () => {
    await initSessionEvents();
    await initProgressEvents();
    await refreshStatus();
    if (appState.status?.migrated_from_tempest) {
      toast("Imported your existing tempest configuration", "success");
    }
    if (!appState.status?.logged_in) {
      navigate("/login");
    } else if (appState.status?.setup_needed) {
      navigate("/setup");
    }
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
      {#if router.path === "/login"}
        <Login />
      {:else if router.path === "/setup"}
        <SetupWizard />
      {:else if router.path === "/plugins"}
        <Placeholder title="Plugins" />
      {:else if router.path === "/doctor"}
        <Placeholder title="Doctor" />
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
