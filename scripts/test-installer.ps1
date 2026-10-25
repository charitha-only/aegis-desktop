# Run only in an ephemeral Windows test runner. Installs into RUNNER_TEMP and never launches UI.
param([Parameter(Mandatory)][string]$Installer)
$ErrorActionPreference = 'Stop'
if (-not $env:GITHUB_ACTIONS -or -not $env:RUNNER_TEMP) { throw 'Installer integration test requires a disposable GitHub Actions runner' }
$testRoot = Join-Path $env:RUNNER_TEMP ('aegis-install-' + [guid]::NewGuid().ToString('N'))
$install = Join-Path $testRoot 'AegisDesktopApplication'
$env:LOCALAPPDATA = Join-Path $testRoot 'userdata'
New-Item -ItemType Directory -Path $testRoot | Out-Null
$setup = Start-Process -FilePath (Resolve-Path -LiteralPath $Installer).Path -ArgumentList @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART','/SP-',"/DIR=`"$install`"","/LOG=`"$testRoot/install.log`"") -WindowStyle Hidden -PassThru -Wait
if ($setup.ExitCode -ne 0) { throw 'Silent installation failed' }
$binary = Join-Path $install 'aegis-core.exe'
if (-not (Test-Path -LiteralPath $binary)) { throw 'Installed core missing' }
$env:AEGIS_BRIDGE_URL = 'http://127.0.0.1:48402'
$core = Start-Process -FilePath $binary -ArgumentList '--port=48402' -WindowStyle Hidden -PassThru
try {
 $ready = $false
 for ($i=0; $i -lt 50; $i++) {
  try { Invoke-RestMethod "$env:AEGIS_BRIDGE_URL/handshake" -TimeoutSec 1 | Out-Null; $ready=$true; break } catch { Start-Sleep -Milliseconds 200 }
 }
 if (-not $ready) { throw 'Installed core failed to start' }
 $monitorReady = $false
 for ($i=0; $i -lt 30; $i++) {
  $status = Invoke-RestMethod "$env:AEGIS_BRIDGE_URL/ipc" -Method Post -ContentType 'application/json' -Body (@{id='installer';command='get_security_status';nonce=(Invoke-RestMethod "$env:AEGIS_BRIDGE_URL/handshake").token;payload=@{}} | ConvertTo-Json)
  if ($status.success -and $status.data.windowsLifecycleMonitorActive) { $monitorReady=$true; break }
  Start-Sleep -Milliseconds 100
 }
 if (-not $monitorReady) { throw 'Installed Windows lifecycle monitor failed' }
 node (Join-Path (Split-Path $PSScriptRoot -Parent) 'aegis-cli/bin/session-smoke.js')
 if ($LASTEXITCODE -ne 0) { throw 'Installed session smoke checks failed' }
} finally { if (-not $core.HasExited) { Stop-Process -Id $core.Id }; $core.WaitForExit() }
$dataDir = Join-Path $env:LOCALAPPDATA 'AegisDesktop/AegisDesktopApplication'
$preservedVaults = @(Get-ChildItem -LiteralPath (Join-Path $dataDir 'sandbox') -File | Where-Object { $_.Extension -in @('.vault','.bak') })
if ($preservedVaults.Count -eq 0) { throw 'Encrypted install test fixtures missing' }
$hashes = @{}; foreach ($file in $preservedVaults) { $hashes[$file.FullName] = (Get-FileHash -LiteralPath $file.FullName).Hash }
$uninstall = Start-Process -FilePath (Join-Path $install 'unins000.exe') -ArgumentList @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART') -WindowStyle Hidden -PassThru -Wait
if ($uninstall.ExitCode -ne 0 -or (Test-Path -LiteralPath $binary)) { throw 'Uninstallation failed' }
foreach ($file in $hashes.Keys) { if ((Get-FileHash -LiteralPath $file).Hash -ne $hashes[$file]) { throw 'Uninstall changed vault data' } }
Write-Output 'Disposable Windows install, installed runtime, uninstall and encrypted data preservation passed.'
