//! # Image Format Utilities
//!
//! Types and helpers for image format detection and conversion.

use std::path::Path;
use std::str::FromStr;

/// Supported image formats for conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Jpeg,
    Png,
    WebP,
    Avif,
    Tiff,
    Gif,
}

impl ImageFormat {
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "png" => Some(Self::Png),
            "webp" => Some(Self::WebP),
            "avif" => Some(Self::Avif),
            "tiff" | "tif" => Some(Self::Tiff),
            "gif" => Some(Self::Gif),
            _ => None,
        }
    }

    /// Get canonical file extension
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::WebP => "webp",
            Self::Avif => "avif",
            Self::Tiff => "tiff",
            Self::Gif => "gif",
        }
    }

    /// Detect format from file path
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }

    /// Get all supported format extensions as a comma-separated string
    pub fn supported_extensions() -> &'static str {
        "jpg, jpeg, png, webp, avif, tiff, tif, gif"
    }
}

impl FromStr for ImageFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_extension(s).ok_or_else(|| {
            format!(
                "Unknown image format: '{}'. Supported: {}",
                s,
                Self::supported_extensions()
            )
        })
    }
}

impl std::fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.extension())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_extension() {
        assert_eq!(ImageFormat::from_extension("jpg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("jpeg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("JPG"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("png"), Some(ImageFormat::Png));
        assert_eq!(ImageFormat::from_extension("webp"), Some(ImageFormat::WebP));
        assert_eq!(ImageFormat::from_extension("avif"), Some(ImageFormat::Avif));
        assert_eq!(ImageFormat::from_extension("tiff"), Some(ImageFormat::Tiff));
        assert_eq!(ImageFormat::from_extension("tif"), Some(ImageFormat::Tiff));
        assert_eq!(ImageFormat::from_extension("gif"), Some(ImageFormat::Gif));
        assert_eq!(ImageFormat::from_extension("xyz"), None);
    }

    #[test]
    fn test_format_extension() {
        assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::WebP.extension(), "webp");
    }

    #[test]
    fn test_format_from_path() {
        assert_eq!(
            ImageFormat::from_path(Path::new("photo.jpg")),
            Some(ImageFormat::Jpeg)
        );
        assert_eq!(
            ImageFormat::from_path(Path::new("/path/to/image.webp")),
            Some(ImageFormat::WebP)
        );
        assert_eq!(ImageFormat::from_path(Path::new("file.unknown")), None);
    }

    #[test]
    fn test_format_from_str() {
        assert_eq!("jpg".parse::<ImageFormat>().unwrap(), ImageFormat::Jpeg);
        assert_eq!("webp".parse::<ImageFormat>().unwrap(), ImageFormat::WebP);
        assert!("xyz".parse::<ImageFormat>().is_err());
    }
}
