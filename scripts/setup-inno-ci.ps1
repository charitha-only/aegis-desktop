# Build dependency installation is restricted to disposable CI runners.
$ErrorActionPreference = 'Stop'
if (-not $env:GITHUB_ACTIONS -or -not $env:RUNNER_TEMP) { throw 'Compiler setup requires a disposable GitHub Actions runner' }
$installer = Join-Path $env:RUNNER_TEMP 'innosetup-6.7.3.exe'
Invoke-WebRequest 'https://github.com/jrsoftware/issrc/releases/download/is-6_7_3/innosetup-6.7.3.exe' -OutFile $installer
$expected = '9c73c3bae7ed48d44112a0f48e66742c00090bdb5bef71d9d3c056c66e97b732'
if ((Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash -ne $expected) { throw 'Inno Setup download digest mismatch' }
$signature = Get-AuthenticodeSignature -LiteralPath $installer
if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'Pyrsys B\.V\.') { throw 'Inno Setup publisher signature invalid' }
$tools = Join-Path $env:RUNNER_TEMP 'aegis-tools/InnoSetup6'
$setup = Start-Process -FilePath $installer -ArgumentList @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART','/SP-','/CURRENTUSER',"/DIR=`"$tools`"") -WindowStyle Hidden -PassThru -Wait
if ($setup.ExitCode -ne 0) { throw 'Inno Setup compiler installation failed' }
$compiler = Join-Path $tools 'ISCC.exe'
if (-not (Test-Path -LiteralPath $compiler)) { throw 'Inno Setup compiler missing after installation' }
"AEGIS_ISCC_PATH=$compiler" >> $env:GITHUB_ENV
Write-Output 'Pinned Inno Setup digest and publisher verified; compiler ready.'
