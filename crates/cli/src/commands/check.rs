use forgekit_core::tools::qpdf::QpdfTool;
use forgekit_core::tools::{Tool, ToolConfig};
use forgekit_core::utils::error::Result;
use forgekit_core::utils::platform::{detect_platform, Platform};

pub fn handle_check_deps() -> Result<()> {
    let platform = detect_platform();
    let platform_name = match platform {
        Platform::MacOS => "macOS",
        Platform::Windows => "Windows",
        Platform::Linux => "Linux",
        Platform::Unknown => "Unknown",
    };

    println!("Checking ForgeKit dependencies...");
    println!("Platform: {}\n", platform_name);

    let tools: Vec<(&'static str, Box<dyn Tool>)> = vec![
        ("qpdf", Box::new(QpdfTool)),
        // TODO: Add other tools (pdfcpu, ocrmypdf, ffmpeg, libvips, etc.)
    ];

    let mut all_ok = true;
    let mut missing_tools = Vec::new();

    for (name, tool) in tools {
        let config = ToolConfig::default();
        match tool.probe(&config) {
            Ok(info) => {
                if info.available {
                    println!("✓ {}: found at {}", name, info.path.display());
                    println!("  Version: {}", info.version);
                } else {
                    println!("✗ {}: not available", name);
                    missing_tools.push(name);
                    all_ok = false;
                }
            }
            Err(e) => {
                println!("✗ {}: not found", name);
                if let forgekit_core::utils::error::ForgeKitError::ToolNotFound { hint, .. } = e {
                    println!("  Install: {}", hint);
                } else {
                    println!("  Error: {}", e);
                }
                missing_tools.push(name);
                all_ok = false;
            }
        }
        println!();
    }

    if all_ok {
        println!("✓ All dependencies are installed!");
    } else {
        println!("✗ Some dependencies are missing.\n");
        println!("Install missing dependencies individually, or install all at once:\n");
        match platform {
            Platform::MacOS => {
                println!("  brew install qpdf pdfcpu tesseract ffmpeg libvips exiftool");
                println!("  pip3 install ocrmypdf");
            }
            Platform::Windows => {
                println!("  winget install qpdf.qpdf pdfcpu.pdfcpu tesseract-ocr ffmpeg");
                println!("  scoop install libvips exiftool");
                println!("  pip install ocrmypdf");
            }
            Platform::Linux => {
                println!("  # Debian/Ubuntu:");
                println!("  sudo apt install qpdf pdfcpu tesseract-ocr ffmpeg libvips-tools libimage-exiftool-perl");
                println!("  pip3 install ocrmypdf");
                println!("\n  # Fedora/RHEL:");
                println!(
                    "  sudo dnf install qpdf pdfcpu tesseract ffmpeg libvips perl-Image-ExifTool"
                );
                println!("  pip3 install ocrmypdf");
                println!("\n  # Arch Linux:");
                println!(
                    "  sudo pacman -S qpdf pdfcpu tesseract ffmpeg libvips perl-image-exiftool"
                );
                println!("  pip3 install ocrmypdf");
            }
            Platform::Unknown => {
                println!("  Install dependencies using your system's package manager");
            }
        }
    }

    Ok(())
}
