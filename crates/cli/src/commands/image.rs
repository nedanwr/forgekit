use clap::{Args, Subcommand};
use forgekit_core::job::JobSpec;
use forgekit_core::utils::error::Result;
use forgekit_core::utils::image::ImageFormat;
use std::path::PathBuf;

#[derive(Subcommand, Clone)]
pub enum ImageCommand {
    /// Convert image to a different format
    ///
    /// Examples:
    ///   forgekit image convert photo.jpg --output photo.webp
    ///   forgekit image convert photo.jpg --output photo.webp --quality 80
    ///   forgekit image convert photo.dng --output photo.jpg --strip
    ///
    /// Supported formats: jpeg/jpg, png, webp, avif, tiff, gif
    /// Format is auto-detected from the output extension, or use --to to specify.
    Convert(ConvertArgs),

    /// Resize image preserving aspect ratio
    ///
    /// Examples:
    ///   forgekit image resize photo.jpg --output thumb.jpg --width 800
    ///   forgekit image resize photo.jpg --output thumb.jpg --height 600
    ///   forgekit image resize photo.jpg --output thumb.jpg --width 800 --height 600
    ///
    /// Specify width, height, or both. Aspect ratio is always preserved.
    Resize(ResizeArgs),

    /// Strip EXIF and other metadata from image
    ///
    /// Examples:
    ///   forgekit image strip photo.jpg --output clean.jpg
    ///
    /// Removes EXIF, XMP, IPTC, ICC profiles, and other metadata.
    /// Useful for privacy before sharing images.
    Strip(StripArgs),
}

#[derive(Args, Clone)]
pub struct ConvertArgs {
    /// Input image file
    #[arg(required = true, help = "Input image file")]
    pub input: PathBuf,

    /// Output image file (defaults to input name with new extension in current dir)
    #[arg(short, long, help = "Output image file")]
    pub output: Option<PathBuf>,

    /// Target format (required if --output not specified)
    #[arg(long, help = "Target format: jpeg, png, webp, avif, tiff, gif")]
    pub to: Option<String>,

    /// Quality (0-100). Applies to JPEG, WebP, and AVIF formats
    #[arg(short, long, help = "Quality (0-100). Applies to JPEG, WebP, AVIF")]
    pub quality: Option<u8>,

    /// Compression level (1-9). Higher = smaller file, slower. For PNG
    #[arg(short, long, help = "Compression (1-9). Default: none (fastest)")]
    pub compression: Option<u8>,

    /// Strip metadata during conversion
    #[arg(long, help = "Strip EXIF and other metadata")]
    pub strip: bool,
}

#[derive(Args, Clone)]
pub struct ResizeArgs {
    /// Input image file
    #[arg(required = true, help = "Input image file")]
    pub input: PathBuf,

    /// Output image file
    #[arg(short, long, required = true, help = "Output image file")]
    pub output: PathBuf,

    /// Target width (preserves aspect ratio if height not specified)
    #[arg(short, long, help = "Target width in pixels")]
    pub width: Option<u32>,

    /// Target height (preserves aspect ratio if width not specified)
    #[arg(short = 'H', long, help = "Target height in pixels")]
    pub height: Option<u32>,
}

#[derive(Args, Clone)]
pub struct StripArgs {
    /// Input image file
    #[arg(required = true, help = "Input image file")]
    pub input: PathBuf,

    /// Output image file
    #[arg(short, long, required = true, help = "Output image file")]
    pub output: PathBuf,
}

pub fn handle_image_command(cmd: ImageCommand, plan_only: bool, json_output: bool) -> Result<()> {
    match cmd {
        ImageCommand::Convert(args) => handle_convert(args, plan_only, json_output),
        ImageCommand::Resize(args) => handle_resize(args, plan_only, json_output),
        ImageCommand::Strip(args) => handle_strip(args, plan_only, json_output),
    }
}

fn handle_convert(args: ConvertArgs, plan_only: bool, json_output: bool) -> Result<()> {
    // Determine format and output path
    let (format, output) = if let Some(output) = args.output {
        // Output provided - get format from --to or output extension
        let fmt = if let Some(ref to) = args.to {
            to.parse::<ImageFormat>().map_err(|_| {
                forgekit_core::utils::error::ForgeKitError::InvalidInput {
                    path: PathBuf::new(),
                    reason: format!(
                        "Unknown format '{}'. Supported: jpeg, png, webp, avif, tiff, gif",
                        to
                    ),
                }
            })?
        } else {
            ImageFormat::from_path(&output).ok_or_else(|| {
                forgekit_core::utils::error::ForgeKitError::InvalidInput {
                    path: output.clone(),
                    reason: "Cannot determine format from output extension. Use --to to specify."
                        .to_string(),
                }
            })?
        };
        (fmt, output)
    } else {
        // No output - require --to flag and derive output path
        let to = args.to.ok_or_else(|| {
            forgekit_core::utils::error::ForgeKitError::InvalidInput {
                path: PathBuf::new(),
                reason: "Either --output or --to is required".to_string(),
            }
        })?;
        let fmt = to.parse::<ImageFormat>().map_err(|_| {
            forgekit_core::utils::error::ForgeKitError::InvalidInput {
                path: PathBuf::new(),
                reason: format!(
                    "Unknown format '{}'. Supported: jpeg, png, webp, avif, tiff, gif",
                    to
                ),
            }
        })?;
        // Derive output: input stem + new extension in current directory
        let stem = args.input.file_stem().ok_or_else(|| {
            forgekit_core::utils::error::ForgeKitError::InvalidInput {
                path: args.input.clone(),
                reason: "Cannot determine filename from input".to_string(),
            }
        })?;
        let output = PathBuf::from(stem).with_extension(fmt.extension());
        (fmt, output)
    };

    // Validate quality range
    if let Some(q) = args.quality {
        if q > 100 {
            return Err(forgekit_core::utils::error::ForgeKitError::InvalidInput {
                path: PathBuf::new(),
                reason: "Quality must be between 0 and 100".to_string(),
            });
        }
    }

    // Validate compression range (1-9) and default to 0 (no compression) if not specified
    let compression = match args.compression {
        Some(c) if c >= 1 && c <= 9 => Some(c),
        Some(_) => {
            return Err(forgekit_core::utils::error::ForgeKitError::InvalidInput {
                path: PathBuf::new(),
                reason: "Compression must be between 1 and 9".to_string(),
            });
        }
        None => Some(0), // Default: no compression (fastest)
    };

    let spec = JobSpec::ImageConvert {
        input: args.input,
        output,
        format,
        quality: args.quality,
        compression,
        strip_metadata: args.strip,
    };

    execute_image_job(&spec, plan_only, json_output)
}

fn handle_resize(args: ResizeArgs, plan_only: bool, json_output: bool) -> Result<()> {
    // Validate that at least one dimension is specified
    if args.width.is_none() && args.height.is_none() {
        return Err(forgekit_core::utils::error::ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: "At least one of --width or --height is required".to_string(),
        });
    }

    let spec = JobSpec::ImageResize {
        input: args.input,
        output: args.output,
        width: args.width,
        height: args.height,
    };

    execute_image_job(&spec, plan_only, json_output)
}

fn handle_strip(args: StripArgs, plan_only: bool, json_output: bool) -> Result<()> {
    let spec = JobSpec::ImageStrip {
        input: args.input,
        output: args.output,
    };

    execute_image_job(&spec, plan_only, json_output)
}

fn execute_image_job(spec: &JobSpec, plan_only: bool, json_output: bool) -> Result<()> {
    if plan_only {
        let plan = forgekit_core::job::executor::execute_job(spec, true)?;
        if json_output {
            let event = forgekit_core::job::progress::ProgressEvent::Progress {
                version: 1,
                job_id: forgekit_core::job::progress::new_job_id(),
                progress: forgekit_core::job::progress::ProgressInfo {
                    current: 0,
                    total: 1,
                    percent: 0,
                    stage: Some("plan".to_string()),
                },
                message: plan.clone(),
            };
            println!("{}", serde_json::to_string(&event).unwrap());
        } else {
            println!("{}", plan);
        }
        Ok(())
    } else {
        let result = forgekit_core::job::executor::execute_job(spec, false)?;
        if json_output {
            let event = forgekit_core::job::progress::ProgressEvent::Complete {
                version: 1,
                job_id: forgekit_core::job::progress::new_job_id(),
                result: forgekit_core::job::progress::JobResult {
                    output: result.clone(),
                    size_bytes: 0,
                    duration_ms: 0,
                },
            };
            println!("{}", serde_json::to_string(&event).unwrap());
        } else {
            println!("{}", result);
        }
        Ok(())
    }
}
