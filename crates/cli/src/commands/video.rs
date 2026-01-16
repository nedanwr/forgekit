use clap::{Args, Subcommand};
use forgekit_core::job::executor::execute_job;
use forgekit_core::job::JobSpec;
use forgekit_core::utils::error::Result;
use std::path::{Path, PathBuf};

#[derive(Subcommand, Clone)]
pub enum VideoCommand {
    /// Transcode video to H.264 format
    ///
    /// Examples:
    ///   forgekit video transcode video.mov --output video.mp4
    ///   forgekit video transcode video.mkv --output video.mp4 --crf 20 --preset slow
    ///   forgekit video transcode video.mp4 --output 720p.mp4 --scale 1280
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

    /// Trim video to a specific time range
    ///
    /// Examples:
    ///   forgekit video trim video.mp4 --start 0:30 --end 2:00 --output clip.mp4
    ///   forgekit video trim video.mp4 --start 30 --end 120 --output clip.mp4
    ///   forgekit video trim movie.mp4 --start 5:00 --output from_5min.mp4
    ///
    /// Time format: seconds (30, 90.5) or MM:SS (1:30) or HH:MM:SS (1:30:00)
    /// Uses stream copy (no re-encoding) for fast trimming.
    Trim(TrimArgs),

    /// Join multiple video files into one
    ///
    /// Examples:
    ///   forgekit video join intro.mp4 main.mp4 outro.mp4 --output final.mp4
    ///   forgekit video join "part_*.mp4" --output full.mp4
    ///   forgekit video join "clip_[0-9][0-9].mp4" --output movie.mp4
    ///
    /// Supports glob patterns. Files are sorted naturally (part_2 before part_10).
    /// All input files must have the same codec, resolution, and frame rate.
    Join(JoinArgs),

    /// Show video file information
    ///
    /// Examples:
    ///   forgekit video info video.mp4
    ///   forgekit video info video.mp4 --json
    ///
    /// Shows duration, resolution, codec, bitrate, and frame rate.
    Info(InfoArgs),

    /// Extract a thumbnail frame from video
    ///
    /// Examples:
    ///   forgekit video thumbnail video.mp4 --time 5 --output thumb.jpg
    ///   forgekit video thumbnail video.mp4 --time 1:30 --output preview.png
    ///
    /// Extracts a single frame at the specified timestamp.
    /// Output format determined by extension (.jpg, .png).
    Thumbnail(ThumbnailArgs),

    /// Convert video clip to animated GIF
    ///
    /// Examples:
    ///   forgekit video gif video.mp4 --output clip.gif
    ///   forgekit video gif video.mp4 --start 10 --duration 5 --output clip.gif
    ///   forgekit video gif video.mp4 --width 480 --fps 15 --output clip.gif
    ///
    /// Creates high-quality GIF with optimized palette.
    Gif(GifArgs),

    /// Change video playback speed
    ///
    /// Examples:
    ///   forgekit video speed video.mp4 --speed 2 --output fast.mp4
    ///   forgekit video speed video.mp4 --speed 0.5 --output slow.mp4
    ///
    /// Speed multiplier: 2 = double speed, 0.5 = half speed.
    /// Audio is adjusted to match video speed.
    Speed(SpeedArgs),

    /// Rotate video by specified degrees
    ///
    /// Examples:
    ///   forgekit video rotate video.mp4 --degrees 90 --output rotated.mp4
    ///   forgekit video rotate video.mp4 --degrees 180 --output flipped.mp4
    ///
    /// Supports 90, 180, or 270 degrees clockwise rotation.
    Rotate(RotateArgs),

    /// Remove audio track from video
    ///
    /// Examples:
    ///   forgekit video mute video.mp4 --output silent.mp4
    ///
    /// Creates a video file with no audio stream.
    Mute(MuteArgs),
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

#[derive(Args, Clone)]
pub struct TrimArgs {
    /// Input video file
    #[arg(required = true, help = "Input video file")]
    pub input: PathBuf,

    /// Output video file
    #[arg(short, long, required = true, help = "Output video file")]
    pub output: PathBuf,

    /// Start time (seconds or MM:SS or HH:MM:SS)
    #[arg(short, long, help = "Start time (e.g., 30, 1:30, 0:01:30)")]
    pub start: Option<String>,

    /// End time (seconds or MM:SS or HH:MM:SS)
    #[arg(short, long, help = "End time (e.g., 120, 2:00, 0:02:00)")]
    pub end: Option<String>,
}

#[derive(Args, Clone)]
pub struct JoinArgs {
    /// Input video files or glob pattern (e.g., "part_*.mp4")
    #[arg(required = true, num_args = 1.., help = "Input files or glob pattern (e.g., part_*.mp4)")]
    pub inputs: Vec<String>,

    /// Output video file
    #[arg(short, long, required = true, help = "Output video file")]
    pub output: PathBuf,
}

#[derive(Args, Clone)]
pub struct InfoArgs {
    /// Input video file
    #[arg(required = true, help = "Input video file")]
    pub input: PathBuf,
}

#[derive(Args, Clone)]
pub struct ThumbnailArgs {
    /// Input video file
    #[arg(required = true, help = "Input video file")]
    pub input: PathBuf,

    /// Output image file (.jpg or .png)
    #[arg(short, long, required = true, help = "Output image file (.jpg or .png)")]
    pub output: PathBuf,

    /// Timestamp to extract frame (seconds or MM:SS or HH:MM:SS)
    #[arg(short, long, required = true, help = "Time to extract frame (e.g., 5, 1:30)")]
    pub time: String,
}

#[derive(Args, Clone)]
pub struct GifArgs {
    /// Input video file
    #[arg(required = true, help = "Input video file")]
    pub input: PathBuf,

    /// Output GIF file
    #[arg(short, long, required = true, help = "Output GIF file")]
    pub output: PathBuf,

    /// Start time (seconds or MM:SS or HH:MM:SS)
    #[arg(short, long, help = "Start time (e.g., 10, 0:30)")]
    pub start: Option<String>,

    /// Duration in seconds (default: 5)
    #[arg(short, long, default_value = "5", help = "Duration in seconds")]
    pub duration: f64,

    /// Output width in pixels (height auto-calculated)
    #[arg(short, long, help = "Output width (height auto-calculated)")]
    pub width: Option<u32>,

    /// Frame rate (default: 10)
    #[arg(short, long, default_value = "10", help = "Frame rate (default: 10)")]
    pub fps: u32,
}

#[derive(Args, Clone)]
pub struct SpeedArgs {
    /// Input video file
    #[arg(required = true, help = "Input video file")]
    pub input: PathBuf,

    /// Output video file
    #[arg(short, long, required = true, help = "Output video file")]
    pub output: PathBuf,

    /// Speed multiplier (e.g., 2 for 2x speed, 0.5 for half speed)
    #[arg(short = 'x', long, required = true, help = "Speed multiplier (e.g., 2, 0.5)")]
    pub speed: f64,
}

#[derive(Args, Clone)]
pub struct RotateArgs {
    /// Input video file
    #[arg(required = true, help = "Input video file")]
    pub input: PathBuf,

    /// Output video file
    #[arg(short, long, required = true, help = "Output video file")]
    pub output: PathBuf,

    /// Rotation angle: 90, 180, or 270 degrees clockwise
    #[arg(short, long, required = true, help = "Rotation: 90, 180, or 270 degrees")]
    pub degrees: u32,
}

#[derive(Args, Clone)]
pub struct MuteArgs {
    /// Input video file
    #[arg(required = true, help = "Input video file")]
    pub input: PathBuf,

    /// Output video file (no audio)
    #[arg(short, long, required = true, help = "Output video file (no audio)")]
    pub output: PathBuf,
}

pub fn handle_video_command(cmd: &VideoCommand, plan_only: bool, json_output: bool) -> Result<()> {
    match cmd {
        VideoCommand::Transcode(args) => handle_transcode(args, plan_only),
        VideoCommand::Trim(args) => handle_trim(args, plan_only),
        VideoCommand::Join(args) => handle_join(args, plan_only),
        VideoCommand::Info(args) => handle_info(args, json_output),
        VideoCommand::Thumbnail(args) => handle_thumbnail(args, plan_only),
        VideoCommand::Gif(args) => handle_gif(args, plan_only),
        VideoCommand::Speed(args) => handle_speed(args, plan_only),
        VideoCommand::Rotate(args) => handle_rotate(args, plan_only),
        VideoCommand::Mute(args) => handle_mute(args, plan_only),
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

fn handle_trim(args: &TrimArgs, plan_only: bool) -> Result<()> {
    let start = args.start.as_ref().map(|s| parse_time(s)).transpose()?;
    let end = args.end.as_ref().map(|s| parse_time(s)).transpose()?;

    if start.is_none() && end.is_none() {
        return Err(forgekit_core::utils::error::ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: "At least one of --start or --end is required".to_string(),
        });
    }

    let spec = JobSpec::VideoTrim {
        input: args.input.clone(),
        output: args.output.clone(),
        start,
        end,
    };

    let result = execute_job(&spec, plan_only)?;
    println!("{}", result);
    Ok(())
}

fn handle_join(args: &JoinArgs, plan_only: bool) -> Result<()> {
    // Expand glob patterns and collect files
    let mut files: Vec<PathBuf> = Vec::new();

    for input in &args.inputs {
        // Check if input contains glob characters
        if input.contains('*') || input.contains('?') || input.contains('[') {
            let paths = glob::glob(input).map_err(|e| {
                forgekit_core::utils::error::ForgeKitError::InvalidInput {
                    path: PathBuf::from(input),
                    reason: format!("Invalid glob pattern: {}", e),
                }
            })?;

            for entry in paths {
                match entry {
                    Ok(path) => files.push(path),
                    Err(e) => {
                        return Err(forgekit_core::utils::error::ForgeKitError::InvalidInput {
                            path: PathBuf::new(),
                            reason: format!("Glob error: {}", e),
                        });
                    }
                }
            }
        } else {
            files.push(PathBuf::from(input));
        }
    }

    // Sort files naturally (so part_2 comes before part_10)
    files.sort_by_key(|a| natural_sort_key(a));

    if files.len() < 2 {
        return Err(forgekit_core::utils::error::ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: format!("At least 2 files required for join, found {}", files.len()),
        });
    }

    let spec = JobSpec::VideoJoin {
        inputs: files,
        output: args.output.clone(),
    };

    let result = execute_job(&spec, plan_only)?;
    println!("{}", result);
    Ok(())
}

/// Generate a sort key for natural sorting (so part_2 < part_10)
fn natural_sort_key(path: &Path) -> Vec<NaturalSortPart> {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let mut parts = Vec::new();
    let mut current_num = String::new();
    let mut current_str = String::new();

    for c in name.chars() {
        if c.is_ascii_digit() {
            if !current_str.is_empty() {
                parts.push(NaturalSortPart::Str(current_str.clone()));
                current_str.clear();
            }
            current_num.push(c);
        } else {
            if !current_num.is_empty() {
                parts.push(NaturalSortPart::Num(current_num.parse().unwrap_or(0)));
                current_num.clear();
            }
            current_str.push(c);
        }
    }

    if !current_num.is_empty() {
        parts.push(NaturalSortPart::Num(current_num.parse().unwrap_or(0)));
    }
    if !current_str.is_empty() {
        parts.push(NaturalSortPart::Str(current_str));
    }

    parts
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
enum NaturalSortPart {
    Num(u64),
    Str(String),
}

/// Parse time string to seconds.
/// Supports: seconds (30, 90.5), MM:SS (1:30), HH:MM:SS (1:30:00)
fn parse_time(s: &str) -> Result<f64> {
    let parts: Vec<&str> = s.split(':').collect();

    match parts.len() {
        1 => {
            // Just seconds
            s.parse::<f64>().map_err(|_| {
                forgekit_core::utils::error::ForgeKitError::InvalidInput {
                    path: PathBuf::new(),
                    reason: format!("Invalid time format '{}'. Use seconds (30), MM:SS (1:30), or HH:MM:SS (1:30:00)", s),
                }
            })
        }
        2 => {
            // MM:SS
            let minutes: f64 = parts[0].parse().map_err(|_| {
                forgekit_core::utils::error::ForgeKitError::InvalidInput {
                    path: PathBuf::new(),
                    reason: format!("Invalid minutes in '{}'", s),
                }
            })?;
            let seconds: f64 = parts[1].parse().map_err(|_| {
                forgekit_core::utils::error::ForgeKitError::InvalidInput {
                    path: PathBuf::new(),
                    reason: format!("Invalid seconds in '{}'", s),
                }
            })?;
            Ok(minutes * 60.0 + seconds)
        }
        3 => {
            // HH:MM:SS
            let hours: f64 = parts[0].parse().map_err(|_| {
                forgekit_core::utils::error::ForgeKitError::InvalidInput {
                    path: PathBuf::new(),
                    reason: format!("Invalid hours in '{}'", s),
                }
            })?;
            let minutes: f64 = parts[1].parse().map_err(|_| {
                forgekit_core::utils::error::ForgeKitError::InvalidInput {
                    path: PathBuf::new(),
                    reason: format!("Invalid minutes in '{}'", s),
                }
            })?;
            let seconds: f64 = parts[2].parse().map_err(|_| {
                forgekit_core::utils::error::ForgeKitError::InvalidInput {
                    path: PathBuf::new(),
                    reason: format!("Invalid seconds in '{}'", s),
                }
            })?;
            Ok(hours * 3600.0 + minutes * 60.0 + seconds)
        }
        _ => Err(forgekit_core::utils::error::ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: format!(
                "Invalid time format '{}'. Use seconds (30), MM:SS (1:30), or HH:MM:SS (1:30:00)",
                s
            ),
        }),
    }
}

fn handle_info(args: &InfoArgs, json_output: bool) -> Result<()> {
    use std::process::Command;

    if !args.input.exists() {
        return Err(forgekit_core::utils::error::ForgeKitError::InvalidInput {
            path: args.input.clone(),
            reason: "File does not exist".to_string(),
        });
    }

    // Get file size
    let file_size = std::fs::metadata(&args.input).map(|m| m.len()).unwrap_or(0);
    let file_size_str = if file_size >= 1_000_000_000 {
        format!("{:.1} GB", file_size as f64 / 1_000_000_000.0)
    } else if file_size >= 1_000_000 {
        format!("{:.1} MB", file_size as f64 / 1_000_000.0)
    } else if file_size >= 1_000 {
        format!("{:.1} KB", file_size as f64 / 1_000.0)
    } else {
        format!("{} bytes", file_size)
    };

    // Use ffprobe to get video info
    let (duration, width, height, codec, fps, bitrate) = if let Ok(output) = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-show_entries",
            "format=duration,bit_rate:stream=width,height,codec_name,r_frame_rate",
            "-select_streams",
            "v:0",
            "-of",
            "csv=p=0",
        ])
        .arg(&args.input)
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.trim().lines().collect();

            // Parse stream info (first line): width,height,codec_name,r_frame_rate
            let (width, height, codec, fps) = if let Some(line) = lines.first() {
                let parts: Vec<&str> = line.split(',').collect();
                let w = parts.first().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
                let h = parts.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
                let c = parts.get(2).unwrap_or(&"?").to_string();
                let fps_str = parts.get(3).unwrap_or(&"0/1");
                // Parse frame rate (e.g., "30/1" or "30000/1001")
                let fps = if let Some((num, den)) = fps_str.split_once('/') {
                    let n: f64 = num.parse().unwrap_or(0.0);
                    let d: f64 = den.parse().unwrap_or(1.0);
                    if d > 0.0 { n / d } else { 0.0 }
                } else {
                    fps_str.parse().unwrap_or(0.0)
                };
                (w, h, c, fps)
            } else {
                (0, 0, "?".to_string(), 0.0)
            };

            // Parse format info (second line): duration,bit_rate
            let (duration, bitrate) = if let Some(line) = lines.get(1) {
                let parts: Vec<&str> = line.split(',').collect();
                let dur = parts
                    .first()
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(format_duration)
                    .unwrap_or_else(|| "?".to_string());
                let br = parts
                    .get(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(|b| format!("{:.1} Mbps", b as f64 / 1_000_000.0))
                    .unwrap_or_else(|| "?".to_string());
                (dur, br)
            } else {
                ("?".to_string(), "?".to_string())
            };

            (duration, width, height, codec, fps, bitrate)
        } else {
            ("?".to_string(), 0, 0, "?".to_string(), 0.0, "?".to_string())
        }
    } else {
        ("?".to_string(), 0, 0, "?".to_string(), 0.0, "?".to_string())
    };

    let resolution = if width > 0 && height > 0 {
        format!("{}x{}", width, height)
    } else {
        "?".to_string()
    };

    let fps_str = if fps > 0.0 {
        format!("{:.2}", fps)
    } else {
        "?".to_string()
    };

    if json_output {
        let info = serde_json::json!({
            "file": args.input.display().to_string(),
            "duration": duration,
            "resolution": resolution,
            "width": width,
            "height": height,
            "codec": codec,
            "fps": fps,
            "bitrate": bitrate,
            "size_bytes": file_size,
            "size": file_size_str,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&info).unwrap_or_default()
        );
    } else {
        println!("File:       {}", args.input.display());
        println!("Duration:   {}", duration);
        println!("Resolution: {}", resolution);
        println!("Codec:      {}", codec);
        println!("Frame Rate: {} fps", fps_str);
        println!("Bitrate:    {}", bitrate);
        println!("Size:       {}", file_size_str);
    }

    Ok(())
}

fn format_duration(seconds: f64) -> String {
    let hours = (seconds / 3600.0).floor() as u32;
    let minutes = ((seconds % 3600.0) / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{}:{:02}", minutes, secs)
    }
}

fn handle_thumbnail(args: &ThumbnailArgs, plan_only: bool) -> Result<()> {
    let timestamp = parse_time(&args.time)?;

    let spec = JobSpec::VideoThumbnail {
        input: args.input.clone(),
        output: args.output.clone(),
        timestamp,
    };

    let result = execute_job(&spec, plan_only)?;
    println!("{}", result);
    Ok(())
}

fn handle_gif(args: &GifArgs, plan_only: bool) -> Result<()> {
    let start = args.start.as_ref().map(|s| parse_time(s)).transpose()?;

    let spec = JobSpec::VideoGif {
        input: args.input.clone(),
        output: args.output.clone(),
        start,
        duration: Some(args.duration),
        width: args.width,
        fps: Some(args.fps),
    };

    let result = execute_job(&spec, plan_only)?;
    println!("{}", result);
    Ok(())
}

fn handle_speed(args: &SpeedArgs, plan_only: bool) -> Result<()> {
    if args.speed <= 0.0 {
        return Err(forgekit_core::utils::error::ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: "Speed must be greater than 0".to_string(),
        });
    }

    let spec = JobSpec::VideoSpeed {
        input: args.input.clone(),
        output: args.output.clone(),
        speed: args.speed,
    };

    let result = execute_job(&spec, plan_only)?;
    println!("{}", result);
    Ok(())
}

fn handle_rotate(args: &RotateArgs, plan_only: bool) -> Result<()> {
    if args.degrees != 90 && args.degrees != 180 && args.degrees != 270 {
        return Err(forgekit_core::utils::error::ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: format!("Invalid rotation angle {}. Use 90, 180, or 270.", args.degrees),
        });
    }

    let spec = JobSpec::VideoRotate {
        input: args.input.clone(),
        output: args.output.clone(),
        degrees: args.degrees,
    };

    let result = execute_job(&spec, plan_only)?;
    println!("{}", result);
    Ok(())
}

fn handle_mute(args: &MuteArgs, plan_only: bool) -> Result<()> {
    let spec = JobSpec::VideoMute {
        input: args.input.clone(),
        output: args.output.clone(),
    };

    let result = execute_job(&spec, plan_only)?;
    println!("{}", result);
    Ok(())
}
