param(
 [Parameter(Mandatory)][string]$SignedFixture,
 [Parameter(Mandatory)][string]$UnsignedFixture,
 [switch]$AllowSelfSigned
)
$ErrorActionPreference = 'Stop'
$signature = Get-AuthenticodeSignature -LiteralPath $SignedFixture
if ($signature.Status -ne 'Valid' -and -not ($AllowSelfSigned -and $signature.Status -eq 'UnknownError' -and $signature.SignerCertificate)) {
 throw 'A genuinely publisher-signed fixture is required'
}
$thumbprint = $signature.SignerCertificate.Thumbprint
$verifyArgs = @{ Path = $SignedFixture; Thumbprint = $thumbprint; VerifyOnly = $true }
if ($AllowSelfSigned) { $verifyArgs['AllowSelfSigned'] = $true }
& "$PSScriptRoot/sign-windows.ps1" @verifyArgs
$wrongThumbprint = if ($thumbprint -eq ('0'*40)) { '1'*40 } else { '0'*40 }
try {
 $wrongArgs = @{ Path = $SignedFixture; Thumbprint = $wrongThumbprint; VerifyOnly = $true }
 if ($AllowSelfSigned) { $wrongArgs['AllowSelfSigned'] = $true }
 & "$PSScriptRoot/sign-windows.ps1" @wrongArgs
 throw 'Wrong publisher unexpectedly accepted'
} catch { if ($_.Exception.Message -notmatch 'signature is missing, invalid or belongs') { throw } }
try {
 $unsignedArgs = @{ Path = $UnsignedFixture; Thumbprint = $thumbprint; VerifyOnly = $true }
 if ($AllowSelfSigned) { $unsignedArgs['AllowSelfSigned'] = $true }
 & "$PSScriptRoot/sign-windows.ps1" @unsignedArgs
 throw 'Unsigned executable unexpectedly accepted'
} catch { if ($_.Exception.Message -notmatch 'signature is missing, invalid or belongs') { throw } }
try {
 & "$PSScriptRoot/build-installer.ps1"
 throw 'Release compilation unexpectedly accepted no signing identity'
} catch { if ($_.Exception.Message -notmatch 'publisher thumbprint is required') { throw } }
Write-Output 'PASS: real publisher fixture accepted; wrong publisher, unsigned core and missing release identity rejected.'
