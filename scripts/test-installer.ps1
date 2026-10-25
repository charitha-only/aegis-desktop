# Run only in an ephemeral Windows test runner. Never launches UI.
param([Parameter(Mandatory)][string]$Installer, [string]$PreviousInstaller)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted' -or -not $env:RUNNER_TEMP) { throw 'Installer test requires a disposable GitHub-hosted runner' }
$root = Split-Path $PSScriptRoot -Parent
$version = (Get-Content (Join-Path $root 'aegis.config.json') -Raw | ConvertFrom-Json).version
$testRoot = Join-Path $env:RUNNER_TEMP ('aegis-install-' + [guid]::NewGuid().ToString('N'))
$install = Join-Path $testRoot 'AegisDesktopApplication'
$env:LOCALAPPDATA = Join-Path $testRoot 'userdata'
$env:AEGIS_BRIDGE_URL = 'http://127.0.0.1:48402'
$env:AEGIS_MIGRATION_TEST_PASSWORD = 'TEST_ONLY-' + [guid]::NewGuid().ToString('N')
New-Item -ItemType Directory -Path $testRoot | Out-Null
$binary = Join-Path $install 'aegis-core.exe'
$dataDir = Join-Path $env:LOCALAPPDATA 'AegisDesktop/AegisDesktopApplication/sandbox'
function Install-TestPackage([string]$Package, [string]$Phase) {
 $setup = Start-Process -FilePath (Resolve-Path -LiteralPath $Package).Path -ArgumentList @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART','/SP-',"/DIR=`"$install`"","/LOG=`"$testRoot/$Phase.log`"") -WindowStyle Hidden -PassThru -Wait
 if ($setup.ExitCode -ne 0 -or -not (Test-Path -LiteralPath $binary)) { throw "Installer phase failed: $Phase" }
}
function Exercise-Core([string]$Mode, [string]$ExpectedVersion) {
 $env:AEGIS_EXPECTED_RUNTIME_VERSION = $ExpectedVersion
 $core = Start-Process -FilePath $binary -ArgumentList '--port=48402' -WindowStyle Hidden -PassThru
 try {
  $ready = $false
  for ($i=0; $i -lt 50; $i++) {
   try {
    $nonce = (Invoke-RestMethod "$env:AEGIS_BRIDGE_URL/handshake" -TimeoutSec 1).token
    $status = Invoke-RestMethod "$env:AEGIS_BRIDGE_URL/ipc" -Method Post -ContentType 'application/json' -Body (@{id='installer';command='get_security_status';nonce=$nonce;payload=@{}} | ConvertTo-Json)
    if ($status.success -and $status.data.windowsLifecycleMonitorActive -and $status.data.engine.Contains($ExpectedVersion)) { $ready=$true; break }
   } catch { }
   Start-Sleep -Milliseconds 100
  }
  if (-not $ready) { throw 'Installed runtime/version/lifecycle monitor unavailable' }
  if ($Mode -eq 'sessions') { node (Join-Path $root 'aegis-cli/bin/session-smoke.js') }
  else { node (Join-Path $root 'aegis-cli/bin/migration-smoke.js') $Mode }
  if ($LASTEXITCODE -ne 0) { throw "Installed runtime check failed: $Mode" }
 } finally { if (-not $core.HasExited) { Stop-Process -Id $core.Id }; $core.WaitForExit() }
}
function Snapshot-Vaults {
 $files = @(Get-ChildItem -LiteralPath $dataDir -File | Where-Object { $_.Extension -in @('.vault','.bak') })
 if ($files.Count -lt 2) { throw 'Encrypted vault and backup fixtures required' }
 $hashes = @{}; foreach ($file in $files) { $hashes[$file.FullName] = (Get-FileHash -LiteralPath $file.FullName).Hash }
 return $hashes
}
function Assert-Preserved($Hashes) {
 foreach ($file in $Hashes.Keys) { if ((Get-FileHash -LiteralPath $file).Hash -ne $Hashes[$file]) { throw 'Installer changed encrypted vault/backup data' } }
}
function Uninstall-TestPackage {
 $uninstall = Start-Process -FilePath (Join-Path $install 'unins000.exe') -ArgumentList @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART') -WindowStyle Hidden -PassThru -Wait
 if ($uninstall.ExitCode -ne 0 -or (Test-Path -LiteralPath $binary)) { throw 'Uninstallation failed' }
}
try {
 if ($PreviousInstaller) {
  Install-TestPackage $PreviousInstaller 'baseline-install'
  Exercise-Core 'seed' '0.22.0'
  $beforeUpgrade = Snapshot-Vaults
  Install-TestPackage $Installer 'upgrade'
  Assert-Preserved $beforeUpgrade
  Exercise-Core 'verify' $version
 } else {
  Install-TestPackage $Installer 'fresh-install'
  Exercise-Core 'seed' $version
 }
 Exercise-Core 'sessions' $version
 $beforeUninstall = Snapshot-Vaults
 Uninstall-TestPackage
 Assert-Preserved $beforeUninstall
 Install-TestPackage $Installer 'reinstall'
 Assert-Preserved $beforeUninstall
 Exercise-Core 'verify' $version
 $afterReinstall = Snapshot-Vaults
 Uninstall-TestPackage
 Assert-Preserved $afterReinstall
 [ordered]@{commit=$env:GITHUB_SHA;runtimeVersion=$version;baselineVersion=$(if($PreviousInstaller){'0.22.0'}else{$null});upgrade=$(if($PreviousInstaller){'passed'}else{'not_run'});reinstall='passed';backupRecovery='passed';uninstallDataPreservation='passed'} | ConvertTo-Json | Set-Content (Join-Path $testRoot 'installer-evidence.json')
 Write-Output 'PASS: installed runtime, upgrade/reinstall, backup recovery and uninstall data preservation.'
} finally { Remove-Item Env:/AEGIS_MIGRATION_TEST_PASSWORD -ErrorAction SilentlyContinue }
