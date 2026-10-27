export function latencySummary(samples) {
  if (!samples.length || samples.some(value => !Number.isFinite(value) || value < 0)) {
    throw new Error('Measurements must be a non-empty array of finite, nonnegative milliseconds');
  }
  const sorted = [...samples].sort((a, b) => a - b);
  const percentile = p => sorted[Math.ceil(p * sorted.length) - 1];
  return {count: sorted.length, minMs: sorted[0], p50Ms: percentile(0.50), p95Ms: percentile(0.95), maxMs: sorted.at(-1)};
}

/** Evaluate explicit core-only budgets; absent measurements never pass. */
export function evaluatePerformanceBudget(report, budget) {
  const observed={startupP95Ms:report?.startup?.p95Ms,ipcP95Ms:report?.ipc?.p95Ms,
    rssBytes:report?.idle?.rssBytes,idleCpuPercent:report?.idle?.cpuPercentOfOneLogicalCore};
  if(report?.schemaVersion !== 2 || !budget || typeof budget !== 'object' || Array.isArray(budget)
    || !Object.keys(budget).length || Object.entries(budget).some(([key,value])=>
      !Object.hasOwn(observed,key) || !Number.isFinite(value) || value<0)) {
    throw new Error('Expected a schema-2 report and nonempty known finite nonnegative performance budgets');
  }
  const checks=Object.entries(budget).map(([metric,limit])=>({metric,limit,
    observed:observed[metric] ?? null,passed:Number.isFinite(observed[metric]) && observed[metric]>=0 && observed[metric]<=limit}));
  return {passed:checks.every(check=>check.passed),checks};
}
