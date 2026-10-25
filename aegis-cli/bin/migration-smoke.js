#!/usr/bin/env node
// Encrypted, synthetic installer migration fixture. Never log passwords or plaintext.
const base = process.env.AEGIS_BRIDGE_URL;
const password = process.env.AEGIS_MIGRATION_TEST_PASSWORD;
const mode = process.argv[2];
if (!base || !password || !['seed', 'verify'].includes(mode)) throw Error('Isolated migration fixture configuration required');
const {token} = await (await fetch(base + '/handshake')).json();
let sequence = 0;
async function call(command, payload = {}) {
  return (await fetch(base + '/ipc', {method:'POST',headers:{'Content-Type':'application/json',Origin:base},
    body:JSON.stringify({id:String(++sequence),command,payload,nonce:token})})).json();
}
const status = await call('get_security_status');
if (!status.success || !status.data.windowsLifecycleMonitorActive ||
    !status.data.engine.includes(process.env.AEGIS_EXPECTED_RUNTIME_VERSION)) throw Error('Migration runtime version/monitor mismatch');
const filename = 'installer-migration.vault';
if (mode === 'seed') {
  for (const content of ['FIRST_TEST_GENERATION', 'PRESERVED_TEST_GENERATION']) {
    if (!(await call('vault_encrypt_store', {filename,content,passphrase:password})).success) throw Error('Migration seed failed');
  }
} else {
  const read = await call('vault_decrypt_retrieve', {filename,passphrase:password});
  if (!read.success || read.data?.decryptedContent !== 'PRESERVED_TEST_GENERATION') throw Error('Migration changed vault contents');
  if (!(await call('vault_restore_backup', {filename,passphrase:password})).success) throw Error('Migrated backup authentication failed');
  const restored = await call('vault_decrypt_retrieve', {filename,passphrase:password});
  if (!restored.success || restored.data?.decryptedContent !== 'FIRST_TEST_GENERATION') throw Error('Migrated backup content mismatch');
  if (!(await call('vault_encrypt_store', {filename,content:'PRESERVED_TEST_GENERATION',passphrase:password})).success) throw Error('Cannot replace migrated vault');
}
console.log('PASS: encrypted installer migration ' + mode);
