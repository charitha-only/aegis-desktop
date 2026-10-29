/** Runtime shapes for explicitly covered core commands; unknown commands stay extensible. */
const record = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === 'object' && !Array.isArray(value);
const text = (value: unknown): value is string => typeof value === 'string' && value.length > 0;
const booleans = (value: Record<string, unknown>, keys: string[]) => keys.every(key => typeof value[key] === 'boolean');
const optionalBooleans = (value: Record<string, unknown>, keys: string[]) =>
  keys.every(key => !(key in value) || typeof value[key] === 'boolean');
const positiveInteger = (value: unknown): value is number => typeof value === 'number' && Number.isSafeInteger(value) && value > 0;
const nonnegativeInteger = (value: unknown): value is number => typeof value === 'number' && Number.isSafeInteger(value) && value >= 0;

export function validCommandData(command: string, data: unknown): boolean {
  if (command === 'fs_read_file') return record(data) && text(data.path) && typeof data.content === 'string';
  if (command === 'fs_write_file') return record(data) && text(data.path) && nonnegativeInteger(data.bytesWritten);
  if (command === 'fs_list_sandbox') {
    return record(data) && Array.isArray(data.entries) && data.entries.length <= 4096
      && data.entries.every(entry => record(entry) && text(entry.name) && typeof entry.isDir === 'boolean' && nonnegativeInteger(entry.sizeBytes));
  }
  if (command === 'vault_session_lock') return record(data) && data.locked === true;
  if (command === 'vault_session_unlock') {
    return record(data) && typeof data.sessionToken === 'string' && /^[a-f0-9]{64}$/.test(data.sessionToken)
      && positiveInteger(data.idleTimeoutSeconds) && positiveInteger(data.absoluteTimeoutSeconds)
      && data.idleTimeoutSeconds <= data.absoluteTimeoutSeconds;
  }
  if (command === 'vault_session_read' || command === 'vault_decrypt_retrieve') {
    return record(data) && text(data.filename) && typeof data.decryptedContent === 'string' && data.integrityVerified === true;
  }
  if (command === 'vault_encrypt_store') {
    return record(data) && text(data.filename) && text(data.status) && positiveInteger(data.envelopeLength);
  }
  if (command === 'platform_info') {
    return record(data) && ['windows','linux','macos'].includes(data.os as string)
      && text(data.architecture) && booleans(data, ['embeddedWebview','lifecycleAdapterCompiled','installerAdapterCompiled','osKeyStore','encryptedFileStorage']);
  }
  if (command === 'platform_directories') {
    return record(data) && ['data','config','cache'].every(key => text(data[key]));
  }
  if (command === 'get_security_status') {
    return record(data) && text(data.engine) && booleans(data, ['nonceVerified','sandboxEnforced'])
      && optionalBooleans(data, ['windowsLifecycleMonitorActive','windowsSessionAvailable','embeddedWindowActive','privacyVaultActive','antiDosRateLimiterActive','airGapPolicyActive','bundlerEngineActive','multiWindowApiActive'])
      && record(data.activeCapabilities)
      && booleans(data.activeCapabilities, ['fsRead','fsWrite','shellExecution','systemInfo','crypto'])
      && optionalBooleans(data.activeCapabilities, ['notifications','encryptedVault','bundler','multiWindow','multiOsPipeline','p2pPluginMarketplace','hsmPkcs11']);
  }
  return true;
}
