<script lang="ts">
  import { router, navigate } from "../router.svelte";
  import { appState, refreshStatus } from "../stores/app.svelte";
  import { logout } from "../api";

  const items = [
    { path: "/", label: "Library", icon: "M4 6h16M4 12h16M4 18h7" },
    {
      path: "/friends",
      label: "Friends",
      icon: "M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0z",
    },
    { path: "/plugins", label: "Plugins", icon: "M13 10V3L4 14h7v7l9-11h-7z" },
    { path: "/doctor", label: "Doctor", icon: "M4.5 12.75l6 6 9-13.5" },
    {
      path: "/settings",
      label: "Settings",
      icon: "M10.5 6h9.75M10.5 6a1.5 1.5 0 11-3 0m3 0a1.5 1.5 0 10-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-9.75 0h9.75",
    },
  ];

  const isActive = (path: string) =>
    path === "/" ? router.path === "/" : router.path.startsWith(path);

  async function handleLogout() {
    await logout();
    await refreshStatus();
    if (!appState.status?.logged_in) navigate("/login");
  }
</script>

<aside class="flex w-56 shrink-0 flex-col border-r border-edge bg-panel">
  <div class="flex items-center gap-2.5 px-5 py-5">
    <div
      class="flex h-8 w-8 items-center justify-center rounded-lg bg-accent font-bold text-white"
    >
      R
    </div>
    <span class="text-lg font-semibold tracking-tight text-white">Riko</span>
  </div>

  <nav class="flex flex-1 flex-col gap-1 px-3">
    {#each items as item (item.path)}
      <button
        class="flex items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition-colors
          {isActive(item.path)
          ? 'bg-panel-hover font-medium text-white'
          : 'text-zinc-400 hover:bg-panel-hover hover:text-zinc-200'}"
        onclick={() => navigate(item.path)}
      >
        <svg
          class="h-4.5 w-4.5"
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="1.8"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" d={item.icon} />
        </svg>
        {item.label}
      </button>
    {/each}
  </nav>

  <div class="border-t border-edge px-5 py-4">
    {#if appState.status?.username}
      <p class="mb-2 truncate text-sm text-zinc-300">{appState.status.username}</p>
    {/if}
    <button
      class="text-xs text-zinc-500 transition-colors hover:text-danger"
      onclick={handleLogout}
    >
      Log out
    </button>
  </div>
</aside>
