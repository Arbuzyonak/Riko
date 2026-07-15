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

export interface GameSession {
  game_id: number;
  pid: number;
  started_at: string;
}

export const launchGame = (gameId: number) =>
  invoke<number>("launch_game", { gameId });

export const stopGame = (gameId: number) =>
  invoke<void>("stop_game", { gameId });

export const getRunningSessions = () =>
  invoke<GameSession[]>("get_running_sessions");

export type SetupStepId =
  | "install_wine"
  | "create_prefix"
  | "install_winetricks_components"
  | "install_gamemode"
  | "install_dxvk"
  | "install_vkd3d"
  | "download_vortex"
  | "register_uri";

export interface PlannedStep {
  step: SetupStepId;
  label: string;
  description: string;
  status: "done" | "needed" | "optional";
  manual_command: string | null;
}

export interface SetupPlan {
  steps: PlannedStep[];
}

export const getSetupPlan = () => invoke<SetupPlan>("get_setup_plan");

export const runSetupStep = (step: SetupStepId) =>
  invoke<void>("run_setup_step", { step });

export const uninstallRiko = () => invoke<void>("uninstall_riko");

export interface ConfigView {
  username: string | null;
  has_session: boolean;
  wine_binary: string;
  wine_env: Record<string, string>;
  filter_wine_noise: boolean;
  auto_update: boolean;
  use_esync: boolean;
  use_fsync: boolean;
  use_gamemode: boolean;
  shader_cache: boolean;
  presence_enabled: boolean;
  wine_prefix: string;
  vortex_exe: string;
  log_file: string;
}

export type ConfigPatch = Partial<{
  wine_binary: string;
  wine_env: Record<string, string>;
  filter_wine_noise: boolean;
  auto_update: boolean;
  use_esync: boolean;
  use_fsync: boolean;
  use_gamemode: boolean;
  shader_cache: boolean;
  presence_enabled: boolean;
}>;

export const getConfig = () => invoke<ConfigView>("get_config");

export const updateConfig = (patch: ConfigPatch) =>
  invoke<ConfigView>("update_config", { patch });

export interface PluginInfo {
  name: string;
  version: string;
  description: string;
  kind: "vulkan-layer" | "binary" | "env-only";
  platforms: string[];
  builtin: boolean;
  installed: boolean;
  built: boolean;
  enabled: boolean;
  supported: boolean;
  build_command: string | null;
}

export const listPlugins = () => invoke<PluginInfo[]>("list_plugins");

export const installPlugin = (name: string) =>
  invoke<PluginInfo>("install_plugin", { name });

export const importPlugin = (path: string) =>
  invoke<PluginInfo>("import_plugin", { path });

export const removePlugin = (name: string) =>
  invoke<void>("remove_plugin", { name });

export const setPluginEnabled = (
  name: string,
  gameId: number | null,
  enabled: boolean | null
) => invoke<PluginInfo[]>("set_plugin_enabled", { name, gameId, enabled });

export interface PerGamePlugins {
  enabled: string[];
  disabled: string[];
}

export const getGamePluginOverrides = (gameId: number) =>
  invoke<PerGamePlugins>("get_game_plugin_overrides", { gameId });

export interface ProgressEvent {
  type: "stage_started" | "stage_progress" | "stage_log" | "stage_finished";
  stage: string;
  label?: string;
  done?: number;
  total?: number | null;
  level?: "info" | "warn" | "error";
  line?: string;
  ok?: boolean;
  detail?: string | null;
}
