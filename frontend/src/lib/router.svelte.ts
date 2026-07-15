export const router = $state({ path: "/" });

function sync() {
  router.path = window.location.hash.slice(1) || "/";
}

sync();
window.addEventListener("hashchange", sync);

export function navigate(path: string) {
  window.location.hash = path;
}

export function routeParam(prefix: string): string | null {
  if (!router.path.startsWith(prefix)) return null;
  return router.path.slice(prefix.length) || null;
}
