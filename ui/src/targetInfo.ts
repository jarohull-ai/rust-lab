import { LanguageTarget } from './types';

export interface NodeTargetMeta {
  lib: string;
  signature: string;
  badge: string;
}

export const NODE_TARGET_META: Record<string, Record<LanguageTarget, NodeTargetMeta>> = {
  http_get: {
    rust: { lib: 'reqwest', signature: 'reqwest::get', badge: 'reqwest' },
    go: { lib: 'net/http', signature: 'http.Get', badge: 'net/http' },
    python: { lib: 'urllib.request', signature: 'urllib.request.urlopen', badge: 'urllib' },
    cpp: { lib: 'cpr / curl', signature: 'cpr::Get', badge: 'cpr' },
  },
  fs_read: {
    rust: { lib: 'std::fs', signature: 'fs::read_to_string', badge: 'std::fs' },
    go: { lib: 'os', signature: 'os.ReadFile', badge: 'os' },
    python: { lib: 'builtins', signature: 'open(path, "r")', badge: 'builtins' },
    cpp: { lib: '<fstream>', signature: 'std::ifstream', badge: '<fstream>' },
  },
  fs_write: {
    rust: { lib: 'std::fs', signature: 'fs::write', badge: 'std::fs' },
    go: { lib: 'os', signature: 'os.WriteFile', badge: 'os' },
    python: { lib: 'builtins', signature: 'open(path, "w")', badge: 'builtins' },
    cpp: { lib: '<fstream>', signature: 'std::ofstream', badge: '<fstream>' },
  },
  log_output: {
    rust: { lib: 'std', signature: 'println!', badge: 'println!' },
    go: { lib: 'fmt', signature: 'fmt.Println', badge: 'fmt' },
    python: { lib: 'builtins', signature: 'print()', badge: 'print' },
    cpp: { lib: '<iostream>', signature: 'std::cout <<', badge: '<iostream>' },
  },
  json_parse: {
    rust: { lib: 'serde_json', signature: 'serde_json::from_str', badge: 'serde_json' },
    go: { lib: 'encoding/json', signature: 'json.Unmarshal', badge: 'encoding/json' },
    python: { lib: 'json', signature: 'json.loads', badge: 'json' },
    cpp: { lib: 'nlohmann_json', signature: 'json::parse', badge: 'nlohmann_json' },
  },
  text_clean: {
    rust: { lib: 'std::str', signature: '.trim() / .to_lowercase()', badge: 'std::str' },
    go: { lib: 'strings', signature: 'strings.TrimSpace', badge: 'strings' },
    python: { lib: 'str', signature: '.strip().lower()', badge: 'str' },
    cpp: { lib: '<algorithm>', signature: 'std::transform', badge: '<string>' },
  },
  condition: {
    rust: { lib: 'core', signature: 'if / else', badge: 'if/else' },
    go: { lib: 'builtin', signature: 'if / else', badge: 'if/else' },
    python: { lib: 'builtin', signature: 'if / else', badge: 'if/else' },
    cpp: { lib: 'core', signature: 'if / else', badge: 'if/else' },
  },
  delay: {
    rust: { lib: 'tokio::time', signature: 'tokio::time::sleep', badge: 'tokio' },
    go: { lib: 'time', signature: 'time.Sleep', badge: 'time' },
    python: { lib: 'asyncio', signature: 'asyncio.sleep', badge: 'asyncio' },
    cpp: { lib: '<chrono>', signature: 'std::this_thread::sleep_for', badge: '<chrono>' },
  },
  string_to_path: {
    rust: { lib: 'std::path', signature: 'PathBuf::from', badge: 'PathBuf' },
    go: { lib: 'path/filepath', signature: 'filepath.Clean', badge: 'filepath' },
    python: { lib: 'pathlib', signature: 'pathlib.Path', badge: 'pathlib' },
    cpp: { lib: '<filesystem>', signature: 'std::filesystem::path', badge: '<filesystem>' },
  },
  json_to_string: {
    rust: { lib: 'serde_json', signature: 'serde_json::to_string', badge: 'serde_json' },
    go: { lib: 'encoding/json', signature: 'json.Marshal', badge: 'encoding/json' },
    python: { lib: 'json', signature: 'json.dumps', badge: 'json' },
    cpp: { lib: 'nlohmann_json', signature: '.dump()', badge: 'nlohmann_json' },
  },
  json_to_bool: {
    rust: { lib: 'serde_json', signature: 'Value::as_bool', badge: 'serde_json' },
    go: { lib: 'strconv', signature: 'strconv.ParseBool', badge: 'strconv' },
    python: { lib: 'builtins', signature: 'bool()', badge: 'bool' },
    cpp: { lib: 'nlohmann_json', signature: '.get<bool>()', badge: 'nlohmann_json' },
  },
};

export function getNodeTargetMeta(kind: string, target: LanguageTarget): NodeTargetMeta {
  return NODE_TARGET_META[kind]?.[target] || {
    lib: target,
    signature: kind,
    badge: target,
  };
}
