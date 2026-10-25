param([Parameter(Mandatory)][string]$SignedFixture, [Parameter(Mandatory)][string]$UnsignedFixture)
$ErrorActionPreference = 'Stop'
$signature = Get-AuthenticodeSignature -LiteralPath $SignedFixture
if ($signature.Status -ne 'Valid') { throw 'A genuinely publisher-signed fixture is required' }
$thumbprint = $signature.SignerCertificate.Thumbprint
& "$PSScriptRoot/sign-windows.ps1" -Path $SignedFixture -Thumbprint $thumbprint -VerifyOnly
$wrongThumbprint = if ($thumbprint -eq ('0'*40)) { '1'*40 } else { '0'*40 }
try {
 & "$PSScriptRoot/sign-windows.ps1" -Path $SignedFixture -Thumbprint $wrongThumbprint -VerifyOnly
 throw 'Wrong publisher unexpectedly accepted'
} catch { if ($_.Exception.Message -notmatch 'signature is missing, invalid or belongs') { throw } }
try {
 & "$PSScriptRoot/sign-windows.ps1" -Path $UnsignedFixture -Thumbprint $thumbprint -VerifyOnly
 throw 'Unsigned executable unexpectedly accepted'
} catch { if ($_.Exception.Message -notmatch 'signature is missing, invalid or belongs') { throw } }
try {
 & "$PSScriptRoot/build-installer.ps1"
 throw 'Release compilation unexpectedly accepted no signing identity'
} catch { if ($_.Exception.Message -notmatch 'publisher thumbprint is required') { throw } }
Write-Output 'PASS: real publisher fixture accepted; wrong publisher, unsigned core and missing release identity rejected.'
