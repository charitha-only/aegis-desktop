import {mkdir, cp, lstat, readdir, readFile, writeFile, unlink} from 'node:fs/promises';
import path from 'node:path';

export async function createProject(source, requestedDestination, {template='react'} = {}) {
  if (!['react','vanilla'].includes(template)) throw new Error('Supported templates: react, vanilla');
  if (!requestedDestination) throw new Error('Usage: aegis create <new-directory>');
  const destination = path.resolve(requestedDestination);
  const name = path.basename(destination);
  if (!/^[A-Za-z][A-Za-z0-9_-]{0,63}$/.test(name)) throw new Error('Project directory name must start with a letter and contain at most 64 letters, digits, underscores or hyphens');
  const relative = path.relative(source, destination);
  if (!relative || (!relative.startsWith('..') && !path.isAbsolute(relative))) throw new Error('Create the application outside the framework source directory');
  const files = ['package.json', 'LICENSE', 'aegis.config.json',
    'src-core/Cargo.toml', 'src-core/Cargo.lock', 'src-core/src', 'src-core/vendor',
    'src-ui/package.json', 'src-ui/package-lock.json', 'src-ui/tsconfig.json', 'src-ui/vite.config.ts', 'src-ui/index.html', 'src-ui/src',
    'aegis-cli/package.json', 'aegis-cli/bin', 'aegis-cli/lib', 'aegis-cli/test'];
  async function rejectLinks(location) {
    const name = path.basename(location).toLowerCase();
    if (name === '.env' || name.startsWith('.env.') || /\.(pfx|p12|pem|key)$/.test(name)
        || ['node_modules', '.git', 'target', 'dist'].includes(name)) throw new Error('Source template contains credential-shaped files or generated output');
    const metadata = await lstat(location);
    if (metadata.isSymbolicLink() || (!metadata.isFile() && !metadata.isDirectory())) throw new Error('Source template contains a linked or special file');
    if (metadata.isDirectory()) for (const entry of await readdir(location)) await rejectLinks(path.join(location, entry));
  }
  for (const file of files) await rejectLinks(path.join(source, file));
  const config = JSON.parse(await readFile(path.join(source,'aegis.config.json'),'utf8'));
  if (!config || typeof config !== 'object' || Array.isArray(config)
      || !config.window || typeof config.window !== 'object' || Array.isArray(config.window)) {
    throw new Error('Source application configuration requires an object with window settings');
  }
  await mkdir(destination);
  for (const file of files) {
    const target = path.join(destination, file);
    await mkdir(path.dirname(target), {recursive: true});
    await cp(path.join(source, file), target, {recursive: true, errorOnExist: true, force: false});
  }
  const configPath = path.join(destination, 'aegis.config.json');
  config.name = name;
  config.appId = `app.${name.toLowerCase()}`;
  config.version = '0.1.0';
  config.description = 'Desktop application built with Aegis';
  config.window.title = name;
  await writeFile(configPath, JSON.stringify(config, null, 2) + '\n');
  await writeFile(path.join(destination, 'src-ui/src/App.tsx'), `import {useState} from 'react';
import {aegis} from './aegis-bridge';
export default function App() {
  const [status, setStatus] = useState('Ready');
  async function connect() {
    const response = await aegis.getSecurityStatus();
    setStatus(response.success ? 'Core connected' : (response.error || 'Connection failed'));
  }
  return <main><h1>${name}</h1><p role="status">{status}</p><button onClick={connect}>Connect to core</button></main>;
}
`);
  await writeFile(path.join(destination, 'src-ui/src/index.css'), ':root{font-family:system-ui;color:#e2e8f0;background:#0f172a}body{margin:0}main{max-width:48rem;margin:10vh auto;padding:2rem}button{font:inherit;padding:.75rem 1rem;border:0;border-radius:.5rem;cursor:pointer}\n');
  await writeFile(path.join(destination,'src-ui/src/main.tsx'), `import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './index.css';
ReactDOM.createRoot(document.getElementById('root')!).render(<React.StrictMode><App /></React.StrictMode>);
`);
  await writeFile(path.join(destination,'src-ui/vite.config.ts'), `import {defineConfig} from 'vite';
import react from '@vitejs/plugin-react';
export default defineConfig({plugins:[react()],server:{port:5173,strictPort:true},build:{target:'esnext',outDir:'dist'}});
`);
  if (template === 'react') {
    await unlink(path.join(destination,'src-ui/src/main.ts')).catch(error => {if(error.code !== 'ENOENT') throw error;});
  }
  if (template === 'vanilla') {
    await unlink(path.join(destination,'src-ui/src/App.tsx'));
    await unlink(path.join(destination,'src-ui/src/main.tsx'));
    await writeFile(path.join(destination,'src-ui/src/main.ts'), `import {aegis} from './aegis-bridge';
import './index.css';
const root = document.getElementById('root');
if (!root) throw new Error('Missing application root');
const main = document.createElement('main');
const title = document.createElement('h1');
title.textContent = '${name}';
const status = document.createElement('p');
status.setAttribute('role', 'status');
status.textContent = 'Ready';
const connect = document.createElement('button');
connect.textContent = 'Connect to core';
connect.addEventListener('click', async () => {
  connect.disabled = true;
  status.textContent = 'Connecting…';
  try {
    const response = await aegis.getSecurityStatus();
    status.textContent = response.success ? 'Core connected' : (response.error || 'Connection failed');
  } catch {status.textContent = 'Connection failed';}
  finally {connect.disabled = false;}
});
main.append(title, status, connect);
root.append(main);
`);
    await writeFile(path.join(destination,'src-ui/vite.config.ts'), `import {defineConfig} from 'vite';
export default defineConfig({server:{port:5173,strictPort:true},build:{target:'esnext',outDir:'dist'}});
`);
  }
  await writeFile(path.join(destination,'src-ui/index.html'), `<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"><title>${name}</title></head>
<body><div id="root"></div><script type="module" src="/src/${template==='vanilla'?'main.ts':'main.tsx'}"></script></body></html>
`);
  await writeFile(path.join(destination, '.gitignore'), 'node_modules/\nsrc-ui/dist/\nsrc-core/target/\ndist-bundle/\nsandbox*/\nWebViewCache/\n.env\n.env.*\n*.pfx\n*.p12\n*.pem\n*.key\n');
  await writeFile(path.join(destination, 'README.md'), `# ${name}\n\nSource project generated by Aegis (${template} starter). The vanilla starter uses the DOM and shared SDK without a React renderer; the shared development dependency lockfile still includes React tooling.\n\nRun \`npm run setup\`, then \`npm run doctor\` to check prerequisites and \`npm run dev\`. Use \`npm run build\` for an optimized core/UI and \`npm run bundle\` for a native portable directory. Use \`npm run package\` for locked release builds plus a verified portable file inventory. Existing bundle output is never overwritten; move it before rebuilding. The inventory is not publisher authentication. Rust and Node.js must be installed.\n\nKeep the configured appId unchanged across releases so package renames preserve the selected data location. The current UI opens in an external browser. Optional embedded windows require the native-desktop feature and platform dependencies; OS key stores and several native integrations remain unavailable. Windows installer creation needs Inno Setup; Linux/macOS portable output is not a signed native installer. Review the framework security and platform documentation before production use.\n`);
  return destination;
}
