const MAX_MESSAGE = 4000;
const nowIso = () => new Date().toISOString();
const daysAgoIso = (n) => new Date(Date.now() - n * 86400000).toISOString();

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    try {
      if (request.method === "POST" && url.pathname === "/heartbeat") {
        return await heartbeat(request, env);
      }
      if (request.method === "POST" && url.pathname === "/error") {
        return await recordError(request, env);
      }
      if (request.method === "GET" && url.pathname === "/") {
        return await dashboard(env, url);
      }
      return new Response("not found", { status: 404 });
    } catch (err) {
      return new Response(`error: ${err.message}`, { status: 500 });
    }
  },
};

function str(value, max) {
  return typeof value === "string" ? value.slice(0, max) : "";
}

async function heartbeat(request, env) {
  const body = await request.json().catch(() => ({}));
  const installId = str(body.install_id, 64);
  if (!installId) return new Response("bad request", { status: 400 });
  const version = str(body.version, 32) || "unknown";
  const os = str(body.os, 16) || "unknown";
  const arch = str(body.arch, 16) || "unknown";
  const now = nowIso();
  await env.DB.prepare(
    `INSERT INTO heartbeats (install_id, version, os, arch, first_seen, last_seen, hits)
     VALUES (?1, ?2, ?3, ?4, ?5, ?5, 1)
     ON CONFLICT(install_id) DO UPDATE SET
       version = ?2, os = ?3, arch = ?4, last_seen = ?5, hits = hits + 1`
  ).bind(installId, version, os, arch, now).run();
  return new Response("ok");
}

async function recordError(request, env) {
  const body = await request.json().catch(() => ({}));
  const message = str(body.message, MAX_MESSAGE);
  if (!message) return new Response("bad request", { status: 400 });
  await env.DB.prepare(
    `INSERT INTO errors (install_id, version, os, kind, message, created_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6)`
  ).bind(
    str(body.install_id, 64) || null,
    str(body.version, 32) || "unknown",
    str(body.os, 16) || "unknown",
    str(body.kind, 32) || "error",
    message,
    nowIso()
  ).run();
  return new Response("ok");
}

async function dashboard(env, url) {
  if (env.DASHBOARD_TOKEN && url.searchParams.get("token") !== env.DASHBOARD_TOKEN) {
    return new Response("unauthorized", { status: 401 });
  }
  const totals = await env.DB.prepare(
    `SELECT
       (SELECT COUNT(*) FROM heartbeats) AS installs,
       (SELECT COUNT(*) FROM heartbeats WHERE last_seen >= ?1) AS active7,
       (SELECT COUNT(*) FROM heartbeats WHERE last_seen >= ?2) AS active30,
       (SELECT COUNT(*) FROM errors) AS errors`
  ).bind(daysAgoIso(7), daysAgoIso(30)).first();

  const versions = (await env.DB.prepare(
    `SELECT version, COUNT(*) AS n FROM heartbeats GROUP BY version ORDER BY n DESC LIMIT 12`
  ).all()).results;
  const oses = (await env.DB.prepare(
    `SELECT os, COUNT(*) AS n FROM heartbeats GROUP BY os ORDER BY n DESC`
  ).all()).results;
  const groups = (await env.DB.prepare(
    `SELECT message, COUNT(*) AS n, MAX(created_at) AS last, version
     FROM errors GROUP BY message ORDER BY n DESC LIMIT 40`
  ).all()).results;

  return new Response(renderHtml(totals, versions, oses, groups), {
    headers: { "content-type": "text/html; charset=utf-8" },
  });
}

function esc(s) {
  return String(s).replace(/[&<>"]/g, (c) =>
    ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c])
  );
}

function bar(rows, key) {
  const max = Math.max(1, ...rows.map((r) => r.n));
  return rows
    .map(
      (r) => `<div class="row"><span class="k">${esc(r[key] || "-")}</span>
        <span class="track"><span class="fill" style="width:${(r.n / max) * 100}%"></span></span>
        <span class="n">${r.n}</span></div>`
    )
    .join("");
}

function renderHtml(t, versions, oses, groups) {
  const errorRows = groups
    .map(
      (g) => `<tr><td class="num">${g.n}</td><td class="msg">${esc(g.message)}</td>
        <td>${esc(g.version)}</td><td class="dim">${esc((g.last || "").replace("T", " ").slice(0, 16))}</td></tr>`
    )
    .join("");
  return `<!doctype html><html><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Riko telemetry</title>
<style>
  :root { color-scheme: dark; }
  body { margin:0; background:#100e18; color:#eeecf6; font:15px/1.5 system-ui, sans-serif; padding:2rem clamp(1rem,4vw,3rem); }
  h1 { font-size:1.5rem; letter-spacing:-.01em; margin:0 0 .3rem; }
  .sub { color:#7d7794; margin:0 0 2rem; font-size:.9rem; }
  .cards { display:grid; grid-template-columns:repeat(auto-fit,minmax(160px,1fr)); gap:1rem; margin-bottom:2rem; }
  .card { background:#191624; border:1px solid #2c2740; border-radius:14px; padding:1.1rem 1.2rem; }
  .card .v { font-size:2rem; font-weight:700; font-variant-numeric:tabular-nums; }
  .card .l { color:#7d7794; font-size:.78rem; text-transform:uppercase; letter-spacing:.06em; margin-top:.2rem; }
  .panel { background:#191624; border:1px solid #2c2740; border-radius:14px; padding:1.2rem 1.3rem; margin-bottom:1.5rem; }
  .panel h2 { font-size:.8rem; text-transform:uppercase; letter-spacing:.06em; color:#b3adc9; margin:0 0 1rem; }
  .row { display:grid; grid-template-columns:8rem 1fr 3rem; align-items:center; gap:.7rem; margin:.35rem 0; font-size:.85rem; }
  .k { color:#b3adc9; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .track { height:.5rem; background:#241f3d; border-radius:999px; overflow:hidden; }
  .fill { display:block; height:100%; background:#9385ff; }
  .n { text-align:right; font-variant-numeric:tabular-nums; color:#7d7794; }
  table { width:100%; border-collapse:collapse; font-size:.84rem; }
  th { text-align:left; color:#7d7794; font-weight:600; font-size:.72rem; text-transform:uppercase; letter-spacing:.05em; padding:.4rem .6rem; }
  td { padding:.45rem .6rem; border-top:1px solid #2c2740; vertical-align:top; }
  td.num { font-variant-numeric:tabular-nums; color:#f0687a; font-weight:700; width:3rem; }
  td.msg { font-family:ui-monospace, monospace; font-size:.8rem; color:#eeecf6; }
  td.dim { color:#7d7794; white-space:nowrap; }
  .empty { color:#7d7794; font-size:.85rem; }
</style></head><body>
  <h1>Riko telemetry</h1>
  <p class="sub">Anonymous, opt-in. Updated live from D1.</p>
  <div class="cards">
    <div class="card"><div class="v">${t.installs}</div><div class="l">Installs</div></div>
    <div class="card"><div class="v">${t.active7}</div><div class="l">Active · 7 days</div></div>
    <div class="card"><div class="v">${t.active30}</div><div class="l">Active · 30 days</div></div>
    <div class="card"><div class="v">${t.errors}</div><div class="l">Error reports</div></div>
  </div>
  <div class="panel"><h2>Versions</h2>${versions.length ? bar(versions, "version") : '<p class="empty">No data yet.</p>'}</div>
  <div class="panel"><h2>Operating systems</h2>${oses.length ? bar(oses, "os") : '<p class="empty">No data yet.</p>'}</div>
  <div class="panel"><h2>Errors by signature</h2>
    ${groups.length ? `<table><thead><tr><th>Count</th><th>Message</th><th>Version</th><th>Last seen</th></tr></thead><tbody>${errorRows}</tbody></table>` : '<p class="empty">No errors reported. 🎉</p>'}
  </div>
</body></html>`;
}
