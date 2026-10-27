import test from 'node:test';
import assert from 'node:assert/strict';
import {latencySummary, evaluatePerformanceBudget} from '../lib/metrics.js';

test('latency report uses nearest-rank percentiles and rejects invalid measurements', () => {
  assert.deepEqual(latencySummary([40, 10, 30, 20]), {count: 4, minMs: 10, p50Ms: 20, p95Ms: 40, maxMs: 40});
  for (const samples of [[], [-1], [NaN], [Infinity]]) assert.throws(() => latencySummary(samples));
});

test('performance budgets compare measured values with explicit limits', () => {
  const result=evaluatePerformanceBudget({schemaVersion:2,startup:{p95Ms:100},ipc:{p95Ms:5},idle:{rssBytes:40000000,cpuPercentOfOneLogicalCore:2}},
    {startupP95Ms:100,ipcP95Ms:4,rssBytes:50000000,idleCpuPercent:3});
  assert.equal(result.passed,false);
  assert.deepEqual(result.checks.map(check=>check.passed),[true,false,true,true]);
  assert.equal(result.checks[1].observed,5);
  assert.equal(result.checks[1].limit,4);
});

test('invalid budgets and incompatible reports are rejected; missing measurements fail', () => {
  const report={schemaVersion:2,startup:{p95Ms:10}};
  for(const budget of [{},null,[],{unknown:1},{startupP95Ms:-1},{startupP95Ms:NaN},{startupP95Ms:'10'}]) {
    assert.throws(()=>evaluatePerformanceBudget(report,budget));
  }
  assert.throws(()=>evaluatePerformanceBudget({schemaVersion:1},{startupP95Ms:10}));
  assert.equal(evaluatePerformanceBudget(report,{rssBytes:100}).passed,false);
  assert.equal(evaluatePerformanceBudget({schemaVersion:2,idle:{rssBytes:NaN}},{rssBytes:100}).passed,false);
});
