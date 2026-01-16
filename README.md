# ForgeKit

Local-first media and PDF toolkit. Fast, lightweight, and privacy-focused.

## Installation

**macOS:** `brew install qpdf ghostscript tesseract ffmpeg libvips exiftool && pip3 install ocrmypdf`

**Linux (Debian/Ubuntu):** `sudo apt install qpdf ghostscript tesseract-ocr ffmpeg libvips-tools libimage-exiftool-perl && pip3 install ocrmypdf`

**Windows:** `winget install qpdf.qpdf ArtifexSoftware.GhostScript tesseract-ocr && scoop install libvips exiftool && pip install ocrmypdf`

Then download the latest release from [GitHub Releases](https://github.com/nedanwr/forgekit/releases).

Run `forgekit check-deps` to verify all dependencies are installed.

## Usage

### PDF Operations

```bash
forgekit pdf merge doc1.pdf doc2.pdf --output merged.pdf
forgekit pdf split book.pdf --output-dir pages/ --pages 1-5
forgekit pdf compress large.pdf --output small.pdf --level high
forgekit pdf ocr scan.pdf --output searchable.pdf --language eng
forgekit pdf metadata doc.pdf --set title="My Doc" --output updated.pdf
```

### Image Operations

```bash
forgekit image convert photo.jpg -t webp --quality 80    # photo.webp
forgekit image resize photo.jpg --width 800              # photo_800w.jpg
forgekit image strip photo.jpg                           # photo_stripped.jpg
forgekit image compress photo.jpg --quality 60           # photo_compressed.jpg
forgekit image info photo.jpg --exif                     # show dimensions + EXIF
```

### Global Options

- `--plan` - Show commands without executing
- `--json` - Output progress as NDJSON

## Page Specification

- Numbers: `1`, `42`
- Ranges: `1-5`, `7-` (7 to end), `-10` (1 to 10)
- Keywords: `odd`, `even`
- Exclusions: `!2`, `!5-10`
- Combined: `1-3,5,7-`, `1-10,!2`

## Exit Codes

- `0` Success
- `1` General error
- `2` Missing tool (run `check-deps`)
- `3` Invalid input

## License

MIT
