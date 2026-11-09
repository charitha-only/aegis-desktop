param(
  [Parameter(Mandatory)][string]$Path,
  [string]$Thumbprint = $env:AEGIS_SIGN_THUMBPRINT,
  [string]$CertPath = $env:AEGIS_SIGN_CERT_PATH,
  [string]$CertPassword = $env:AEGIS_SIGN_CERT_PASSWORD,
  [string]$TimestampUrl = 'http://timestamp.digicert.com',
  [switch]$AllowSelfSigned,
  [switch]$VerifyOnly,
  # Azure Trusted Signing options
  [string]$AzureEndpoint = $env:AZURE_TRUSTED_SIGNING_ENDPOINT,
  [string]$AzureAccount = $env:AZURE_TRUSTED_SIGNING_ACCOUNT,
  [string]$AzureProfile = $env:AZURE_TRUSTED_SIGNING_PROFILE
)

$ErrorActionPreference = 'Stop'
$target = (Resolve-Path -LiteralPath $Path).Path
$ext = [IO.Path]::GetExtension($target).ToLowerInvariant()
if ($ext -ne '.exe' -and $ext -ne '.dll') {
  throw 'Signing accepts an explicit executable (.exe or .dll) only'
}

# Resolve signtool.exe if available
$tool = Get-Command signtool.exe -ErrorAction SilentlyContinue
$toolPath = if ($tool) { $tool.Source } else {
  Get-ChildItem 'C:/Program Files (x86)/Windows Kits/10/bin/*/x64/signtool.exe' -ErrorAction SilentlyContinue |
    Sort-Object FullName -Descending |
    Select-Object -First 1 -ExpandProperty FullName
}

if (-not $VerifyOnly) {
  # ── Mode 1: Azure Trusted Signing (Cloud-based HSM / Enterprise) ───────
  if ($AzureEndpoint -and $AzureProfile) {
    $azureTool = Get-Command AzureSignTool.exe -ErrorAction SilentlyContinue
    if (-not $azureTool) {
      throw 'Azure Trusted Signing requested but AzureSignTool.exe was not found. Install with: dotnet tool install --global AzureSignTool'
    }
    Write-Output "Signing with Azure Trusted Signing profile '$AzureProfile'..."
    $azArgs = @('sign', '-kvu', $AzureEndpoint, '-kvc', $AzureProfile, '-tr', $TimestampUrl, '-td', 'sha256')
    if ($AzureAccount) { $azArgs += @('-kvi', $AzureAccount) }
    $azArgs += $target
    & $azureTool.Source @azArgs
    if ($LASTEXITCODE -ne 0) { throw 'Azure Trusted Signing failed' }
  }
  # ── Mode 2: PFX / PKCS#12 File ──────────────────────────────────────────
  elseif ($CertPath) {
    if (-not (Test-Path -LiteralPath $CertPath)) { throw "Certificate file not found: $CertPath" }
    $certResolved = (Resolve-Path -LiteralPath $CertPath).Path
    Write-Output "Loading certificate from file: $certResolved"
    if ($CertPassword) {
      $secPass = ConvertTo-SecureString $CertPassword -AsPlainText -Force
      $certificate = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new($certResolved, $secPass, [System.Security.Cryptography.X509Certificates.X509KeyStorageFlags]::Exportable)
    } else {
      $certificate = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new($certResolved)
    }
    if (-not $certificate.HasPrivateKey -or $certificate.NotAfter -le (Get-Date)) {
      throw 'A valid publisher code-signing certificate with a private key is required'
    }
    if ($Thumbprint -and $certificate.Thumbprint -ne $Thumbprint) {
      throw "Certificate thumbprint mismatch: expected $Thumbprint, file has $($certificate.Thumbprint)"
    }
    $Thumbprint = $certificate.Thumbprint

    if ($toolPath) {
      $signArgs = @('sign', '/f', $certResolved, '/fd', 'SHA256', '/tr', $TimestampUrl, '/td', 'SHA256')
      if ($CertPassword) { $signArgs += @('/p', $CertPassword) }
      $signArgs += $target
      & $toolPath @signArgs
      if ($LASTEXITCODE -ne 0) { throw 'Authenticode signing with signtool failed' }
    } else {
      Set-AuthenticodeSignature -FilePath $target -Certificate $certificate -TimestampServer $TimestampUrl -HashAlgorithm SHA256 | Out-Null
    }
  }
  # ── Mode 3: Certificate Store (CurrentUser or LocalMachine) ─────────────
  else {
    # If no thumbprint provided, auto-discover code signing certificate
    if (-not $Thumbprint) {
      $candidates = Get-ChildItem Cert:\CurrentUser\My, Cert:\LocalMachine\My -ErrorAction SilentlyContinue |
        Where-Object {
          $_.HasPrivateKey -and
          $_.NotAfter -gt (Get-Date) -and
          ($_.EnhancedKeyUsageList.ObjectId -contains '1.3.6.1.5.5.7.3.3')
        } | Sort-Object NotAfter -Descending

      if ($candidates) {
        $certificate = $candidates[0]
        $Thumbprint = $certificate.Thumbprint
        Write-Output "Auto-selected code signing certificate: $Thumbprint ($($certificate.Subject))"
      } else {
        throw 'No code signing certificate specified and none found in Certificate Stores'
      }
    }

    $certificate = Get-Item -LiteralPath ('Cert:/CurrentUser/My/' + $Thumbprint) -ErrorAction SilentlyContinue
    $storeName = 'My'
    $isLocalMachine = $false
    if (-not $certificate) {
      $certificate = Get-Item -LiteralPath ('Cert:/LocalMachine/My/' + $Thumbprint) -ErrorAction SilentlyContinue
      $isLocalMachine = $true
    }
    if (-not $certificate) {
      throw "Certificate with thumbprint $Thumbprint not found in CurrentUser\My or LocalMachine\My"
    }
    if (-not $certificate.HasPrivateKey -or $certificate.NotAfter -le (Get-Date)) {
      throw 'A valid publisher code-signing certificate with a private key is required'
    }
    if (-not ($certificate.EnhancedKeyUsageList.ObjectId -contains '1.3.6.1.5.5.7.3.3')) {
      throw 'Certificate must permit code signing (Enhanced Key Usage 1.3.6.1.5.5.7.3.3)'
    }

    if ($toolPath) {
      $signArgs = @('sign', '/sha1', $Thumbprint, '/s', 'My', '/fd', 'SHA256', '/tr', $TimestampUrl, '/td', 'SHA256')
      if ($isLocalMachine) { $signArgs += '/sm' }
      $signArgs += $target
      & $toolPath @signArgs
      if ($LASTEXITCODE -ne 0) { throw 'Authenticode signing failed with signtool' }
    } else {
      Set-AuthenticodeSignature -FilePath $target -Certificate $certificate -TimestampServer $TimestampUrl -HashAlgorithm SHA256 | Out-Null
    }
  }
}

# ── Verification Phase ───────────────────────────────────────────────────
$signature = Get-AuthenticodeSignature -LiteralPath $target
if ($signature.Status -eq 'NotSigned') {
  throw 'Publisher signature is missing, invalid or belongs to a different certificate'
}
if ($Thumbprint -and $signature.SignerCertificate.Thumbprint -ne $Thumbprint) {
  throw "Publisher signature is missing, invalid or belongs to a different certificate (Expected: $Thumbprint, Found: $($signature.SignerCertificate.Thumbprint))"
}
if ($signature.Status -ne 'Valid' -and -not $AllowSelfSigned) {
  throw "Publisher signature status is '$($signature.Status)'. Use a trusted CA certificate (DigiCert, Sectigo, etc.) for public releases or pass -AllowSelfSigned for local test builds."
}
if (-not $signature.TimeStamperCertificate) {
  throw 'A trusted RFC 3161 timestamp is required for release signing'
}

$signer = $signature.SignerCertificate
$stamper = $signature.TimeStamperCertificate
Write-Output "✅ Authenticode signature verified successfully:"
Write-Output "   Target:    $target"
Write-Output "   Subject:   $($signer.Subject)"
Write-Output "   Issuer:    $($signer.Issuer)"
Write-Output "   Status:    $($signature.Status)"
Write-Output "   Timestamp: $($stamper.Subject) ($($stamper.NotBefore))"
