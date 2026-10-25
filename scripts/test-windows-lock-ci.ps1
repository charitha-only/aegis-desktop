# Real workstation locking is permitted only in an explicitly enabled disposable hosted runner.
param([Parameter(Mandatory)][string]$Binary)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted' -or -not $env:RUNNER_TEMP -or $env:AEGIS_ALLOW_DISPOSABLE_LOCK_TEST -ne 'true') {
 throw 'Real Windows lock test requires explicit opt-in in a disposable GitHub-hosted runner'
}
$testRoot = Join-Path $env:RUNNER_TEMP ('aegis-real-lock-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $testRoot | Out-Null
$env:LOCALAPPDATA = Join-Path $testRoot 'userdata'
$base = 'http://127.0.0.1:48404'
$core = Start-Process -FilePath (Resolve-Path -LiteralPath $Binary).Path -ArgumentList '--port=48404' -WindowStyle Hidden -PassThru
try {
 $script:nonce = $null
 for ($i=0; $i -lt 50; $i++) {
  try { $script:nonce=(Invoke-RestMethod "$base/handshake" -TimeoutSec 1).token; break } catch { Start-Sleep -Milliseconds 100 }
 }
 if (-not $script:nonce) { throw 'Lock-test core did not start' }
 function Call-Core([string]$Command, $Payload=@{}) {
  Invoke-RestMethod "$base/ipc" -Method Post -ContentType 'application/json' -Body (@{id='native-lock-test';command=$Command;nonce=$script:nonce;payload=$Payload} | ConvertTo-Json -Depth 8)
 }
 $ready=$false
 for ($i=0; $i -lt 30; $i++) {
  $status=Call-Core 'get_security_status'
  if ($status.success -and $status.data.windowsLifecycleMonitorActive -and $status.data.windowsSessionAvailable) { $ready=$true; break }
  Start-Sleep -Milliseconds 100
 }
 if (-not $ready) { throw 'An unlocked interactive Windows session and active monitor are required' }
 $password='TEST_ONLY-'+[guid]::NewGuid().ToString('N')
 if (-not (Call-Core 'vault_encrypt_store' @{filename='lock-test.vault';content='TEST_ONLY';passphrase=$password}).success) { throw 'Lock fixture creation failed' }
 $session=Call-Core 'vault_session_unlock' @{filename='lock-test.vault';passphrase=$password}
 if (-not $session.success) { throw 'Cannot unlock lock-test vault' }
 if (-not (Call-Core 'secrets_store' @{key='lock-test';value='TEST_ONLY';masterPassphrase=$password}).success) { throw 'Cannot seed ephemeral lock-test secret' }
 Add-Type -TypeDefinition 'using System.Runtime.InteropServices; public static class AegisDisposableLock { [DllImport("user32.dll", SetLastError=true)] [return: MarshalAs(UnmanagedType.Bool)] public static extern bool LockWorkStation(); }'
 if (-not [AegisDisposableLock]::LockWorkStation()) { throw 'Windows rejected the disposable workstation lock request' }
 $locked=$false
 for ($i=0; $i -lt 50; $i++) {
  $state=Call-Core 'get_security_status'
  if (-not $state.data.windowsSessionAvailable) { $locked=$true; break }
  Start-Sleep -Milliseconds 200
 }
 if (-not $locked) { throw 'Windows did not report a locked session' }
 $read=Call-Core 'vault_session_read' @{filename='lock-test.vault';sessionToken=$session.data.sessionToken}
 if ($read.success -or $read.error -notmatch 'session.*(locked|expired|invalid)') { throw 'Locked Windows session retained vault access' }
 if ((Call-Core 'vault_session_unlock' @{filename='lock-test.vault';passphrase=$password}).success) { throw 'Vault unlocked while Windows was locked' }
 $cleared=$false
 for ($i=0; $i -lt 30; $i++) {
  if ((Call-Core 'secrets_list').data.secrets.Count -eq 0) { $cleared=$true; break }
  Start-Sleep -Milliseconds 200
 }
 if (-not $cleared) { throw 'Windows lock did not clear ephemeral secrets' }
 [ordered]@{commit=$env:GITHUB_SHA;os=[Environment]::OSVersion.VersionString;runtime=$status.data.engine;realWorkstationLock='passed';sessionRevocation='passed';unlockWhileLocked='rejected';ephemeralSecretCleanup='passed';realSuspendResume='not_run';publisherSigning='not_run';independentAudit='pending_external_reviewer'} | ConvertTo-Json | Set-Content (Join-Path $testRoot 'windows-lock-evidence.json')
 Write-Output 'PASS: real disposable Windows lock, session revocation and ephemeral secret cleanup.'
} finally { if (-not $core.HasExited) { Stop-Process -Id $core.Id }; $core.WaitForExit(); $script:nonce=$null }
