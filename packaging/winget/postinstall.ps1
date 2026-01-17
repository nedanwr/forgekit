# ForgeKit Windows Dependency Installer
# Run this script after installing ForgeKit to install all required dependencies.
# Usage: .\postinstall.ps1

Write-Host "ForgeKit Dependency Installer" -ForegroundColor Green
Write-Host "==============================" -ForegroundColor Green
Write-Host ""

# Check for winget
if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
    Write-Host "Error: winget not found. Please install App Installer from Microsoft Store." -ForegroundColor Red
    exit 1
}

# Install winget packages
Write-Host "Installing winget packages..." -ForegroundColor Cyan
$wingetPackages = @(
    "qpdf.qpdf",
    "ArtifexSoftware.GhostScript",
    "UB-Mannheim.TesseractOCR",
    "Gyan.FFmpeg",
    "Python.Python.3"
)

foreach ($pkg in $wingetPackages) {
    Write-Host "  Installing $pkg..." -ForegroundColor Yellow
    winget install --id $pkg --silent --accept-package-agreements --accept-source-agreements 2>$null
}

# Check for scoop (needed for libvips and exiftool)
if (Get-Command scoop -ErrorAction SilentlyContinue) {
    Write-Host "Installing scoop packages..." -ForegroundColor Cyan
    scoop install libvips exiftool 2>$null
} else {
    Write-Host ""
    Write-Host "Note: Scoop not found. For full functionality, install Scoop and run:" -ForegroundColor Yellow
    Write-Host "  scoop install libvips exiftool" -ForegroundColor White
    Write-Host ""
    Write-Host "To install Scoop: https://scoop.sh" -ForegroundColor Gray
}

# Install ocrmypdf via pip
Write-Host "Installing Python packages..." -ForegroundColor Cyan
if (Get-Command python -ErrorAction SilentlyContinue) {
    python -m pip install --quiet --upgrade pip 2>$null
    python -m pip install --quiet ocrmypdf 2>$null
    Write-Host "  Installed ocrmypdf" -ForegroundColor Yellow
} else {
    Write-Host "  Warning: Python not found. Restart terminal and run: pip install ocrmypdf" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Installation complete!" -ForegroundColor Green
Write-Host "Run 'forgekit check-deps' to verify all dependencies." -ForegroundColor Gray

