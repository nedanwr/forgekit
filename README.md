# ForgeKit

Local-first media and PDF toolkit. Fast, lightweight, and privacy-focused.

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

### Dependencies

ForgeKit requires external tools to be installed. Check if all dependencies are available:

```bash
forgekit check-deps
```

### macOS (Homebrew)

```bash
brew install qpdf pdfcpu tesseract ffmpeg libvips exiftool
pip3 install ocrmypdf
```

### Windows (winget/scoop)

```powershell
winget install qpdf qpdf
winget install pdfcpu pdfcpu
winget install tesseract-ocr
winget install ffmpeg
scoop install libvips exiftool
pip install ocrmypdf
```

### Linux (apt/pacman)

**Debian/Ubuntu:**
```bash
sudo apt install qpdf pdfcpu tesseract-ocr ffmpeg libvips-tools libimage-exiftool-perl
pip3 install ocrmypdf
```

**Arch:**
```bash
sudo pacman -S qpdf pdfcpu tesseract ffmpeg libvips perl-image-exiftool
pip3 install ocrmypdf
```

## Features

### PDF Operations

- **Merge**: Combine multiple PDFs into one
- **Split**: Extract pages by ranges or keywords
- **Linearize**: Optimize for fast web view
- **Compress**: Reduce file size with presets (coming soon)
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

Contributions welcome! Please open an issue or pull request.

## License

MIT OR Apache-2.0

