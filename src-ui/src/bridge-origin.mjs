/** Resolve the IPC origin without fetching or opening any desktop surface. */
export function resolveBridgeOrigin({production, appOrigin, configuredOrigin}) {
  const candidate = production ? appOrigin : (configuredOrigin === undefined ? 'http://127.0.0.1:48291' : configuredOrigin);
  const match = typeof candidate === 'string' && /^http:\/\/127\.0\.0\.1:([1-9][0-9]{0,4})\/?$/.exec(candidate);
  if (!match || Number(match[1]) > 65535) {
    throw new Error('Invalid IPC bridge origin: expected http://127.0.0.1:<port>');
  }
  return `http://127.0.0.1:${match[1]}`;
}
