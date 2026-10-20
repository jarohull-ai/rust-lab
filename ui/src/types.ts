export type PinType =
  | 'string'
  | 'number'
  | 'boolean'
  | 'json'
  | 'http_response'
  | 'file_path'
  | 'any';

export interface TypedPin {
  name: string;
  pin_type: PinType;
  required: boolean;
  description: string;
}

export interface NodeSpec {
  name: string;
  version: string;
  description: string;
  inputs: TypedPin[];
  outputs: TypedPin[];
  requires_network: boolean;
  requires_fs_write: boolean;
}

export interface NodeInstance {
  id: string;
  kind: string;
  config: Record<string, any>;
  after?: string[];
}

export interface Port {
  node: string;
  port: string;
}

export interface EdgeSpec {
  from: Port;
  to: Port;
}

export interface GraphSpec {
  graph_version: string;
  nodes: NodeInstance[];
  edges: EdgeSpec[];
  security?: Record<string, unknown>;
}

export interface RustLabProject {
  format: 'rust-lab-project';
  version: 1;
  name: string;
  graph: GraphSpec;
  layout: Record<string, { x: number; y: number }>;
}

export interface ValidateResponse {
  valid: boolean;
  errors: string[];
  execution_order: string[];
}

export interface ExecutionReport {
  executed: string[];
  skipped: string[];
  outputs: Record<string, Record<string, any>>;
}

export interface ExportResponse {
  project_name: string;
  cargo_toml: string;
  main_rs: string;
  nodes_rs: string;
}

export interface CompileResponse {
  success: boolean;
  stdout: string;
  stderr: string;
}

export interface UiLog {
  id: number;
  timestamp: number;
  icon: string;
  key: string;
  params?: Record<string, string | number>;
}

export const PIN_COLORS: Record<PinType, { bg: string; border: string; text: string; hex: string }> = {
  string: { bg: 'bg-emerald-500/20', border: 'border-emerald-500', text: 'text-emerald-400', hex: '#10b981' },
  number: { bg: 'bg-blue-500/20', border: 'border-blue-500', text: 'text-blue-400', hex: '#3b82f6' },
  boolean: { bg: 'bg-purple-500/20', border: 'border-purple-500', text: 'text-purple-400', hex: '#a855f7' },
  json: { bg: 'bg-amber-500/20', border: 'border-amber-500', text: 'text-amber-400', hex: '#f59e0b' },
  http_response: { bg: 'bg-cyan-500/20', border: 'border-cyan-500', text: 'text-cyan-400', hex: '#06b6d4' },
  file_path: { bg: 'bg-orange-500/20', border: 'border-orange-500', text: 'text-orange-400', hex: '#f97316' },
  any: { bg: 'bg-slate-500/20', border: 'border-slate-400', text: 'text-slate-300', hex: '#94a3b8' },
};
