#!/usr/bin/env node
import assert from 'node:assert/strict';
import net from 'node:net';
import {spawn} from 'node:child_process';
import {mkdtemp, mkdir, cp, readFile, writeFile, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';

const fixture = await mkdtemp(path.join(tmpdir(), 'aegis-ipc-deadline-'));
const executable = `aegis-core${process.platform === 'win32' ? '.exe' : ''}`;
const env = {...process.env, HOME: fixture, LOCALAPPDATA: path.join(fixture, 'local'), APPDATA: path.join(fixture, 'roaming'),
  XDG_DATA_HOME: path.join(fixture, 'data'), XDG_CONFIG_HOME: path.join(fixture, 'config'), XDG_CACHE_HOME: path.join(fixture, 'cache')};
async function boot(app) {
  const child = spawn(path.join(app, executable), ['--port=0'], {cwd: fixture, env, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe']});
  const done = new Promise(resolve => child.once('close', resolve));
  try {
    const base = await new Promise((resolve, reject) => {
      let output = '';
      const timer = setTimeout(() => reject(new Error('IPC deadline fixture startup timed out')), 15000);
      child.once('error', error => {clearTimeout(timer); reject(error);});
      child.once('exit', () => {clearTimeout(timer); reject(new Error('IPC deadline fixture exited before ready'));});
      child.stderr.resume();
      child.stdout.on('data', data => {
        output = (output + data.toString()).slice(-8192);
        const match = output.match(/IPC Bridge Server actively listening on (http:\/\/127\.0\.0\.1:\d+)/);
        if (match) {clearTimeout(timer); resolve(match[1]);}
      });
    });
    const url = new URL(base);
    const cases = [
      ['incomplete body with a slow trickle', `POST /ipc HTTP/1.1\r\nHost: ${url.host}\r\nContent-Type: application/json\r\nContent-Length: 65536\r\n\r\n{`, true],
      ['incomplete headers', `POST /ipc HTTP/1.1\r\nHost: ${url.host}\r\n`, false],
      ['incomplete small buffered body', `POST /ipc HTTP/1.1\r\nHost: ${url.host}\r\nContent-Type: application/json\r\nContent-Length: 64\r\n\r\n{`, false],
      ['oversized rejected body', `POST /ipc HTTP/1.1\r\nHost: ${url.host}\r\nContent-Type: application/json\r\nContent-Length: 4294967295\r\n\r\n{`, false],
    ];
    for (const [name, request, trickle] of cases) {
      const stalled = net.createConnection({host: url.hostname, port: Number(url.port)});
      stalled.on('error', () => {});
      stalled.resume();
      let timer, ticker;
      try {
        await new Promise((resolve, reject) => {stalled.once('connect', resolve); stalled.once('error', reject);});
        const closed = new Promise((resolve, reject) => {
          timer = setTimeout(() => reject(new Error(`${name}: connection exceeded the absolute transport budget`)), 8000);
          stalled.once('close', () => {clearTimeout(timer); resolve();});
        });
        // Attach immediately so failure while probing cannot create an unhandled rejection.
        closed.catch(() => {});
        stalled.write(request);
        if (trickle) ticker = setInterval(() => {if (!stalled.destroyed) stalled.write(' ');}, 400);
        await new Promise(resolve => setTimeout(resolve, 300));
        const response = await fetch(`${base}/handshake`, {headers: {Connection: 'close'}, signal: AbortSignal.timeout(8000)});
        assert.equal(response.status, 200, `${name}: subsequent handshake must recover`);
        assert.equal(response.headers.get('connection'), 'close', 'Clients must reconnect instead of reusing an expired ingress budget');
        assert.equal(typeof (await response.json()).token, 'string');
        await closed;
        console.log(`PASS: bounded IPC transport: ${name}`);
      } finally {clearTimeout(timer); clearInterval(ticker); stalled.destroy();}
    }

    const oversizedHeader = net.createConnection({host: url.hostname, port: Number(url.port)});
    oversizedHeader.on('error', () => {});
    oversizedHeader.resume();
    let headerTimer;
    try {
      await new Promise((resolve, reject) => {oversizedHeader.once('connect', resolve); oversizedHeader.once('error', reject);});
      const closed = new Promise((resolve, reject) => {
        headerTimer = setTimeout(() => reject(new Error('Oversized header must be rejected before the ingress deadline')), 2000);
        oversizedHeader.once('close', resolve);
      });
      oversizedHeader.write(`GET /handshake HTTP/1.1\r\nX-Large: ${'x'.repeat(9000)}`);
      await closed;
    } finally {clearTimeout(headerTimer); oversizedHeader.destroy();}
    console.log('PASS: oversized header rejected before further input');
    const held = [];
    try {
      for (let i = 0; i < 32; i++) {
        const socket = net.createConnection({host: url.hostname, port: Number(url.port)});
        held.push(socket);
        socket.on('error', () => {});
        socket.resume();
        await new Promise((resolve, reject) => {socket.once('connect', resolve); socket.once('error', reject);});
        socket.write(`GET /handshake HTTP/1.1\r\nHost: ${url.host}\r\n`);
      }
      await new Promise(resolve => setTimeout(resolve, 200));
      await assert.rejects(fetch(`${base}/handshake`, {headers: {Connection: 'close'}, signal: AbortSignal.timeout(2000)}),
        error => error.name !== 'TimeoutError',
        'Connections above the admission limit must be refused before creating more work');
    } finally {for (const socket of held) socket.destroy();}
    await new Promise(resolve => setTimeout(resolve, 300));
    assert.equal((await fetch(`${base}/handshake`, {signal: AbortSignal.timeout(8000)})).status, 200,
      'Server must recover when connection slots are released');
    console.log('PASS: bounded connection admission and slot recovery');


  } finally {
    if (child.pid && child.exitCode === null) child.kill();
    await done;
  }
}
try {
  let app = path.join(fixture, 'FirstPackageFolder');
  await mkdir(path.join(app, 'ui'), {recursive: true});
  await writeFile(path.join(app, 'ui/index.html'), '<!doctype html><title>Owned IPC deadline fixture</title>');
  await cp(path.resolve('src-core/target/release', executable), path.join(app, executable));
  const config = JSON.parse(await readFile('aegis.config.json', 'utf8'));
  config.appId = 'org.example.ipc-deadline-fixture';
  await writeFile(path.join(app, 'aegis.config.json'), JSON.stringify(config));
  await boot(app);
} finally {
  const relative = path.relative(tmpdir(), fixture);
  if (!relative.startsWith('..') && !path.isAbsolute(relative)) await rm(fixture, {recursive: true, force: true});
}
