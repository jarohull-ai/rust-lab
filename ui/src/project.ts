import type { Edge, Node } from '@xyflow/react';
import type { EdgeSpec, GraphSpec, LanguageTarget, NodeInstance, RustLabProject } from './types';

export const PROJECT_FORMAT = 'rust-lab-project' as const;
export const PROJECT_VERSION = 1 as const;
export const AUTOSAVE_KEY = 'rust-lab.project.autosave.v1';

const DEFAULT_PROJECT_NAME = 'Untitled Project';
const EDGE_STYLE = { stroke: '#38bdf8', strokeWidth: 2 };

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function cloneRecord(value: unknown): Record<string, unknown> {
  if (!isRecord(value)) return {};
  return JSON.parse(JSON.stringify(value)) as Record<string, unknown>;
}

export function normalizeProjectName(value: string): string {
  const withoutExtension = value.trim().replace(/\.rustlab(?:\.json)?$/i, '').trim();
  return (withoutExtension || DEFAULT_PROJECT_NAME).slice(0, 80);
}

export function projectFileName(name: string): string {
  const safe = normalizeProjectName(name)
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
  return `${safe || 'rust-lab-project'}.rustlab.json`;
}

export function createProjectDocument(
  name: string,
  nodes: Node[],
  edges: Edge[],
  target: LanguageTarget = 'rust',
): RustLabProject {
  const nodeIds = new Set(nodes.map((node) => node.id));
  const graphNodes: NodeInstance[] = nodes.map((node) => {
    const instance: NodeInstance = {
      id: node.id,
      kind: String(node.data.kind ?? ''),
      config: cloneRecord(node.data.config),
    };
    if (Array.isArray(node.data.after) && node.data.after.every((id) => typeof id === 'string')) {
      instance.after = [...node.data.after];
    }
    return instance;
  });
  const graphEdges: EdgeSpec[] = edges
    .filter(
      (edge) =>
        nodeIds.has(edge.source) &&
        nodeIds.has(edge.target) &&
        typeof edge.sourceHandle === 'string' &&
        typeof edge.targetHandle === 'string',
    )
    .map((edge) => ({
      from: { node: edge.source, port: edge.sourceHandle as string },
      to: { node: edge.target, port: edge.targetHandle as string },
    }));

  return {
    format: PROJECT_FORMAT,
    version: PROJECT_VERSION,
    name: normalizeProjectName(name),
    target,
    graph: {
      graph_version: '0.1',
      nodes: graphNodes,
      edges: graphEdges,
    },
    layout: Object.fromEntries(
      nodes.map((node) => [node.id, { x: node.position.x, y: node.position.y }]),
    ),
  };
}

function validateGraph(graph: unknown): asserts graph is GraphSpec {
  if (!isRecord(graph) || typeof graph.graph_version !== 'string') {
    throw new Error('Project graph is missing a valid graph_version.');
  }
  if (!Array.isArray(graph.nodes) || !Array.isArray(graph.edges)) {
    throw new Error('Project graph must contain node and edge arrays.');
  }

  const ids = new Set<string>();
  for (const rawNode of graph.nodes) {
    if (
      !isRecord(rawNode) ||
      typeof rawNode.id !== 'string' ||
      rawNode.id.length === 0 ||
      typeof rawNode.kind !== 'string' ||
      rawNode.kind.length === 0 ||
      !isRecord(rawNode.config)
    ) {
      throw new Error('Project contains an invalid node.');
    }
    if (ids.has(rawNode.id)) throw new Error(`Project contains duplicate node id: ${rawNode.id}`);
    ids.add(rawNode.id);
    if (
      rawNode.after !== undefined &&
      (!Array.isArray(rawNode.after) || !rawNode.after.every((id) => typeof id === 'string'))
    ) {
      throw new Error(`Node ${rawNode.id} contains an invalid after list.`);
    }
  }

  for (const rawEdge of graph.edges) {
    if (!isRecord(rawEdge) || !isRecord(rawEdge.from) || !isRecord(rawEdge.to)) {
      throw new Error('Project contains an invalid edge.');
    }
    const endpoints = [rawEdge.from, rawEdge.to];
    if (
      endpoints.some(
        (endpoint) =>
          typeof endpoint.node !== 'string' ||
          !ids.has(endpoint.node) ||
          typeof endpoint.port !== 'string' ||
          endpoint.port.length === 0,
      )
    ) {
      throw new Error('Project edge references an unknown node or invalid port.');
    }
  }
}

export function parseProjectDocument(text: string): RustLabProject {
  let raw: unknown;
  try {
    raw = JSON.parse(text);
  } catch {
    throw new Error('The selected file is not valid JSON.');
  }

  if (!isRecord(raw) || raw.format !== PROJECT_FORMAT || raw.version !== PROJECT_VERSION) {
    throw new Error('This is not a supported Rust Lab project file.');
  }
  if (typeof raw.name !== 'string') throw new Error('Project name is missing.');
  validateGraph(raw.graph);
  if (!isRecord(raw.layout)) throw new Error('Project layout is missing.');

  const layout: Record<string, { x: number; y: number }> = {};
  for (const [nodeId, position] of Object.entries(raw.layout)) {
    if (
      !isRecord(position) ||
      typeof position.x !== 'number' ||
      !Number.isFinite(position.x) ||
      typeof position.y !== 'number' ||
      !Number.isFinite(position.y)
    ) {
      throw new Error(`Project contains an invalid position for node ${nodeId}.`);
    }
    layout[nodeId] = { x: position.x, y: position.y };
  }

  return {
    format: PROJECT_FORMAT,
    version: PROJECT_VERSION,
    name: normalizeProjectName(raw.name),
    graph: raw.graph,
    layout,
  };
}

export function projectToWorkspace(project: RustLabProject): { nodes: Node[]; edges: Edge[] } {
  const nodes: Node[] = project.graph.nodes.map((node, index) => ({
    id: node.id,
    type: 'custom',
    position: project.layout[node.id] ?? {
      x: 80 + (index % 3) * 320,
      y: 100 + Math.floor(index / 3) * 220,
    },
    data: {
      kind: node.kind,
      config: cloneRecord(node.config),
      ...(node.after?.length ? { after: [...node.after] } : {}),
      status: 'idle',
    },
  }));

  const edges: Edge[] = project.graph.edges.map((edge, index) => ({
    id: `project-edge-${index}-${edge.from.node}-${edge.to.node}`,
    source: edge.from.node,
    sourceHandle: edge.from.port,
    target: edge.to.node,
    targetHandle: edge.to.port,
    animated: true,
    style: EDGE_STYLE,
  }));

  return { nodes, edges };
}
