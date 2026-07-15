import { getAppStatus, type AppStatus } from "../api";

export const appState = $state({
  status: null as AppStatus | null,
  loading: true,
});

export async function refreshStatus() {
  appState.loading = true;
  try {
    appState.status = await getAppStatus();
  } finally {
    appState.loading = false;
  }
}
