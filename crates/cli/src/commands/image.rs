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

    /// Output image file
    #[arg(short, long, required = true, help = "Output image file")]
    pub output: PathBuf,

    /// Target format (auto-detected from output extension if not specified)
    #[arg(long, help = "Target format: jpeg, png, webp, avif, tiff, gif")]
    pub to: Option<String>,

    /// Quality (0-100). Applies to JPEG, WebP, and AVIF formats
    #[arg(short, long, help = "Quality (0-100). Applies to JPEG, WebP, AVIF")]
    pub quality: Option<u8>,

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
    // Determine format from --to flag or output extension
    let format = if let Some(ref to) = args.to {
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
        ImageFormat::from_path(&args.output).ok_or_else(|| {
            forgekit_core::utils::error::ForgeKitError::InvalidInput {
                path: args.output.clone(),
                reason:
                    "Cannot determine format from output extension. Use --to to specify format."
                        .to_string(),
            }
        })?
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

    let spec = JobSpec::ImageConvert {
        input: args.input,
        output: args.output,
        format,
        quality: args.quality,
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
