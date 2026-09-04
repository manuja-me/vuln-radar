<#
.SYNOPSIS
    Submits or validates VulnRadar WinGet manifests to microsoft/winget-pkgs.
.DESCRIPTION
    Validates the local manifests using 'winget validate' and submits
    a Pull Request directly to microsoft/winget-pkgs using 'wingetcreate'.
#>

[CmdletBinding()]
param(
    [switch]$Submit,
    [string]$Version = "0.7.0"
)

$ErrorActionPreference = "Stop"

$ManifestDir = Join-Path $PSScriptRoot "manifests\m\manuja-me\VulnRadar\$Version"

if (-not (Test-Path $ManifestDir)) {
    Write-Error "Manifest directory not found: $ManifestDir"
}

Write-Host "==> Validating WinGet manifests using 'winget validate'..." -ForegroundColor Cyan
winget validate --manifest $ManifestDir

if ($LASTEXITCODE -ne 0) {
    Write-Error "Manifest validation failed!"
}
Write-Host "Manifest validation succeeded!`n" -ForegroundColor Green

if (-not $Submit) {
    Write-Host "Manifests are valid and ready for submission!" -ForegroundColor Yellow
    Write-Host "To submit to microsoft/winget-pkgs:"
    Write-Host "  .\packaging\winget\submit.ps1 -Submit`n"
    return
}

# Ensure wingetcreate is installed
if (-not (Get-Command wingetcreate -ErrorAction SilentlyContinue)) {
    Write-Host "==> Installing wingetcreate..." -ForegroundColor Cyan
    winget install Microsoft.WingetCreate --accept-source-agreements --accept-package-agreements
}

# Fetch GitHub token from gh CLI if available
$token = $null
if (Get-Command gh -ErrorAction SilentlyContinue) {
    $token = (gh auth token 2>$null)
}

Write-Host "==> Submitting manifests to microsoft/winget-pkgs using wingetcreate..." -ForegroundColor Cyan

if ($token) {
    wingetcreate submit --token $token --prtitle "New package: manuja-me.VulnRadar version $Version" $ManifestDir
} else {
    wingetcreate submit --prtitle "New package: manuja-me.VulnRadar version $Version" $ManifestDir
}

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n🎉 Manifest submitted successfully to microsoft/winget-pkgs!" -ForegroundColor Green
} else {
    Write-Error "wingetcreate submission failed. Ensure you have GitHub permissions."
}
