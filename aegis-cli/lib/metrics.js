export function latencySummary(samples) {
  if (!samples.length || samples.some(value => !Number.isFinite(value) || value < 0)) {
    throw new Error('Measurements must be a non-empty array of finite, nonnegative milliseconds');
  }
  const sorted = [...samples].sort((a, b) => a - b);
  const percentile = p => sorted[Math.ceil(p * sorted.length) - 1];
  return {count: sorted.length, minMs: sorted[0], p50Ms: percentile(0.50), p95Ms: percentile(0.95), maxMs: sorted.at(-1)};
}
