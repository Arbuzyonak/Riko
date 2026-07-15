import { getPlaytime, type PlaytimeEntry } from "../api";

export const playtimeState = $state({
  entries: {} as Record<number, PlaytimeEntry>,
  loaded: false,
});

export async function loadPlaytime(force = false) {
  if (playtimeState.loaded && !force) return;
  try {
    playtimeState.entries = await getPlaytime();
    playtimeState.loaded = true;
  } catch {
    playtimeState.loaded = true;
  }
}

export function formatPlaytime(secs: number): string {
  if (secs < 60) return "under a minute";
  const minutes = Math.floor(secs / 60);
  if (minutes < 60) return `${minutes} min`;
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  return rest > 0 ? `${hours}h ${rest}m` : `${hours}h`;
}
