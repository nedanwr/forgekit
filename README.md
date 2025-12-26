# ForgeKit

Local-first media and PDF toolkit. Fast, lightweight, and privacy-focused.

**Repository**: https://github.com/nedanwar/forgekit

## Quick Start

### Installation

1. Install dependencies (see [Installation](#installation) section below)
2. Download the latest release from GitHub Releases
3. Extract and add to your PATH

### Examples

**PDF Operations:**

```bash
# Merge multiple PDFs
forgekit pdf merge doc1.pdf doc2.pdf --output merged.pdf

# Merge with linearization (fast web view)
forgekit pdf merge *.pdf --output all.pdf --linearize

# Split PDF by pages
forgekit pdf split book.pdf --output-dir pages/ --pages 1-5,10-20

# Extract odd pages
forgekit pdf split book.pdf --output-dir pages/ --pages odd

# See what commands would run (without executing)
forgekit pdf merge a.pdf b.pdf --output c.pdf --plan

# JSON output for scripting
forgekit pdf merge *.pdf --output out.pdf --json | jq -r '.result.output'
```

**Page Specification:**

- Numbers: `1`, `42`
- Ranges: `1-5`, `10-20`, `7-` (7 to end), `-10` (1 to 10)
- Keywords: `odd`, `even`, `first`, `last`
- Exclusions: `!2`, `!5-10`
- Combined: `1-3,5,7-`, `odd`, `even`, `1-10,!2,!5`

## Installation

### Package Manager Installation (Recommended)

Install ForgeKit using your system's package manager. Dependencies are automatically installed alongside ForgeKit - no additional commands needed.

**macOS (Homebrew):**

```bash
brew install forgekit
```

**Windows (winget):**

```powershell
winget install forgekit
```

**Debian/Ubuntu:**

```bash
sudo apt install forgekit
```

**Fedora/RHEL:**

```bash
sudo dnf install forgekit
```

**Arch Linux:**

```bash
# Available via AUR (when published)
yay -S forgekit
# or
pacman -S forgekit
```

### Manual Installation

If package manager installation isn't available, you can install ForgeKit manually and then install dependencies separately.

**1. Install ForgeKit binary:**

Download the latest release from [GitHub Releases](https://github.com/nedanwar/forgekit/releases) and add to your PATH.

**2. Install dependencies:**

Check which dependencies are missing:

```bash
forgekit check-deps
```

Then install them based on your platform:

**macOS (Homebrew):**

```bash
brew install qpdf ghostscript tesseract ffmpeg libvips exiftool
pip3 install ocrmypdf
```

**Windows (winget/scoop):**

```powershell
winget install qpdf.qpdf ArtifexSoftware.GhostScript tesseract-ocr Gyan.FFmpeg
scoop install libvips exiftool
pip install ocrmypdf
```

**Linux:**

**Debian/Ubuntu:**

```bash
sudo apt install qpdf ghostscript tesseract-ocr ffmpeg libvips-tools libimage-exiftool-perl python3-pip
pip3 install ocrmypdf
```

**Fedora/RHEL:**

```bash
sudo dnf install qpdf ghostscript tesseract ffmpeg libvips perl-Image-ExifTool python3-pip
pip3 install ocrmypdf
```

**Arch Linux:**

```bash
sudo pacman -S qpdf ghostscript tesseract ffmpeg libvips perl-image-exiftool python-pip
pip3 install ocrmypdf
```

**Note:** If you have a package manager available, we strongly recommend using package manager installation instead (see above). It automatically handles all dependencies including Python and ocrmypdf.

## Features

### PDF Operations

- **Merge**: Combine multiple PDFs into one
- **Split**: Extract pages by ranges or keywords
- **Linearize**: Optimize for fast web view
- **Compress**: Reduce file size with Ghostscript presets
- **OCR**: Add searchable text layer (coming soon)
- **Metadata**: View/edit PDF metadata (coming soon)

### Image Operations (coming soon)

- Convert formats (WebP, AVIF, JPEG)
- Resize with aspect ratio preservation
- Strip EXIF metadata

### Media Operations (coming soon)

- Video transcoding (H.264)
- Audio conversion and normalization

## Global Flags

- `--json`: Output progress as NDJSON (one event per line)
- `--plan`: Show underlying commands without executing
- `--dry-run`: Validate inputs and show plan, don't execute
- `--log-level <level>`: Set log level (debug|info|warn|error)
- `--force`: Overwrite existing files
- `--tools.<name>=path`: Override tool path (e.g., `--tools.qpdf=/usr/local/bin/qpdf`)

## Exit Codes

- `0`: Success
- `1`: General error (processing failed)
- `2`: Missing tool (with install hint)
- `3`: Invalid input (file not found, invalid pages spec, etc.)
- `4`: Permission denied
- `5`: Disk full
- `130`: Cancelled (SIGINT)

## JSON Output

When using `--json`, ForgeKit outputs NDJSON (newline-delimited JSON) events:

```json
{"type":"progress","job_id":"abc123","progress":{"current":1,"total":3,"percent":33},"message":"Merging page 1/3"}
{"type":"progress","job_id":"abc123","progress":{"current":2,"total":3,"percent":67},"message":"Merging page 2/3"}
{"type":"complete","job_id":"abc123","result":{"output":"merged.pdf","size_bytes":123456,"duration_ms":1234}}
```

## Troubleshooting

### Tool not found

If you see "Tool 'qpdf' not found", install the required dependencies (see [Installation](#installation)).

You can also override tool paths:

```bash
forgekit pdf merge a.pdf b.pdf --output c.pdf --tools.qpdf=/custom/path/to/qpdf
```

### Permission denied

Ensure you have read access to input files and write access to output directories.

### Invalid page spec

Page numbers must be >= 1. Ranges must have start <= end. Use `--help` for examples.

## Contributing

Contributions welcome! Please open an issue or pull request on [GitHub](https://github.com/nedanwar/forgekit).

## License

MIT
