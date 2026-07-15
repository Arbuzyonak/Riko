import { listen } from "@tauri-apps/api/event";
import type { ProgressEvent } from "../api";

const LOG_LIMIT = 300;

export interface StageState {
  label: string;
  done: number;
  total: number | null;
  lines: { level: string; line: string }[];
  finished: boolean;
  ok: boolean | null;
  detail: string | null;
}

export const progressState = $state({
  stages: {} as Record<string, StageState>,
  active: null as string | null,
});

function stage(name: string): StageState {
  return (progressState.stages[name] ??= {
    label: name,
    done: 0,
    total: null,
    lines: [],
    finished: false,
    ok: null,
    detail: null,
  });
}

export function resetProgress() {
  progressState.stages = {};
  progressState.active = null;
}

function apply(event: ProgressEvent) {
  const s = stage(event.stage);
  switch (event.type) {
    case "stage_started":
      s.label = event.label ?? event.stage;
      s.finished = false;
      s.ok = null;
      progressState.active = event.stage;
      break;
    case "stage_progress":
      s.done = event.done ?? 0;
      s.total = event.total ?? null;
      progressState.active = event.stage;
      break;
    case "stage_log":
      s.lines.push({ level: event.level ?? "info", line: event.line ?? "" });
      if (s.lines.length > LOG_LIMIT) s.lines.splice(0, s.lines.length - LOG_LIMIT);
      progressState.active = event.stage;
      break;
    case "stage_finished":
      s.finished = true;
      s.ok = event.ok ?? null;
      s.detail = event.detail ?? null;
      break;
  }
}

let initialized = false;

export async function initProgressEvents() {
  if (initialized) return;
  initialized = true;
  for (const channel of [
    "setup://progress",
    "plugin://progress",
    "update://progress",
    "wine://progress",
  ]) {
    await listen<ProgressEvent>(channel, ({ payload }) => apply(payload));
  }
}
