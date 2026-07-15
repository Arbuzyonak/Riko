import { invoke } from "@tauri-apps/api/core";

export interface AppStatus {
  logged_in: boolean;
  username: string | null;
  setup_needed: boolean;
  migrated_from_tempest: boolean;
}

export interface Game {
  id: number;
  name: string;
  description: string | null;
  thumbnail_url: string | null;
  creator: string | null;
}

export const getAppStatus = () => invoke<AppStatus>("get_app_status");

export const login = (username: string, password: string) =>
  invoke<string>("login", { username, password });

export const logout = () => invoke<void>("logout");

export const listGames = (refresh: boolean) =>
  invoke<Game[]>("list_games", { refresh });
