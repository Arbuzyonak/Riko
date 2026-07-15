import { listGames, type Game } from "../api";

export const gamesState = $state({
  games: [] as Game[],
  loading: false,
  loaded: false,
  error: null as string | null,
});

export async function loadGames(refresh: boolean) {
  gamesState.loading = true;
  gamesState.error = null;
  try {
    gamesState.games = await listGames(refresh);
    gamesState.loaded = true;
  } catch (e) {
    gamesState.error = String(e);
  } finally {
    gamesState.loading = false;
  }
}
