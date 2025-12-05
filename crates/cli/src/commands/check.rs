use forgekit_core::tools::{Tool, ToolConfig};
use forgekit_core::tools::qpdf::QpdfTool;
use forgekit_core::utils::error::Result;

pub fn handle_check_deps() -> Result<()> {
    println!("Checking ForgeKit dependencies...\n");

    let tools: Vec<(&'static str, Box<dyn Tool>)> = vec![
        ("qpdf", Box::new(QpdfTool)),
        // TODO: Add other tools (pdfcpu, ocrmypdf, ffmpeg, libvips, etc.)
    ];

    let mut all_ok = true;

    for (name, tool) in tools {
        let config = ToolConfig::default();
        match tool.probe(&config) {
            Ok(info) => {
                if info.available {
                    println!("✓ {}: found at {}", name, info.path.display());
                    println!("  Version: {}", info.version);
                } else {
                    println!("✗ {}: not available", name);
                    all_ok = false;
                }
            }
            Err(e) => {
                println!("✗ {}: {}", name, e);
                if let forgekit_core::utils::error::ForgeKitError::ToolNotFound { hint, .. } = e {
                    println!("  Hint: {}", hint);
                }
                all_ok = false;
            }
        }
        println!();
    }

    if all_ok {
        println!("All dependencies are installed!");
    } else {
        println!("Some dependencies are missing. Install them using the hints above.");
    }
    
    Ok(())
}

