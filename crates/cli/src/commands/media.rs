use clap::{Args, Subcommand};
use forgekit_core::job::executor::execute_job;
use forgekit_core::job::JobSpec;
use forgekit_core::utils::error::Result;
use std::path::PathBuf;

#[derive(Subcommand, Clone)]
pub enum MediaCommand {
    /// Transcode video to H.264 format
    ///
    /// Examples:
    ///   forgekit media transcode video.mov --output video.mp4
    ///   forgekit media transcode video.mkv --output video.mp4 --crf 20 --preset slow
    ///   forgekit media transcode video.mp4 --output 720p.mp4 --scale 1280
    ///
    /// CRF (Constant Rate Factor): 0-51, lower = better quality, larger file
    ///   0     = Lossless
    ///   18-23 = Visually lossless to good quality (default: 23)
    ///   24-28 = Medium quality, good compression
    ///   29+   = Low quality
    ///
    /// Presets: ultrafast, superfast, veryfast, faster, fast, medium, slow, slower, veryslow
    /// Slower presets = better compression, smaller file size for same quality
    Transcode(TranscodeArgs),
}

#[derive(Args, Clone)]
pub struct TranscodeArgs {
    /// Input video file
    #[arg(required = true, help = "Input video file")]
    pub input: PathBuf,

    /// Output video file
    #[arg(short, long, required = true, help = "Output video file (must be .mp4)")]
    pub output: PathBuf,

    /// CRF quality (0-51, lower = better quality). Default: 23
    #[arg(
        short,
        long,
        default_value = "23",
        help = "CRF quality (0-51, default: 23)"
    )]
    pub crf: u8,

    /// Encoder preset (ultrafast to veryslow). Default: medium
    #[arg(
        short,
        long,
        default_value = "medium",
        help = "Preset: ultrafast, superfast, veryfast, faster, fast, medium, slow, slower, veryslow"
    )]
    pub preset: String,

    /// Scale video width (height auto-calculated to preserve aspect ratio)
    #[arg(
        short,
        long,
        help = "Scale to width (height auto-calculated to preserve aspect ratio)"
    )]
    pub scale: Option<i32>,

    /// Re-encode audio to AAC 128kbps (default: copy audio stream)
    #[arg(
        long,
        default_value = "false",
        help = "Re-encode audio to AAC 128kbps"
    )]
    pub reencode_audio: bool,
}

pub fn handle_media_command(cmd: &MediaCommand, plan_only: bool, _json_output: bool) -> Result<()> {
    match cmd {
        MediaCommand::Transcode(args) => handle_transcode(args, plan_only),
    }
}

fn handle_transcode(args: &TranscodeArgs, plan_only: bool) -> Result<()> {
    // Validate CRF range
    if args.crf > 51 {
        return Err(forgekit_core::utils::error::ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: format!("CRF must be 0-51, got {}", args.crf),
        });
    }

    // Validate preset
    let valid_presets = [
        "ultrafast",
        "superfast",
        "veryfast",
        "faster",
        "fast",
        "medium",
        "slow",
        "slower",
        "veryslow",
    ];
    if !valid_presets.contains(&args.preset.as_str()) {
        return Err(forgekit_core::utils::error::ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: format!(
                "Invalid preset '{}'. Valid presets: {}",
                args.preset,
                valid_presets.join(", ")
            ),
        });
    }

    // Convert scale width to (width, -1) tuple for aspect ratio preservation
    let scale = args.scale.map(|w| (w, -1));

    let spec = JobSpec::VideoTranscode {
        input: args.input.clone(),
        output: args.output.clone(),
        crf: args.crf,
        preset: args.preset.clone(),
        scale,
        copy_audio: !args.reencode_audio,
    };

    let result = execute_job(&spec, plan_only)?;
    println!("{}", result);
    Ok(())
}
