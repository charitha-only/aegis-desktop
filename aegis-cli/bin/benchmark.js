#!/usr/bin/env node
import {spawn, spawnSync} from 'node:child_process';
import {mkdtemp, rm, stat} from 'node:fs/promises';
import {tmpdir, cpus, totalmem, release} from 'node:os';
import path from 'node:path';
import {performance} from 'node:perf_hooks';
import {latencySummary} from '../lib/metrics.js';

const binary = path.resolve(process.argv[2] || `src-core/target/release/aegis-core${process.platform === 'win32' ? '.exe' : ''}`);
const trials = 3;
async function withCore(run) {
  const fixture = await mkdtemp(path.join(tmpdir(), 'aegis-benchmark-'));
  const started = performance.now();
  const child = spawn(binary, ['--port=0'], {cwd: fixture, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe']});
  const done = new Promise(resolve => child.once('close', resolve));
  try {
    const base = await new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error('Core startup timed out')), 15000);
      let output = '';
      child.once('error', error => {clearTimeout(timer); reject(error);});
      child.once('exit', () => {clearTimeout(timer); reject(new Error('Core exited before ready'));});
      child.stderr.resume();
      child.stdout.on('data', data => {
        output = (output + data.toString()).slice(-8192);
        const match = output.match(/IPC Bridge Server actively listening on (http:\/\/127\.0\.0\.1:\d+)/);
        if (match) {clearTimeout(timer); resolve(match[1]);}
      });
    });
    return await run({base, pid: child.pid, startupMs: performance.now() - started});
  } finally {
    if (child.pid && child.exitCode === null) child.kill();
    await done;
    const relative = path.relative(tmpdir(), fixture);
    if (!relative.startsWith('..') && !path.isAbsolute(relative)) await rm(fixture, {recursive: true, force: true});
  }
}

function sampleProcess(pid) {
  if (!Number.isSafeInteger(pid) || pid <= 0) throw new Error('Invalid owned process ID');
  if (process.platform === 'win32') {
    const result = spawnSync('powershell', ['-NoProfile', '-NonInteractive', '-Command',
      `Get-Process -Id ${pid} | Select-Object WorkingSet64,PeakWorkingSet64,CPU | ConvertTo-Json -Compress`], {encoding: 'utf8', windowsHide: true, timeout: 10000});
    if (result.status !== 0) return null;
    const value = JSON.parse(result.stdout);
    return {rssBytes: value.WorkingSet64, peakRssBytes: value.PeakWorkingSet64, cpuSeconds: value.CPU || 0};
  }
  const result = spawnSync('ps', ['-o', 'rss=,time=', '-p', String(pid)], {encoding: 'utf8', timeout: 5000});
  if (result.status !== 0) return null;
  const [rss, time] = result.stdout.trim().split(/\s+/);
  const [days, clock] = time.includes('-') ? time.split('-') : ['0', time];
  const seconds = clock.split(':').reduce((sum, part) => sum * 60 + Number(part), 0) + Number(days) * 86400;
  return {rssBytes: Number(rss) * 1024, peakRssBytes: null, cpuSeconds: seconds};
}

const startup = [];
for (let i = 0; i < trials; i++) startup.push(await withCore(async core => core.startupMs));
const workload = await withCore(async ({base, pid}) => {
  const handshake = await fetch(`${base}/handshake`, {signal: AbortSignal.timeout(5000)}).then(r => r.json());
  const ipc = [];
  for (let i = 0; i < 20; i++) {
    const start = performance.now();
    const response = await fetch(`${base}/ipc`, {method: 'POST', headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({id: String(i), nonce: handshake.token, command: 'sys_info', payload: {}}), signal: AbortSignal.timeout(5000)}).then(r => r.json());
    if (!response.success) throw new Error('Benchmark IPC request failed');
    ipc.push(performance.now() - start);
  }
  const before = sampleProcess(pid);
  const start = performance.now();
  await new Promise(resolve => setTimeout(resolve, 2000));
  const after = sampleProcess(pid);
  const elapsedMs = performance.now() - start;
  return {ipc: latencySummary(ipc), idle: before && after ? {
    durationMs: elapsedMs, rssBytes: after.rssBytes, peakRssBytes: after.peakRssBytes,
    cpuPercentOfOneLogicalCore: Math.max(0, after.cpuSeconds - before.cpuSeconds) / (elapsedMs / 1000) * 100,
  } : {unavailable: true}};
});
console.log(JSON.stringify({schemaVersion: 1, measuredAt: new Date().toISOString(),
  environment: {os: process.platform, osRelease: release(), architecture: process.arch, cpu: cpus()[0]?.model, logicalCpus: cpus().length, ramBytes: totalmem()},
  binaryBytes: (await stat(binary)).size, startup: latencySummary(startup), ...workload,
  limitations: ['Core process only; no UI rendering or webview cost measured', 'OS filesystem caches are uncontrolled; not a cold-boot benchmark', 'CPU sampling is coarse and excludes child processes; this is not a release performance budget', 'Physical constrained-device acceptance remains required'],
}, null, 2));
