import { getGameStats, type GameStats } from "../api";

const REFRESH_MS = 60_000;

export const statsState = $state({
  entries: {} as Record<number, GameStats>,
  loaded: false,
});

let polling = false;

export async function loadStats() {
  try {
    statsState.entries = await getGameStats();
    statsState.loaded = true;
  } catch {
    statsState.loaded = true;
  }
  if (!polling) {
    polling = true;
    setInterval(() => {
      getGameStats()
        .then((entries) => (statsState.entries = entries))
        .catch(() => {});
    }, REFRESH_MS);
  }
}

export function formatVisits(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 10_000) return `${Math.round(n / 1_000)}k`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return `${n}`;
}
