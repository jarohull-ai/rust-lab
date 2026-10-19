import type { Edge, Node } from '@xyflow/react';
import type { ExecutionReport, GraphSpec } from './types';

export const LESSON_IDS = ['text_pipeline', 'http_json', 'conditions'] as const;
export type LessonId = (typeof LESSON_IDS)[number];

export interface LessonSetup {
  nodes: Node[];
  edges: Edge[];
}

export function createLessonSetup(lesson: LessonId): LessonSetup {
  if (lesson === 'text_pipeline') {
    return {
      nodes: [
        {
          id: 'lesson_cleaner',
          type: 'custom',
          position: { x: 160, y: 180 },
          data: {
            kind: 'text_clean',
            config: { text: '   LEARN RUST BY BUILDING   ', trim: true, to_lower: true },
            status: 'idle',
          },
        },
        {
          id: 'lesson_logger',
          type: 'custom',
          position: { x: 520, y: 180 },
          data: { kind: 'log_output', config: {}, status: 'idle' },
        },
      ],
      edges: [],
    };
  }

  if (lesson === 'http_json') {
    return {
      nodes: [
        {
          id: 'lesson_fetch',
          type: 'custom',
          position: { x: 80, y: 180 },
          data: {
            kind: 'http_get',
            config: { url: 'https://httpbin.org/json' },
            status: 'idle',
          },
        },
        {
          id: 'lesson_parse',
          type: 'custom',
          position: { x: 390, y: 180 },
          data: {
            kind: 'json_parse',
            config: { path: 'slideshow.title' },
            status: 'idle',
          },
        },
        {
          id: 'lesson_convert',
          type: 'custom',
          position: { x: 700, y: 180 },
          data: { kind: 'json_to_string', config: {}, status: 'idle' },
        },
        {
          id: 'lesson_logger',
          type: 'custom',
          position: { x: 1010, y: 180 },
          data: { kind: 'log_output', config: {}, status: 'idle' },
        },
      ],
      edges: [],
    };
  }

  return {
    nodes: [
      {
        id: 'lesson_parse_bool',
        type: 'custom',
        position: { x: 80, y: 180 },
        data: {
          kind: 'json_parse',
          config: { json_str: 'true' },
          status: 'idle',
        },
      },
      {
        id: 'lesson_to_bool',
        type: 'custom',
        position: { x: 390, y: 180 },
        data: { kind: 'json_to_bool', config: {}, status: 'idle' },
      },
      {
        id: 'lesson_condition',
        type: 'custom',
        position: { x: 700, y: 180 },
        data: {
          kind: 'condition',
          config: { true_branch: '', false_branch: '' },
          status: 'idle',
        },
      },
      {
        id: 'lesson_true',
        type: 'custom',
        position: { x: 1040, y: 70 },
        data: {
          kind: 'log_output',
          config: { text: 'The condition is true.' },
          status: 'idle',
        },
      },
      {
        id: 'lesson_false',
        type: 'custom',
        position: { x: 1040, y: 310 },
        data: {
          kind: 'log_output',
          config: { text: 'The condition is false.' },
          status: 'idle',
        },
      },
    ],
    edges: [],
  };
}

function hasNode(graph: GraphSpec, id: string, kind: string): boolean {
  return graph.nodes.some((node) => node.id === id && node.kind === kind);
}

function hasEdge(
  graph: GraphSpec,
  fromNode: string,
  fromPort: string,
  toNode: string,
  toPort: string
): boolean {
  return graph.edges.some(
    (edge) =>
      edge.from.node === fromNode &&
      edge.from.port === fromPort &&
      edge.to.node === toNode &&
      edge.to.port === toPort
  );
}

function executed(report: ExecutionReport | null, nodeIds: string[]): boolean {
  return !!report && nodeIds.every((id) => report.executed.includes(id));
}

export function evaluateLesson(
  lesson: LessonId,
  graph: GraphSpec,
  report: ExecutionReport | null
): boolean[] {
  if (lesson === 'text_pipeline') {
    return [
      hasNode(graph, 'lesson_cleaner', 'text_clean') &&
        hasNode(graph, 'lesson_logger', 'log_output'),
      hasEdge(graph, 'lesson_cleaner', 'cleaned', 'lesson_logger', 'text'),
      executed(report, ['lesson_cleaner', 'lesson_logger']),
    ];
  }

  if (lesson === 'http_json') {
    return [
      hasNode(graph, 'lesson_fetch', 'http_get') &&
        hasNode(graph, 'lesson_parse', 'json_parse') &&
        hasNode(graph, 'lesson_convert', 'json_to_string') &&
        hasNode(graph, 'lesson_logger', 'log_output'),
      hasEdge(graph, 'lesson_fetch', 'body', 'lesson_parse', 'json_str'),
      hasEdge(graph, 'lesson_parse', 'value', 'lesson_convert', 'value'),
      hasEdge(graph, 'lesson_convert', 'text', 'lesson_logger', 'text'),
      executed(report, [
        'lesson_fetch',
        'lesson_parse',
        'lesson_convert',
        'lesson_logger',
      ]),
    ];
  }

  const condition = graph.nodes.find((node) => node.id === 'lesson_condition');
  const branchTargetsAreSet =
    condition?.config.true_branch === 'lesson_true' &&
    condition?.config.false_branch === 'lesson_false';
  const branchExecuted =
    !!report &&
    report.executed.includes('lesson_condition') &&
    report.executed.includes('lesson_true') &&
    report.skipped.includes('lesson_false');

  return [
    hasNode(graph, 'lesson_parse_bool', 'json_parse') &&
      hasNode(graph, 'lesson_to_bool', 'json_to_bool') &&
      hasNode(graph, 'lesson_condition', 'condition'),
    hasEdge(graph, 'lesson_parse_bool', 'value', 'lesson_to_bool', 'value'),
    hasEdge(graph, 'lesson_to_bool', 'value', 'lesson_condition', 'value'),
    branchTargetsAreSet,
    branchExecuted,
  ];
}

export function nextLesson(lesson: LessonId): LessonId | null {
  const index = LESSON_IDS.indexOf(lesson);
  return LESSON_IDS[index + 1] ?? null;
}
