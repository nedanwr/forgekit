# ForgeKit Beta-MVP Roadmap (CLI-First)

**Last Updated**: January 2026
**Status**: Core foundation complete, PDF operations complete, Image operations complete, Audio operations complete
**Current Version**: v0.0.7

## Versioning Strategy

**Version Format**: `MAJOR.MINOR.PATCH` (Semantic Versioning)

- **MVP Release**: `0.1.0` - Beta-MVP feature complete
- **GA Release**: `1.0.0` - Production-ready, stable API
- **Pre-MVP**: `0.0.x` - Development releases leading to MVP

### Release Roadmap

#### v0.0.1 - Core Foundation

**Status**: ✅ Completed

**Deliverables**:

- Rust workspace structure (core + CLI)
- Tool trait system and qpdf adapter
- Error handling with exit codes
- Basic job execution framework
- Unit tests for core functionality
- `.gitignore` and basic project structure

#### v0.0.2 - PDF Basic Operations ✅

**Status**: Completed (Current)

**Deliverables**:

- PDF merge command (`pdf merge`) with linearize support
- PDF split command (`pdf split`) with pages grammar
- Pages grammar parser (1-3,5,7-,odd,even,!2) with 11 comprehensive tests
- JSON progress output (NDJSON with version 1 schema)
- `--plan` and `--dry-run` flags working
- Comprehensive tests (20 tests: 13 unit, 2 integration, 5 doc)
- Comprehensive documentation (README, CONTRIBUTING, inline docs)
- `check-deps` command (basic implementation)
- Error handling with actionable hints

#### v0.0.3 - Dependency Management ✅

**Status**: Completed

**Deliverables**:

- ✅ Enhanced `check-deps` command with platform-specific install instructions
- ✅ Package dependency declarations:
  - Debian/Ubuntu (`.deb` package metadata with Depends field)
  - RPM (Fedora/RHEL package metadata with Requires field)
  - Homebrew formula (Ruby with `depends_on` statements)
  - winget manifest (YAML with Dependencies section)
- ✅ Post-install hooks for automatic Python package installation (ocrmypdf)
- ✅ Improved tool detection (PATH-first, config fallback)
- ✅ Error messages with actionable install hints per platform
- ✅ Documentation for dependency installation per platform
- ✅ Tests for dependency checking (10 integration tests covering tool probing, platform detection, install hints, and error handling)

#### v0.0.4 - PDF Advanced Operations ✅

**Status**: Completed

**Deliverables**:

- ✅ PDF compress command (`pdf compress`) with preset support
- ✅ PDF linearize command (`pdf linearize`) as standalone subcommand
- ✅ PDF reorder command (`pdf reorder`) with page ordering
- ✅ PDF extract command (`pdf extract`) with page selection
- ✅ ghostscript tool adapter (probe, version, execute)
- ✅ Preset system foundation (YAML loader and parser)
- ✅ Tests for new PDF operations
- ✅ Integration with existing pages grammar parser

#### v0.0.5 - PDF OCR and Metadata ✅

**Status**: Completed

**Deliverables**:

- ✅ PDF OCR command (`pdf ocr`) with language selection
- ✅ PDF metadata command (`pdf metadata`) with get/set operations
- ✅ ocrmypdf tool adapter (Python wrapper handling)
- ✅ exiftool adapter for metadata operations
- ✅ OCR progress reporting (basic progress events)
- ✅ Metadata read/write operations (get all, get field, set fields)
- ✅ Tests for OCR and metadata operations (9 new tests)
- ✅ Error handling for OCR failures
- ✅ OCR options: --skip-text, --deskew, --force-ocr
- ✅ Integration with check-deps command

#### v0.0.6 - Image Operations ✅

**Status**: Completed

**Deliverables**:

- ✅ Image convert command (`image convert`) with format selection and optional output
- ✅ Image resize command (`image resize`) with aspect ratio preservation
- ✅ Image strip command (`image strip`) for EXIF removal
- ✅ Image compress command (`image compress`) for quality reduction
- ✅ Image info command (`image info`) for dimensions/format/metadata
- ✅ libvips tool adapter with compression parameter support
- ✅ Optional output paths with smart auto-naming (e.g., `photo_800w.jpg`, `photo_stripped.jpg`)
- ✅ PNG compression control (1-9) for conversion
- ✅ RAW format input support (DNG, CR2, NEF, etc.)

#### v0.0.7 - Audio Operations ✅

**Status**: Completed

**Deliverables**:

- ✅ Audio convert command (`audio convert`) with format/bitrate selection
- ✅ Audio normalize command (`audio normalize`) with EBU R128 support
- ✅ Audio info command (`audio info`) for file information
- ✅ ffmpeg audio adapter (codec selection, bitrate control)
- ✅ Loudness normalization support (EBU R128, Streaming, custom LUFS)
- ✅ Supported formats: MP3, AAC, Opus, FLAC, WAV, OGG, M4A
- ✅ Tests for audio operations (12 new tests)

#### v0.0.8 - Video Operations 📋

**Status**: Pending

**Deliverables**:

- Media transcode command (`media transcode`) with preset support
- ffmpeg video adapter (H.264 only, software x264)
- Video preset (H.264 1080p with CRF 23)
- Progress parsing from ffmpeg stderr (time= and Duration=)
- CRF-based quality control (0-51 range)
- Scale filter with aspect ratio preservation
- Tests for video operations (duration, codec, resolution)
- Golden tests for video transcoding

#### v0.0.9 - Package Creation and CI/CD 📋

**Status**: Pending

**Deliverables**:

- Debian package (`.deb`) with dependency declarations
- RPM package (`.rpm`) with dependency declarations
- Homebrew formula with dependencies (Ruby)
- winget manifest with dependencies (YAML)
- GitHub Actions CI/CD workflow:
  - Build binaries for macOS, Windows, Linux
  - Create platform-specific packages
  - Run full test suite on all platforms
  - Generate checksums (SHA256)
  - Upload artifacts to GitHub Releases
- Automated package building scripts
- Release automation (version bumping, changelog)
- Package repository setup (Homebrew tap, winget source)

#### v0.1.0 - MVP Release (Beta-MVP) 📋

**Status**: Pending

**Deliverables**:

- ✅ All CLI commands implemented and tested:
  - PDF: merge, split, compress, linearize, reorder, extract, ocr, metadata
  - Image: convert, resize, strip, compress, info
  - Audio: convert, normalize
  - Media: transcode
- ✅ All package formats available (deb, rpm, Homebrew, winget)
- ✅ Comprehensive documentation (README, CONTRIBUTING, ROADMAP)
- ✅ CI/CD passing on all platforms (macOS, Windows, Linux)
- ✅ Golden tests for all operations
- ✅ Preset system complete (YAML loader, all presets defined)
- ✅ Production-ready error handling with actionable hints
- ✅ Stable JSON progress API (version 1 schema)
- ✅ Dependency management working (automatic installation)
- ✅ Installation instructions for all platforms
- ✅ Binary-only releases as fallback option

**Post-MVP (v0.1.x)**:

- Bug fixes and stability improvements
- Performance optimizations
- Additional presets (user-requested)
- Documentation improvements
- Minor feature additions (non-breaking)

**v1.0.0 - GA Release** 📋
**Status**: Post-MVP

**Deliverables**:

- Stable API (no breaking changes from 0.1.0)
- Comprehensive test coverage (>90%)
- Production-hardened error handling
- Full documentation suite (user guide, API docs)
- Community feedback incorporated
- Performance optimizations completed
- Security audit completed
- Long-term support commitment
- Migration guide from 0.x to 1.0

**v1.0.0 - GA Release**:

- Stable API (no breaking changes)
- Comprehensive test coverage
- Production-hardened
- Full documentation
- Community feedback incorporated

## Current Implementation Status

### ✅ Completed Features

- **Core Foundation**: Tool trait system, error handling, job specs
- **PDF Operations**: Merge, split, compress, linearize, reorder, extract, OCR, metadata
- **Image Operations**: Convert, resize, strip, compress, info (via libvips)
- **Audio Operations**: Convert, normalize, info (via ffmpeg)
- **Pages Grammar**: Full parser with comprehensive tests (11 tests)
- **Progress Reporting**: NDJSON output with versioned schema
- **CLI Flags**: `--json`, `--plan`, `--dry-run` working
- **Documentation**: Comprehensive inline docs, README, CONTRIBUTING guide
- **Dependency Checking**: `check-deps` command implemented

**Current Version**: v0.0.7 (completed)

### 🔄 In Progress

- **v0.0.8**: Video Operations

### 📋 Pending Features

- Media: transcode
- Preset system (YAML) ✅
- Package creation (deb, rpm, Homebrew, winget)
- CI/CD setup with package building

## 1. Product Goals and Non-Goals

### Beta-MVP Goals (CLI-First)

- **Speed**: Sub-second startup, near-native tool performance, minimal overhead
- **Small binaries**: CLI <15 MB stripped
- **Offline privacy**: Zero network access, all processing local, no telemetry
- **CLI excellence**: Consistent subcommands, clear flags, comprehensive --help, working examples
- **Automation-ready**: Stable JSON progress/events, --plan/--dry-run, predictable exit codes
- **Reliability**: Atomic writes, safe temp handling, clear error messages with actionable hints
- **Dependency clarity**: Dependencies automatically installed via package managers

### Non-Goals (Beta-MVP)

- No GUI (Tauri deferred to post-beta-MVP)
- No server/daemon/localhost API (strictly enforced)
- No web app or browser-based UI
- No advanced PDF forms editing or redaction guarantees
- No complex video filters, color grading, or subtitle editing
- No smartcards/HSM signing or network services
- No cloud sync or multi-device features
- No hardware-accelerated video (software x264 only)
- No H.265/AV1 codecs (H.264 only for beta-MVP)

## 2. Personas and Top Workflows

### Personas

1. **Power user (Bob)**: CLI scripting, batch processing, automation (primary beta-MVP user)
2. **Creator (Carol)**: Batch media conversion, consistent quality presets
3. **Non-technical user (Alice)**: Deferred to post-beta-MVP (GUI)

### Workflows (8-12 mapped to CLI commands)

**PDF Workflows:**

- `pdf merge`: Combine multiple PDFs into one (Bob: `forgekit pdf merge *.pdf --output merged.pdf`)
- `pdf split`: Extract pages into separate files (Bob: `forgekit pdf split doc.pdf --pages 1-5,10-20`)
- `pdf reorder`: Rearrange pages (Carol: `forgekit pdf reorder book.pdf --order 10-1,5,7-9`)
- `pdf compress`: Reduce file size with presets (Bob: `forgekit pdf compress large.pdf --output small.pdf --preset web`)
- `pdf linearize`: Fast web view optimization (Bob: `forgekit pdf linearize *.pdf`)
- `pdf extract`: Extract specific pages (Carol: `forgekit pdf extract --pages odd,1-10`)
- `pdf ocr`: Add searchable text layer (Bob: `forgekit pdf ocr scan.pdf --output searchable.pdf`)
- `pdf metadata`: View/edit title/author/subject (Bob: `forgekit pdf metadata doc.pdf --set title="Report"`)

**Image Workflows:**

- `image convert`: Format conversion with quality presets (Carol: `forgekit image convert *.jpg --to webp --preset web`)
- `image resize`: Smart resize with aspect ratio (Bob: `forgekit image resize photo.jpg --width 1920 --output resized.jpg`)
- `image strip`: Remove EXIF metadata (Bob: `forgekit image strip photos/ --recursive`)

**Media Workflows:**

- `media transcode`: Video conversion H.264 (Carol: `forgekit media transcode video.mp4 --preset h264-1080p`)
- `audio convert`: Format conversion (Bob: `forgekit audio convert *.mp3 --to opus --preset opus-128k`)
- `audio normalize`: Loudness normalization (Carol: `forgekit audio normalize *.wav --ebu-r128`)

## 3. Architecture Decision Records

### ADR-0001: Rust Core + CLI, No GUI/Web/Daemon (Beta-MVP)

**Decision**: Single Rust codebase with two components: `crates/core` (pure logic), `crates/cli` (clap CLI). No GUI, no HTTP servers, no localhost APIs, no background daemons. Tauri GUI deferred to post-beta-MVP.

**Rationale**:

- Rust provides performance, safety, and small binaries
- CLI-first enables automation and scripting
- Single codebase reduces maintenance burden
- GUI can consume same core crate later

**Trade-offs**:

- ✅ Fast, lightweight, offline-first
- ✅ Excellent for automation and batch processing
- ✅ No GUI complexity in beta-MVP
- ❌ No GUI (by design for beta-MVP)

### ADR-0002: External Tools Strategy

**Decision**: Treat external tools as dependencies that are automatically installed alongside ForgeKit. Similar to how `apt install` or `brew install` handles dependencies, ForgeKit installation packages/scripts will automatically install required tools. Users don't need to manually install dependencies.

**Rationale**:

- **Familiar pattern**: Users understand "install package → dependencies come with it"
- **Smaller downloads**: ForgeKit binary is small, dependencies installed via system package managers
- **System integration**: Uses native package managers (apt, brew, winget) for dependency management
- **Automatic updates**: Dependencies can be updated via system package managers
- **Disk efficiency**: Shared dependencies across applications
- **Standard practice**: Follows conventions users expect from system packages

**Trade-offs**:

- ✅ Familiar installation pattern (like any system package)
- ✅ Smaller ForgeKit download (just the binary)
- ✅ Dependencies managed by system package managers
- ✅ Automatic dependency resolution
- ❌ Requires package manager (apt/brew/winget) - but most users have these
- ❌ Installation requires admin/root privileges (standard for system packages)
- ❌ Platform-specific packaging needed (deb, rpm, pkg, msi, etc.)

**Minimum versions**:

- qpdf: 10.0+
- ghostscript: 0.4+
- ocrmypdf: 14.0+ (Python 3.8+)
- tesseract: 5.0+
- ffmpeg: 5.0+
- libvips: 8.12+
- exiftool: 12.0+

### ADR-0003: Progress/Event JSON Format

**Decision**: Line-delimited JSON (NDJSON) for progress events. Stable contract: `{"type": "progress|complete|error", "version": 1, "job_id": "...", "progress": {...}, "message": "..."}`. Exit codes: 0=success, 1=error, 2=missing tool, 3=invalid input.

**Rationale**:

- Streamable, parseable, debuggable
- CLI and future GUI can share parser
- Exit codes enable scripting
- Versioned schema for future compatibility

**Example event stream**:

```json
{"type":"progress","version":1,"job_id":"abc123","progress":{"current":1,"total":3,"percent":33},"message":"Merging page 1/3"}
{"type":"progress","version":1,"job_id":"abc123","progress":{"current":2,"total":3,"percent":67},"message":"Merging page 2/3"}
{"type":"complete","version":1,"job_id":"abc123","result":{"output":"merged.pdf","size_bytes":123456,"duration_ms":1234}}
```

### ADR-0004: Preset System

**Decision**: Versioned YAML presets stored in `presets/` directory. Declarative mapping to tool flags. CLI: `--preset <name>`.

**Rationale**:

- Versioned for future compatibility
- YAML is human-readable
- Easy to extend without code changes
- Reproducible quality settings

**Preset structure**:

```yaml
version: 1
presets:
  pdf-compress-web:
    tool: ghostscript
    args: ["optimize", "-level=2"]
  image-webp-web:
    tool: libvips
    args: ["--quality=70", "--strip"]
```

### ADR-0005: Build/Package Policy

**Decision**: Rust release builds with `strip=true`, LTO enabled. Target size: CLI <15 MB (excluding system deps). Binary releases only (no GUI packaging in beta-MVP).

**Rationale**:

- Stripped binaries remove debug symbols
- LTO optimizes across crates
- Binary releases are simplest for CLI
- Realistic targets for beta-MVP

**Packaging**:

- macOS: Binary release (`.tar.gz` or `.zip`)
- Windows: Portable ZIP with `.exe`
- Linux: Binary release (`.tar.gz`)
- GitHub Releases with checksums (SHA256)

## 4. System Architecture Detail

### Module Layout (`crates/core/src/`)

```
lib.rs                    # Public API exports
job/                      # Job model and execution
  mod.rs
  spec.rs                 # JobSpec enum (PdfMerge, ImageConvert, etc.)
  executor.rs             # JobExecutor trait and implementations
  progress.rs             # ProgressEvent, ProgressReporter trait
tools/                    # External tool adapters
  mod.rs
  trait_def.rs           # Tool trait: probe(), version(), execute()
  qpdf.rs                # qpdf adapter
  ghostscript.rs              # ghostscript adapter
  ocrmypdf.rs            # ocrmypdf adapter
  ffmpeg.rs              # ffmpeg adapter
  libvips.rs             # libvips adapter
  exiftool.rs            # exiftool adapter
pipeline/                 # Multi-step pipelines
  mod.rs
  builder.rs             # PipelineBuilder
config/                   # Configuration resolution
  mod.rs
  probe.rs               # Tool detection and PATH probing
  paths.rs               # Config file paths, tool overrides
presets/                  # Preset loading and application
  mod.rs
  loader.rs              # YAML preset loader
  mapper.rs              # Map presets to tool args
utils/                    # Utilities
  temp.rs                # Temp file creation, atomic writes
  pages.rs               # Pages grammar parser (1-3,5,7-,odd,even,!2)
  error.rs               # Error taxonomy, ExitCode enum
```

### Tool Trait Design

```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn probe(&self, config: &ToolConfig) -> Result<ToolInfo>;
    fn version(&self, path: &PathBuf) -> Result<String>;
}

pub struct ToolInfo {
    pub path: PathBuf,
    pub version: String,
    pub available: bool,
}
```

### JobSpec Enum

```rust
pub enum JobSpec {
    PdfMerge { inputs: Vec<PathBuf>, output: PathBuf, linearize: bool },
    PdfSplit { input: PathBuf, output_dir: PathBuf, pages: PageSpec },
    PdfReorder { input: PathBuf, output: PathBuf, order: Vec<PageSpec> },
    PdfCompress { input: PathBuf, output: PathBuf, level: CompressionLevel },
    PdfLinearize { input: PathBuf, output: PathBuf },
    PdfExtract { input: PathBuf, output: PathBuf, pages: PageSpec },
    PdfOcr { input: PathBuf, output: PathBuf, language: String },
    PdfMetadata { input: PathBuf, action: MetadataAction },
    ImageConvert { input: PathBuf, output: PathBuf, format: ImageFormat, quality: u8 },
    ImageResize { input: PathBuf, output: PathBuf, dimensions: Dimensions, preserve_aspect: bool },
    ImageStrip { input: PathBuf, output: PathBuf },
    MediaTranscode { input: PathBuf, output: PathBuf, codec: VideoCodec, crf: u8, scale: Option<Dimensions> },
    AudioConvert { input: PathBuf, output: PathBuf, format: AudioFormat, bitrate: u32 },
    AudioNormalize { input: PathBuf, output: PathBuf, target: LoudnessTarget },
}
```

### Progress Parsing Patterns

**qpdf**: Parse stderr for "Processing page X of Y" → `current=X, total=Y`

**ghostscript**: Parse JSON output mode if available, else stderr "page X/Y"

**ocrmypdf**: Parse progress bar: `[████████░░░░░░░░] 50%` → estimate from file size

**ffmpeg**: Parse `time=00:01:23.45` and `Duration: 00:05:00.00` → calculate percent

**libvips**: No progress (fast), emit start/complete events

### Config Resolution Order

1. `--tools.<name>=path` (highest priority - explicit override)
2. Bundled tools in `tools/` directory (relative to binary location)
3. `~/.config/forgekit/tools.toml` (user config)
4. `$XDG_CONFIG_HOME/forgekit/tools.toml` (Linux)
5. `%APPDATA%/forgekit/tools.toml` (Windows)
6. PATH probe (lowest priority - fallback)

### Error Taxonomy

```rust
pub enum ForgeKitError {
    ToolNotFound { tool: String, hint: String },
    ToolVersionMismatch { tool: String, required: String, found: String },
    InvalidInput { path: PathBuf, reason: String },
    ProcessingFailed { tool: String, stderr: String },
    PermissionDenied { path: PathBuf },
    DiskFull { path: PathBuf },
    Cancelled,
}
```

**Exit codes**: 0=success, 1=general error, 2=missing tool, 3=invalid input, 4=permission denied, 5=disk full

## 5. CLI Spec

### Global Flags

```
--json              # JSON output (NDJSON for progress)
--plan              # Show underlying commands without executing
--dry-run           # Validate inputs, show plan, don't execute
--log-level <level> # debug|info|warn|error (default: info)
--force             # Overwrite existing files
--tools.<name>=path # Override tool path (e.g., --tools.qpdf=/usr/local/bin/qpdf)
--config <path>     # Override config file location
```

### PDF Subcommands

```
forgekit pdf merge <inputs>... --output <path> [--linearize]
forgekit pdf split <input> --output-dir <path> --pages <spec>
forgekit pdf reorder <input> --output <path> --order <spec>...
forgekit pdf compress <input> --output <path> [--preset <name>|--level <1-5>]
forgekit pdf linearize <input> --output <path>
forgekit pdf extract <input> --output <path> --pages <spec>
forgekit pdf ocr <input> --output <path> [--language <lang>] [--preset <name>]
forgekit pdf metadata <input> [--get <key>|--set <key=value>...] [--output <path>]
forgekit pdf encrypt <input> --output <path> --password <pwd> [--owner-password <pwd>]
forgekit pdf decrypt <input> --output <path> --password <pwd>
```

### Image Subcommands

```
forgekit image convert <input> --output <path> --to <format> [--preset <name>|--quality <0-100>]
forgekit image resize <input> --output <path> --width <px> [--height <px>] [--preset <name>]
forgekit image strip <input> --output <path>
```

### Media Subcommands

```
forgekit media transcode <input> --output <path> [--preset <name>|--codec h264 --crf <0-51> --scale <WxH>]
forgekit audio convert <input> --output <path> --to <format> [--preset <name>|--bitrate <kbps>]
forgekit audio normalize <input> --output <path> [--preset <name>|--ebu-r128|--peak <dB>]
```

### Pages Grammar

**Grammar**: `PAGE_SPEC := RANGE | NUMBER | KEYWORD | EXCLUSION`

- `RANGE`: `NUMBER-NUMBER` (e.g., `1-5`, `10-20`)
- `NUMBER`: integer (e.g., `1`, `42`)
- `KEYWORD`: `odd`, `even`, `first`, `last`
- `EXCLUSION`: `!NUMBER` or `!RANGE` (e.g., `!2`, `!5-10`)
- Comma-separated: `1-3,5,7-` (7- means 7 to end), `-10` (means 1-10)
- Examples: `1-3,5,7-`, `-10`, `odd`, `even`, `1-10,!2,!5`

**Error messages**: "Invalid page spec 'xyz': expected number, range, or keyword (odd/even/first/last)"

### JSON Output Schemas

**Progress event**:

```json
{
  "type": "progress",
  "version": 1,
  "job_id": "abc123",
  "progress": {
    "current": 2,
    "total": 5,
    "percent": 40,
    "stage": "merging"
  },
  "message": "Processing page 2/5"
}
```

**Complete event**:

```json
{
  "type": "complete",
  "version": 1,
  "job_id": "abc123",
  "result": {
    "output": "/path/to/output.pdf",
    "size_bytes": 123456,
    "duration_ms": 1234
  }
}
```

**Error event**:

```json
{
  "type": "error",
  "version": 1,
  "job_id": "abc123",
  "error": {
    "code": "TOOL_NOT_FOUND",
    "message": "qpdf not found in PATH",
    "hint": "Bundled tool missing - reinstall ForgeKit"
  }
}
```

### Example CLI Invocations

```bash
# PDF merge with linearize
forgekit pdf merge doc1.pdf doc2.pdf --output merged.pdf --linearize

# PDF split with pages spec
forgekit pdf split book.pdf --output-dir pages/ --pages 1-5,10-20,odd

# PDF compress with preset
forgekit pdf compress large.pdf --output small.pdf --preset web

# Image convert with quality
forgekit image convert photo.jpg --output photo.webp --to webp --quality 80

# Video transcode with preset
forgekit media transcode video.mp4 --output video_h264.mp4 --preset h264-1080p

# JSON output for scripting
forgekit pdf merge *.pdf --output out.pdf --json | jq -r '.result.output'
```

## 6. CLI UX Excellence (Beta-MVP Focus)

### CLI Design Principles

**Consistency**:

- All subcommands follow `forgekit <category> <action> [args]`
- Global flags work uniformly: `--json`, `--plan`, `--dry-run`, `--log-level`, `--force`
- Error messages follow format: `Error: <what> - <why> - <how to fix>`

**Transparency**:

- `--plan` shows exact underlying commands (e.g., `qpdf --linearize input.pdf output.pdf`)
- `--dry-run` validates inputs, shows plan, exits without executing
- `--log-level debug` shows tool invocations, temp file paths, progress parsing

**Help Quality**:

- `forgekit --help`: Overview with examples
- `forgekit pdf --help`: PDF subcommands with brief descriptions
- `forgekit pdf merge --help`: Full flag documentation, examples, exit codes
- Examples in help text show real-world usage

**JSON Output Stability**:

- NDJSON format (one event per line)
- Schema versioned: `{"version": 1, "type": "...", ...}`
- Progress events include `job_id`, `progress`, `message`
- Complete events include `result` with `output`, `size_bytes`, `duration_ms`
- Error events include `error` with `code`, `message`, `hint`

**Exit Codes**:

- `0`: Success
- `1`: General error (processing failed)
- `2`: Missing tool (dependency not installed, run `forgekit check-deps` for install instructions)
- `3`: Invalid input (file not found, invalid pages spec, etc.)
- `4`: Permission denied
- `5`: Disk full
- `130`: Cancelled (SIGINT)

### Example Help Output

```
$ forgekit pdf merge --help
Merge multiple PDFs into a single file.

USAGE:
    forgekit pdf merge <INPUTS>... --output <OUTPUT>

ARGS:
    <INPUTS>...    Input PDF files (at least 2 required)

OPTIONS:
    -o, --output <OUTPUT>    Output PDF file path
        --linearize          Optimize for fast web view
    -f, --force              Overwrite existing output file
        --json               Output progress as NDJSON
        --plan               Show underlying commands without executing
        --dry-run            Validate inputs and show plan, don't execute

EXAMPLES:
    # Merge two PDFs
    forgekit pdf merge doc1.pdf doc2.pdf --output merged.pdf

    # Merge with linearization and JSON progress
    forgekit pdf merge *.pdf --output all.pdf --linearize --json

    # See what commands would run
    forgekit pdf merge a.pdf b.pdf --output c.pdf --plan

EXIT CODES:
    0  Success
    1  Processing failed (check logs)
    2  Tool not found (dependency missing, install via package manager)
    3  Invalid input (file not found or invalid format)
    4  Permission denied
    5  Disk full

SEE ALSO:
    forgekit pdf split --help
    forgekit pdf compress --help
```

### GUI (Post-Beta-MVP)

**Deferred to post-beta-MVP**:

- Tauri GUI will call same `crates/core` functions
- GUI will consume JSON progress events
- GUI will show CLI command equivalent for reproducibility
- No GUI work in beta-MVP milestones

## 7. Dependency and Packaging Plan

### Required Dependencies and Versions

**Dependency management**: External tools are declared as dependencies in ForgeKit's package definitions. Installation scripts/packages automatically install these dependencies using system package managers.

**Required dependencies**:

- **qpdf** 11.x+ (PDF manipulation)
- **ghostscript** 0.4+ (PDF compression/optimization)
- **tesseract** 5.0+ (OCR engine)
- **ocrmypdf** 14.0+ (Python wrapper for OCR, installed via pip)
- **ffmpeg** 5.0+ (audio/video processing)
- **libvips** 8.12+ (image processing)
- **exiftool** 12.0+ (metadata handling)

**Tool detection order**:

1. System PATH (where package manager installs them)
2. Config file overrides (`--tools.<name>=path`)
3. Explicit path overrides (`--tools.<name>=path` CLI flag)

**Package dependency declarations**:

**Debian/Ubuntu** (`.deb` package):

```deb
Depends: qpdf (>= 11.0), ghostscript, tesseract-ocr (>= 5.0), ffmpeg (>= 5.0), libvips-tools (>= 8.12), libimage-exiftool-perl (>= 12.0), python3, python3-pip
```

**macOS** (Homebrew formula):

```ruby
depends_on "qpdf" => ">= 11.0"
depends_on "ghostscript"
depends_on "tesseract" => ">= 5.0"
depends_on "ffmpeg" => ">= 5.0"
depends_on "libvips" => ">= 8.12"
depends_on "exiftool" => ">= 12.0"
depends_on "python@3"
```

**Windows** (winget manifest):

```yaml
Dependencies:
  - PackageIdentifier: qpdf.qpdf
    MinimumVersion: 11.0.0
  - PackageIdentifier: ghostscript.ghostscript
  - PackageIdentifier: tesseract-ocr
  - PackageIdentifier: ffmpeg
  - PackageIdentifier: Python.Python.3
```

**Installation**: Install ForgeKit via package manager → dependencies are automatically installed.

### Binary Size Targets

- **ForgeKit binary**: <15 MB stripped (best: 8 MB, likely: 12 MB, worst: 18 MB)
- **Package size**: Binary only (~10-15 MB) + package metadata
- **Total installation size**: ~50-100 MB (including dependencies installed by package manager)
- **Tauri app**: Deferred to post-beta-MVP

**Note**: Dependencies are installed separately by package managers, so ForgeKit download stays small.

### Packaging Formats (Beta-MVP)

**macOS**:

- **Homebrew formula** (primary): `brew install forgekit`
  - Automatically installs dependencies (qpdf, ghostscript, tesseract, etc.)
  - Binary installed to `/opt/homebrew/bin/forgekit` or `/usr/local/bin/forgekit`
- **Binary release** (fallback): `.tar.gz` with just the binary
  - Users manually install dependencies via `brew install qpdf ghostscript ...`
- Optional: `.pkg` installer (post-beta-MVP)

**Windows**:

- **winget package** (primary): `winget install forgekit`
  - Automatically installs dependencies via winget
  - Binary installed to `%LOCALAPPDATA%\Microsoft\WinGet\Packages\`
- **Scoop manifest** (alternative): `scoop install forgekit`
  - Automatically installs dependencies via scoop
- **Binary release** (fallback): `.zip` with just the binary
  - Users manually install dependencies via winget/scoop

**Linux**:

- **Debian/Ubuntu**: `.deb` package
  - `sudo dpkg -i forgekit.deb` or `sudo apt install ./forgekit.deb`
  - Automatically installs dependencies via apt
- **Fedora/RHEL**: `.rpm` package
  - `sudo rpm -i forgekit.rpm` or `sudo dnf install forgekit.rpm`
  - Automatically installs dependencies via dnf/yum
- **Arch Linux**: `.pkg.tar.zst` package
  - `sudo pacman -U forgekit.pkg.tar.zst`
  - Automatically installs dependencies via pacman
- **Binary release** (fallback): `.tar.gz` with just the binary
  - Users manually install dependencies via their package manager

**Distribution**:

- GitHub Releases with platform-specific packages
- Package repositories (Homebrew tap, winget source, apt repo, etc.)
- Checksums (SHA256) for verification
- Installation automatically handles dependencies

## 8. Security and Privacy

### Network Access

- No network access in CLI (strictly enforced)
- No telemetry, no analytics, no crash reporting in MVP

### Sandboxed Permissions

- macOS: No special permissions needed for CLI
- Windows: No special permissions needed
- Linux: No special permissions needed

### Temp File Hygiene

- Use `std::env::temp_dir()` with unique filenames: `forgekit-{uuid}.tmp`
- Atomic writes: write to temp, then `rename()` to final location
- Cleanup on cancellation or error
- No overwrite unless `--force` flag

### Password Handling

- Never log passwords (redact in logs: `password=***`)
- CLI: `--password` flag (warn about shell history) or `--password-file` (read from file)
- Environment variable: `FORGEKIT_PDF_PASSWORD` (for scripting)

## 9. Testing/CI

### Fixture Set

**PDFs**:

- `fixtures/pdf/simple.pdf` (5 pages, 100 KB)
- `fixtures/pdf/large.pdf` (100 pages, 10 MB)
- `fixtures/pdf/encrypted.pdf` (password: "test")

**Images**:

- `fixtures/image/small.jpg` (100x100, 50 KB)
- `fixtures/image/large.png` (4000x3000, 5 MB)
- `fixtures/image/with_exif.jpg` (contains EXIF data)

**Media**:

- `fixtures/video/short.mp4` (10 seconds, H.264, 5 MB)
- `fixtures/audio/sample.mp3` (30 seconds, 128 kbps, 500 KB)

### Test Types

**Golden tests**:

- PDF merge: output page count = sum of input pages
- PDF split: extracted pages match source pages
- OCR: extracted text contains expected keywords
- Image resize: output dimensions match target
- Video transcode: output duration matches input (±1 second)

**Progress snapshot tests**:

- Parse qpdf stderr → progress events
- Parse ffmpeg stdout → progress events

**Exit code tests**:

- Missing tool → exit code 2
- Invalid input → exit code 3
- Permission denied → exit code 4

### CI Setup (GitHub Actions)

**Matrix**: macOS-12, windows-2022, ubuntu-22.04

**Steps**:

1. Install Rust toolchain
2. Cache dependencies (`target/`)
3. Install test tools (qpdf, ffmpeg, etc.) via package managers
4. Run unit tests
5. Run integration tests (with fixtures)
6. Build release binaries
7. Upload artifacts

**Target**: <15 minutes per OS

## 10. Preset Definitions

### PDF Compress Presets

```yaml
version: 1
presets:
  pdf-compress-web:
    tool: ghostscript
    description: "Web-optimized (smallest size, lower quality)"
    args: ["optimize", "-level=2", "-compression=compress"]

  pdf-compress-screen:
    tool: ghostscript
    description: "Screen viewing (balanced)"
    args: ["optimize", "-level=3", "-compression=compress"]

  pdf-compress-printer:
    tool: ghostscript
    description: "Print quality (higher quality)"
    args: ["optimize", "-level=4", "-compression=compress"]

  pdf-compress-hq:
    tool: ghostscript
    description: "High quality (minimal compression)"
    args: ["optimize", "-level=5", "-compression=compress"]
```

### Video Presets (Beta-MVP: H.264 Only)

```yaml
version: 1
presets:
  video-h264-1080p:
    tool: ffmpeg
    description: "H.264, CRF 23, 1080p"
    args:
      [
        "-c:v",
        "libx264",
        "-crf",
        "23",
        "-preset",
        "medium",
        "-vf",
        "scale=1920:1080:force_original_aspect_ratio=decrease",
      ]
```

### Audio Presets

```yaml
version: 1
presets:
  audio-opus-128k:
    tool: ffmpeg
    description: "Opus 128 kbps"
    args: ["-c:a", "libopus", "-b:a", "128k"]

  audio-aac-192k:
    tool: ffmpeg
    description: "AAC 192 kbps"
    args: ["-c:a", "aac", "-b:a", "192k"]

  audio-normalize-ebu:
    tool: ffmpeg
    description: "EBU R128 loudness normalization"
    args: ["-af", "loudnorm=I=-16:TP=-1.5:LRA=11"]
```

### Image Presets

```yaml
version: 1
presets:
  image-webp-web:
    tool: libvips
    description: "WebP quality 70, strip EXIF"
    args: ["--quality=70", "--strip"]

  image-avif-web:
    tool: libvips
    description: "AVIF quality 45, strip EXIF"
    args: ["--quality=45", "--strip"]

  image-jpeg-web:
    tool: libvips
    description: "JPEG quality 82, strip EXIF"
    args: ["--quality=82", "--strip"]
```

**Preset storage**: `presets/presets.yaml` (included in package) + `~/.config/forgekit/presets.yaml` (user overrides)

## 11. Milestones and Estimates

### Milestone 1: Core Foundation (Week 1-2) ✅ COMPLETED

**Acceptance criteria**:

- ✅ `crates/core` structure with Tool trait
- ✅ Tool probe/version detection for qpdf
- ✅ Basic JobSpec enum (PdfMerge, PdfSplit)
- ✅ Temp file utilities with atomic writes
- ✅ Unit tests for tool detection
- ✅ Error taxonomy with exit codes
- ✅ Comprehensive documentation

**Risks**: Tool detection may fail on Windows PATH

**Mitigation**: Test on Windows early, document PATH issues

**Estimate**: Best 1 week, likely 1.5 weeks, worst 2 weeks

**Status**: ✅ Completed

### Milestone 2: PDF Basic Operations (Week 2-3) ✅ COMPLETED

**Acceptance criteria**:

- ✅ qpdf merge/split implementations
- ✅ Pages grammar parser with comprehensive tests (11 tests)
- ✅ CLI `pdf merge` and `pdf split` subcommands
- ✅ JSON progress output (NDJSON format with version field)
- ✅ `--plan` and `--dry-run` flags working
- ✅ Comprehensive help text with examples
- ✅ Integration tests for JSON format

**Risks**: Pages grammar edge cases

**Mitigation**: Comprehensive test cases, clear error messages

**Estimate**: Best 1 week, likely 1.5 weeks, worst 2 weeks

**Status**: ✅ Completed

**Note**: Linearize is supported as a flag on merge, but not as a separate subcommand yet.

### Milestone 3: PDF Advanced Operations (Week 3-4)

**Acceptance criteria**:

- ghostscript compress/optimize
- OCR with ocrmypdf (basic)
- Metadata read/write with exiftool
- CLI `pdf compress/ocr/metadata` subcommands
- Preset system (YAML loader)

**Risks**: OCR performance slow on large PDFs

**Mitigation**: Show progress, allow cancellation

**Estimate**: Best 1 week, likely 1.5 weeks, worst 2 weeks

**Status**: 🔄 Pending

### Milestone 4: Image Operations (Week 4-5) ✅ COMPLETED

**Acceptance criteria**:

- ✅ libvips convert/resize/strip/compress
- ✅ CLI `image convert/resize/strip/compress/info` subcommands
- ✅ Optional output with smart auto-naming
- ✅ PNG compression control (1-9)
- ✅ RAW format input support

**Risks**: libvips not available on Windows

**Mitigation**: libvips available via scoop, clear install hints

**Estimate**: Best 1 week, likely 1 week, worst 1.5 weeks

**Status**: ✅ Completed

### Milestone 5: Audio Operations (Week 5-6)

**Acceptance criteria**:

- ffmpeg audio convert/normalize
- CLI `audio convert/normalize` subcommands
- Audio presets (Opus, AAC, EBU R128)
- Golden tests (duration, bitrate)

**Risks**: None significant

**Estimate**: Best 0.5 weeks, likely 1 week, worst 1 week

**Status**: 🔄 Pending

### Milestone 6: Video Operations (Week 6-7)

**Acceptance criteria**:

- ffmpeg H.264 transcode with CRF (software x264 only)
- CLI `media transcode` subcommand
- Video preset (H.264 1080p)
- Progress parsing from ffmpeg stderr
- Golden tests (duration, codec)

**Risks**: Transcoding slow on large files

**Mitigation**: Show progress, allow cancellation

**Estimate**: Best 1 week, likely 1 week, worst 1.5 weeks

**Status**: 🔄 Pending

### Milestone 7: CLI Polish and Documentation (Week 7-8) ✅ COMPLETED

**Acceptance criteria**:

- ✅ Comprehensive --help for PDF subcommands (merge, split)
- ✅ Examples in help text and README
- ✅ --plan and --dry-run work for PDF commands
- ✅ JSON progress events stable and documented (version 1 schema)
- ✅ Exit codes documented and tested
- ✅ CONTRIBUTING.md with codebase overview
- ✅ Comprehensive inline documentation throughout codebase
- ✅ All doc tests passing (5 tests)

**Risks**: None significant

**Estimate**: Best 1 week, likely 1 week, worst 1.5 weeks

**Status**: ✅ Completed

**Note**: Help and documentation are complete for implemented features. Will expand as more commands are added.

### Milestone 8: Package Creation and Dependency Management (Week 8-9)

**Acceptance criteria**:

- Create platform-specific packages (deb, rpm, Homebrew formula, winget manifest)
- Packages declare dependencies that are automatically installed
- `forgekit check-deps` verifies all dependencies are installed
- Installation scripts handle dependency installation
- README includes platform-specific installation instructions
- Package repositories set up (Homebrew tap, winget source, apt repo)

**Risks**:

- Package manager complexity across platforms
- Dependency version conflicts
- Package repository setup and maintenance
- Users without package managers (fallback needed)

**Mitigation**:

- Provide binary-only releases as fallback
- Clear dependency documentation
- Test installation on fresh VMs per OS
- Use standard package formats (deb, rpm, Homebrew, winget)

**Estimate**: Best 1.5 weeks, likely 2 weeks, worst 3 weeks

**Status**: 🔄 Pending

### Milestone 9: Error Handling and Hints (Week 9-10) ✅ COMPLETED

**Acceptance criteria**:

- ✅ Comprehensive error taxonomy (`ForgeKitError` enum)
- ✅ Install hints per OS in error messages
- ✅ Exit codes standardized and mapped from errors
- ✅ Error messages actionable with hints
- ✅ JSON error events with code, message, and hint

**Risks**: Install hints may be incomplete

**Mitigation**: Test on fresh VMs per OS

**Estimate**: Best 0.5 weeks, likely 1 week, worst 1 week

**Status**: ✅ Completed

**Note**: Config file support for tool overrides is deferred to when more tools are added.

### Milestone 10: Beta-MVP Release Preparation (Week 10-11)

**Acceptance criteria**:

- All CLI commands working with examples
- Golden tests passing in CI (macOS, Windows, Linux)
- README complete with quick start and examples
- Preset reference documented
- Platform-specific packages built (deb, rpm, Homebrew formula, winget manifest)
- Binary-only releases as fallback option
- Release workflow (GitHub Actions) that:
  - Builds ForgeKit binary per platform
  - Creates platform-specific packages
  - Generates checksums (SHA256)
  - Uploads to GitHub Releases
  - Publishes to package repositories (Homebrew tap, winget source, etc.)

**Risks**:

- CI failures on Windows/Linux
- Package creation complexity across platforms
- Package repository setup and maintenance

**Mitigation**:

- Test early, fix platform-specific issues
- Start with binary-only releases, add packages incrementally
- Use established package formats and tools

**Estimate**: Best 2 weeks, likely 2.5 weeks, worst 3.5 weeks

**Status**: 🔄 Pending

**Total estimate (Beta-MVP)**: Best 6 weeks, likely 9 weeks, worst 12 weeks

**Post-Beta-MVP**:

- Tauri GUI (calls same core crate)
- Hardware-accelerated H.264
- H.265/AV1 codecs
- Advanced PDF features (signing, etc.)

## 12. Risks and Mitigations

### Risk 1: External Tool Availability on Windows

**Impact**: Medium (dependencies must be available via package managers)

**Probability**: Medium

**Mitigation**:

- Dependencies automatically installed via winget/scoop
- Binary-only release as fallback for users without package managers
- Clear documentation for manual dependency installation
- Support portable tool paths via `--tools.<name>=path` for advanced users

### Risk 2: OCR Performance on Large PDFs

**Impact**: Medium (slow UX)

**Probability**: High

**Mitigation**:

- Show progress bars
- Allow cancellation
- Consider page-by-page processing (future)

### Risk 3: libvips Not Available on Windows

**Impact**: Low (scoop has libvips)

**Probability**: Low

**Mitigation**:

- libvips available via scoop on Windows
- Clear install hints in error messages

### Risk 4: Licensing Caveats (GPL tools)

**Impact**: Low (tools are external dependencies, installed via package managers)

**Probability**: Low

**Mitigation**:

- No bundling in MVP (users install tools)
- Document license compatibility in README

## 13. Documentation Scaffold

### README.md Structure

```markdown
# ForgeKit

Local-first media and PDF toolkit.

## Quick Start

[Install instructions per OS]

## Examples

[CLI examples for each subcommand]

## Installation

### macOS

[Homebrew commands]

### Windows

[winget/scoop commands]

### Linux

[apt/pacman commands]

## Troubleshooting

- Tool not found: [PATH issues, config file]
- Permission denied: [file access]
- Large file processing: [memory/disk]

## Contributing

[Simple guidelines]
```

### docs/adr/0001-rust-core-cli-tauri.md

[Brief summary of ADR-0001]

### docs/adr/0002-external-tools-strategy.md

[Brief summary of ADR-0002]

### docs/adr/0003-progress-json-format.md

[Brief summary of ADR-0003]

### docs/adr/0004-preset-system.md

[Brief summary of ADR-0004]

### docs/adr/0005-build-package-policy.md

[Brief summary of ADR-0005]

### docs/presets.md

[Reference for all presets with examples]

### docs/troubleshooting.md

[Common issues: missing deps, permissions, PATH, performance]

## 14. Next 5 Work Sessions (1-2 Hours Each)

### Session 1: Project Scaffold and Tool Detection ✅ COMPLETED

**Task**: Create Rust workspace with `crates/core`, `crates/cli`. Implement Tool trait and qpdf probe/version detection. Add basic error types and exit codes.

**Done definition**:

- ✅ `cargo new --workspace` structure created
- ✅ Tool trait defined in `crates/core/src/tools/trait_def.rs`
- ✅ qpdf adapter with `probe()` and `version()` methods
- ✅ Error taxonomy with exit codes (ExitCode enum)
- ✅ Unit test: probe finds qpdf in PATH
- ✅ `cargo test` passes
- ✅ Comprehensive module documentation added

**Files created**:

- ✅ `Cargo.toml` (workspace)
- ✅ `crates/core/Cargo.toml`, `crates/core/src/lib.rs`, `crates/core/src/tools/`
- ✅ `crates/core/src/utils/error.rs` (ExitCode enum)
- ✅ `crates/cli/Cargo.toml`, `crates/cli/src/main.rs`
- ✅ `.gitignore` for Rust projects

### Session 2: PDF Merge Implementation with --plan ✅ COMPLETED

**Task**: Implement qpdf merge in `crates/core`. Add JobSpec::PdfMerge. Create temp file utils. Wire CLI `pdf merge` subcommand with `--plan` flag showing underlying commands.

**Done definition**:

- ✅ `JobSpec::PdfMerge` enum variant
- ✅ qpdf merge execution in `crates/core/src/job/executor.rs`
- ✅ Temp file creation with unique names (UUID-based)
- ✅ CLI `pdf merge` subcommand with `--plan` flag
- ✅ `--plan` shows underlying qpdf commands
- ✅ Linearize flag support
- ✅ Progress reporting integration

**Files created**:

- ✅ `crates/core/src/job/spec.rs`
- ✅ `crates/core/src/job/executor.rs`
- ✅ `crates/core/src/utils/temp.rs`
- ✅ `crates/cli/src/commands/pdf.rs`
- ✅ `fixtures/pdf/` directory created

### Session 3: Pages Grammar Parser and PDF Split ✅ COMPLETED

**Task**: Implement pages grammar parser supporting `1-3,5,7-`, `odd`, `even`, `!2`. Add comprehensive tests. Integrate with `pdf split` subcommand. Add `--plan` support.

**Done definition**:

- ✅ `PageSpec` enum and parser in `crates/core/src/utils/pages.rs`
- ✅ Tests: `1-3,5`, `odd`, `even`, `!2`, `7-`, `-10` (11 comprehensive tests)
- ✅ CLI `pdf split --pages 1-3` subcommand implemented
- ✅ `--plan` shows underlying qpdf commands
- ✅ Error messages clear for invalid specs
- ✅ Conversion to qpdf page format

**Files created**:

- ✅ `crates/core/src/utils/pages.rs` with comprehensive parser
- ✅ `crates/core/src/utils/pages.rs` tests (11 tests, all passing)
- ✅ Extended `crates/cli/src/commands/pdf.rs` with split handler

### Session 4: JSON Progress Output and --dry-run ✅ COMPLETED

**Task**: Implement ProgressEvent struct and NDJSON output. Wire progress reporting for qpdf merge. Add `--json` and `--dry-run` flags to CLI. Test JSON parsing.

**Done definition**:

- ✅ `ProgressEvent` struct in `crates/core/src/job/progress.rs` (versioned schema)
- ✅ NDJSON output when `--json` flag used
- ✅ Progress events emitted during merge (start, progress, complete)
- ✅ `--dry-run` validates inputs, shows plan, exits without executing
- ✅ `forgekit pdf merge --json` outputs valid NDJSON
- ✅ Golden test: JSON event stream matches expected format
- ✅ Integration tests for JSON format (2 tests)

**Files created**:

- ✅ `crates/core/src/job/progress.rs` with versioned events
- ✅ Extended `crates/cli/src/main.rs` with `--json` and `--dry-run` flags
- ✅ Progress reporter trait implementation (JsonProgressReporter, NoOpProgressReporter)
- ✅ `tests/golden/progress_merge.json` (expected JSON output)
- ✅ `crates/core/tests/integration_test.rs` (moved from workspace root)

### Session 5: Comprehensive --help and Examples ✅ COMPLETED

**Task**: Add comprehensive --help for all PDF subcommands with examples. Create README with quick start. Add `check-deps` command for dependency probing.

**Done definition**:

- ✅ `forgekit pdf --help` shows all subcommands with brief descriptions
- ✅ `forgekit pdf merge --help` shows full flags, examples, exit codes
- ✅ `forgekit pdf split --help` shows page spec documentation and examples
- ✅ `forgekit check-deps` lists tools, shows status (found/missing), install hints
- ✅ README.md includes quick start, install instructions, examples
- ✅ At least one working example per subcommand documented
- ✅ CONTRIBUTING.md with codebase overview and contribution guide
- ✅ Comprehensive inline documentation throughout codebase

**Files created**:

- ✅ Extended `crates/cli/src/commands/pdf.rs` with comprehensive help text
- ✅ `crates/cli/src/commands/check.rs` (check-deps command)
- ✅ `README.md` with quick start and examples
- ✅ `CONTRIBUTING.md` with architecture overview
- ✅ `ROADMAP.md` (this file)

---

**End of Roadmap**
