# Post-installation script for ForgeKit winget package
# Automatically installs ocrmypdf Python package

# Check if Python and pip are available
if (Get-Command python -ErrorAction SilentlyContinue) {
    Write-Host "Installing Python dependencies for ForgeKit..." -ForegroundColor Cyan
    python -m pip install --quiet ocrmypdf 2>$null
} elseif (Get-Command python3 -ErrorAction SilentlyContinue) {
    Write-Host "Installing Python dependencies for ForgeKit..." -ForegroundColor Cyan
    python3 -m pip install --quiet ocrmypdf 2>$null
} elseif (Get-Command pip -ErrorAction SilentlyContinue) {
    Write-Host "Installing Python dependencies for ForgeKit..." -ForegroundColor Cyan
    pip install --quiet ocrmypdf 2>$null
}

