<script lang="ts">
  import { getFriends, type Friend } from "../lib/api";
  import { toast } from "../lib/stores/toast.svelte";

  const REFRESH_MS = 60_000;

  let friends = $state<Friend[] | null>(null);
  let refreshing = $state(false);

  const onlineCount = $derived(
    (friends ?? []).filter((f) => f.online_status !== "offline").length
  );

  $effect(() => {
    if (friends === null) refresh();
    const timer = setInterval(refresh, REFRESH_MS);
    return () => clearInterval(timer);
  });

  async function refresh() {
    refreshing = true;
    try {
      friends = await getFriends();
    } catch (e) {
      if (friends === null) toast(String(e), "error");
      friends ??= [];
    } finally {
      refreshing = false;
    }
  }

  function statusLabel(status: string): string {
    if (status === "in_game") return "In game";
    if (status === "online") return "Online";
    return "Offline";
  }
</script>

<div class="flex flex-col gap-6 p-8">
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-semibold tracking-tight text-white">Friends</h1>
      <p class="mt-1 text-sm text-zinc-500">
        {#if friends === null}
          Loading…
        {:else}
          {onlineCount} of {friends.length} online
        {/if}
      </p>
    </div>
    <button
      class="shrink-0 rounded-lg border border-edge bg-panel px-3.5 py-2 text-sm text-zinc-300 transition-colors hover:bg-panel-hover disabled:opacity-50"
      disabled={refreshing}
      onclick={refresh}
    >
      {refreshing ? "Refreshing…" : "Refresh"}
    </button>
  </div>

  {#if friends !== null && friends.length === 0}
    <div
      class="flex flex-col items-center gap-2 rounded-xl border border-dashed border-edge py-20 text-center"
    >
      <p class="text-zinc-300">No friends yet</p>
      <p class="text-sm text-zinc-500">Add friends on playvortex.io and they'll show up here.</p>
    </div>
  {:else}
    <div class="flex flex-col divide-y divide-edge rounded-xl border border-edge bg-panel px-5">
      {#each friends ?? [] as friend (friend.id)}
        <div class="flex items-center justify-between py-3">
          <div class="flex items-center gap-3">
            <div class="relative">
              {#if friend.avatar}
                <img
                  src={friend.avatar}
                  alt=""
                  class="h-10 w-10 rounded-full object-cover {friend.online_status === 'offline'
                    ? 'opacity-40 grayscale'
                    : ''}"
                />
              {:else}
                <div
                  class="flex h-10 w-10 items-center justify-center rounded-full bg-accent/20 text-sm font-semibold text-accent uppercase {friend.online_status ===
                  'offline'
                    ? 'opacity-40'
                    : ''}"
                >
                  {friend.username.slice(0, 1)}
                </div>
              {/if}
              <span
                class="absolute -right-0.5 -bottom-0.5 h-3 w-3 rounded-full border-2 border-panel {friend.online_status ===
                'in_game'
                  ? 'bg-accent'
                  : friend.online_status === 'online'
                    ? 'bg-ok'
                    : 'bg-zinc-600'}"
              ></span>
            </div>
            <div>
              <p
                class="text-sm {friend.online_status === 'offline'
                  ? 'text-zinc-500'
                  : 'text-zinc-200'}"
              >
                {friend.username}
              </p>
              <p
                class="text-xs {friend.online_status === 'in_game'
                  ? 'text-accent'
                  : friend.online_status === 'online'
                    ? 'text-ok'
                    : 'text-zinc-600'}"
              >
                {statusLabel(friend.online_status)}
              </p>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
