/** Runtime shapes for explicitly covered core commands; unknown commands stay extensible. */
const record = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === 'object' && !Array.isArray(value);
const text = (value: unknown): value is string => typeof value === 'string' && value.length > 0;
const booleans = (value: Record<string, unknown>, keys: string[]) => keys.every(key => typeof value[key] === 'boolean');
const optionalBooleans = (value: Record<string, unknown>, keys: string[]) =>
  keys.every(key => !(key in value) || typeof value[key] === 'boolean');
const positiveInteger = (value: unknown): value is number => typeof value === 'number' && Number.isSafeInteger(value) && value > 0;
const nonnegativeInteger = (value: unknown): value is number => typeof value === 'number' && Number.isSafeInteger(value) && value >= 0;

const managedWindowId = (value: unknown): value is string => typeof value === 'string' && /^child-[1-9][0-9]{0,19}$/.test(value);

export function validCommandData(command: string, data: unknown): boolean {
  if (command === 'window_list') return record(data) && data.responseKind === 'capabilities'
    && booleans(data, ['multiWindowSupported','managedWindowEnumerationAvailable','managedWindowCloseAvailable','externalBrowserLaunchAvailable'])
    && text(data.apiVersion) && text(data.platformHint) && Array.isArray(data.spawnMethods) && data.spawnMethods.every(text)
    && record(data.capabilities) && booleans(data.capabilities, ['modal','kiosk','alwaysOnTop','customSize','customUrl']);
  if (command === 'managed_window_focus') return record(data) && managedWindowId(data.windowId) && data.action === 'focus' && data.requested === true && data.stateConfirmed === false;
  if (command === 'managed_window_set_minimized') return record(data) && managedWindowId(data.windowId) && data.action === 'setMinimized' && data.requested === true && typeof data.requestedMinimized === 'boolean' && data.stateConfirmed === false;
  if (command === 'managed_window_set_maximized') return record(data) && managedWindowId(data.windowId) && data.action === 'setMaximized' && data.requested === true && typeof data.requestedMaximized === 'boolean' && data.stateConfirmed === false;
  if (command === 'managed_window_set_fullscreen') return record(data) && managedWindowId(data.windowId) && data.action === 'setFullscreen' && data.requested === true && typeof data.requestedFullscreen === 'boolean' && data.stateConfirmed === false;
  if (command === 'managed_window_set_title') return record(data) && managedWindowId(data.windowId) && data.action === 'setTitle' && data.requested === true && text(data.title) && data.stateConfirmed === false;
  if (command === 'managed_window_list') return record(data) && Array.isArray(data.windowIds) && data.windowIds.length <= 8
    && data.windowIds.every(managedWindowId) && new Set(data.windowIds).size === data.windowIds.length;
  if (command === 'managed_window_close') return record(data) && managedWindowId(data.windowId) && data.closed === true;
  if (command === 'managed_window_create') return record(data) && managedWindowId(data.windowId) && data.backend === 'NATIVE_EMBEDDED' && data.documentReady === false;
  if (command === 'fs_read_file') return record(data) && text(data.path) && typeof data.content === 'string';
  if (command === 'fs_write_file') return record(data) && text(data.path) && nonnegativeInteger(data.bytesWritten);
  if (command === 'fs_file_hash') return record(data) && text(data.path) && text(data.sha256);
  if (command === 'fs_verify_integrity') return record(data) && text(data.path) && typeof data.verified === 'boolean';
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
  if (command === 'handshake_rotate') {
    return record(data) && data.rotated === true && typeof data.sessionToken === 'string' && nonnegativeInteger(data.createdAt);
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
  if (command === 'updater_status') {
    return record(data) && text(data.autoUpdaterVersion) && text(data.currentVersion)
      && text(data.stagingDir) && typeof data.hasStagedUpdates === 'boolean'
      && Array.isArray(data.stagedFiles) && text(data.verificationAlgorithm)
      && text(data.securityPolicy) && typeof data.airGapCompatible === 'boolean'
      && typeof data.autoApply === 'boolean';
  }
  if (command === 'updater_check') {
    return record(data) && ['update_available', 'up_to_date', 'parse_error'].includes(data.status as string);
  }
  if (command === 'updater_verify') {
    return record(data) && typeof data.verified === 'boolean' && text(data.filePath) && text(data.sha256);
  }
  if (command === 'updater_journal') {
    return record(data) && ['IDLE', 'STAGED', 'VERIFIED', 'APPLIED', 'ROLLED_BACK'].includes(data.state as string)
      && text(data.active_version);
  }
  if (command === 'updater_rollback') {
    return record(data) && data.rolledBack === true;
  }
  if (command === 'updater_apply') {
    return record(data) && data.applied === true && text(data.targetVersion);
  }
  if (command === 'updater_stage') {
    return record(data) && text(data.staged_path) && text(data.version) && typeof data.ready_to_apply === 'boolean';
  }
  return true;
}
