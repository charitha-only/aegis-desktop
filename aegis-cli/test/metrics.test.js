import test from 'node:test';
import assert from 'node:assert/strict';
import {latencySummary} from '../lib/metrics.js';

test('latency report uses nearest-rank percentiles and rejects invalid measurements', () => {
  assert.deepEqual(latencySummary([40, 10, 30, 20]), {count: 4, minMs: 10, p50Ms: 20, p95Ms: 40, maxMs: 40});
  for (const samples of [[], [-1], [NaN], [Infinity]]) assert.throws(() => latencySummary(samples));
});
