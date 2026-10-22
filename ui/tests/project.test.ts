import assert from 'node:assert/strict';
import test from 'node:test';
import type { Edge, Node } from '@xyflow/react';
import {
  createProjectDocument,
  normalizeProjectName,
  parseProjectDocument,
  projectFileName,
  projectToWorkspace,
} from '../src/project.ts';

const nodes: Node[] = [
  {
    id: 'cleaner',
    type: 'custom',
    position: { x: 25, y: 40 },
    data: { kind: 'text_clean', config: { text: 'Hello' }, status: 'executed' },
  },
  {
    id: 'logger',
    type: 'custom',
    position: { x: 350, y: 40 },
    data: { kind: 'log_output', config: {}, status: 'idle' },
  },
];

const edges: Edge[] = [
  {
    id: 'edge-1',
    source: 'cleaner',
    sourceHandle: 'cleaned',
    target: 'logger',
    targetHandle: 'text',
  },
];

test('project files preserve graph data and canvas positions', () => {
  const project = createProjectDocument('First Project', nodes, edges);
  const parsed = parseProjectDocument(JSON.stringify(project));
  const restored = projectToWorkspace(parsed);

  assert.equal(parsed.name, 'First Project');
  assert.deepEqual(parsed.layout.cleaner, { x: 25, y: 40 });
  assert.equal(restored.nodes[0].data.status, 'idle');
  assert.equal(restored.edges[0].sourceHandle, 'cleaned');
  assert.equal(restored.edges[0].targetHandle, 'text');
});

test('project parser rejects malformed and dangling graph data', () => {
  assert.throws(() => parseProjectDocument('{broken'));
  assert.throws(() =>
    parseProjectDocument(
      JSON.stringify({
        format: 'rust-lab-project',
        version: 1,
        name: 'Broken',
        graph: {
          graph_version: '0.1',
          nodes: [{ id: 'one', kind: 'log_output', config: {} }],
          edges: [{ from: { node: 'missing', port: 'x' }, to: { node: 'one', port: 'text' } }],
        },
        layout: {},
      }),
    ),
  );
});

test('project names produce portable Rust Lab filenames', () => {
  assert.equal(normalizeProjectName(' Demo.rustlab.json '), 'Demo');
  assert.equal(projectFileName('My First Rust App'), 'my-first-rust-app.rustlab.json');
});
