/** Runtime shapes for explicitly covered core commands; unknown commands stay extensible. */
const record = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === 'object' && !Array.isArray(value);
const text = (value: unknown): value is string => typeof value === 'string' && value.length > 0;
const booleans = (value: Record<string, unknown>, keys: string[]) => keys.every(key => typeof value[key] === 'boolean');
const optionalBooleans = (value: Record<string, unknown>, keys: string[]) =>
  keys.every(key => !(key in value) || typeof value[key] === 'boolean');

export function validCommandData(command: string, data: unknown): boolean {
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
