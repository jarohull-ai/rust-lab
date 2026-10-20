/**
 * Resolves the full URL for an API endpoint.
 * In desktop mode (Tauri), requests are directed to the local desktop API at http://127.0.0.1:3030.
 * In web mode, relative paths (/api/...) are used, allowing Vite's dev proxy or the web host to route them.
 */
export function getApiUrl(path: string): string {
  const isDesktop =
    typeof window !== 'undefined' &&
    (Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__) ||
      window.location.protocol === 'tauri:' ||
      window.location.protocol === 'asset:' ||
      window.location.hostname.includes('tauri.localhost') ||
      (window.location.hostname === 'localhost' &&
        window.location.port !== '3030' &&
        window.location.port !== '5173'));

  const base = isDesktop ? 'http://127.0.0.1:3030' : '';
  const cleanPath = path.startsWith('/') ? path : `/${path}`;
  return `${base}${cleanPath}`;
}

export async function apiFetch(path: string, init?: RequestInit): Promise<Response> {
  const url = getApiUrl(path);
  const headers = new Headers(init?.headers);
  if (!headers.has('X-Requested-With')) {
    headers.set('X-Requested-With', 'Polyglot-Lab');
  }
  return fetch(url, { ...init, headers });
}

export async function apiFetchJson<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await apiFetch(path, init);
  const body = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error((body as { error?: string }).error ?? `HTTP ${response.status}`);
  }
  return body as T;
}
