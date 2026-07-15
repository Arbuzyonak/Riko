CREATE TABLE IF NOT EXISTS heartbeats (
  install_id TEXT PRIMARY KEY,
  version    TEXT NOT NULL,
  os         TEXT NOT NULL,
  arch       TEXT NOT NULL,
  first_seen TEXT NOT NULL,
  last_seen  TEXT NOT NULL,
  hits       INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS errors (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  install_id TEXT,
  version    TEXT NOT NULL,
  os         TEXT NOT NULL,
  kind       TEXT NOT NULL,
  message    TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_errors_created ON errors (created_at);
CREATE INDEX IF NOT EXISTS idx_heartbeats_last_seen ON heartbeats (last_seen);
