param(
  [string]$BundleName = 'AegisDesktopApplication',
  [string]$ExpectedThumbprint = $env:AEGIS_SIGN_THUMBPRINT,
  [string]$CertPath = $env:AEGIS_SIGN_CERT_PATH,
  [string]$CertPassword = $env:AEGIS_SIGN_CERT_PASSWORD,
  [switch]$AllowUnsignedForTesting,
  [switch]$AllowSelfSigned
)
$ErrorActionPreference = 'Stop'
if ($BundleName -notmatch '^[A-Za-z0-9_-]+$') { throw 'Invalid bundle name' }
$root = Split-Path $PSScriptRoot -Parent
$binary = Join-Path $root "dist-bundle/$BundleName/aegis-core.exe"

if (-not $AllowUnsignedForTesting) {
  if (-not $ExpectedThumbprint -and -not $CertPath) {
    throw 'A publisher thumbprint is required; unsigned compilation is for testing only'
  }

  $verifyArgs = @{ Path = $binary; VerifyOnly = $true }
  if ($ExpectedThumbprint) { $verifyArgs['Thumbprint'] = $ExpectedThumbprint }
  if ($CertPath) { $verifyArgs['CertPath'] = $CertPath }
  if ($CertPassword) { $verifyArgs['CertPassword'] = $CertPassword }
  if ($AllowSelfSigned) { $verifyArgs['AllowSelfSigned'] = $true }

  & "$PSScriptRoot/sign-windows.ps1" @verifyArgs
}

$compiler = Get-Command ISCC.exe -ErrorAction SilentlyContinue
$compilerPath = if ($env:AEGIS_ISCC_PATH) { $env:AEGIS_ISCC_PATH } elseif ($compiler) { $compiler.Source } else { 'C:/Program Files (x86)/Inno Setup 6/ISCC.exe' }
if (-not (Test-Path -LiteralPath $compilerPath)) { throw 'Install Inno Setup 6 to compile the installer' }

$issFile = Join-Path $root "dist-bundle/$BundleName-installer.iss"
Write-Output "Compiling installer: $issFile..."
& $compilerPath $issFile
if ($LASTEXITCODE -ne 0) { throw 'Installer compilation failed' }

# Find and sign the generated installer executable
$installerDir = Join-Path $root 'dist-bundle/installers'
if (Test-Path $installerDir) {
  $installers = Get-ChildItem $installerDir -Filter '*.exe' | Sort-Object LastWriteTime -Descending
  if ($installers) {
    $latestInstaller = $installers[0].FullName
    if (-not $AllowUnsignedForTesting) {
      Write-Output "Signing installer package: $latestInstaller"
      $signArgs = @{ Path = $latestInstaller }
      if ($ExpectedThumbprint) { $signArgs['Thumbprint'] = $ExpectedThumbprint }
      if ($CertPath) { $signArgs['CertPath'] = $CertPath }
      if ($CertPassword) { $signArgs['CertPassword'] = $CertPassword }
      if ($AllowSelfSigned) { $signArgs['AllowSelfSigned'] = $true }

      & "$PSScriptRoot/sign-windows.ps1" @signArgs
      Write-Output "Installer signed and verified: $latestInstaller"
    } else {
      Write-Output "TEST ONLY: installer has not been publisher-signed ($latestInstaller)."
    }
  }
}
