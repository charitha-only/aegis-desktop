# Known-advisory scanning is automated evidence, not an independent application audit.
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted' -or -not $env:RUNNER_TEMP) { throw 'Dependency audit setup requires a disposable hosted runner' }
$root = Split-Path $PSScriptRoot -Parent
$tools = Join-Path $env:RUNNER_TEMP ('aegis-audit-tools-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tools | Out-Null
$archive = Join-Path $tools 'cargo-audit.zip'
Invoke-WebRequest 'https://github.com/rustsec/rustsec/releases/download/cargo-audit/v0.22.2/cargo-audit-x86_64-pc-windows-msvc-v0.22.2.zip' -OutFile $archive
if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash -ne '0a7316540862c13d954f648917ceacca593747baed6eec180fafa590be2710ab') { throw 'RustSec tool digest mismatch' }
Expand-Archive -LiteralPath $archive -DestinationPath (Join-Path $tools 'extracted')
$audit = Join-Path $tools 'extracted/cargo-audit-x86_64-pc-windows-msvc-v0.22.2/cargo-audit.exe'
$output = Join-Path $env:RUNNER_TEMP 'aegis-dependency-evidence'
New-Item -ItemType Directory -Path $output -Force | Out-Null
& $audit audit --file (Join-Path $root 'src-core/Cargo.lock') --db (Join-Path $tools 'advisory-db') --deny warnings --format json > (Join-Path $output 'rustsec.json')
$rustExit = $LASTEXITCODE
npm audit --prefix (Join-Path $root 'src-ui') --json > (Join-Path $output 'npm-audit.json')
$npmExit = $LASTEXITCODE
[ordered]@{commit=$env:GITHUB_SHA;cargoLockSha256=(Get-FileHash (Join-Path $root 'src-core/Cargo.lock')).Hash;uiLockSha256=(Get-FileHash (Join-Path $root 'src-ui/package-lock.json')).Hash;rustSecExitCode=$rustExit;npmExitCode=$npmExit;independentApplicationAudit='not_performed'} | ConvertTo-Json | Set-Content (Join-Path $output 'provenance.json')
if ($rustExit -ne 0 -or $npmExit -ne 0) { throw 'Known dependency advisory scan failed; inspect evidence artifacts' }
Write-Output 'PASS: RustSec and npm known-advisory checks; application audit remains separate.'
