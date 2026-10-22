# Secure script to set Figma Personal Access Token in mcp_config.json without leaking to terminal
$ErrorActionPreference = "Stop"
$configFile = "C:\Users\Ketan\.gemini\config\mcp_config.json"

if (-not (Test-Path $configFile)) {
    Write-Error "Config file not found at: $configFile"
    exit 1
}

Write-Host "`n=== Setup Figma MCP Access Token ===" -ForegroundColor Cyan
Write-Host "You can obtain a Personal Access Token in Figma:" -ForegroundColor DarkGray
Write-Host "  1. Open Figma -> Click your Profile -> Settings" -ForegroundColor DarkGray
Write-Host "  2. Scroll down to 'Personal access tokens'" -ForegroundColor DarkGray
Write-Host "  3. Click 'Generate new token' and copy it (starts with figd_)" -ForegroundColor DarkGray
Write-Host "====================================`n" -ForegroundColor Cyan

$secureToken = Read-Host "Paste your Figma Personal Access Token (input hidden)" -AsSecureString
$bstr = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($secureToken)
$plainToken = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto($bstr)
[System.Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr)

if ([string]::IsNullOrWhiteSpace($plainToken)) {
    Write-Warning "No token entered. Aborting."
    exit 0
}

# Read JSON, update FIGMA_API_KEY, and save cleanly
$json = Get-Content -Path $configFile -Raw -Encoding UTF8 | ConvertFrom-Json

if ($null -eq $json.mcpServers.figma) {
    Write-Error "Figma server entry missing from mcp_config.json"
    exit 1
}

$json.mcpServers.figma.env.FIGMA_API_KEY = $plainToken

$updatedJson = $json | ConvertTo-Json -Depth 10
Set-Content -Path $configFile -Value $updatedJson -Encoding UTF8

Write-Host "`n[SUCCESS] FIGMA_API_KEY successfully configured in $configFile!" -ForegroundColor Green
Write-Host "The Figma MCP server is now active and ready to connect to your Figma files.`n" -ForegroundColor Cyan
