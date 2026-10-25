param(
 [string]$BundleName = 'AegisDesktopApplication',
 [string]$ExpectedThumbprint,
 [switch]$AllowUnsignedForTesting
)
$ErrorActionPreference = 'Stop'
if ($BundleName -notmatch '^[A-Za-z0-9_-]+$') { throw 'Invalid bundle name' }
$root = Split-Path $PSScriptRoot -Parent
$binary = Join-Path $root "dist-bundle/$BundleName/aegis-core.exe"
if (-not $AllowUnsignedForTesting) {
 if (-not $ExpectedThumbprint) { throw 'A publisher thumbprint is required; unsigned compilation is for testing only' }
 & "$PSScriptRoot/sign-windows.ps1" -Path $binary -Thumbprint $ExpectedThumbprint -VerifyOnly
}
$compiler = Get-Command ISCC.exe -ErrorAction SilentlyContinue
$compilerPath = if ($compiler) { $compiler.Source } else { 'C:/Program Files (x86)/Inno Setup 6/ISCC.exe' }
if (-not (Test-Path -LiteralPath $compilerPath)) { throw 'Install Inno Setup 6 to compile the installer' }
& $compilerPath (Join-Path $root "dist-bundle/$BundleName-installer.iss")
if ($LASTEXITCODE -ne 0) { throw 'Installer compilation failed' }
if ($AllowUnsignedForTesting) { Write-Output 'TEST ONLY: installer has not been publisher-signed.' }
