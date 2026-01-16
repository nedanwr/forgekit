use clap::{Args, Subcommand};
use forgekit_core::job::executor::execute_job;
use forgekit_core::job::JobSpec;
use forgekit_core::utils::audio::{AudioFormat, LoudnessTarget};
use forgekit_core::utils::error::Result;
use std::path::PathBuf;

#[derive(Subcommand, Clone)]
pub enum AudioCommand {
    /// Convert audio to a different format
    ///
    /// Examples:
    ///   forgekit audio convert song.wav --output song.mp3
    ///   forgekit audio convert song.wav --output song.mp3 --bitrate 320
    ///   forgekit audio convert song.wav -t opus --bitrate 128
    ///
    /// Supported formats: mp3, aac, opus, flac, wav, ogg, m4a
    Convert(ConvertArgs),

    /// Normalize audio loudness
    ///
    /// Examples:
    ///   forgekit audio normalize song.wav --output normalized.wav
    ///   forgekit audio normalize song.wav --output normalized.wav --target streaming
    ///   forgekit audio normalize song.wav --output normalized.wav --lufs -16
    ///
    /// Targets:
    ///   ebu-r128  - Broadcast standard (-23 LUFS)
    ///   streaming - Streaming platforms (-14 LUFS)
    ///   --lufs N  - Custom LUFS target
    Normalize(NormalizeArgs),

    /// Extract audio from video file
    ///
    /// Examples:
    ///   forgekit audio extract video.mp4 --output audio.mp3
    ///   forgekit audio extract video.mp4 --output audio.mp3 --bitrate 192
    ///   forgekit audio extract video.mp4 -t opus --bitrate 128
    ///
    /// Extracts the audio stream from a video file.
    /// Supported output formats: mp3, aac, opus, flac, wav, ogg, m4a
    Extract(ExtractArgs),

    /// Trim audio to a specific time range
    ///
    /// Examples:
    ///   forgekit audio trim song.mp3 --start 0:30 --end 2:00 --output clip.mp3
    ///   forgekit audio trim song.mp3 --start 30 --end 120 --output clip.mp3
    ///   forgekit audio trim podcast.mp3 --start 5:00 --output from_5min.mp3
    ///
    /// Time format: seconds (30, 90.5) or MM:SS (1:30) or HH:MM:SS (1:30:00)
    Trim(TrimArgs),

    /// Join multiple audio files into one
    ///
    /// Examples:
    ///   forgekit audio join intro.mp3 main.mp3 outro.mp3 --output podcast.mp3
    ///   forgekit audio join part1.wav part2.wav --output full.wav
    ///
    /// Files are concatenated in the order specified.
    Join(JoinArgs),

    /// Adjust audio volume/gain
    ///
    /// Examples:
    ///   forgekit audio volume quiet.wav --gain +6dB --output louder.wav
    ///   forgekit audio volume loud.mp3 --gain -3dB --output quieter.mp3
    ///   forgekit audio volume song.wav --gain 6 --output boosted.wav
    ///
    /// Gain is specified in decibels (dB). Positive values boost, negative reduce.
    Volume(VolumeArgs),

    /// Convert stereo audio to mono
    ///
    /// Examples:
    ///   forgekit audio mono stereo.wav --output mono.wav
    ///   forgekit audio mono interview.mp3 --output interview_mono.mp3
    ///
    /// Downmixes stereo channels to a single mono channel.
    Mono(MonoArgs),

    /// Show audio file information
    ///
    /// Examples:
    ///   forgekit audio info song.mp3
    ///
    /// Shows duration, format, bitrate, channels, and sample rate.
    Info(InfoArgs),
}

#[derive(Args, Clone)]
pub struct ConvertArgs {
    /// Input audio file
    #[arg(required = true, help = "Input audio file")]
    pub input: PathBuf,

    /// Output audio file (defaults to input name with new extension in current dir)
    #[arg(short, long, help = "Output audio file")]
    pub output: Option<PathBuf>,

    /// Target format (required if --output not specified)
    #[arg(
        short = 't',
        long,
        help = "Target format: mp3, aac, opus, flac, wav, ogg, m4a"
    )]
    pub to: Option<String>,

    /// Bitrate in kbps (e.g., 128, 192, 320). For lossy formats only
    #[arg(short, long, help = "Bitrate in kbps (e.g., 128, 192, 320)")]
    pub bitrate: Option<u32>,
}

#[derive(Args, Clone)]
pub struct NormalizeArgs {
    /// Input audio file
    #[arg(required = true, help = "Input audio file")]
    pub input: PathBuf,

    /// Output audio file (defaults to input_normalized.ext in current dir)
    #[arg(short, long, help = "Output audio file")]
    pub output: Option<PathBuf>,

    /// Loudness target preset (ebu-r128 or streaming)
    #[arg(
        long,
        default_value = "ebu-r128",
        help = "Target: ebu-r128 (-23 LUFS) or streaming (-14 LUFS)"
    )]
    pub target: String,

    /// Custom LUFS target (overrides --target)
    #[arg(long, help = "Custom LUFS target (e.g., -16)")]
    pub lufs: Option<f32>,
}

#[derive(Args, Clone)]
pub struct ExtractArgs {
    /// Input video file
    #[arg(required = true, help = "Input video file")]
    pub input: PathBuf,

    /// Output audio file (defaults to input name with new extension in current dir)
    #[arg(short, long, help = "Output audio file")]
    pub output: Option<PathBuf>,

    /// Target format (required if --output not specified)
    #[arg(
        short = 't',
        long,
        help = "Target format: mp3, aac, opus, flac, wav, ogg, m4a"
    )]
    pub to: Option<String>,

    /// Bitrate in kbps (e.g., 128, 192, 320). For lossy formats only
    #[arg(short, long, help = "Bitrate in kbps (e.g., 128, 192, 320)")]
    pub bitrate: Option<u32>,
}

#[derive(Args, Clone)]
pub struct TrimArgs {
    /// Input audio file
    #[arg(required = true, help = "Input audio file")]
    pub input: PathBuf,

    /// Output audio file
    #[arg(short, long, required = true, help = "Output audio file")]
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
    /// Input audio files (at least 2)
    #[arg(required = true, num_args = 2.., help = "Input audio files to join")]
    pub inputs: Vec<PathBuf>,

    /// Output audio file
    #[arg(short, long, required = true, help = "Output audio file")]
    pub output: PathBuf,
}

#[derive(Args, Clone)]
pub struct VolumeArgs {
    /// Input audio file
    #[arg(required = true, help = "Input audio file")]
    pub input: PathBuf,

    /// Output audio file
    #[arg(short, long, required = true, help = "Output audio file")]
    pub output: PathBuf,

    /// Gain adjustment in dB (e.g., +6, -3, 6dB, -3dB)
    #[arg(short, long, required = true, help = "Gain in dB (e.g., +6, -3, 6dB)")]
    pub gain: String,
}

#[derive(Args, Clone)]
pub struct MonoArgs {
    /// Input audio file
    #[arg(required = true, help = "Input audio file")]
    pub input: PathBuf,

    /// Output audio file (defaults to input_mono.ext in current dir)
    #[arg(short, long, help = "Output audio file")]
    pub output: Option<PathBuf>,
}

#[derive(Args, Clone)]
pub struct InfoArgs {
    /// Input audio file
    #[arg(required = true, help = "Input audio file")]
    pub input: PathBuf,
}

pub fn handle_audio_command(cmd: &AudioCommand, plan_only: bool, json_output: bool) -> Result<()> {
    match cmd {
        AudioCommand::Convert(args) => handle_convert(args, plan_only),
        AudioCommand::Normalize(args) => handle_normalize(args, plan_only),
        AudioCommand::Extract(args) => handle_extract(args, plan_only),
        AudioCommand::Trim(args) => handle_trim(args, plan_only),
        AudioCommand::Join(args) => handle_join(args, plan_only),
        AudioCommand::Volume(args) => handle_volume(args, plan_only),
        AudioCommand::Mono(args) => handle_mono(args, plan_only),
        AudioCommand::Info(args) => handle_info(args, json_output),
    }
}

fn handle_convert(args: &ConvertArgs, plan_only: bool) -> Result<()> {
    // Determine output format and path
    let (format, output) = if let Some(ref output) = args.output {
        let fmt = AudioFormat::from_path(output).ok_or_else(|| {
            forgekit_core::utils::error::ForgeKitError::InvalidInput {
                path: output.clone(),
                reason: "Cannot determine format from output extension. Use --to to specify."
                    .to_string(),
            }
        })?;
        (fmt, output.clone())
    } else {
        // No output - require --to flag and derive output path
        let to = args.to.as_ref().ok_or_else(|| {
            forgekit_core::utils::error::ForgeKitError::InvalidInput {
                path: PathBuf::new(),
                reason: "Either --output or --to is required".to_string(),
            }
        })?;
        let fmt = to.parse::<AudioFormat>().map_err(|_| {
            forgekit_core::utils::error::ForgeKitError::InvalidInput {
                path: PathBuf::new(),
                reason: format!(
                    "Unknown format '{}'. Supported: {}",
                    to,
                    AudioFormat::supported_extensions()
                ),
            }
        })?;
        let stem = args.input.file_stem().ok_or_else(|| {
            forgekit_core::utils::error::ForgeKitError::InvalidInput {
                path: args.input.clone(),
                reason: "Cannot determine filename from input".to_string(),
            }
        })?;
        let new_name = format!("{}.{}", stem.to_string_lossy(), fmt.extension());
        (fmt, PathBuf::from(new_name))
    };

    let spec = JobSpec::AudioConvert {
        input: args.input.clone(),
        output,
        format,
        bitrate: args.bitrate,
    };

    let result = execute_job(&spec, plan_only)?;
    println!("{}", result);
    Ok(())
}

fn handle_normalize(args: &NormalizeArgs, plan_only: bool) -> Result<()> {
    // Determine output path
    let output = if let Some(ref output) = args.output {
        output.clone()
    } else {
        let stem = args.input.file_stem().ok_or_else(|| {
            forgekit_core::utils::error::ForgeKitError::InvalidInput {
                path: args.input.clone(),
                reason: "Cannot determine filename from input".to_string(),
            }
        })?;
        let ext = args.input.extension().unwrap_or_default();
        let new_name = format!(
            "{}_normalized.{}",
            stem.to_string_lossy(),
            ext.to_string_lossy()
        );
        PathBuf::from(new_name)
    };

    // Determine loudness target
    let target = if let Some(lufs) = args.lufs {
        LoudnessTarget::Custom(lufs)
    } else {
        match args.target.to_lowercase().as_str() {
            "ebu-r128" | "ebu" | "broadcast" => LoudnessTarget::EbuR128,
            "streaming" | "stream" => LoudnessTarget::Streaming,
            _ => {
                return Err(forgekit_core::utils::error::ForgeKitError::InvalidInput {
                    path: PathBuf::new(),
                    reason: format!(
                        "Unknown target '{}'. Use: ebu-r128, streaming, or --lufs for custom",
                        args.target
                    ),
                });
            }
        }
    };

    let spec = JobSpec::AudioNormalize {
        input: args.input.clone(),
        output,
        target,
    };

    let result = execute_job(&spec, plan_only)?;
    println!("{}", result);
    Ok(())
}

fn handle_extract(args: &ExtractArgs, plan_only: bool) -> Result<()> {
    // Determine output format and path
    let (format, output) = if let Some(ref output) = args.output {
        let fmt = AudioFormat::from_path(output).ok_or_else(|| {
            forgekit_core::utils::error::ForgeKitError::InvalidInput {
                path: output.clone(),
                reason: "Cannot determine format from output extension. Use --to to specify."
                    .to_string(),
            }
        })?;
        (fmt, output.clone())
    } else {
        // No output - require --to flag and derive output path
        let to = args.to.as_ref().ok_or_else(|| {
            forgekit_core::utils::error::ForgeKitError::InvalidInput {
                path: PathBuf::new(),
                reason: "Either --output or --to is required".to_string(),
            }
        })?;
        let fmt = to.parse::<AudioFormat>().map_err(|_| {
            forgekit_core::utils::error::ForgeKitError::InvalidInput {
                path: PathBuf::new(),
                reason: format!(
                    "Unknown format '{}'. Supported: {}",
                    to,
                    AudioFormat::supported_extensions()
                ),
            }
        })?;
        let stem = args.input.file_stem().ok_or_else(|| {
            forgekit_core::utils::error::ForgeKitError::InvalidInput {
                path: args.input.clone(),
                reason: "Cannot determine filename from input".to_string(),
            }
        })?;
        let new_name = format!("{}.{}", stem.to_string_lossy(), fmt.extension());
        (fmt, PathBuf::from(new_name))
    };

    let spec = JobSpec::AudioExtract {
        input: args.input.clone(),
        output,
        format,
        bitrate: args.bitrate,
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

    let spec = JobSpec::AudioTrim {
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
    let spec = JobSpec::AudioJoin {
        inputs: args.inputs.clone(),
        output: args.output.clone(),
    };

    let result = execute_job(&spec, plan_only)?;
    println!("{}", result);
    Ok(())
}

fn handle_volume(args: &VolumeArgs, plan_only: bool) -> Result<()> {
    let gain_db = parse_gain(&args.gain)?;

    let spec = JobSpec::AudioVolume {
        input: args.input.clone(),
        output: args.output.clone(),
        gain_db,
    };

    let result = execute_job(&spec, plan_only)?;
    println!("{}", result);
    Ok(())
}

fn handle_mono(args: &MonoArgs, plan_only: bool) -> Result<()> {
    let output = if let Some(ref output) = args.output {
        output.clone()
    } else {
        let stem = args.input.file_stem().ok_or_else(|| {
            forgekit_core::utils::error::ForgeKitError::InvalidInput {
                path: args.input.clone(),
                reason: "Cannot determine filename from input".to_string(),
            }
        })?;
        let ext = args.input.extension().unwrap_or_default();
        let new_name = format!("{}_mono.{}", stem.to_string_lossy(), ext.to_string_lossy());
        PathBuf::from(new_name)
    };

    let spec = JobSpec::AudioMono {
        input: args.input.clone(),
        output,
    };

    let result = execute_job(&spec, plan_only)?;
    println!("{}", result);
    Ok(())
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

/// Parse gain string to decibels.
/// Supports: +6, -3, 6dB, -3dB, +6dB
fn parse_gain(s: &str) -> Result<f64> {
    let s = s.trim().to_lowercase();
    let s = s.strip_suffix("db").unwrap_or(&s);

    s.parse::<f64>().map_err(
        |_| forgekit_core::utils::error::ForgeKitError::InvalidInput {
            path: PathBuf::new(),
            reason: format!("Invalid gain '{}'. Use a number like +6, -3, or 6dB", s),
        },
    )
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
    let file_size_str = if file_size >= 1_000_000 {
        format!("{:.1} MB", file_size as f64 / 1_000_000.0)
    } else if file_size >= 1_000 {
        format!("{:.1} KB", file_size as f64 / 1_000.0)
    } else {
        format!("{} bytes", file_size)
    };

    // Use ffprobe to get audio info
    let (duration, format, bitrate, channels, sample_rate) = if let Ok(output) =
        Command::new("ffprobe")
            .args([
                "-v",
                "quiet",
                "-show_entries",
                "format=duration,format_name,bit_rate:stream=channels,sample_rate",
                "-of",
                "csv=p=0",
            ])
            .arg(&args.input)
            .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.trim().lines().collect();

            // Parse stream info (first line): channels,sample_rate
            let (channels, sample_rate) = if let Some(line) = lines.first() {
                let parts: Vec<&str> = line.split(',').collect();
                (
                    parts.first().unwrap_or(&"?").to_string(),
                    parts.get(1).unwrap_or(&"?").to_string(),
                )
            } else {
                ("?".to_string(), "?".to_string())
            };

            // Parse format info (second line): duration,format_name,bit_rate
            let (duration, format, bitrate) = if let Some(line) = lines.get(1) {
                let parts: Vec<&str> = line.split(',').collect();
                let dur = parts
                    .first()
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(format_duration)
                    .unwrap_or_else(|| "?".to_string());
                let fmt = parts.get(1).unwrap_or(&"?").to_string();
                let br = parts
                    .get(2)
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(|b| format!("{} kbps", b / 1000))
                    .unwrap_or_else(|| "?".to_string());
                (dur, fmt, br)
            } else {
                ("?".to_string(), "?".to_string(), "?".to_string())
            };

            (duration, format, bitrate, channels, sample_rate)
        } else {
            (
                "?".to_string(),
                "?".to_string(),
                "?".to_string(),
                "?".to_string(),
                "?".to_string(),
            )
        }
    } else {
        (
            "?".to_string(),
            "?".to_string(),
            "?".to_string(),
            "?".to_string(),
            "?".to_string(),
        )
    };

    if json_output {
        let info = serde_json::json!({
            "file": args.input.display().to_string(),
            "duration": duration,
            "format": format,
            "bitrate": bitrate,
            "channels": channels.parse::<u32>().unwrap_or(0),
            "sample_rate": sample_rate.parse::<u32>().unwrap_or(0),
            "size_bytes": file_size,
            "size": file_size_str,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&info).unwrap_or_default()
        );
    } else {
        println!("File:        {}", args.input.display());
        println!("Duration:    {}", duration);
        println!("Format:      {}", format);
        println!("Bitrate:     {}", bitrate);
        println!("Channels:    {}", channels);
        println!("Sample Rate: {} Hz", sample_rate);
        println!("Size:        {}", file_size_str);
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
