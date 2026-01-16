//! # Audio Format Utilities
//!
//! Types and helpers for audio format detection and conversion.

use std::path::Path;
use std::str::FromStr;

/// Supported audio formats for conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Mp3,
    Aac,
    Opus,
    Flac,
    Wav,
    Ogg,
    M4a,
}

impl AudioFormat {
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp3" => Some(Self::Mp3),
            "aac" => Some(Self::Aac),
            "opus" => Some(Self::Opus),
            "flac" => Some(Self::Flac),
            "wav" => Some(Self::Wav),
            "ogg" | "oga" => Some(Self::Ogg),
            "m4a" => Some(Self::M4a),
            _ => None,
        }
    }

    /// Get canonical file extension
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
            Self::Aac => "aac",
            Self::Opus => "opus",
            Self::Flac => "flac",
            Self::Wav => "wav",
            Self::Ogg => "ogg",
            Self::M4a => "m4a",
        }
    }

    /// Detect format from file path
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }

    /// Get ffmpeg codec name for this format
    pub fn ffmpeg_codec(&self) -> &'static str {
        match self {
            Self::Mp3 => "libmp3lame",
            Self::Aac => "aac",
            Self::Opus => "libopus",
            Self::Flac => "flac",
            Self::Wav => "pcm_s16le",
            Self::Ogg => "libvorbis",
            Self::M4a => "aac",
        }
    }

    /// Get all supported format extensions as a comma-separated string
    pub fn supported_extensions() -> &'static str {
        "mp3, aac, opus, flac, wav, ogg, m4a"
    }

    /// Check if this format supports bitrate setting
    pub fn supports_bitrate(&self) -> bool {
        matches!(
            self,
            Self::Mp3 | Self::Aac | Self::Opus | Self::Ogg | Self::M4a
        )
    }
}

impl FromStr for AudioFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_extension(s).ok_or_else(|| {
            format!(
                "Unknown audio format: '{}'. Supported: {}",
                s,
                Self::supported_extensions()
            )
        })
    }
}

impl std::fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.extension())
    }
}

/// Loudness normalization target
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoudnessTarget {
    /// EBU R128 broadcast standard (-23 LUFS)
    EbuR128,
    /// Streaming standard (-14 LUFS)
    Streaming,
    /// Custom LUFS target
    Custom(f32),
}

impl LoudnessTarget {
    /// Get the target integrated loudness in LUFS
    pub fn lufs(&self) -> f32 {
        match self {
            Self::EbuR128 => -23.0,
            Self::Streaming => -14.0,
            Self::Custom(lufs) => *lufs,
        }
    }

    /// Get the true peak limit in dBTP
    pub fn true_peak(&self) -> f32 {
        match self {
            Self::EbuR128 => -1.0,
            Self::Streaming => -1.0,
            Self::Custom(_) => -1.0,
        }
    }
}

impl std::fmt::Display for LoudnessTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EbuR128 => write!(f, "EBU R128 (-23 LUFS)"),
            Self::Streaming => write!(f, "Streaming (-14 LUFS)"),
            Self::Custom(lufs) => write!(f, "{} LUFS", lufs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_extension() {
        assert_eq!(AudioFormat::from_extension("mp3"), Some(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_extension("MP3"), Some(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_extension("aac"), Some(AudioFormat::Aac));
        assert_eq!(AudioFormat::from_extension("opus"), Some(AudioFormat::Opus));
        assert_eq!(AudioFormat::from_extension("flac"), Some(AudioFormat::Flac));
        assert_eq!(AudioFormat::from_extension("wav"), Some(AudioFormat::Wav));
        assert_eq!(AudioFormat::from_extension("ogg"), Some(AudioFormat::Ogg));
        assert_eq!(AudioFormat::from_extension("oga"), Some(AudioFormat::Ogg));
        assert_eq!(AudioFormat::from_extension("m4a"), Some(AudioFormat::M4a));
        assert_eq!(AudioFormat::from_extension("xyz"), None);
    }

    #[test]
    fn test_format_extension() {
        assert_eq!(AudioFormat::Mp3.extension(), "mp3");
        assert_eq!(AudioFormat::Opus.extension(), "opus");
        assert_eq!(AudioFormat::Flac.extension(), "flac");
    }

    #[test]
    fn test_format_from_path() {
        assert_eq!(
            AudioFormat::from_path(Path::new("song.mp3")),
            Some(AudioFormat::Mp3)
        );
        assert_eq!(
            AudioFormat::from_path(Path::new("/path/to/audio.opus")),
            Some(AudioFormat::Opus)
        );
        assert_eq!(AudioFormat::from_path(Path::new("file.unknown")), None);
    }

    #[test]
    fn test_format_from_str() {
        assert_eq!("mp3".parse::<AudioFormat>().unwrap(), AudioFormat::Mp3);
        assert_eq!("opus".parse::<AudioFormat>().unwrap(), AudioFormat::Opus);
        assert!("xyz".parse::<AudioFormat>().is_err());
    }

    #[test]
    fn test_ffmpeg_codec() {
        assert_eq!(AudioFormat::Mp3.ffmpeg_codec(), "libmp3lame");
        assert_eq!(AudioFormat::Opus.ffmpeg_codec(), "libopus");
        assert_eq!(AudioFormat::Flac.ffmpeg_codec(), "flac");
    }

    #[test]
    fn test_supports_bitrate() {
        assert!(AudioFormat::Mp3.supports_bitrate());
        assert!(AudioFormat::Opus.supports_bitrate());
        assert!(!AudioFormat::Flac.supports_bitrate());
        assert!(!AudioFormat::Wav.supports_bitrate());
    }

    #[test]
    fn test_loudness_target_lufs() {
        assert_eq!(LoudnessTarget::EbuR128.lufs(), -23.0);
        assert_eq!(LoudnessTarget::Streaming.lufs(), -14.0);
        assert_eq!(LoudnessTarget::Custom(-16.0).lufs(), -16.0);
    }

    #[test]
    fn test_loudness_target_display() {
        assert_eq!(LoudnessTarget::EbuR128.to_string(), "EBU R128 (-23 LUFS)");
        assert_eq!(
            LoudnessTarget::Streaming.to_string(),
            "Streaming (-14 LUFS)"
        );
        assert_eq!(LoudnessTarget::Custom(-16.0).to_string(), "-16 LUFS");
    }
}
