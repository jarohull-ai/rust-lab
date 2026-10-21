/**
 * Desktop sessions receive an instance-specific loopback URL and secret from
 * the trusted Tauri command channel. Web development keeps relative URLs.
 */
interface ApiSession {
  baseUrl: string;
  token: string;
}

function isDesktopRuntime(): boolean {
  return (
    typeof window !== 'undefined' &&
    (Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__) ||
      window.location.protocol === 'tauri:' ||
      window.location.protocol === 'asset:' ||
      window.location.hostname.includes('tauri.localhost') ||
      (window.location.hostname === 'localhost' &&
        window.location.port !== '3030' &&
        window.location.port !== '5173'))
  );
}

let desktopSession: Promise<ApiSession> | undefined;

async function getDesktopSession(): Promise<ApiSession | undefined> {
  if (!isDesktopRuntime()) return undefined;
  const tauri = (
    window as unknown as {
      __TAURI__?: { core?: { invoke?: <T>(command: string) => Promise<T> } };
    }
  ).__TAURI__;
  const invoke = tauri?.core?.invoke;
  if (!invoke) {
    throw new Error('The trusted Tauri command channel is unavailable');
  }
  desktopSession ??= invoke<ApiSession>('api_session');
  return desktopSession;
}

export function getApiUrl(path: string, base = ''): string {
  const cleanPath = path.startsWith('/') ? path : `/${path}`;
  return `${base}${cleanPath}`;
}

export async function apiFetch(path: string, init?: RequestInit): Promise<Response> {
  const session = await getDesktopSession();
  const url = getApiUrl(path, session?.baseUrl);
  const headers = new Headers(init?.headers);
  if (!headers.has('X-Requested-With')) {
    headers.set('X-Requested-With', 'Polyglot-Lab');
  }
  const documentToken =
    typeof document === 'undefined'
      ? undefined
      : document
          .querySelector<HTMLMetaElement>('meta[name="rust-lab-token"]')
          ?.content.trim();
  const token = session?.token ?? documentToken ?? import.meta.env.VITE_RUST_LAB_API_TOKEN;
  if (token && !headers.has('X-Lab-Token')) {
    headers.set('X-Lab-Token', token);
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
