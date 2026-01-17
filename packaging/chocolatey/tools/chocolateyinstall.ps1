$ErrorActionPreference = 'Stop'

$packageName = 'forgekit'
$version = '0.0.9'
$url64 = "https://github.com/nedanwr/forgekit/releases/download/v$version/forgekit-v$version-x86_64-pc-windows-msvc.zip"
$checksum64 = 'PLACEHOLDER_SHA256'

$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"

# Download and extract
$packageArgs = @{
  packageName   = $packageName
  unzipLocation = $toolsDir
  url64bit      = $url64
  checksum64    = $checksum64
  checksumType64= 'sha256'
}
Install-ChocolateyZipPackage @packageArgs

# Install ocrmypdf via pip (dependencies already installed by Chocolatey)
Write-Host "Installing ocrmypdf Python package..." -ForegroundColor Cyan
try {
    & python -m pip install --quiet --upgrade pip 2>$null
    & python -m pip install --quiet ocrmypdf 2>$null
    Write-Host "Successfully installed ocrmypdf" -ForegroundColor Green
} catch {
    Write-Warning "Could not install ocrmypdf. Run manually: pip install ocrmypdf"
}

# Install libvips via scoop if available (not in Chocolatey)
if (Get-Command scoop -ErrorAction SilentlyContinue) {
    Write-Host "Installing libvips via scoop..." -ForegroundColor Cyan
    & scoop install libvips 2>$null
} else {
    Write-Warning "libvips not installed (requires Scoop). Image operations may be limited."
    Write-Warning "To install: Install Scoop (scoop.sh), then run: scoop install libvips"
}

Write-Host ""
Write-Host "ForgeKit installed successfully!" -ForegroundColor Green
Write-Host "Run 'forgekit check-deps' to verify all dependencies." -ForegroundColor Gray
