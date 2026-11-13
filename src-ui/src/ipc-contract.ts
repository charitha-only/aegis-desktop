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
  if (command === 'updater_stage' || command === 'updater_stage_bytes') {
    return record(data) && text(data.staged_path) && text(data.version) && typeof data.ready_to_apply === 'boolean';
  }
  if (command === 'plugin_manifest_list') {
    return record(data) && Array.isArray(data.manifests) && typeof data.count === 'number';
  }
  if (command === 'plugin_manifest_validate') {
    return record(data) && typeof data.valid === 'boolean';
  }
  if (command === 'plugin_manifest_register') {
    return record(data) && data.registered === true && text(data.pluginId);
  }
  if (command === 'plugin_sandboxed_execute') {
    return record(data) && text(data.pluginId) && text(data.action) && 'result' in data;
  }
  if (command === 'banking_screen_protection_set' || command === 'banking_screen_protection_status') {
    return record(data) && typeof data.protected === 'boolean' && text(data.platform);
  }
  if (command === 'banking_biometric_hardware_status') {
    return record(data) && typeof data.biometric_enrolled === 'boolean';
  }
  if (command === 'banking_transaction_propose' || command === 'banking_transaction_approve' || command === 'banking_transaction_reject') {
    return record(data) && text(data.tx_id) && text(data.maker_id);
  }
  if (command === 'banking_ledger_verify') {
    return record(data) && typeof data.verified === 'boolean';
  }
  if (command === 'rasp_evaluate') {
    return record(data) && typeof data.debugger_detected === 'boolean' && typeof data.memory_hooks_detected === 'boolean'
      && typeof data.fail_closed_containment_active === 'boolean' && ['SECURE', 'ELEVATED_RISK', 'COMPROMISED_CONTAINED'].includes(data.system_integrity_tier as string);
  }
  if (command === 'rasp_containment_reset') {
    return record(data) && text(data.status);
  }
  if (command === 'network_pinning_evaluate') {
    return record(data) && typeof data.pin_verified === 'boolean' && typeof data.mitm_interception_detected === 'boolean'
      && typeof data.mtls_client_authenticated === 'boolean' && text(data.observed_spki_pin);
  }
  if (command === 'network_pinning_enroll') {
    return record(data) && text(data.status) && text(data.host);
  }
  if (command === 'fido2_token_enumerate') {
    return record(data) && Array.isArray(data.devices) && typeof data.count === 'number';
  }
  if (command === 'fido2_token_register') {
    return record(data) && text(data.credential_id) && text(data.public_key_hex);
  }
  if (command === 'fido2_token_sign') {
    return record(data) && text(data.credential_id) && text(data.signature_hex) && typeof data.user_present === 'boolean';
  }
  if (command === 'fido2_token_verify') {
    return record(data) && typeof data.verified === 'boolean';
  }
  if (command === 'p2p_sync_init') {
    return record(data) && text(data.session_id) && text(data.initiator_public_hex);
  }
  if (command === 'p2p_sync_complete') {
    return record(data) && text(data.session_id) && typeof data.session_established === 'boolean';
  }
  if (command === 'p2p_sync_encrypt') {
    return record(data) && text(data.session_id) && text(data.ciphertext_hex) && text(data.auth_tag_hex);
  }
  if (command === 'p2p_sync_decrypt') {
    return record(data) && typeof data.verified === 'boolean' && text(data.plaintextHex);
  }
  // ── v1.3.0 Enterprise Pillars Contracts ────────────────────────────────────
  if (command === 'db_query') {
    return record(data) && text(data.table) && Array.isArray(data.rows) && typeof data.count === 'number';
  }
  if (command === 'db_insert' || command === 'db_update') {
    return record(data) && text(data.status) && text(data.table);
  }
  if (command === 'db_stats') {
    return record(data) && typeof data.version === 'number' && typeof data.total_tables === 'number';
  }
  if (command === 'db_transaction') {
    return record(data) && text(data.status);
  }
  if (command === 'pcr_enclave_status') {
    return record(data) && text(data.tpm_version) && Array.isArray(data.pcr_registers);
  }
  if (command === 'pcr_enclave_seal' || command === 'pcr_enclave_unseal') {
    return record(data) && text(data.packageId || (data as any).package_id);
  }
  if (command === 'pcr_enclave_tamper_sim') {
    return record(data) && (data as any).tampered === true;
  }
  if (command === 'rbac_identity' || command === 'rbac_switch_role') {
    return record(data) && text((data as any).user_id) && text((data as any).display_name);
  }
  if (command === 'rbac_check') {
    return record(data) && typeof (data as any).allowed === 'boolean';
  }
  if (command === 'rbac_matrix') {
    return record(data) && Array.isArray((data as any).roles);
  }
  if (command === 'rbac_audit') {
    return record(data) && Array.isArray((data as any).auditLog);
  }
  if (command === 'traffic_recent') {
    return record(data) && Array.isArray((data as any).flows);
  }
  if (command === 'traffic_stats') {
    return record(data) && typeof (data as any).total_flows_inspected === 'number';
  }
  if (command === 'traffic_killswitch') {
    return record(data) && typeof (data as any).killswitchActive === 'boolean';
  }
  if (command === 'traffic_simulate') {
    return record(data) && text((data as any).target_host);
  }
  if (command === 'compliance_generate') {
    return record(data) && record((data as any).payload) && text((data as any).payload_sha256) && text((data as any).signature_hex);
  }
  if (command === 'compliance_verify') {
    return record(data) && typeof (data as any).verified === 'boolean';
  }
  if (command === 'updater_delta_verify') {
    return record(data) && typeof (data as any).valid === 'boolean';
  }
  if (command === 'updater_delta_create') {
    return record(data) && text((data as any).magic);
  }
  if (command === 'updater_delta_apply') {
    return record(data) && (data as any).success === true;
  }
  if (command === 'pq_kem_keypair') {
    return record(data) && text((data as any).hybrid_public_key_hex);
  }
  if (command === 'pq_kem_encapsulate') {
    return record(data) && text((data as any).shared_secret_hex);
  }
  if (command === 'pq_kem_decapsulate') {
    return record(data) && typeof (data as any).verified === 'boolean';
  }
  if (command === 'pq_kem_metadata') {
    return record(data) && text((data as any).standard);
  }
  if (command === 'piv_state') {
    return record(data) && Array.isArray((data as any).slots);
  }
  if (command === 'piv_pin_verify') {
    return record(data) && typeof (data as any).authenticated === 'boolean';
  }
  if (command === 'piv_logout') {
    return record(data) && (data as any).sessionUnlocked === false;
  }
  if (command === 'piv_sign') {
    return record(data) && text((data as any).signature_hex);
  }
  if (command === 'piv_cert_export') {
    return record(data) && text((data as any).slot_id);
  }
  if (command === 'tray_status_get') {
    return record(data) && typeof (data as any).daemonActive === 'boolean';
  }
  if (command === 'tray_quick_killswitch') {
    return record(data) && typeof (data as any).killswitchActive === 'boolean';
  }
  if (command === 'tray_menu_action') {
    return record(data);
  }
  return true;
}
