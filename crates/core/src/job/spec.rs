//! # Job Specifications
//!
//! A `JobSpec` describes what you want to do - it's pure data, no execution logic.
//! Think of it as a recipe: "merge these PDFs", "extract these pages", etc.
//!
//! ## Why Separate Specs from Execution?
//!
//! Separating specs from execution gives us:
//! - **Testability**: Easy to test parsing/validation without running tools
//! - **Transparency**: `--plan` flag can show what would happen without doing it
//! - **Serialization**: Could save jobs to disk, queue them, etc. (future)
//! - **Clarity**: The executor code is cleaner when it just focuses on "how", not "what"

use crate::utils::audio::{AudioFormat, LoudnessTarget};
use crate::utils::image::ImageFormat;
use crate::utils::pages::PageSpec;
use std::path::PathBuf;

/// Action to perform on PDF metadata
#[derive(Debug, Clone)]
pub enum MetadataAction {
    /// Read all metadata as JSON
    GetAll,
    /// Read a specific field
    Get(String),
    /// Set one or more fields (field name, value)
    Set(Vec<(String, String)>),
}

/// A job specification describing what operation to perform.
///
/// This is pure data - it doesn't execute anything. The executor (`job::executor`)
/// takes a `JobSpec` and actually runs it by calling the appropriate external tools.
///
/// ## Adding a New Job Type
///
/// To add a new operation (e.g., `PdfCompress`):
///
/// 1. Add a variant to this enum with the necessary fields
/// 2. Add a match arm in `executor.rs` to handle it
/// 3. Add a CLI subcommand in `cli/src/commands/`
/// 4. Update `description()` to include it
#[derive(Debug, Clone)]
pub enum JobSpec {
    /// Merge multiple PDFs into a single file.
    ///
    /// `linearize` optimizes the PDF for fast web viewing by reorganizing
    /// the internal structure (useful for large PDFs viewed in browsers).
    PdfMerge {
        /// Input PDF files to merge (at least 2 required).
        inputs: Vec<PathBuf>,
        /// Output PDF file path.
        output: PathBuf,
        /// Whether to linearize the output PDF.
        linearize: bool,
    },
    /// Split a PDF into separate files by page ranges.
    ///
    /// Uses `PageSpec` to define which pages to extract. Can extract multiple
    /// ranges (e.g., pages 1-5 and 10-20) into separate files.
    PdfSplit {
        /// Input PDF file to split.
        input: PathBuf,
        /// Directory where split files will be saved.
        output_dir: PathBuf,
        /// Page specifications defining which pages to extract.
        pages: Vec<PageSpec>,
    },
    /// Compress a PDF using Ghostscript with compression level control.
    ///
    /// Uses Ghostscript with optimized QFactor settings for fast compression.
    PdfCompress {
        /// Input PDF file to compress.
        input: PathBuf,
        /// Output PDF file path.
        output: PathBuf,
        /// Compression level (light, standard, high).
        /// Default is "standard" (balanced compression).
        level: String,
    },
    /// Linearize a PDF for fast web viewing.
    ///
    /// Optimizes the PDF's internal structure for fast web viewing by reorganizing
    /// the internal structure. This is a standalone operation separate from merge.
    PdfLinearize {
        /// Input PDF file to linearize.
        input: PathBuf,
        /// Output PDF file path.
        output: PathBuf,
    },
    /// Reorder pages in a PDF.
    ///
    /// Reorders pages according to the specified order (1-indexed page numbers).
    PdfReorder {
        /// Input PDF file to reorder.
        input: PathBuf,
        /// Output PDF file path.
        output: PathBuf,
        /// Page order (1-indexed). Example: [3, 1, 2] means page 3, then 1, then 2.
        page_order: Vec<u32>,
    },
    /// Extract specific pages from a PDF.
    ///
    /// Extracts pages matching the page specification. Can output to a single PDF
    /// file or separate image files per page.
    PdfExtract {
        /// Input PDF file to extract from.
        input: PathBuf,
        /// Output PDF file path (when format is "pdf").
        output: Option<PathBuf>,
        /// Output directory path (when format is "images").
        output_dir: Option<PathBuf>,
        /// Page specifications defining which pages to extract.
        pages: Vec<PageSpec>,
        /// Output format: "pdf" or "images".
        format: String,
    },
    /// Add OCR text layer to a scanned PDF.
    ///
    /// Uses ocrmypdf to add a searchable text layer to scanned PDFs.
    /// The original image quality is preserved.
    PdfOcr {
        /// Input PDF file to OCR.
        input: PathBuf,
        /// Output PDF file path.
        output: PathBuf,
        /// OCR language (e.g., "eng", "deu", "fra"). Default is "eng".
        language: String,
        /// Skip pages that already have text (faster for mixed documents).
        skip_text: bool,
        /// Deskew pages before OCR (corrects tilted scans).
        deskew: bool,
        /// Force OCR even if text already exists (redo OCR).
        force_ocr: bool,
    },
    /// Read or write PDF metadata.
    ///
    /// Uses exiftool to get/set PDF metadata fields like title, author, subject, etc.
    PdfMetadata {
        /// Input PDF file.
        input: PathBuf,
        /// Output PDF file path (only needed for set operations, optional for get).
        output: Option<PathBuf>,
        /// The metadata action to perform.
        action: MetadataAction,
    },

    // ========== Image Operations ==========
    /// Convert image to a different format.
    ///
    /// Uses libvips for conversion.
    /// Supports JPEG, PNG, WebP, AVIF, TIFF, and GIF formats.
    ImageConvert {
        /// Input image file.
        input: PathBuf,
        /// Output image file.
        output: PathBuf,
        /// Target format (auto-detected from output extension if not specified).
        format: ImageFormat,
        /// Quality (0-100). Format-dependent: JPEG/WebP/AVIF use this.
        quality: Option<u8>,
        /// Compression level (0-9). For PNG: 0=fastest, 9=smallest.
        compression: Option<u8>,
        /// Strip metadata during conversion.
        strip_metadata: bool,
    },

    /// Resize image with aspect ratio preservation.
    ///
    /// Resizes to fit within the specified dimensions while preserving
    /// the original aspect ratio. Specify width, height, or both.
    ImageResize {
        /// Input image file.
        input: PathBuf,
        /// Output image file.
        output: PathBuf,
        /// Target width (if only width specified, height calculated to preserve ratio).
        width: Option<u32>,
        /// Target height (if only height specified, width calculated to preserve ratio).
        height: Option<u32>,
    },

    /// Strip EXIF and other metadata from image.
    ///
    /// Removes EXIF, XMP, IPTC, ICC profiles, and other metadata.
    /// Useful for privacy before sharing images.
    ImageStrip {
        /// Input image file.
        input: PathBuf,
        /// Output image file.
        output: PathBuf,
    },

    // ========== Audio Operations ==========
    /// Convert audio to a different format.
    ///
    /// Uses ffmpeg for conversion. Supports MP3, AAC, Opus, FLAC, WAV, OGG, M4A.
    AudioConvert {
        /// Input audio file.
        input: PathBuf,
        /// Output audio file.
        output: PathBuf,
        /// Target format.
        format: AudioFormat,
        /// Bitrate in kbps (e.g., 128, 192, 320). Not all formats support bitrate.
        bitrate: Option<u32>,
    },

    /// Normalize audio loudness.
    ///
    /// Uses ffmpeg loudnorm filter for EBU R128 compliant loudness normalization.
    AudioNormalize {
        /// Input audio file.
        input: PathBuf,
        /// Output audio file.
        output: PathBuf,
        /// Loudness target (EBU R128, Streaming, or custom LUFS).
        target: LoudnessTarget,
    },

    /// Extract audio from video file.
    ///
    /// Uses ffmpeg to extract the audio stream from a video file.
    AudioExtract {
        /// Input video file.
        input: PathBuf,
        /// Output audio file.
        output: PathBuf,
        /// Target audio format.
        format: AudioFormat,
        /// Bitrate in kbps (e.g., 128, 192, 320). Optional.
        bitrate: Option<u32>,
    },

    /// Trim audio to a specific time range.
    ///
    /// Uses ffmpeg to extract a segment from start to end time.
    AudioTrim {
        /// Input audio file.
        input: PathBuf,
        /// Output audio file.
        output: PathBuf,
        /// Start time in seconds (e.g., 30.0 for 0:30).
        start: Option<f64>,
        /// End time in seconds (e.g., 120.0 for 2:00).
        end: Option<f64>,
    },

    /// Join multiple audio files into one.
    ///
    /// Uses ffmpeg concat demuxer to concatenate audio files.
    AudioJoin {
        /// Input audio files to join (at least 2 required).
        inputs: Vec<PathBuf>,
        /// Output audio file.
        output: PathBuf,
    },

    /// Adjust audio volume/gain.
    ///
    /// Uses ffmpeg volume filter to adjust gain in decibels.
    AudioVolume {
        /// Input audio file.
        input: PathBuf,
        /// Output audio file.
        output: PathBuf,
        /// Gain adjustment in decibels (e.g., 6.0 for +6dB, -3.0 for -3dB).
        gain_db: f64,
    },

    /// Convert stereo audio to mono.
    ///
    /// Uses ffmpeg to downmix stereo to mono.
    AudioMono {
        /// Input audio file.
        input: PathBuf,
        /// Output audio file.
        output: PathBuf,
    },
}

impl JobSpec {
    /// Get a human-readable description of what this job does.
    ///
    /// Useful for logging, progress messages, and `--plan` output.
    pub fn description(&self) -> String {
        match self {
            JobSpec::PdfMerge {
                inputs, linearize, ..
            } => {
                let linearize_str = if *linearize { " (linearized)" } else { "" };
                format!("Merge {} PDFs{}", inputs.len(), linearize_str)
            }
            JobSpec::PdfSplit { pages, .. } => {
                format!("Split PDF into {} page spec(s)", pages.len())
            }
            JobSpec::PdfCompress { level, .. } => {
                format!("Compress PDF with {} compression", level)
            }
            JobSpec::PdfLinearize { .. } => "Linearize PDF".to_string(),
            JobSpec::PdfReorder { page_order, .. } => {
                format!("Reorder PDF pages ({} pages)", page_order.len())
            }
            JobSpec::PdfExtract { pages, format, .. } => {
                format!(
                    "Extract {} page spec(s) from PDF as {}",
                    pages.len(),
                    format
                )
            }
            JobSpec::PdfOcr { language, .. } => {
                format!("OCR PDF with language '{}'", language)
            }
            JobSpec::PdfMetadata { action, .. } => match action {
                MetadataAction::GetAll => "Read all PDF metadata".to_string(),
                MetadataAction::Get(field) => format!("Read PDF metadata field '{}'", field),
                MetadataAction::Set(fields) => {
                    format!("Set {} PDF metadata field(s)", fields.len())
                }
            },
            JobSpec::ImageConvert {
                format, quality, ..
            } => {
                if let Some(q) = quality {
                    format!("Convert image to {} (quality {})", format.extension(), q)
                } else {
                    format!("Convert image to {}", format.extension())
                }
            }
            JobSpec::ImageResize { width, height, .. } => match (width, height) {
                (Some(w), Some(h)) => format!("Resize image to {}x{}", w, h),
                (Some(w), None) => format!("Resize image to width {}", w),
                (None, Some(h)) => format!("Resize image to height {}", h),
                (None, None) => "Resize image".to_string(),
            },
            JobSpec::ImageStrip { .. } => "Strip image metadata".to_string(),
            JobSpec::AudioConvert {
                format, bitrate, ..
            } => {
                if let Some(br) = bitrate {
                    format!("Convert audio to {} ({}kbps)", format.extension(), br)
                } else {
                    format!("Convert audio to {}", format.extension())
                }
            }
            JobSpec::AudioNormalize { target, .. } => {
                format!("Normalize audio to {}", target)
            }
            JobSpec::AudioExtract {
                format, bitrate, ..
            } => {
                if let Some(br) = bitrate {
                    format!("Extract audio as {} ({}kbps)", format.extension(), br)
                } else {
                    format!("Extract audio as {}", format.extension())
                }
            }
            JobSpec::AudioTrim { start, end, .. } => match (start, end) {
                (Some(s), Some(e)) => format!("Trim audio from {:.1}s to {:.1}s", s, e),
                (Some(s), None) => format!("Trim audio from {:.1}s to end", s),
                (None, Some(e)) => format!("Trim audio from start to {:.1}s", e),
                (None, None) => "Trim audio".to_string(),
            },
            JobSpec::AudioJoin { inputs, .. } => {
                format!("Join {} audio files", inputs.len())
            }
            JobSpec::AudioVolume { gain_db, .. } => {
                if *gain_db >= 0.0 {
                    format!("Adjust audio volume by +{:.1}dB", gain_db)
                } else {
                    format!("Adjust audio volume by {:.1}dB", gain_db)
                }
            }
            JobSpec::AudioMono { .. } => "Convert audio to mono".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::pages::PageSpec;

    #[test]
    fn test_pdf_compress_description_with_level() {
        let spec = JobSpec::PdfCompress {
            input: PathBuf::from("input.pdf"),
            output: PathBuf::from("output.pdf"),
            level: "high".to_string(),
        };
        assert_eq!(spec.description(), "Compress PDF with high compression");
    }

    #[test]
    fn test_pdf_compress_description_standard() {
        let spec = JobSpec::PdfCompress {
            input: PathBuf::from("input.pdf"),
            output: PathBuf::from("output.pdf"),
            level: "standard".to_string(),
        };
        assert_eq!(spec.description(), "Compress PDF with standard compression");
    }

    #[test]
    fn test_pdf_linearize_description() {
        let spec = JobSpec::PdfLinearize {
            input: PathBuf::from("input.pdf"),
            output: PathBuf::from("output.pdf"),
        };
        assert_eq!(spec.description(), "Linearize PDF");
    }

    #[test]
    fn test_pdf_reorder_description() {
        let spec = JobSpec::PdfReorder {
            input: PathBuf::from("input.pdf"),
            output: PathBuf::from("output.pdf"),
            page_order: vec![3, 1, 2],
        };
        assert_eq!(spec.description(), "Reorder PDF pages (3 pages)");
    }

    #[test]
    fn test_pdf_extract_description() {
        let pages = vec![
            PageSpec::Range {
                start: 1,
                end: Some(5),
            },
            PageSpec::Page(10),
        ];
        let spec = JobSpec::PdfExtract {
            input: PathBuf::from("input.pdf"),
            output: Some(PathBuf::from("output.pdf")),
            output_dir: None,
            pages,
            format: "pdf".to_string(),
        };
        assert_eq!(spec.description(), "Extract 2 page spec(s) from PDF as pdf");
    }

    #[test]
    fn test_pdf_ocr_description() {
        let spec = JobSpec::PdfOcr {
            input: PathBuf::from("scan.pdf"),
            output: PathBuf::from("searchable.pdf"),
            language: "eng".to_string(),
            skip_text: true,
            deskew: false,
            force_ocr: false,
        };
        assert_eq!(spec.description(), "OCR PDF with language 'eng'");
    }

    #[test]
    fn test_pdf_metadata_get_all_description() {
        let spec = JobSpec::PdfMetadata {
            input: PathBuf::from("doc.pdf"),
            output: None,
            action: MetadataAction::GetAll,
        };
        assert_eq!(spec.description(), "Read all PDF metadata");
    }

    #[test]
    fn test_pdf_metadata_get_field_description() {
        let spec = JobSpec::PdfMetadata {
            input: PathBuf::from("doc.pdf"),
            output: None,
            action: MetadataAction::Get("title".to_string()),
        };
        assert_eq!(spec.description(), "Read PDF metadata field 'title'");
    }

    #[test]
    fn test_pdf_metadata_set_description() {
        let spec = JobSpec::PdfMetadata {
            input: PathBuf::from("doc.pdf"),
            output: Some(PathBuf::from("updated.pdf")),
            action: MetadataAction::Set(vec![
                ("title".to_string(), "My Document".to_string()),
                ("author".to_string(), "John Doe".to_string()),
            ]),
        };
        assert_eq!(spec.description(), "Set 2 PDF metadata field(s)");
    }

    #[test]
    fn test_image_convert_description() {
        let spec = JobSpec::ImageConvert {
            input: PathBuf::from("photo.jpg"),
            output: PathBuf::from("photo.webp"),
            format: ImageFormat::WebP,
            quality: Some(80),
            compression: None,
            strip_metadata: false,
        };
        assert_eq!(spec.description(), "Convert image to webp (quality 80)");
    }

    #[test]
    fn test_image_convert_description_no_quality() {
        let spec = JobSpec::ImageConvert {
            input: PathBuf::from("photo.jpg"),
            output: PathBuf::from("photo.png"),
            format: ImageFormat::Png,
            quality: None,
            compression: Some(0),
            strip_metadata: true,
        };
        assert_eq!(spec.description(), "Convert image to png");
    }

    #[test]
    fn test_image_resize_description() {
        let spec = JobSpec::ImageResize {
            input: PathBuf::from("photo.jpg"),
            output: PathBuf::from("thumb.jpg"),
            width: Some(800),
            height: Some(600),
        };
        assert_eq!(spec.description(), "Resize image to 800x600");
    }

    #[test]
    fn test_image_resize_description_width_only() {
        let spec = JobSpec::ImageResize {
            input: PathBuf::from("photo.jpg"),
            output: PathBuf::from("thumb.jpg"),
            width: Some(800),
            height: None,
        };
        assert_eq!(spec.description(), "Resize image to width 800");
    }

    #[test]
    fn test_image_strip_description() {
        let spec = JobSpec::ImageStrip {
            input: PathBuf::from("photo.jpg"),
            output: PathBuf::from("clean.jpg"),
        };
        assert_eq!(spec.description(), "Strip image metadata");
    }

    #[test]
    fn test_image_resize_description_height_only() {
        let spec = JobSpec::ImageResize {
            input: PathBuf::from("photo.jpg"),
            output: PathBuf::from("thumb.jpg"),
            width: None,
            height: Some(600),
        };
        assert_eq!(spec.description(), "Resize image to height 600");
    }

    // Compress tests (compress uses ImageConvert with compression settings)

    #[test]
    fn test_image_compress_jpeg_description() {
        // JPEG compress shows quality
        let spec = JobSpec::ImageConvert {
            input: PathBuf::from("photo.jpg"),
            output: PathBuf::from("photo_compressed.jpg"),
            format: ImageFormat::Jpeg,
            quality: Some(80),
            compression: None,
            strip_metadata: true,
        };
        assert_eq!(spec.description(), "Convert image to jpg (quality 80)");
    }

    #[test]
    fn test_image_compress_png_description() {
        // PNG compress doesn't show quality (uses compression level instead)
        let spec = JobSpec::ImageConvert {
            input: PathBuf::from("image.png"),
            output: PathBuf::from("image_compressed.png"),
            format: ImageFormat::Png,
            quality: None,
            compression: Some(9),
            strip_metadata: true,
        };
        assert_eq!(spec.description(), "Convert image to png");
    }

    // Audio tests

    #[test]
    fn test_audio_convert_description_with_bitrate() {
        let spec = JobSpec::AudioConvert {
            input: PathBuf::from("audio.wav"),
            output: PathBuf::from("audio.mp3"),
            format: AudioFormat::Mp3,
            bitrate: Some(192),
        };
        assert_eq!(spec.description(), "Convert audio to mp3 (192kbps)");
    }

    #[test]
    fn test_audio_convert_description_no_bitrate() {
        let spec = JobSpec::AudioConvert {
            input: PathBuf::from("audio.wav"),
            output: PathBuf::from("audio.flac"),
            format: AudioFormat::Flac,
            bitrate: None,
        };
        assert_eq!(spec.description(), "Convert audio to flac");
    }

    #[test]
    fn test_audio_normalize_ebu_r128_description() {
        let spec = JobSpec::AudioNormalize {
            input: PathBuf::from("audio.wav"),
            output: PathBuf::from("audio_normalized.wav"),
            target: LoudnessTarget::EbuR128,
        };
        assert_eq!(spec.description(), "Normalize audio to EBU R128 (-23 LUFS)");
    }

    #[test]
    fn test_audio_normalize_streaming_description() {
        let spec = JobSpec::AudioNormalize {
            input: PathBuf::from("audio.wav"),
            output: PathBuf::from("audio_normalized.wav"),
            target: LoudnessTarget::Streaming,
        };
        assert_eq!(
            spec.description(),
            "Normalize audio to Streaming (-14 LUFS)"
        );
    }

    #[test]
    fn test_audio_normalize_custom_description() {
        let spec = JobSpec::AudioNormalize {
            input: PathBuf::from("audio.wav"),
            output: PathBuf::from("audio_normalized.wav"),
            target: LoudnessTarget::Custom(-16.0),
        };
        assert_eq!(spec.description(), "Normalize audio to -16 LUFS");
    }

    #[test]
    fn test_audio_extract_description_with_bitrate() {
        let spec = JobSpec::AudioExtract {
            input: PathBuf::from("video.mp4"),
            output: PathBuf::from("audio.mp3"),
            format: AudioFormat::Mp3,
            bitrate: Some(192),
        };
        assert_eq!(spec.description(), "Extract audio as mp3 (192kbps)");
    }

    #[test]
    fn test_audio_extract_description_no_bitrate() {
        let spec = JobSpec::AudioExtract {
            input: PathBuf::from("video.mp4"),
            output: PathBuf::from("audio.flac"),
            format: AudioFormat::Flac,
            bitrate: None,
        };
        assert_eq!(spec.description(), "Extract audio as flac");
    }

    #[test]
    fn test_audio_trim_description() {
        let spec = JobSpec::AudioTrim {
            input: PathBuf::from("song.mp3"),
            output: PathBuf::from("clip.mp3"),
            start: Some(30.0),
            end: Some(120.0),
        };
        assert_eq!(spec.description(), "Trim audio from 30.0s to 120.0s");
    }

    #[test]
    fn test_audio_trim_description_start_only() {
        let spec = JobSpec::AudioTrim {
            input: PathBuf::from("song.mp3"),
            output: PathBuf::from("clip.mp3"),
            start: Some(60.0),
            end: None,
        };
        assert_eq!(spec.description(), "Trim audio from 60.0s to end");
    }

    #[test]
    fn test_audio_trim_description_end_only() {
        let spec = JobSpec::AudioTrim {
            input: PathBuf::from("song.mp3"),
            output: PathBuf::from("clip.mp3"),
            start: None,
            end: Some(90.0),
        };
        assert_eq!(spec.description(), "Trim audio from start to 90.0s");
    }

    #[test]
    fn test_audio_join_description() {
        let spec = JobSpec::AudioJoin {
            inputs: vec![
                PathBuf::from("part1.mp3"),
                PathBuf::from("part2.mp3"),
                PathBuf::from("part3.mp3"),
            ],
            output: PathBuf::from("joined.mp3"),
        };
        assert_eq!(spec.description(), "Join 3 audio files");
    }

    #[test]
    fn test_audio_volume_positive_description() {
        let spec = JobSpec::AudioVolume {
            input: PathBuf::from("quiet.wav"),
            output: PathBuf::from("louder.wav"),
            gain_db: 6.0,
        };
        assert_eq!(spec.description(), "Adjust audio volume by +6.0dB");
    }

    #[test]
    fn test_audio_volume_negative_description() {
        let spec = JobSpec::AudioVolume {
            input: PathBuf::from("loud.wav"),
            output: PathBuf::from("quieter.wav"),
            gain_db: -3.0,
        };
        assert_eq!(spec.description(), "Adjust audio volume by -3.0dB");
    }

    #[test]
    fn test_audio_mono_description() {
        let spec = JobSpec::AudioMono {
            input: PathBuf::from("stereo.wav"),
            output: PathBuf::from("mono.wav"),
        };
        assert_eq!(spec.description(), "Convert audio to mono");
    }
}
