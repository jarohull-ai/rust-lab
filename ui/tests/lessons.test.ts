import assert from 'node:assert/strict';
import test from 'node:test';
import { createLessonSetup, evaluateLesson, nextLesson } from '../src/lessons.ts';
import type { ExecutionReport, GraphSpec } from '../src/types.ts';

function graph(nodes: GraphSpec['nodes'], edges: GraphSpec['edges']): GraphSpec {
  return { graph_version: '0.1', nodes, edges };
}

test('lesson starters are intentionally incomplete', () => {
  for (const lesson of ['text_pipeline', 'http_json', 'conditions'] as const) {
    const setup = createLessonSetup(lesson);
    const spec = graph(
      setup.nodes.map((node) => ({
        id: node.id,
        kind: String(node.data.kind),
        config: (node.data.config as Record<string, unknown>) ?? {},
      })),
      []
    );

    assert.equal(evaluateLesson(lesson, spec, null).every(Boolean), false);
  }
});

test('text lesson requires the correct connection and a successful run', () => {
  const spec = graph(
    [
      { id: 'lesson_cleaner', kind: 'text_clean', config: { text: 'hello' } },
      { id: 'lesson_logger', kind: 'log_output', config: {} },
    ],
    [
      {
        from: { node: 'lesson_cleaner', port: 'cleaned' },
        to: { node: 'lesson_logger', port: 'text' },
      },
    ]
  );
  const beforeRun = evaluateLesson('text_pipeline', spec, null);
  assert.deepEqual(beforeRun, [true, true, false]);

  const report: ExecutionReport = {
    executed: ['lesson_cleaner', 'lesson_logger'],
    skipped: [],
    outputs: {},
  };
  assert.equal(evaluateLesson('text_pipeline', spec, report).every(Boolean), true);
});

test('condition lesson recognizes explicit conversion and branch execution', () => {
  const spec = graph(
    [
      { id: 'lesson_parse_bool', kind: 'json_parse', config: { json_str: 'true' } },
      { id: 'lesson_to_bool', kind: 'json_to_bool', config: {} },
      {
        id: 'lesson_condition',
        kind: 'condition',
        config: { true_branch: 'lesson_true', false_branch: 'lesson_false' },
      },
      { id: 'lesson_true', kind: 'log_output', config: { text: 'true' } },
      { id: 'lesson_false', kind: 'log_output', config: { text: 'false' } },
    ],
    [
      {
        from: { node: 'lesson_parse_bool', port: 'value' },
        to: { node: 'lesson_to_bool', port: 'value' },
      },
      {
        from: { node: 'lesson_to_bool', port: 'value' },
        to: { node: 'lesson_condition', port: 'value' },
      },
    ]
  );
  const report: ExecutionReport = {
    executed: [
      'lesson_parse_bool',
      'lesson_to_bool',
      'lesson_condition',
      'lesson_true',
    ],
    skipped: ['lesson_false'],
    outputs: {},
  };

  assert.equal(evaluateLesson('conditions', spec, report).every(Boolean), true);
});

test('lesson order ends after the third lesson', () => {
  assert.equal(nextLesson('text_pipeline'), 'http_json');
  assert.equal(nextLesson('http_json'), 'conditions');
  assert.equal(nextLesson('conditions'), null);
});
