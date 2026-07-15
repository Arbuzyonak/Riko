# Riko telemetry worker

A tiny, serverless intake + dashboard for **opt-in, anonymous** usage and crash
data. Runs on Cloudflare Workers + D1. There is no server to keep online and it
costs ~nothing at this scale.

## What the client sends (only when the user opts in)

- **Heartbeat** (`POST /heartbeat`) on startup: a random install ID, app
  version, OS, and CPU arch. That's it.
- **Error report** (`POST /error`) on a panic: the panic message, version, OS,
  and (if known) the install ID.

It never sends the Vortex account, game activity, file paths, or `riko.log`.
The toggle is **off by default** (Settings → "Share anonymous usage & crash
reports"). See `crates/riko-core/src/telemetry.rs`.

## Endpoints

| method | path         | purpose                                  |
|--------|--------------|------------------------------------------|
| POST   | `/heartbeat` | upsert an install's last-seen + version  |
| POST   | `/error`     | append a crash/error report              |
| GET    | `/`          | the dashboard (installs, active, errors) |

## Deploy

```sh
cd telemetry-worker
npm install -g wrangler          # or use npx wrangler ...
wrangler d1 create riko-telemetry            # paste database_id into wrangler.toml
wrangler d1 execute riko-telemetry --file schema.sql
wrangler deploy                               # prints your https://riko-telemetry.<subdomain>.workers.dev
```

Then point the launcher at it. Either edit
`crates/riko-core/src/telemetry.rs` `DEFAULT_ENDPOINT`, or set per install in
`config.toml`:

```toml
[telemetry]
enabled = true
endpoint = "https://riko-telemetry.<subdomain>.workers.dev"
```

### Protect the dashboard (optional)

```sh
wrangler secret put DASHBOARD_TOKEN     # then open  /?token=<value>
```

Without a token the dashboard is public-read (it contains no personal data, but
error messages can be noisy — a token is recommended).

## Private server instead of a VPS?

You don't need a VPS for this. If you'd rather self-host the same logic on a box
you own, run any small HTTP service that implements the three routes against
SQLite and expose it with a **Cloudflare Tunnel** (`cloudflared`) so you get
public HTTPS without opening ports or exposing your home IP. The Worker path
above is simply the lowest-maintenance option.
