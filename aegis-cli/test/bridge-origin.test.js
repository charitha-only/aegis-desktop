import test from 'node:test';
import assert from 'node:assert/strict';
import {resolveBridgeOrigin} from '../../src-ui/src/bridge-origin.mjs';

test('development clients can select a separate explicit loopback core origin', () => {
  assert.equal(resolveBridgeOrigin({production: false, appOrigin: 'http://localhost:5173', configuredOrigin: 'http://127.0.0.1:49321'}),
    'http://127.0.0.1:49321');
  assert.equal(resolveBridgeOrigin({production: false}), 'http://127.0.0.1:48291');
});

test('configured bridge addresses must be explicit IPv4 loopback HTTP origins', () => {
  for (const configuredOrigin of [
    'https://example.com', 'http://example.com:49321', 'http://localhost:49321',
    'http://127.0.0.1:0', 'http://127.0.0.1:65536', 'http://127.0.0.1:049321',
    'http://127.0.0.1:49321/private', 'http://127.0.0.1:49321?target=x',
    'http://127.0.0.1:49321#fragment', 'http://user:pass@127.0.0.1:49321',
    'http://127.0.0.1:49321@evil.example', 'http://127.1:49321', '', null, 49321,
  ]) {
    assert.throws(() => resolveBridgeOrigin({production: false, configuredOrigin}), /Invalid IPC bridge origin/);
  }
});

test('packaged clients remain on their own origin and ignore development overrides', () => {
  assert.equal(resolveBridgeOrigin({production: true, appOrigin: 'http://127.0.0.1:61234', configuredOrigin: 'https://untrusted.example'}),
    'http://127.0.0.1:61234');
  assert.equal(resolveBridgeOrigin({production: false, configuredOrigin: 'http://127.0.0.1:49321/'}), 'http://127.0.0.1:49321');
  for (const appOrigin of [undefined, 'file://', 'https://untrusted.example', 'http://localhost:5173']) {
    assert.throws(() => resolveBridgeOrigin({production: true, appOrigin}), /Invalid IPC bridge origin/);
  }
});
