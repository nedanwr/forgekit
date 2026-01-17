$ErrorActionPreference = 'Stop'

$packageName = 'forgekit'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"

# Remove the binary
$exePath = Join-Path $toolsDir 'forgekit.exe'
if (Test-Path $exePath) {
    Remove-Item $exePath -Force
}

Write-Host "ForgeKit uninstalled." -ForegroundColor Green
Write-Host "Note: Dependencies (qpdf, ffmpeg, etc.) were not removed." -ForegroundColor Gray
