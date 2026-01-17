//! # ffmpeg Tool Adapter
//!
//! ffmpeg is a powerful multimedia framework for audio/video conversion.
//! We use the `ffmpeg` CLI for:
//! - Audio format conversion
//! - Bitrate transcoding
//! - Loudness normalization (EBU R128)
//!
//! ## Minimum Version: 5.0+

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::tools::{Tool, ToolConfig, ToolInfo};
use crate::utils::audio::{AudioFormat, LoudnessTarget};
use crate::utils::error::{ForgeKitError, Result};
use crate::utils::platform::ToolInstallHints;

/// ffmpeg tool adapter.
pub struct FfmpegTool;

impl Tool for FfmpegTool {
    fn name(&self) -> &'static str {
        "ffmpeg"
    }

    fn probe(&self, config: &ToolConfig) -> Result<ToolInfo> {
        // Check override path first
        if let Some(ref path) = config.override_path {
            if path.exists() {
                let version = self.version(path)?;
                return Ok(ToolInfo {
                    path: path.clone(),
                    version,
                    available: true,
                });
            }
        }

        // Probe PATH for 'ffmpeg' command
        let which_output = if cfg!(target_os = "windows") {
            Command::new("where").arg("ffmpeg").output()
        } else {
            Command::new("which").arg("ffmpeg").output()
        };

        let path = match which_output {
            Ok(output) if output.status.success() => {
                let path_str = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !path_str.is_empty() {
                    PathBuf::from(path_str)
                } else {
                    PathBuf::from("ffmpeg")
                }
            }
            _ => PathBuf::from("ffmpeg"),
        };

        // Verify it works
        let output = Command::new(&path).arg("-version").output().map_err(|_| {
            ForgeKitError::ToolNotFound {
                tool: "ffmpeg".to_string(),
                hint: ToolInstallHints::for_tool("ffmpeg"),
            }
        })?;

        if !output.status.success() {
            return Err(ForgeKitError::ToolNotFound {
                tool: "ffmpeg".to_string(),
                hint: ToolInstallHints::for_tool("ffmpeg"),
            });
        }

        let version = self.version(&path)?;

        Ok(ToolInfo {
            path,
            version,
            available: true,
        })
    }

    fn version(&self, path: &Path) -> Result<String> {
        let output = Command::new(path)
            .arg("-version")
            .output()
            .map_err(|e| ForgeKitError::Other(anyhow::anyhow!("Failed to run ffmpeg: {}", e)))?;

        if !output.status.success() {
            return Err(ForgeKitError::Other(anyhow::anyhow!(
                "ffmpeg -version failed"
            )));
        }

        // ffmpeg -version outputs something like: "ffmpeg version 6.1 Copyright..."
        let version = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .to_string();

        // Extract just the version part
        let version = version
            .split_whitespace()
            .nth(2)
            .unwrap_or(&version)
            .to_string();

        Ok(version.trim().to_string())
    }
}

impl FfmpegTool {
    /// Convert audio to a different format with optional bitrate.
    ///
    /// Uses `ffmpeg -i input -c:a codec -b:a bitrate output` syntax.
    pub fn convert(
        &self,
        tool_path: &Path,
        input: &Path,
        output: &Path,
        format: &AudioFormat,
        bitrate: Option<u32>,
    ) -> Result<()> {
        let mut cmd = Command::new(tool_path);
        cmd.arg("-y"); // Overwrite output
        cmd.arg("-i").arg(input);
        cmd.arg("-c:a").arg(format.ffmpeg_codec());

        if let Some(br) = bitrate {
            if format.supports_bitrate() {
                cmd.arg("-b:a").arg(format!("{}k", br));
            }
        }

        // For opus in ogg container, we need to specify format
        if matches!(format, AudioFormat::Opus) {
            cmd.arg("-f").arg("opus");
        }

        cmd.arg(output);

        let output_result = cmd
            .output()
            .map_err(|e| ForgeKitError::Other(anyhow::anyhow!("Failed to run ffmpeg: {}", e)))?;

        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "ffmpeg".to_string(),
                stderr: stderr.to_string(),
            });
        }

        Ok(())
    }

    /// Normalize audio loudness using EBU R128 standard.
    ///
    /// Uses two-pass loudnorm filter for accurate normalization.
    pub fn normalize(
        &self,
        tool_path: &Path,
        input: &Path,
        output: &Path,
        target: &LoudnessTarget,
    ) -> Result<()> {
        let lufs = target.lufs();
        let tp = target.true_peak();

        // Single-pass loudnorm with linear normalization
        let loudnorm_filter = format!("loudnorm=I={}:TP={}:LRA=11:print_format=summary", lufs, tp);

        let mut cmd = Command::new(tool_path);
        cmd.arg("-y"); // Overwrite output
        cmd.arg("-i").arg(input);
        cmd.arg("-af").arg(&loudnorm_filter);
        cmd.arg(output);

        let output_result = cmd
            .output()
            .map_err(|e| ForgeKitError::Other(anyhow::anyhow!("Failed to run ffmpeg: {}", e)))?;

        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "ffmpeg".to_string(),
                stderr: stderr.to_string(),
            });
        }

        Ok(())
    }

    /// Extract audio from video file.
    ///
    /// Uses `ffmpeg -i input -vn -c:a codec output` to strip video and keep audio.
    pub fn extract(
        &self,
        tool_path: &Path,
        input: &Path,
        output: &Path,
        format: &AudioFormat,
        bitrate: Option<u32>,
    ) -> Result<()> {
        let mut cmd = Command::new(tool_path);
        cmd.arg("-y"); // Overwrite output
        cmd.arg("-i").arg(input);
        cmd.arg("-vn"); // No video - strip video stream
        cmd.arg("-c:a").arg(format.ffmpeg_codec());

        if let Some(br) = bitrate {
            if format.supports_bitrate() {
                cmd.arg("-b:a").arg(format!("{}k", br));
            }
        }

        // For opus in ogg container, we need to specify format
        if matches!(format, AudioFormat::Opus) {
            cmd.arg("-f").arg("opus");
        }

        cmd.arg(output);

        let output_result = cmd
            .output()
            .map_err(|e| ForgeKitError::Other(anyhow::anyhow!("Failed to run ffmpeg: {}", e)))?;

        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "ffmpeg".to_string(),
                stderr: stderr.to_string(),
            });
        }

        Ok(())
    }

    /// Generate plan string for audio extraction (for --plan flag).
    pub fn plan_extract(
        input: &Path,
        output: &Path,
        format: &AudioFormat,
        bitrate: Option<u32>,
    ) -> String {
        let mut parts = vec![
            "ffmpeg".to_string(),
            "-y".to_string(),
            "-i".to_string(),
            input.display().to_string(),
            "-vn".to_string(),
            "-c:a".to_string(),
            format.ffmpeg_codec().to_string(),
        ];

        if let Some(br) = bitrate {
            if format.supports_bitrate() {
                parts.push("-b:a".to_string());
                parts.push(format!("{}k", br));
            }
        }

        if matches!(format, AudioFormat::Opus) {
            parts.push("-f".to_string());
            parts.push("opus".to_string());
        }

        parts.push(output.display().to_string());
        parts.join(" ")
    }

    /// Generate plan string for audio conversion (for --plan flag).
    pub fn plan_convert(
        input: &Path,
        output: &Path,
        format: &AudioFormat,
        bitrate: Option<u32>,
    ) -> String {
        let mut parts = vec![
            "ffmpeg".to_string(),
            "-y".to_string(),
            "-i".to_string(),
            input.display().to_string(),
            "-c:a".to_string(),
            format.ffmpeg_codec().to_string(),
        ];

        if let Some(br) = bitrate {
            if format.supports_bitrate() {
                parts.push("-b:a".to_string());
                parts.push(format!("{}k", br));
            }
        }

        if matches!(format, AudioFormat::Opus) {
            parts.push("-f".to_string());
            parts.push("opus".to_string());
        }

        parts.push(output.display().to_string());
        parts.join(" ")
    }

    /// Generate plan string for audio normalization (for --plan flag).
    pub fn plan_normalize(input: &Path, output: &Path, target: &LoudnessTarget) -> String {
        let lufs = target.lufs();
        let tp = target.true_peak();

        format!(
            "ffmpeg -y -i {} -af \"loudnorm=I={}:TP={}:LRA=11:print_format=summary\" {}",
            input.display(),
            lufs,
            tp,
            output.display()
        )
    }

    /// Trim audio to a specific time range.
    ///
    /// Uses `-ss` for start time and `-to` for end time.
    pub fn trim(
        &self,
        tool_path: &Path,
        input: &Path,
        output: &Path,
        start: Option<f64>,
        end: Option<f64>,
    ) -> Result<()> {
        let mut cmd = Command::new(tool_path);
        cmd.arg("-y"); // Overwrite output

        // -ss before -i for fast seeking
        if let Some(s) = start {
            cmd.arg("-ss").arg(format!("{}", s));
        }

        cmd.arg("-i").arg(input);

        // -to after -i for accurate end time (relative to start if -ss used before -i)
        if let Some(e) = end {
            if let Some(s) = start {
                // Duration from start
                cmd.arg("-t").arg(format!("{}", e - s));
            } else {
                cmd.arg("-to").arg(format!("{}", e));
            }
        }

        cmd.arg("-c").arg("copy"); // Stream copy for speed
        cmd.arg(output);

        let output_result = cmd
            .output()
            .map_err(|e| ForgeKitError::Other(anyhow::anyhow!("Failed to run ffmpeg: {}", e)))?;

        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "ffmpeg".to_string(),
                stderr: stderr.to_string(),
            });
        }

        Ok(())
    }

    /// Generate plan string for audio trimming (for --plan flag).
    pub fn plan_trim(input: &Path, output: &Path, start: Option<f64>, end: Option<f64>) -> String {
        let mut parts = vec!["ffmpeg".to_string(), "-y".to_string()];

        if let Some(s) = start {
            parts.push("-ss".to_string());
            parts.push(format!("{}", s));
        }

        parts.push("-i".to_string());
        parts.push(input.display().to_string());

        if let Some(e) = end {
            if let Some(s) = start {
                parts.push("-t".to_string());
                parts.push(format!("{}", e - s));
            } else {
                parts.push("-to".to_string());
                parts.push(format!("{}", e));
            }
        }

        parts.push("-c".to_string());
        parts.push("copy".to_string());
        parts.push(output.display().to_string());

        parts.join(" ")
    }

    /// Join multiple audio files into one.
    ///
    /// Uses ffmpeg concat demuxer with a temporary file list.
    pub fn join(&self, tool_path: &Path, inputs: &[PathBuf], output: &Path) -> Result<()> {
        use std::io::Write;

        // Create temporary file with list of inputs
        let temp_dir = std::env::temp_dir();
        let list_file = temp_dir.join(format!("ffmpeg_concat_{}.txt", std::process::id()));

        {
            let mut file = std::fs::File::create(&list_file).map_err(|e| {
                ForgeKitError::Other(anyhow::anyhow!("Failed to create concat list: {}", e))
            })?;

            for input in inputs {
                // Escape single quotes in paths
                let escaped = input.display().to_string().replace('\'', "'\\''");
                writeln!(file, "file '{}'", escaped).map_err(|e| {
                    ForgeKitError::Other(anyhow::anyhow!("Failed to write concat list: {}", e))
                })?;
            }
        }

        let mut cmd = Command::new(tool_path);
        cmd.arg("-y");
        cmd.arg("-f").arg("concat");
        cmd.arg("-safe").arg("0"); // Allow absolute paths
        cmd.arg("-i").arg(&list_file);
        cmd.arg("-c").arg("copy"); // Stream copy
        cmd.arg(output);

        let output_result = cmd
            .output()
            .map_err(|e| ForgeKitError::Other(anyhow::anyhow!("Failed to run ffmpeg: {}", e)))?;

        // Clean up temp file
        let _ = std::fs::remove_file(&list_file);

        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "ffmpeg".to_string(),
                stderr: stderr.to_string(),
            });
        }

        Ok(())
    }

    /// Generate plan string for audio joining (for --plan flag).
    pub fn plan_join(inputs: &[PathBuf], output: &Path) -> String {
        let input_list: Vec<String> = inputs.iter().map(|p| p.display().to_string()).collect();
        format!(
            "ffmpeg -y -f concat -safe 0 -i <list: {}> -c copy {}",
            input_list.join(", "),
            output.display()
        )
    }

    /// Adjust audio volume/gain.
    ///
    /// Uses ffmpeg volume filter with dB adjustment.
    pub fn volume(
        &self,
        tool_path: &Path,
        input: &Path,
        output: &Path,
        gain_db: f64,
    ) -> Result<()> {
        let mut cmd = Command::new(tool_path);
        cmd.arg("-y");
        cmd.arg("-i").arg(input);
        cmd.arg("-af").arg(format!("volume={}dB", gain_db));
        cmd.arg(output);

        let output_result = cmd
            .output()
            .map_err(|e| ForgeKitError::Other(anyhow::anyhow!("Failed to run ffmpeg: {}", e)))?;

        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "ffmpeg".to_string(),
                stderr: stderr.to_string(),
            });
        }

        Ok(())
    }

    /// Generate plan string for volume adjustment (for --plan flag).
    pub fn plan_volume(input: &Path, output: &Path, gain_db: f64) -> String {
        format!(
            "ffmpeg -y -i {} -af \"volume={}dB\" {}",
            input.display(),
            gain_db,
            output.display()
        )
    }

    /// Convert stereo audio to mono.
    ///
    /// Uses ffmpeg pan filter to downmix to mono.
    pub fn mono(&self, tool_path: &Path, input: &Path, output: &Path) -> Result<()> {
        let mut cmd = Command::new(tool_path);
        cmd.arg("-y");
        cmd.arg("-i").arg(input);
        cmd.arg("-ac").arg("1"); // Set audio channels to 1 (mono)
        cmd.arg(output);

        let output_result = cmd
            .output()
            .map_err(|e| ForgeKitError::Other(anyhow::anyhow!("Failed to run ffmpeg: {}", e)))?;

        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "ffmpeg".to_string(),
                stderr: stderr.to_string(),
            });
        }

        Ok(())
    }

    /// Generate plan string for mono conversion (for --plan flag).
    pub fn plan_mono(input: &Path, output: &Path) -> String {
        format!(
            "ffmpeg -y -i {} -ac 1 {}",
            input.display(),
            output.display()
        )
    }

    // ========== Video Operations ==========

    /// Transcode video to H.264 using software x264 encoder.
    ///
    /// Uses CRF (Constant Rate Factor) for quality control.
    /// CRF 0 = lossless, CRF 51 = worst quality. Default is 23.
    #[allow(clippy::too_many_arguments)]
    pub fn transcode(
        &self,
        tool_path: &Path,
        input: &Path,
        output: &Path,
        crf: u8,
        preset: &str,
        scale: Option<(i32, i32)>,
        copy_audio: bool,
    ) -> Result<()> {
        let mut cmd = Command::new(tool_path);
        cmd.arg("-y"); // Overwrite output
        cmd.arg("-i").arg(input);

        // Video codec: H.264 with x264
        cmd.arg("-c:v").arg("libx264");
        cmd.arg("-crf").arg(crf.to_string());
        cmd.arg("-preset").arg(preset);

        // Scale filter if specified
        if let Some((width, height)) = scale {
            let scale_filter = if height == -1 {
                // Preserve aspect ratio, scale to width
                format!("scale={}:-2", width) // -2 ensures divisible by 2
            } else {
                format!("scale={}:{}", width, height)
            };
            cmd.arg("-vf").arg(scale_filter);
        }

        // Audio handling
        if copy_audio {
            cmd.arg("-c:a").arg("copy");
        } else {
            cmd.arg("-c:a").arg("aac");
            cmd.arg("-b:a").arg("128k");
        }

        cmd.arg(output);

        let output_result = cmd
            .output()
            .map_err(|e| ForgeKitError::Other(anyhow::anyhow!("Failed to run ffmpeg: {}", e)))?;

        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "ffmpeg".to_string(),
                stderr: stderr.to_string(),
            });
        }

        Ok(())
    }

    /// Generate plan string for video transcode (for --plan flag).
    pub fn plan_transcode(
        input: &Path,
        output: &Path,
        crf: u8,
        preset: &str,
        scale: Option<(i32, i32)>,
        copy_audio: bool,
    ) -> String {
        let mut parts = vec![
            "ffmpeg".to_string(),
            "-y".to_string(),
            "-i".to_string(),
            input.display().to_string(),
            "-c:v".to_string(),
            "libx264".to_string(),
            "-crf".to_string(),
            crf.to_string(),
            "-preset".to_string(),
            preset.to_string(),
        ];

        if let Some((width, height)) = scale {
            let scale_filter = if height == -1 {
                format!("scale={}:-2", width)
            } else {
                format!("scale={}:{}", width, height)
            };
            parts.push("-vf".to_string());
            parts.push(scale_filter);
        }

        if copy_audio {
            parts.push("-c:a".to_string());
            parts.push("copy".to_string());
        } else {
            parts.push("-c:a".to_string());
            parts.push("aac".to_string());
            parts.push("-b:a".to_string());
            parts.push("128k".to_string());
        }

        parts.push(output.display().to_string());
        parts.join(" ")
    }

    /// Trim video to a specific time range using stream copy (fast, no re-encoding).
    pub fn video_trim(
        &self,
        tool_path: &Path,
        input: &Path,
        output: &Path,
        start: Option<f64>,
        end: Option<f64>,
    ) -> Result<()> {
        let mut cmd = Command::new(tool_path);
        cmd.arg("-y"); // Overwrite output

        // Add start time before input for fast seeking
        if let Some(start) = start {
            cmd.arg("-ss").arg(format!("{:.3}", start));
        }

        cmd.arg("-i").arg(input);

        // Add end time / duration
        if let Some(end) = end {
            if let Some(start) = start {
                // Duration from start
                let duration = end - start;
                cmd.arg("-t").arg(format!("{:.3}", duration));
            } else {
                // End time from beginning
                cmd.arg("-to").arg(format!("{:.3}", end));
            }
        }

        // Use stream copy for fast trimming (no re-encoding)
        cmd.arg("-c").arg("copy");
        cmd.arg(output);

        let output_result = cmd.output()?;
        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "ffmpeg".to_string(),
                stderr: stderr.to_string(),
            });
        }
        Ok(())
    }

    /// Generate plan string for video trim.
    pub fn plan_video_trim(
        input: &Path,
        output: &Path,
        start: Option<f64>,
        end: Option<f64>,
    ) -> String {
        let mut parts = vec!["ffmpeg".to_string(), "-y".to_string()];

        if let Some(start) = start {
            parts.push("-ss".to_string());
            parts.push(format!("{:.3}", start));
        }

        parts.push("-i".to_string());
        parts.push(input.display().to_string());

        if let Some(end) = end {
            if let Some(start) = start {
                let duration = end - start;
                parts.push("-t".to_string());
                parts.push(format!("{:.3}", duration));
            } else {
                parts.push("-to".to_string());
                parts.push(format!("{:.3}", end));
            }
        }

        parts.push("-c".to_string());
        parts.push("copy".to_string());
        parts.push(output.display().to_string());
        parts.join(" ")
    }

    /// Join multiple video files using concat demuxer.
    /// All inputs must have the same codec/resolution/frame rate.
    pub fn video_join(&self, tool_path: &Path, inputs: &[PathBuf], output: &Path) -> Result<()> {
        use std::io::Write;

        // Create concat file list
        let temp_dir = std::env::temp_dir();
        let concat_file = temp_dir.join(format!("forgekit_concat_{}.txt", uuid::Uuid::new_v4()));

        {
            let mut file = std::fs::File::create(&concat_file)?;
            for input in inputs {
                writeln!(file, "file '{}'", input.canonicalize()?.display())?;
            }
        }

        let mut cmd = Command::new(tool_path);
        cmd.arg("-y")
            .arg("-f")
            .arg("concat")
            .arg("-safe")
            .arg("0")
            .arg("-i")
            .arg(&concat_file)
            .arg("-c")
            .arg("copy")
            .arg(output);

        let output_result = cmd.output()?;

        // Clean up concat file
        let _ = std::fs::remove_file(&concat_file);

        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "ffmpeg".to_string(),
                stderr: stderr.to_string(),
            });
        }
        Ok(())
    }

    /// Generate plan string for video join.
    pub fn plan_video_join(inputs: &[PathBuf], output: &Path) -> String {
        let files_str = inputs
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "ffmpeg -y -f concat -safe 0 -i <concat_list: {}> -c copy {}",
            files_str,
            output.display()
        )
    }

    /// Extract a thumbnail frame from video at a specific timestamp.
    pub fn video_thumbnail(
        &self,
        tool_path: &Path,
        input: &Path,
        output: &Path,
        timestamp: f64,
    ) -> Result<()> {
        let mut cmd = Command::new(tool_path);
        cmd.arg("-y")
            .arg("-ss")
            .arg(format!("{:.3}", timestamp))
            .arg("-i")
            .arg(input)
            .arg("-frames:v")
            .arg("1")
            .arg("-q:v")
            .arg("2")
            .arg(output);

        let output_result = cmd.output()?;
        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "ffmpeg".to_string(),
                stderr: stderr.to_string(),
            });
        }
        Ok(())
    }

    /// Generate plan string for video thumbnail.
    pub fn plan_video_thumbnail(input: &Path, output: &Path, timestamp: f64) -> String {
        format!(
            "ffmpeg -y -ss {:.3} -i {} -frames:v 1 -q:v 2 {}",
            timestamp,
            input.display(),
            output.display()
        )
    }

    /// Convert video to a different format.
    /// For GIF: uses palette generation for quality.
    /// For other formats: uses stream copy when possible.
    #[allow(clippy::too_many_arguments)]
    pub fn video_convert(
        &self,
        tool_path: &Path,
        input: &Path,
        output: &Path,
        format: &str,
        start: Option<f64>,
        duration: Option<f64>,
        width: Option<u32>,
        fps: Option<u32>,
    ) -> Result<()> {
        let mut cmd = Command::new(tool_path);
        cmd.arg("-y");

        if let Some(s) = start {
            cmd.arg("-ss").arg(format!("{:.3}", s));
        }

        cmd.arg("-i").arg(input);

        if let Some(d) = duration {
            cmd.arg("-t").arg(format!("{:.3}", d));
        }

        if format == "gif" {
            // GIF needs special palette generation for quality
            let fps_val = fps.unwrap_or(10);
            let filter = if let Some(w) = width {
                format!("fps={},scale={}:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse", fps_val, w)
            } else {
                format!(
                    "fps={},split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
                    fps_val
                )
            };
            cmd.arg("-filter_complex").arg(&filter);
        } else {
            // For other formats, use stream copy when possible
            cmd.arg("-c").arg("copy");
        }

        cmd.arg(output);

        let output_result = cmd.output()?;
        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "ffmpeg".to_string(),
                stderr: stderr.to_string(),
            });
        }
        Ok(())
    }

    /// Generate plan string for video format conversion.
    #[allow(clippy::too_many_arguments)]
    pub fn plan_video_convert(
        input: &Path,
        output: &Path,
        format: &str,
        start: Option<f64>,
        duration: Option<f64>,
        width: Option<u32>,
        fps: Option<u32>,
    ) -> String {
        let mut parts = vec!["ffmpeg".to_string(), "-y".to_string()];

        if let Some(s) = start {
            parts.push("-ss".to_string());
            parts.push(format!("{:.3}", s));
        }

        parts.push("-i".to_string());
        parts.push(input.display().to_string());

        if let Some(d) = duration {
            parts.push("-t".to_string());
            parts.push(format!("{:.3}", d));
        }

        if format == "gif" {
            let fps_val = fps.unwrap_or(10);
            let filter = if let Some(w) = width {
                format!("fps={},scale={}:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse", fps_val, w)
            } else {
                format!(
                    "fps={},split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
                    fps_val
                )
            };
            parts.push("-filter_complex".to_string());
            parts.push(format!("\"{}\"", filter));
        } else {
            parts.push("-c".to_string());
            parts.push("copy".to_string());
        }

        parts.push(output.display().to_string());
        parts.join(" ")
    }

    /// Change video playback speed.
    pub fn video_speed(
        &self,
        tool_path: &Path,
        input: &Path,
        output: &Path,
        speed: f64,
    ) -> Result<()> {
        let mut cmd = Command::new(tool_path);
        cmd.arg("-y").arg("-i").arg(input);

        // Video speed: setpts=PTS/speed (e.g., 2x = PTS/2, 0.5x = PTS/0.5)
        // Audio speed: atempo filter (only supports 0.5 to 2.0, chain for more)
        let video_filter = format!("setpts=PTS/{}", speed);

        // Audio tempo - chain atempo filters if needed for extreme speeds
        let audio_filters = build_atempo_chain(speed);

        cmd.arg("-filter:v").arg(&video_filter);

        if !audio_filters.is_empty() {
            cmd.arg("-filter:a").arg(&audio_filters);
        } else {
            cmd.arg("-an"); // No audio if speed is too extreme
        }

        cmd.arg(output);

        let output_result = cmd.output()?;
        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "ffmpeg".to_string(),
                stderr: stderr.to_string(),
            });
        }
        Ok(())
    }

    /// Generate plan string for video speed change.
    pub fn plan_video_speed(input: &Path, output: &Path, speed: f64) -> String {
        let video_filter = format!("setpts=PTS/{}", speed);
        let audio_filters = build_atempo_chain(speed);

        if !audio_filters.is_empty() {
            format!(
                "ffmpeg -y -i {} -filter:v {} -filter:a {} {}",
                input.display(),
                video_filter,
                audio_filters,
                output.display()
            )
        } else {
            format!(
                "ffmpeg -y -i {} -filter:v {} -an {}",
                input.display(),
                video_filter,
                output.display()
            )
        }
    }

    /// Rotate video by specified degrees.
    pub fn video_rotate(
        &self,
        tool_path: &Path,
        input: &Path,
        output: &Path,
        degrees: u32,
    ) -> Result<()> {
        let transpose = match degrees {
            90 => "transpose=1",              // 90 clockwise
            180 => "transpose=1,transpose=1", // 180
            270 => "transpose=2",             // 90 counter-clockwise (270 clockwise)
            _ => {
                return Err(ForgeKitError::InvalidInput {
                    path: input.to_path_buf(),
                    reason: format!("Invalid rotation angle {}. Use 90, 180, or 270.", degrees),
                })
            }
        };

        let mut cmd = Command::new(tool_path);
        cmd.arg("-y")
            .arg("-i")
            .arg(input)
            .arg("-vf")
            .arg(transpose)
            .arg("-c:a")
            .arg("copy")
            .arg(output);

        let output_result = cmd.output()?;
        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "ffmpeg".to_string(),
                stderr: stderr.to_string(),
            });
        }
        Ok(())
    }

    /// Generate plan string for video rotation.
    pub fn plan_video_rotate(input: &Path, output: &Path, degrees: u32) -> String {
        let transpose = match degrees {
            90 => "transpose=1",
            180 => "transpose=1,transpose=1",
            270 => "transpose=2",
            _ => "transpose=1",
        };
        format!(
            "ffmpeg -y -i {} -vf {} -c:a copy {}",
            input.display(),
            transpose,
            output.display()
        )
    }

    /// Remove audio track from video.
    pub fn video_mute(&self, tool_path: &Path, input: &Path, output: &Path) -> Result<()> {
        let mut cmd = Command::new(tool_path);
        cmd.arg("-y")
            .arg("-i")
            .arg(input)
            .arg("-c:v")
            .arg("copy")
            .arg("-an")
            .arg(output);

        let output_result = cmd.output()?;
        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "ffmpeg".to_string(),
                stderr: stderr.to_string(),
            });
        }
        Ok(())
    }

    /// Generate plan string for video mute.
    pub fn plan_video_mute(input: &Path, output: &Path) -> String {
        format!(
            "ffmpeg -y -i {} -c:v copy -an {}",
            input.display(),
            output.display()
        )
    }

    /// Stitch image sequence into video or GIF.
    pub fn video_stitch(
        &self,
        tool_path: &Path,
        inputs: &[PathBuf],
        output: &Path,
        format: &str,
        fps: u32,
        width: Option<u32>,
    ) -> Result<()> {
        use std::io::Write;

        // Create a concat file listing all images
        let temp_dir = std::env::temp_dir();
        let concat_file = temp_dir.join(format!("forgekit_stitch_{}.txt", uuid::Uuid::new_v4()));

        {
            let mut file = std::fs::File::create(&concat_file)?;
            for input in inputs {
                // Each image shown for 1/fps duration
                writeln!(file, "file '{}'", input.canonicalize()?.display())?;
                writeln!(file, "duration {:.6}", 1.0 / fps as f64)?;
            }
            // Repeat last frame to avoid ffmpeg duration issue
            if let Some(last) = inputs.last() {
                writeln!(file, "file '{}'", last.canonicalize()?.display())?;
            }
        }

        let mut cmd = Command::new(tool_path);
        cmd.arg("-y")
            .arg("-f")
            .arg("concat")
            .arg("-safe")
            .arg("0")
            .arg("-i")
            .arg(&concat_file);

        if format == "gif" {
            // GIF with palette generation
            let filter = if let Some(w) = width {
                format!("fps={},scale={}:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse", fps, w)
            } else {
                format!(
                    "fps={},split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
                    fps
                )
            };
            cmd.arg("-filter_complex").arg(&filter);
        } else {
            // Video output
            cmd.arg("-c:v")
                .arg("libx264")
                .arg("-pix_fmt")
                .arg("yuv420p")
                .arg("-r")
                .arg(fps.to_string());
        }

        cmd.arg(output);

        let output_result = cmd.output()?;

        // Clean up concat file
        let _ = std::fs::remove_file(&concat_file);

        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(ForgeKitError::ProcessingFailed {
                tool: "ffmpeg".to_string(),
                stderr: stderr.to_string(),
            });
        }
        Ok(())
    }

    /// Generate plan string for video stitch.
    pub fn plan_video_stitch(
        inputs: &[PathBuf],
        output: &Path,
        format: &str,
        fps: u32,
        width: Option<u32>,
    ) -> String {
        let files_preview = if inputs.len() <= 3 {
            inputs
                .iter()
                .map(|p| {
                    p.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                })
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            format!(
                "{}, ... ({} files)",
                inputs[0].file_name().unwrap_or_default().to_string_lossy(),
                inputs.len()
            )
        };

        if format == "gif" {
            let filter = if let Some(w) = width {
                format!("fps={},scale={}:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse", fps, w)
            } else {
                format!(
                    "fps={},split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
                    fps
                )
            };
            format!(
                "ffmpeg -y -f concat -i <{}> -filter_complex \"{}\" {}",
                files_preview,
                filter,
                output.display()
            )
        } else {
            format!(
                "ffmpeg -y -f concat -i <{}> -c:v libx264 -pix_fmt yuv420p -r {} {}",
                files_preview,
                fps,
                output.display()
            )
        }
    }
}

/// Build atempo filter chain for speed changes.
/// atempo only supports 0.5 to 2.0, so we chain multiple for extreme speeds.
fn build_atempo_chain(speed: f64) -> String {
    if !(0.5..=2.0).contains(&speed) {
        // For extreme speeds, chain atempo filters
        let mut filters = Vec::new();
        let mut remaining = speed;

        while remaining > 2.0 {
            filters.push("atempo=2.0".to_string());
            remaining /= 2.0;
        }
        while remaining < 0.5 {
            filters.push("atempo=0.5".to_string());
            remaining /= 0.5;
        }

        if (remaining - 1.0).abs() > 0.01 {
            filters.push(format!("atempo={:.3}", remaining));
        }

        if filters.is_empty() {
            String::new()
        } else {
            filters.join(",")
        }
    } else if (speed - 1.0).abs() < 0.01 {
        String::new() // No change needed
    } else {
        format!("atempo={:.3}", speed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffmpeg_name() {
        let tool = FfmpegTool;
        assert_eq!(tool.name(), "ffmpeg");
    }

    #[test]
    fn test_ffmpeg_probe() {
        let tool = FfmpegTool;
        let config = ToolConfig::default();

        // This test will pass if ffmpeg is installed, skip otherwise
        if let Ok(info) = tool.probe(&config) {
            assert!(info.available);
            assert!(!info.version.is_empty());
        }
    }

    #[test]
    fn test_plan_convert_mp3() {
        let input = PathBuf::from("audio.wav");
        let output = PathBuf::from("audio.mp3");

        let plan = FfmpegTool::plan_convert(&input, &output, &AudioFormat::Mp3, Some(192));

        assert!(plan.contains("ffmpeg"));
        assert!(plan.contains("-i audio.wav"));
        assert!(plan.contains("-c:a libmp3lame"));
        assert!(plan.contains("-b:a 192k"));
        assert!(plan.contains("audio.mp3"));
    }

    #[test]
    fn test_plan_convert_opus() {
        let input = PathBuf::from("audio.wav");
        let output = PathBuf::from("audio.opus");

        let plan = FfmpegTool::plan_convert(&input, &output, &AudioFormat::Opus, Some(128));

        assert!(plan.contains("ffmpeg"));
        assert!(plan.contains("-c:a libopus"));
        assert!(plan.contains("-b:a 128k"));
        assert!(plan.contains("-f opus"));
    }

    #[test]
    fn test_plan_convert_flac_no_bitrate() {
        let input = PathBuf::from("audio.wav");
        let output = PathBuf::from("audio.flac");

        let plan = FfmpegTool::plan_convert(&input, &output, &AudioFormat::Flac, Some(320));

        // FLAC doesn't support bitrate, so it shouldn't be in the plan
        assert!(plan.contains("-c:a flac"));
        assert!(!plan.contains("-b:a"));
    }

    #[test]
    fn test_plan_normalize_ebu_r128() {
        let input = PathBuf::from("audio.wav");
        let output = PathBuf::from("audio_normalized.wav");

        let plan = FfmpegTool::plan_normalize(&input, &output, &LoudnessTarget::EbuR128);

        assert!(plan.contains("ffmpeg"));
        assert!(plan.contains("-af"));
        assert!(plan.contains("loudnorm"));
        assert!(plan.contains("I=-23"));
        assert!(plan.contains("TP=-1"));
    }

    #[test]
    fn test_plan_normalize_streaming() {
        let input = PathBuf::from("audio.wav");
        let output = PathBuf::from("audio_normalized.wav");

        let plan = FfmpegTool::plan_normalize(&input, &output, &LoudnessTarget::Streaming);

        assert!(plan.contains("I=-14"));
    }

    #[test]
    fn test_plan_normalize_custom() {
        let input = PathBuf::from("audio.wav");
        let output = PathBuf::from("audio_normalized.wav");

        let plan = FfmpegTool::plan_normalize(&input, &output, &LoudnessTarget::Custom(-16.0));

        assert!(plan.contains("I=-16"));
    }

    #[test]
    fn test_plan_extract_mp3() {
        let input = PathBuf::from("video.mp4");
        let output = PathBuf::from("audio.mp3");

        let plan = FfmpegTool::plan_extract(&input, &output, &AudioFormat::Mp3, Some(192));

        assert!(plan.contains("ffmpeg"));
        assert!(plan.contains("-i video.mp4"));
        assert!(plan.contains("-vn"));
        assert!(plan.contains("-c:a libmp3lame"));
        assert!(plan.contains("-b:a 192k"));
        assert!(plan.contains("audio.mp3"));
    }

    #[test]
    fn test_plan_extract_opus() {
        let input = PathBuf::from("video.mkv");
        let output = PathBuf::from("audio.opus");

        let plan = FfmpegTool::plan_extract(&input, &output, &AudioFormat::Opus, Some(128));

        assert!(plan.contains("-vn"));
        assert!(plan.contains("-c:a libopus"));
        assert!(plan.contains("-f opus"));
    }

    #[test]
    fn test_plan_extract_flac_no_bitrate() {
        let input = PathBuf::from("video.mov");
        let output = PathBuf::from("audio.flac");

        // FLAC doesn't support bitrate
        let plan = FfmpegTool::plan_extract(&input, &output, &AudioFormat::Flac, Some(320));

        assert!(plan.contains("-vn"));
        assert!(plan.contains("-c:a flac"));
        assert!(!plan.contains("-b:a"));
    }

    #[test]
    fn test_plan_trim() {
        let input = PathBuf::from("song.mp3");
        let output = PathBuf::from("clip.mp3");

        let plan = FfmpegTool::plan_trim(&input, &output, Some(30.0), Some(120.0));

        assert!(plan.contains("ffmpeg"));
        assert!(plan.contains("-ss 30"));
        assert!(plan.contains("-t 90")); // duration = 120 - 30
        assert!(plan.contains("-c copy"));
    }

    #[test]
    fn test_plan_trim_start_only() {
        let input = PathBuf::from("song.mp3");
        let output = PathBuf::from("clip.mp3");

        let plan = FfmpegTool::plan_trim(&input, &output, Some(60.0), None);

        assert!(plan.contains("-ss 60"));
        assert!(!plan.contains("-t "));
    }

    #[test]
    fn test_plan_trim_end_only() {
        let input = PathBuf::from("song.mp3");
        let output = PathBuf::from("clip.mp3");

        let plan = FfmpegTool::plan_trim(&input, &output, None, Some(90.0));

        assert!(!plan.contains("-ss"));
        assert!(plan.contains("-to 90"));
    }

    #[test]
    fn test_plan_join() {
        let inputs = vec![PathBuf::from("part1.mp3"), PathBuf::from("part2.mp3")];
        let output = PathBuf::from("joined.mp3");

        let plan = FfmpegTool::plan_join(&inputs, &output);

        assert!(plan.contains("ffmpeg"));
        assert!(plan.contains("-f concat"));
        assert!(plan.contains("-c copy"));
        assert!(plan.contains("part1.mp3"));
        assert!(plan.contains("part2.mp3"));
    }

    #[test]
    fn test_plan_volume_positive() {
        let input = PathBuf::from("quiet.wav");
        let output = PathBuf::from("louder.wav");

        let plan = FfmpegTool::plan_volume(&input, &output, 6.0);

        assert!(plan.contains("ffmpeg"));
        assert!(plan.contains("-af"));
        assert!(plan.contains("volume=6dB"));
    }

    #[test]
    fn test_plan_volume_negative() {
        let input = PathBuf::from("loud.wav");
        let output = PathBuf::from("quieter.wav");

        let plan = FfmpegTool::plan_volume(&input, &output, -3.0);

        assert!(plan.contains("volume=-3dB"));
    }

    #[test]
    fn test_plan_mono() {
        let input = PathBuf::from("stereo.wav");
        let output = PathBuf::from("mono.wav");

        let plan = FfmpegTool::plan_mono(&input, &output);

        assert!(plan.contains("ffmpeg"));
        assert!(plan.contains("-ac 1"));
    }
}
