param (
    [ValidateSet("full", "variants", "tokens")]
    [string]$Target = "full"
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$targetFile = switch ($Target) {
    "full"     { Join-Path $scriptDir "rustic_player_full_ui.svg" }
    "variants" { Join-Path $scriptDir "floating_glass_dock_variants.svg" }
    "tokens"   { Join-Path $scriptDir "rustic_player_design_tokens.svg" }
}

if (-not (Test-Path $targetFile)) {
    Write-Error "Target file not found: $targetFile"
    exit 1
}

$svgContent = Get-Content -Path $targetFile -Raw -Encoding UTF8
Set-Clipboard -Value $svgContent

Write-Host "`n========================================================" -ForegroundColor Green
Write-Host " [SUCCESS] Copied '$Target' SVG vector data to Clipboard!" -ForegroundColor Cyan
Write-Host " Source: $targetFile" -ForegroundColor DarkGray
Write-Host "========================================================" -ForegroundColor Green
Write-Host "Next Step:" -ForegroundColor Yellow
Write-Host "  1. Switch to your Figma desktop window"
Write-Host "  2. Open any Figma design file / canvas"
Write-Host "  3. Press Ctrl + V to paste!"
Write-Host "     Figma will instantly instantiate native vector frames & layers."
Write-Host "========================================================`n"
