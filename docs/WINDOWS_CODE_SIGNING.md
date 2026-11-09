# 🔏 Windows Authenticode Code Signing & Commercial CA Guide

This document describes the complete Code Signing architecture for Aegis Desktop, explaining how to transition from local development certificates to **Commercial Extended Validation (EV)** and **Azure Trusted Signing** for public distribution without Windows SmartScreen warnings.

---

## 1. Why Code Signing Matters (SmartScreen & Zero-Trust)

When users download and launch an executable on Windows 10 or 11, the operating system verifies its **Microsoft Authenticode** digital signature:

| Signature Type | SmartScreen Behavior | User Experience | Use Case |
| :--- | :--- | :--- | :--- |
| **Unsigned Binary** | 🛑 Blocked with red alert ("Unknown Publisher") | User must click "More info" → "Run anyway" | Development testing |
| **Self-Signed Certificate** | ⚠️ Warning ("Publisher: Aegis Desktop Authority (Untrusted Root)") | Warns unless Root CA is manually installed | Internal / Local QA |
| **Commercial Standard CA (OV)** | 🟡 Neutral / Trust builds progressively over download volume | Initial warning disappears after reputation | Small distribution |
| **Commercial EV Certificate (Hardware Token)** | 🟢 **Zero Warnings — Instant Reputation** | Seamless immediate installation | Enterprise / Commercial |
| **Azure Trusted Signing (Microsoft Cloud)** | 🟢 **Zero Warnings — Instant Reputation** | Seamless immediate installation | Modern Cloud CI/CD |

---

## 2. Choosing a Commercial Provider

### Option A: Azure Trusted Signing (Recommended for Cloud & GitHub Actions)
- **Cost**: ~\$9.99 / month (No \$400/year physical USB dongle required).
- **Security**: Microsoft-managed cloud HSM (FIPS 140-2 Level 3).
- **Reputation**: Immediate Windows SmartScreen trust.
- **Workflow**: Automated via GitHub Actions without storing private keys on build machines.

### Option B: DigiCert / Sectigo EV Code Signing (Enterprise Standard)
- **Cost**: ~\$400 – \$700 / year.
- **Security**: Hardware cryptographic token (SafeNet eToken or YubiKey).
- **Reputation**: Immediate SmartScreen trust upon first download.
- **Workflow**: Plugged into a build machine or accessed via network HSM.

### Option C: Organization Validated (OV) PFX Certificate
- **Cost**: ~\$200 – \$300 / year.
- **Security**: Password-protected `.pfx` or `.p12` file.
- **Reputation**: SmartScreen reputation builds progressively as more users install the software.

---

## 3. Aegis Code Signing Tooling (`scripts/sign-windows.ps1`)

Aegis includes a dual-engine signing script that supports **Azure Trusted Signing**, **PFX files**, **Hardware Tokens / Certificate Stores**, and **PowerShell native signing**:

```powershell
scripts/sign-windows.ps1
  -Path <file.exe>
  [-Thumbprint <40-hex-sha1>]
  [-CertPath <path-to.pfx>]
  [-CertPassword <password>]
  [-TimestampUrl <rfc3161-url>]
  [-AllowSelfSigned]
  [-VerifyOnly]
  [-AzureEndpoint <url>]
  [-AzureAccount <account>]
  [-AzureProfile <profile>]
```

### Supported Modes:

#### 1. Hardware Token / Windows Certificate Store (EV DigiCert / Sectigo)
When using an EV USB token, the certificate is automatically registered in Windows Certificate Manager (`Cert:\CurrentUser\My` or `Cert:\LocalMachine\My`).

```powershell
# Sign with your certificate thumbprint:
powershell -ExecutionPolicy Bypass -File scripts/sign-windows.ps1 `
  -Path dist-bundle/AegisDesktopApplication/aegis-core.exe `
  -Thumbprint 3A4F829C10B2...
```

#### 2. PFX / PKCS#12 File (Standard Commercial CA)
```powershell
# Sign using a secure PFX file:
powershell -ExecutionPolicy Bypass -File scripts/sign-windows.ps1 `
  -Path dist-bundle/AegisDesktopApplication/aegis-core.exe `
  -CertPath "C:\certs\my-publisher.pfx" `
  -CertPassword "SecretPassword123"
```
Or configure via environment variables:
```powershell
$env:AEGIS_SIGN_CERT_PATH = "C:\certs\my-publisher.pfx"
$env:AEGIS_SIGN_CERT_PASSWORD = "SecretPassword123"
powershell -ExecutionPolicy Bypass -File scripts/sign-windows.ps1 -Path dist-bundle/AegisDesktopApplication/aegis-core.exe
```

#### 3. Azure Trusted Signing (Microsoft Cloud HSM)
```powershell
# Install AzureSignTool (one time):
dotnet tool install --global AzureSignTool

# Sign via Azure Trusted Signing:
powershell -ExecutionPolicy Bypass -File scripts/sign-windows.ps1 `
  -Path dist-bundle/AegisDesktopApplication/aegis-core.exe `
  -AzureEndpoint "https://wus.codesigning.azure.net" `
  -AzureAccount "MyAegisAccount" `
  -AzureProfile "ReleaseProfile"
```

#### 4. Local Development / CI Testing (Self-Signed)
```powershell
# Generate a local development certificate:
powershell -ExecutionPolicy Bypass -File scripts/create-dev-cert.ps1

# Sign and verify locally:
powershell -ExecutionPolicy Bypass -File scripts/sign-windows.ps1 `
  -Path dist-bundle/AegisDesktopApplication/aegis-core.exe `
  -AllowSelfSigned
```

---

## 4. End-to-End Packaging & Installer Signing

A commercial release requires signing **both** the internal executable (`aegis-core.exe`) and the customer installer (`AegisDesktopApplication-Setup.exe`):

```powershell
# 1. Package the release:
Remove-Item -Recurse -Force dist-bundle -ErrorAction SilentlyContinue
npm run package

# 2. Sign the core executable:
powershell -ExecutionPolicy Bypass -File scripts/sign-windows.ps1 `
  -Path dist-bundle/AegisDesktopApplication/aegis-core.exe `
  -Thumbprint <COMMERCIAL_THUMBPRINT>

# 3. Compile and sign the Inno Setup installer:
powershell -ExecutionPolicy Bypass -File scripts/build-installer.ps1 `
  -ExpectedThumbprint <COMMERCIAL_THUMBPRINT>

# 4. Verify signatures:
powershell -ExecutionPolicy Bypass -File scripts/sign-windows.ps1 `
  -Path dist-bundle/installers/AegisDesktopApplication-Setup-v*.exe `
  -Thumbprint <COMMERCIAL_THUMBPRINT> `
  -VerifyOnly
```

---

## 5. GitHub Actions Automated Signing Workflow

To automate commercial signing in CI/CD, add the following step to your `.github/workflows/release.yml`:

```yaml
- name: Sign Executables with Azure Trusted Signing
  uses: azure/trusted-signing-action@v0.4.1
  with:
    azure-tenant-id: ${{ secrets.AZURE_TENANT_ID }}
    azure-client-id: ${{ secrets.AZURE_CLIENT_ID }}
    azure-client-secret: ${{ secrets.AZURE_CLIENT_SECRET }}
    endpoint: https://wus.codesigning.azure.net
    trusted-signing-account-name: aegis-signing-account
    certificate-profile-name: aegis-prod-profile
    files-folder: dist-bundle/AegisDesktopApplication
    files-folder-filter: exe,dll
    file-digest: SHA256
    timestamp-rfc3161: http://timestamp.digicert.com
    timestamp-digest: SHA256
```

---

## 6. Verifying a Signed Binary

You can verify any binary on Windows using PowerShell:

```powershell
$sig = Get-AuthenticodeSignature .\aegis-core.exe
[PSCustomObject]@{
    Status      = $sig.Status
    Subject     = $sig.SignerCertificate.Subject
    Issuer      = $sig.SignerCertificate.Issuer
    Thumbprint  = $sig.SignerCertificate.Thumbprint
    Timestamp   = $sig.TimeStamperCertificate.NotBefore
} | Format-List
```

When signed with a trusted commercial CA, `Status` will report `Valid` and Windows will execute the software with **zero SmartScreen warnings**.
