# Contributing to ForgeKit

Thanks for your interest in contributing! This guide will help you understand the codebase and get started.

## Getting Started

1. **Fork and clone** the repository
2. **Install dependencies**:

   ```bash
   # Install Rust (if you haven't already)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Install external tools (see README.md for details)
   brew install qpdf  # macOS
   # or apt install qpdf  # Linux
   ```

3. **Build the project**:
   ```bash
   cargo build
   ```
4. **Run tests**:
   ```bash
   cargo test
   ```

## Codebase Overview

ForgeKit is organized as a Rust workspace with two main crates:

- **`crates/core`** - The core library (pure Rust logic, no CLI dependencies)
- **`crates/cli`** - The command-line interface (thin wrapper around core)

### Architecture

```
┌─────────────┐
│   CLI       │  ← Argument parsing, output formatting
│  (clap)     │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    Core     │  ← Job specs, tool adapters, execution
│  Library    │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  External   │  ← qpdf, ffmpeg, libvips, etc.
│   Tools     │
└─────────────┘
```

### Key Concepts

**Jobs** (`crates/core/src/job/spec.rs`)

- A `JobSpec` describes what you want to do (merge PDFs, resize images, etc.)
- It's pure data - no execution logic
- Examples: `PdfMerge`, `PdfSplit`, `PdfCompress`

**Tools** (`crates/core/src/tools/`)

- Adapters for external command-line tools
- Each tool implements the `Tool` trait
- Handles finding the tool, checking version, and running it
- See `qpdf.rs` for a complete example

**Executors** (`crates/core/src/job/executor.rs`)

- Takes a `JobSpec` and actually runs it
- Handles tool detection, command construction, progress reporting
- Returns success message or detailed error

**Progress** (`crates/core/src/job/progress.rs`)

- Events emitted during job execution
- NDJSON format (one event per line)
- Used for `--json` flag and future GUI

## Adding a New Feature

### Example: Adding PDF Compression

1. **Add a `JobSpec` variant** (`crates/core/src/job/spec.rs`):

   ```rust
   PdfCompress {
       input: PathBuf,
       output: PathBuf,
       level: CompressionLevel,
   }
   ```

2. **Create a tool adapter** (`crates/core/src/tools/gs.rs`):

   ```rust
   pub struct GsTool;

   impl Tool for GsTool {
       fn name(&self) -> &'static str { "gs" }
       fn probe(&self, config: &ToolConfig) -> Result<ToolInfo> { /* ... */ }
       fn version(&self, path: &PathBuf) -> Result<String> { /* ... */ }
   }
   ```

3. **Add execution logic** (`crates/core/src/job/executor.rs`):

   ```rust
   JobSpec::PdfCompress { input, output, level } => {
       execute_pdf_compress(input, output, level, plan_only, reporter)
   }
   ```

4. **Add CLI subcommand** (`crates/cli/src/commands/pdf.rs`):

   ```rust
   Compress(CompressArgs),
   ```

5. **Wire it up** (`crates/cli/src/main.rs`):

   ```rust
   Commands::Pdf(PdfCommand::Compress(args)) => {
       handle_pdf_compress(args, plan_only, json_output)
   }
   ```

6. **Add tests** - See `tests/` directory for examples

## Code Style

- **Use `cargo fmt`** to format code
- **Use `cargo clippy`** to catch common issues
- **Follow Rust naming conventions** (snake_case for functions, PascalCase for types)
- **Add doc comments** for public APIs (use `///` for items, `//!` for modules)
- **Keep functions focused** - if a function does too much, split it

## Testing

We use several types of tests:

- **Unit tests** - Test individual functions/modules
- **Integration tests** - Test full workflows with real files
- **Golden tests** - Compare output against expected results

Run all tests:

```bash
cargo test
```

Run with output:

```bash
cargo test -- --nocapture
```

## Documentation

- **Public APIs** should have doc comments explaining what they do and why
- **Complex logic** should have inline comments explaining the "why"
- **Examples** in doc comments help users understand usage

Generate docs:

```bash
cargo doc --open
```

## Commit Messages

Keep commits focused and descriptive:

```
Add PDF compression support

- Add PdfCompress JobSpec variant
- Implement Ghostscript tool adapter
- Add pdf compress CLI subcommand
- Add tests for compression levels
```

## Questions?

- Check the [README.md](README.md) for usage examples
- Check the [ROADMAP.md](ROADMAP.md) for planned features
- Open an issue if you're stuck or have questions

Thanks for contributing! 🎉
