param(
 [Parameter(Mandatory)][string]$Path,
 [Parameter(Mandatory)][ValidatePattern('^[0-9A-Fa-f]{40}$')][string]$Thumbprint,
 [string]$TimestampUrl = 'http://timestamp.digicert.com',
 [switch]$VerifyOnly
)
$ErrorActionPreference = 'Stop'
$target = (Resolve-Path -LiteralPath $Path).Path
if ([IO.Path]::GetExtension($target) -ne '.exe') { throw 'Signing accepts an explicit executable only' }
if (-not $VerifyOnly) {
 $certificate = Get-Item -LiteralPath ('Cert:/CurrentUser/My/' + $Thumbprint)
 if (-not $certificate.HasPrivateKey -or $certificate.NotAfter -le (Get-Date)) { throw 'A valid publisher code-signing certificate with a private key is required' }
 if (-not ($certificate.EnhancedKeyUsageList.ObjectId -contains '1.3.6.1.5.5.7.3.3')) { throw 'Certificate must permit code signing' }
 $tool = Get-Command signtool.exe -ErrorAction SilentlyContinue
 if (-not $tool) {
  $toolPath = Get-ChildItem 'C:/Program Files (x86)/Windows Kits/10/bin/*/x64/signtool.exe' -ErrorAction SilentlyContinue | Sort-Object FullName -Descending | Select-Object -First 1 -ExpandProperty FullName
 } else { $toolPath = $tool.Source }
 if (-not $toolPath) { throw 'Install Windows SDK SignTool before signing' }
 & $toolPath sign /sha1 $Thumbprint /s My /fd SHA256 /tr $TimestampUrl /td SHA256 $target
 if ($LASTEXITCODE -ne 0) { throw 'Authenticode signing failed' }
}
$signature = Get-AuthenticodeSignature -LiteralPath $target
if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Thumbprint -ne $Thumbprint) { throw 'Publisher signature is missing, invalid or belongs to a different certificate' }
if (-not $signature.TimeStamperCertificate) { throw 'A trusted timestamp is required for release signing' }
Write-Output 'Publisher Authenticode signature and timestamp verified.'
