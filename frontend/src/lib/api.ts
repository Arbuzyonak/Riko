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

export const launchGame = (gameId: number, username?: string) =>
  invoke<number>("launch_game", { gameId, username: username ?? null });

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
  minimize_while_playing: boolean;
  presence_enabled: boolean;
  telemetry_enabled: boolean;
  community_shaders: boolean;
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
  minimize_while_playing: boolean;
  presence_enabled: boolean;
  telemetry_enabled: boolean;
  community_shaders: boolean;
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
  missing_requirement: string | null;
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

export interface AccountView {
  username: string;
  active: boolean;
}

export const listAccounts = () => invoke<AccountView[]>("list_accounts");

export const switchAccount = (username: string) =>
  invoke<AccountView[]>("switch_account", { username });

export const removeAccount = (username: string) =>
  invoke<AccountView[]>("remove_account", { username });

export interface InstalledWine {
  name: string;
  wine_binary: string;
}

export interface AvailableWine {
  name: string;
  download_url: string;
  size_mb: number;
}

export interface WineVersions {
  installed: InstalledWine[];
  available: AvailableWine[];
  active_binary: string;
}

export const listWineVersions = () => invoke<WineVersions>("list_wine_versions");

export const installWineVersion = (url: string) =>
  invoke<InstalledWine>("install_wine_version", { url });

export const removeWineVersion = (name: string) =>
  invoke<void>("remove_wine_version", { name });

export interface LaunchOverrides {
  wine_binary: string | null;
  use_esync: boolean | null;
  use_fsync: boolean | null;
  use_gamemode: boolean | null;
  env: Record<string, string>;
}

export interface UpdateInfo {
  current: string;
  latest: string;
  release_url: string;
}

export const checkRikoUpdate = () =>
  invoke<UpdateInfo | null>("check_riko_update");

export interface MarketplaceEntry {
  name: string;
  version: string;
  description: string;
  kind: string;
  platforms: string[];
  download_url: string;
  sha256: string;
  size_bytes: number;
  author: string | null;
  homepage: string | null;
  installed: boolean;
}

export const listMarketplace = () =>
  invoke<MarketplaceEntry[]>("list_marketplace");

export const installMarketplacePlugin = (name: string) =>
  invoke<PluginInfo>("install_marketplace_plugin", { name });

export const getLaunchOverrides = (gameId: number) =>
  invoke<LaunchOverrides>("get_launch_overrides", { gameId });

export const setLaunchOverrides = (gameId: number, overrides: LaunchOverrides) =>
  invoke<void>("set_launch_overrides", { gameId, overrides });

export const createShortcut = (gameId: number) =>
  invoke<string>("create_shortcut", { gameId });

export interface Friend {
  id: number;
  username: string;
  online_status: "in_game" | "online" | "offline" | string;
  avatar: string | null;
}

export const getFriends = () => invoke<Friend[]>("get_friends");

export interface GameStats {
  visits: number;
  active: number;
}

export const getGameStats = () =>
  invoke<Record<number, GameStats>>("get_game_stats");

export interface PlaytimeEntry {
  total_secs: number;
  last_played: string | null;
  launches: number;
}

export const getPlaytime = () =>
  invoke<Record<number, PlaytimeEntry>>("get_playtime");

export interface SessionRecord {
  game_id: number;
  started_at: string;
  duration_secs: number;
}

export const getSessions = () => invoke<SessionRecord[]>("get_sessions");

export type FixKind =
  | { kind: "command"; shell: string }
  | { kind: "run_setup" }
  | { kind: "run_login" }
  | { kind: "run_update" }
  | { kind: "register_uri" };

export type FixAction = { label: string } & FixKind;

export interface CheckResult {
  id: string;
  name: string;
  passed: boolean;
  detail: string;
  fix: FixAction | null;
}

export const runDoctor = () => invoke<CheckResult[]>("run_doctor");

export const applyFix = (fix: FixKind) => invoke<void>("apply_fix", { fix });

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
