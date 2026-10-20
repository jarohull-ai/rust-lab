import type { Edge, Node } from '@xyflow/react';
import type { ExecutionReport, GraphSpec, LanguageTarget } from './types';

export const LESSON_IDS = [
  'text_pipeline',
  'http_json',
  'conditions',
  'file_read',
  'file_write',
] as const;
export type LessonId = (typeof LESSON_IDS)[number];

export interface LessonSetup {
  nodes: Node[];
  edges: Edge[];
}

export function createLessonSetup(lesson: LessonId, target: LanguageTarget = 'rust'): LessonSetup {
  let sampleText = '   LEARN RUST BY BUILDING   ';
  if (target === 'go') sampleText = '   LEARN GO WITH GOROUTINES   ';
  if (target === 'python') sampleText = '   LEARN PYTHON ASYNC PIPELINES   ';
  if (target === 'cpp') sampleText = '   LEARN C++20 MODERN PIPELINES   ';

  if (lesson === 'text_pipeline') {
    return {
      nodes: [
        {
          id: 'lesson_cleaner',
          type: 'custom',
          position: { x: 160, y: 180 },
          data: {
            kind: 'text_clean',
            config: { text: sampleText, trim: true, to_lower: true },
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

  if (lesson === 'conditions') {
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

  if (lesson === 'file_read') {
    return {
      nodes: [
        {
          id: 'lesson_read_path',
          type: 'custom',
          position: { x: 60, y: 180 },
          data: {
            kind: 'string_to_path',
            config: { path: 'data/sample.txt' },
            status: 'idle',
          },
        },
        {
          id: 'lesson_reader',
          type: 'custom',
          position: { x: 370, y: 180 },
          data: { kind: 'fs_read', config: {}, status: 'idle' },
        },
        {
          id: 'lesson_file_cleaner',
          type: 'custom',
          position: { x: 680, y: 180 },
          data: { kind: 'text_clean', config: { to_upper: true }, status: 'idle' },
        },
        {
          id: 'lesson_file_logger',
          type: 'custom',
          position: { x: 990, y: 180 },
          data: { kind: 'log_output', config: {}, status: 'idle' },
        },
      ],
      edges: [],
    };
  }

  return {
    nodes: [
      {
        id: 'lesson_write_path',
        type: 'custom',
        position: { x: 80, y: 180 },
        data: {
          kind: 'string_to_path',
          config: { path: 'output/lesson-result.txt' },
          status: 'idle',
        },
      },
      {
        id: 'lesson_write_text',
        type: 'custom',
        position: { x: 390, y: 180 },
        data: {
          kind: 'text_clean',
          config: { text: '  MY FIRST RUST LAB FILE  ', trim: true, to_lower: true },
          status: 'idle',
        },
      },
      {
        id: 'lesson_writer',
        type: 'custom',
        position: { x: 720, y: 180 },
        data: { kind: 'fs_write', config: {}, status: 'idle' },
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

  if (lesson === 'file_read') {
    return [
      hasNode(graph, 'lesson_read_path', 'string_to_path') &&
        hasNode(graph, 'lesson_reader', 'fs_read') &&
        hasNode(graph, 'lesson_file_cleaner', 'text_clean') &&
        hasNode(graph, 'lesson_file_logger', 'log_output'),
      hasEdge(graph, 'lesson_read_path', 'path', 'lesson_reader', 'path'),
      hasEdge(graph, 'lesson_reader', 'content', 'lesson_file_cleaner', 'text'),
      hasEdge(graph, 'lesson_file_cleaner', 'cleaned', 'lesson_file_logger', 'text'),
      executed(report, [
        'lesson_read_path',
        'lesson_reader',
        'lesson_file_cleaner',
        'lesson_file_logger',
      ]),
    ];
  }

  if (lesson === 'file_write') {
    return [
      hasNode(graph, 'lesson_write_path', 'string_to_path') &&
        hasNode(graph, 'lesson_write_text', 'text_clean') &&
        hasNode(graph, 'lesson_writer', 'fs_write'),
      hasEdge(graph, 'lesson_write_path', 'path', 'lesson_writer', 'path'),
      hasEdge(graph, 'lesson_write_text', 'cleaned', 'lesson_writer', 'content'),
      executed(report, ['lesson_write_path', 'lesson_write_text', 'lesson_writer']),
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
