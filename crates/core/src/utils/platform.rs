//! # Platform Detection and Install Hints
//!
//! Detects the current platform and provides platform-specific installation
//! instructions for external tools. This enables `check-deps` and error messages
//! to show the correct install commands for each OS.

/// Operating system platform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// macOS (Darwin)
    MacOS,
    /// Windows
    Windows,
    /// Linux (various distributions)
    Linux,
    /// Unknown/unsupported platform
    Unknown,
}

/// Detect the current platform
pub fn detect_platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::MacOS
    } else if cfg!(target_os = "windows") {
        Platform::Windows
    } else if cfg!(target_os = "linux") {
        Platform::Linux
    } else {
        Platform::Unknown
    }
}

/// Linux distribution detection (if possible)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxDistro {
    /// Debian/Ubuntu (apt-based)
    Debian,
    /// Fedora/RHEL/CentOS (dnf/yum-based)
    Fedora,
    /// Arch Linux (pacman-based)
    Arch,
    /// Unknown/other
    Unknown,
}

/// Detect Linux distribution by checking common files
pub fn detect_linux_distro() -> LinuxDistro {
    if std::path::Path::new("/etc/debian_version").exists() {
        LinuxDistro::Debian
    } else if std::path::Path::new("/etc/fedora-release").exists()
        || std::path::Path::new("/etc/redhat-release").exists()
    {
        LinuxDistro::Fedora
    } else if std::path::Path::new("/etc/arch-release").exists() {
        LinuxDistro::Arch
    } else {
        LinuxDistro::Unknown
    }
}

impl Platform {
    /// Get install instructions for a tool on this platform
    pub fn install_hint(&self, tool: &str) -> String {
        match self {
            Platform::MacOS => format!("brew install {}", tool),
            Platform::Windows => format!("winget install {}", tool),
            Platform::Linux => {
                let distro = detect_linux_distro();
                match distro {
                    LinuxDistro::Debian => format!("sudo apt install {}", tool),
                    LinuxDistro::Fedora => format!("sudo dnf install {}", tool),
                    LinuxDistro::Arch => format!("sudo pacman -S {}", tool),
                    LinuxDistro::Unknown => {
                        format!("Install {} using your system's package manager", tool)
                    }
                }
            }
            Platform::Unknown => format!("Install {} using your system's package manager", tool),
        }
    }

    /// Get install instructions for multiple tools
    pub fn install_hints(&self, tools: &[&str]) -> String {
        match self {
            Platform::MacOS => format!("brew install {}", tools.join(" ")),
            Platform::Windows => {
                // Windows winget needs separate commands or a manifest
                tools
                    .iter()
                    .map(|tool| format!("winget install {}", tool))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            Platform::Linux => {
                let distro = detect_linux_distro();
                match distro {
                    LinuxDistro::Debian => format!("sudo apt install {}", tools.join(" ")),
                    LinuxDistro::Fedora => format!("sudo dnf install {}", tools.join(" ")),
                    LinuxDistro::Arch => format!("sudo pacman -S {}", tools.join(" ")),
                    LinuxDistro::Unknown => format!(
                        "Install {} using your system's package manager",
                        tools.join(", ")
                    ),
                }
            }
            Platform::Unknown => format!(
                "Install {} using your system's package manager",
                tools.join(", ")
            ),
        }
    }
}

/// Tool-specific install hints (handles special cases like Python packages)
pub struct ToolInstallHints;

impl ToolInstallHints {
    /// Get install hint for a specific tool, handling special cases
    pub fn for_tool(tool: &str) -> String {
        let platform = detect_platform();

        match tool {
            "qpdf" => platform.install_hint("qpdf"),
            "gs" | "ghostscript" => match platform {
                Platform::MacOS => "brew install ghostscript".to_string(),
                Platform::Windows => "winget install ArtifexSoftware.GhostScript".to_string(),
                Platform::Linux => {
                    let distro = detect_linux_distro();
                    match distro {
                        LinuxDistro::Debian => "sudo apt install ghostscript".to_string(),
                        LinuxDistro::Fedora => "sudo dnf install ghostscript".to_string(),
                        LinuxDistro::Arch => "sudo pacman -S ghostscript".to_string(),
                        LinuxDistro::Unknown => {
                            "Install ghostscript using your package manager".to_string()
                        }
                    }
                }
                Platform::Unknown => "Install ghostscript using your package manager".to_string(),
            },
            "tesseract" => match platform {
                Platform::MacOS => "brew install tesseract".to_string(),
                Platform::Windows => "winget install tesseract-ocr".to_string(),
                Platform::Linux => {
                    let distro = detect_linux_distro();
                    match distro {
                        LinuxDistro::Debian => "sudo apt install tesseract-ocr".to_string(),
                        LinuxDistro::Fedora => "sudo dnf install tesseract".to_string(),
                        LinuxDistro::Arch => "sudo pacman -S tesseract".to_string(),
                        LinuxDistro::Unknown => {
                            "Install tesseract-ocr using your package manager".to_string()
                        }
                    }
                }
                Platform::Unknown => "Install tesseract using your package manager".to_string(),
            },
            "ocrmypdf" => {
                // Python package - needs pip
                match platform {
                    Platform::MacOS => "pip3 install ocrmypdf".to_string(),
                    Platform::Windows => "pip install ocrmypdf".to_string(),
                    Platform::Linux => "pip3 install ocrmypdf".to_string(),
                    Platform::Unknown => "pip install ocrmypdf".to_string(),
                }
            }
            "ffmpeg" => platform.install_hint("ffmpeg"),
            "libvips" => match platform {
                Platform::MacOS => "brew install libvips".to_string(),
                Platform::Windows => "scoop install libvips".to_string(),
                Platform::Linux => {
                    let distro = detect_linux_distro();
                    match distro {
                        LinuxDistro::Debian => "sudo apt install libvips-tools".to_string(),
                        LinuxDistro::Fedora => "sudo dnf install libvips".to_string(),
                        LinuxDistro::Arch => "sudo pacman -S libvips".to_string(),
                        LinuxDistro::Unknown => {
                            "Install libvips-tools using your package manager".to_string()
                        }
                    }
                }
                Platform::Unknown => "Install libvips using your package manager".to_string(),
            },
            "imagemagick" | "convert" => match platform {
                Platform::MacOS => "brew install imagemagick".to_string(),
                Platform::Windows => "scoop install imagemagick".to_string(),
                Platform::Linux => {
                    let distro = detect_linux_distro();
                    match distro {
                        LinuxDistro::Debian => "sudo apt install imagemagick".to_string(),
                        LinuxDistro::Fedora => "sudo dnf install ImageMagick".to_string(),
                        LinuxDistro::Arch => "sudo pacman -S imagemagick".to_string(),
                        LinuxDistro::Unknown => {
                            "Install imagemagick using your package manager".to_string()
                        }
                    }
                }
                Platform::Unknown => "Install imagemagick using your package manager".to_string(),
            },
            "exiftool" => match platform {
                Platform::MacOS => "brew install exiftool".to_string(),
                Platform::Windows => "scoop install exiftool".to_string(),
                Platform::Linux => {
                    let distro = detect_linux_distro();
                    match distro {
                        LinuxDistro::Debian => {
                            "sudo apt install libimage-exiftool-perl".to_string()
                        }
                        LinuxDistro::Fedora => "sudo dnf install perl-Image-ExifTool".to_string(),
                        LinuxDistro::Arch => "sudo pacman -S perl-image-exiftool".to_string(),
                        LinuxDistro::Unknown => {
                            "Install libimage-exiftool-perl using your package manager".to_string()
                        }
                    }
                }
                Platform::Unknown => "Install exiftool using your package manager".to_string(),
            },
            _ => platform.install_hint(tool),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = detect_platform();
        // Should detect something (not Unknown) on most systems
        assert_ne!(platform, Platform::Unknown);
    }

    #[test]
    fn test_install_hint_macos() {
        let platform = Platform::MacOS;
        assert_eq!(platform.install_hint("qpdf"), "brew install qpdf");
    }

    #[test]
    fn test_tool_install_hints() {
        // Test that we get some hint (not empty)
        let hint = ToolInstallHints::for_tool("qpdf");
        assert!(!hint.is_empty());
        assert!(hint.contains("qpdf") || hint.contains("install"));
    }
}
