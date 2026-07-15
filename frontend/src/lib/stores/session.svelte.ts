import { listen } from "@tauri-apps/api/event";
import { getRunningSessions } from "../api";
import { navigate } from "../router.svelte";
import { toast } from "./toast.svelte";

const LOG_LIMIT = 500;

export interface LogEntry {
  line: string;
  isStderr: boolean;
}

export const sessionState = $state({
  running: {} as Record<number, { pid: number; startedAt: string }>,
  logs: {} as Record<number, LogEntry[]>,
});

export function isRunning(gameId: number): boolean {
  return gameId in sessionState.running;
}

interface GameEventPayload {
  game_id: number;
  type: "started" | "exited";
  pid?: number;
  code?: number | null;
  duration_secs?: number;
}

interface LogBatchPayload {
  game_id: number;
  lines: { line: string; is_stderr: boolean }[];
}

let initialized = false;

export async function initSessionEvents() {
  if (initialized) return;
  initialized = true;

  const sessions = await getRunningSessions().catch(() => []);
  for (const s of sessions) {
    sessionState.running[s.game_id] = { pid: s.pid, startedAt: s.started_at };
  }

  await listen<GameEventPayload>("game://event", ({ payload }) => {
    if (payload.type === "started") {
      sessionState.running[payload.game_id] = {
        pid: payload.pid ?? 0,
        startedAt: new Date().toISOString(),
      };
      sessionState.logs[payload.game_id] = [];
    } else if (payload.type === "exited") {
      delete sessionState.running[payload.game_id];
      const mins = Math.round((payload.duration_secs ?? 0) / 60);
      const codeInfo =
        payload.code === 0 || payload.code == null
          ? "Game session ended"
          : `Game exited with code ${payload.code}`;
      toast(`${codeInfo} (${mins} min)`, payload.code ? "error" : "info");
    }
  });

  await listen<LogBatchPayload>("game://logs", ({ payload }) => {
    const entries = (sessionState.logs[payload.game_id] ??= []);
    for (const l of payload.lines) {
      entries.push({ line: l.line, isStderr: l.is_stderr });
    }
    if (entries.length > LOG_LIMIT) {
      entries.splice(0, entries.length - LOG_LIMIT);
    }
  });

  await listen<string>("game://launch-error", ({ payload }) => {
    toast(payload, "error");
  });

  await listen<number>("nav://game", ({ payload }) => {
    navigate(`/game/${payload}`);
  });

  await listen("auth://expired", () => {
    toast("Your session expired; please sign in again", "error");
    navigate("/login");
  });
}
