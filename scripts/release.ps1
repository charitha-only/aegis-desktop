# ============================================================
# release.ps1 — Version bump + GitHub push script
# Usage:
#   .\scripts\release.ps1          # patch bump (0.29.19 → 0.29.20)
#   .\scripts\release.ps1 minor    # minor bump (0.29.19 → 0.30.0)
#   .\scripts\release.ps1 major    # major bump (0.29.19 → 1.0.0)
#   .\scripts\release.ps1 1.2.3    # exact version
# ============================================================

param(
    [string]$BumpType = "patch"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ── 1. Working directory check ───────────────────────────────
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

Write-Host "`n🔍 Checking git status..." -ForegroundColor Cyan
$status = git status --porcelain
if ($status) {
    Write-Host "❌ Working tree is not clean. Commit or stash changes first:" -ForegroundColor Red
    git status --short
    exit 1
}

# ── 2. Read current version ──────────────────────────────────
$pkgPath = Join-Path $root "package.json"
$pkg = Get-Content $pkgPath -Raw | ConvertFrom-Json
$currentVersion = $pkg.version
Write-Host "📦 Current version: $currentVersion" -ForegroundColor Yellow

# ── 3. Calculate new version ─────────────────────────────────
function Bump-Version($version, $type) {
    $parts = $version -split "\."
    $major = [int]$parts[0]
    $minor = [int]$parts[1]
    $patch = [int]$parts[2]

    switch ($type) {
        "major" { return "$($major + 1).0.0" }
        "minor" { return "$major.$($minor + 1).0" }
        "patch" { return "$major.$minor.$($patch + 1)" }
        default {
            # Treat as exact version if it matches x.y.z
            if ($type -match "^\d+\.\d+\.\d+$") {
                return $type
            }
            Write-Host "❌ Unknown bump type: $type" -ForegroundColor Red
            exit 1
        }
    }
}

$newVersion = Bump-Version $currentVersion $BumpType
Write-Host "🚀 New version:     $newVersion" -ForegroundColor Green

# ── 4. Confirm ───────────────────────────────────────────────
$confirm = Read-Host "`nProceed with $currentVersion → $newVersion? (y/N)"
if ($confirm -notmatch "^[Yy]$") {
    Write-Host "⛔ Aborted." -ForegroundColor Red
    exit 0
}

# ── 5. Update package.json ───────────────────────────────────
Write-Host "`n📝 Updating package.json..." -ForegroundColor Cyan
$pkgRaw = Get-Content $pkgPath -Raw
$pkgRaw = $pkgRaw -replace '"version": "' + [regex]::Escape($currentVersion) + '"', '"version": "' + $newVersion + '"'
Set-Content -Path $pkgPath -Value $pkgRaw -NoNewline
Write-Host "   ✅ package.json → $newVersion"

# ── 6. Ask for release notes ─────────────────────────────────
Write-Host "`n📄 Enter release notes (press Enter twice when done):" -ForegroundColor Cyan
$lines = @()
while ($true) {
    $line = Read-Host
    if ($line -eq "" -and ($lines.Count -eq 0 -or $lines[-1] -eq "")) { break }
    $lines += $line
}
$releaseNotes = ($lines | Where-Object { $_ -ne "" }) -join "`n- "
if ($releaseNotes) { $releaseNotes = "- " + $releaseNotes }
else { $releaseNotes = "- Version bump to $newVersion" }

# ── 7. Update CHANGELOG.md ───────────────────────────────────
Write-Host "`n📋 Updating CHANGELOG.md..." -ForegroundColor Cyan
$changelogPath = Join-Path $root "CHANGELOG.md"
$today = Get-Date -Format "yyyy-MM-dd"
$newEntry = "# v$newVersion — $today`n`n$releaseNotes`n`n"

if (Test-Path $changelogPath) {
    $existing = Get-Content $changelogPath -Raw
    Set-Content -Path $changelogPath -Value ($newEntry + $existing) -NoNewline
} else {
    Set-Content -Path $changelogPath -Value $newEntry -NoNewline
}
Write-Host "   ✅ CHANGELOG.md updated"

# ── 8. Git commit ────────────────────────────────────────────
Write-Host "`n📦 Creating git commit..." -ForegroundColor Cyan
git add package.json CHANGELOG.md
git commit -m "chore: bump version to v$newVersion"
Write-Host "   ✅ Committed"

# ── 9. Git tag ───────────────────────────────────────────────
Write-Host "`n🏷️  Creating git tag v$newVersion..." -ForegroundColor Cyan
git tag -a "v$newVersion" -m "Release v$newVersion"
Write-Host "   ✅ Tagged"

# ── 10. Push to GitHub ───────────────────────────────────────
Write-Host "`n🚀 Pushing to GitHub..." -ForegroundColor Cyan
git push origin main
git push origin "v$newVersion"
Write-Host "   ✅ Pushed to GitHub" -ForegroundColor Green

Write-Host "`n✨ Done! v$newVersion is now live on GitHub." -ForegroundColor Green
Write-Host "   https://github.com/charitha-only/aegis-desktop/releases/tag/v$newVersion`n" -ForegroundColor Blue
