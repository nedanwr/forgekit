# ForgeKit Package Dependencies

This directory contains package dependency declarations for different package managers.
These files ensure that when users install ForgeKit via their system's package manager,
all required external tools are automatically installed as dependencies.

## Package Formats

### Debian/Ubuntu (`.deb`)

- **File**: `debian/control`
- **Usage**: Included in `.deb` package metadata
- **Dependencies**: qpdf, pdfcpu, tesseract-ocr, ffmpeg, libvips-tools, libimage-exiftool-perl, python3-pip

### Fedora/RHEL/CentOS (`.rpm`)

- **File**: `rpm/forgekit.spec`
- **Usage**: Used to build `.rpm` packages with `rpmbuild`
- **Dependencies**: qpdf, pdfcpu, tesseract, ffmpeg, libvips, perl-Image-ExifTool, python3-pip

### macOS (Homebrew)

- **File**: `homebrew/forgekit.rb`
- **Usage**: Homebrew formula file
- **Dependencies**: qpdf, pdfcpu, tesseract, ffmpeg, libvips, exiftool, python@3

### Windows (winget)

- **File**: `winget/forgekit.yaml`
- **Usage**: winget manifest for Windows Package Manager
- **Dependencies**: qpdf.qpdf, pdfcpu.pdfcpu, tesseract-ocr, Gyan.FFmpeg, libvips.libvips, exiftool.exiftool, Python.Python.3

## Building Packages

### Debian Package

```bash
dpkg-buildpackage -us -uc
```

### RPM Package

```bash
rpmbuild -ba rpm/forgekit.spec
```

### Homebrew Formula

```bash
# Copy to Homebrew tap repository
cp homebrew/forgekit.rb /path/to/homebrew-tap/Formula/forgekit.rb
```

### winget Manifest

```bash
# Submit to winget-pkgs repository
# See: https://github.com/microsoft/winget-pkgs
```

## Post-Installation Scripts

All package formats include post-installation hooks that automatically install the `ocrmypdf` Python package:

- **Homebrew**: Uses `post_install` hook to run `pip3 install ocrmypdf`
- **Debian/RPM**: Includes `postinst` script that installs `ocrmypdf` via pip3
- **winget**: Post-install script (`postinstall.ps1`) is included (installer must execute it)

This ensures **single-command installation** - users don't need to manually install Python dependencies.

### How Package Managers Handle Existing Dependencies

**Important**: Package managers automatically skip installing dependencies that are already present:

- **Homebrew**: If Python@3 is already installed, Homebrew will use the existing installation
- **apt/dnf**: If Python3 is already installed, the package manager will skip installing it again
- **winget**: If Python is already installed, winget will skip installing it

The post-install scripts detect and use the existing Python installation, whether it was installed by the package manager or was already present on the system.

## Notes

- Python packages (like `ocrmypdf`) are automatically installed via post-install scripts
- Some tools may have different names across distributions (e.g., `libimage-exiftool-perl` vs `perl-Image-ExifTool`)
- Windows dependencies use winget package identifiers, which may differ from tool names
- For winget, the installer executable should execute `postinstall.ps1` after installation
