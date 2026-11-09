param(
  [string]$Subject = 'CN=Aegis Desktop Authority',
  [int]$YearsValid = 3
)
$ErrorActionPreference = 'Stop'

# Check if an existing valid code signing certificate matches the subject
$existing = Get-ChildItem Cert:\CurrentUser\My | Where-Object {
  $_.Subject -eq $Subject -and
  $_.HasPrivateKey -and
  $_.NotAfter -gt (Get-Date) -and
  ($_.EnhancedKeyUsageList.ObjectId -contains '1.3.6.1.5.5.7.3.3')
} | Sort-Object NotAfter -Descending | Select-Object -First 1

if ($existing) {
  Write-Output "Found existing valid code signing certificate: $($existing.Thumbprint)"
  Write-Output "Subject: $($existing.Subject)"
  Write-Output "Expires: $($existing.NotAfter)"
  return $existing.Thumbprint
}

Write-Output "Generating self-signed code signing certificate for '$Subject'..."
$cert = New-SelfSignedCertificate `
  -Type CodeSigningCert `
  -Subject $Subject `
  -CertStoreLocation Cert:\CurrentUser\My `
  -NotAfter (Get-Date).AddYears($YearsValid) `
  -KeyExportPolicy Exportable `
  -KeyLength 2048 `
  -HashAlgorithm SHA256

Write-Output "Created certificate with Thumbprint: $($cert.Thumbprint)"
Write-Output "Subject: $($cert.Subject)"
Write-Output "Expires: $($cert.NotAfter)"
return $cert.Thumbprint
