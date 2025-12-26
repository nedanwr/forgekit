// Integration tests for ForgeKit Core
// These tests verify that components work together correctly

use forgekit_core::job::progress::ProgressEvent;
use forgekit_core::tools::qpdf::QpdfTool;
use forgekit_core::tools::{Tool, ToolConfig};
use forgekit_core::utils::error::ForgeKitError;
use forgekit_core::utils::platform::{detect_platform, Platform, ToolInstallHints};
use std::path::PathBuf;

#[test]
fn test_json_progress_format() {
    let event = ProgressEvent::Progress {
        version: 1,
        job_id: "test-123".to_string(),
        progress: forgekit_core::job::progress::ProgressInfo {
            current: 1,
            total: 3,
            percent: 33,
            stage: Some("merging".to_string()),
        },
        message: "Processing page 1/3".to_string(),
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"type\":\"progress\""));
    assert!(json.contains("\"version\":1"));
    assert!(json.contains("\"job_id\":\"test-123\""));
    assert!(json.contains("\"current\":1"));
    assert!(json.contains("\"total\":3"));
}

#[test]
fn test_json_complete_format() {
    let event = ProgressEvent::Complete {
        version: 1,
        job_id: "test-123".to_string(),
        result: forgekit_core::job::progress::JobResult {
            output: "/path/to/output.pdf".to_string(),
            size_bytes: 123456,
            duration_ms: 1234,
        },
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"type\":\"complete\""));
    assert!(json.contains("\"version\":1"));
    assert!(json.contains("\"output\":\"/path/to/output.pdf\""));
    assert!(json.contains("\"size_bytes\":123456"));
}

// Dependency checking integration tests

#[test]
fn test_tool_probe_with_override_path() {
    // Test that override_path that doesn't exist falls back to PATH
    // (This is the designed fallback behavior)
    let tool = QpdfTool;
    let config = ToolConfig {
        override_path: Some(PathBuf::from("/nonexistent/qpdf")),
    };

    let result = tool.probe(&config);
    // Result depends on whether qpdf is installed on the system
    // If qpdf is on PATH, probe succeeds (fallback worked)
    // If qpdf is not on PATH, probe fails
    match result {
        Ok(info) => {
            // qpdf is installed - fallback to PATH worked
            assert!(info.available);
            // The path should NOT be the nonexistent override path
            assert_ne!(info.path, PathBuf::from("/nonexistent/qpdf"));
        }
        Err(ForgeKitError::ToolNotFound { hint, .. }) => {
            // qpdf not installed - that's fine
            assert!(!hint.is_empty());
        }
        Err(e) => panic!("Unexpected error type: {:?}", e),
    }
}

#[test]
fn test_tool_probe_path_fallback() {
    // Test that PATH is checked when override_path is not set
    let tool = QpdfTool;
    let config = ToolConfig::default();

    let result = tool.probe(&config);
    // This will succeed if qpdf is installed, fail if not
    // Either way, we verify the probe doesn't panic
    match result {
        Ok(info) => {
            assert!(info.available);
            assert!(!info.version.is_empty());
            assert!(!info.path.as_os_str().is_empty());
        }
        Err(ForgeKitError::ToolNotFound { hint, .. }) => {
            // Tool not found - verify hint is provided
            assert!(!hint.is_empty());
            assert!(hint.contains("qpdf") || hint.contains("install"));
        }
        Err(e) => {
            panic!("Unexpected error type: {:?}", e);
        }
    }
}

#[test]
fn test_platform_detection_consistency() {
    // Test that platform detection is consistent
    let platform1 = detect_platform();
    let platform2 = detect_platform();

    // Should return the same platform on consecutive calls
    assert_eq!(platform1, platform2);

    // Should detect a known platform (not Unknown) on most systems
    // (This might fail on exotic systems, but that's okay)
    if platform1 != Platform::Unknown {
        // Verify platform-specific install hints work
        let hint = platform1.install_hint("qpdf");
        assert!(!hint.is_empty());
        assert!(hint.contains("qpdf"));
    }
}

#[test]
fn test_tool_install_hints_platform_specific() {
    // Test that install hints are platform-specific
    let tools = vec![
        "qpdf",
        "gs",
        "tesseract",
        "ocrmypdf",
        "ffmpeg",
        "libvips",
        "exiftool",
    ];

    for tool_name in tools {
        let hint = ToolInstallHints::for_tool(tool_name);

        // Verify hint is not empty
        assert!(
            !hint.is_empty(),
            "Install hint for {} should not be empty",
            tool_name
        );

        // Verify hint contains the tool name or "install"
        assert!(
            hint.contains(tool_name) || hint.contains("install"),
            "Install hint for {} should mention the tool or installation: {}",
            tool_name,
            hint
        );

        // Verify platform-specific commands
        let platform = detect_platform();
        match platform {
            Platform::MacOS => {
                assert!(
                    hint.contains("brew") || hint.contains("pip"),
                    "macOS hint for {} should mention brew or pip: {}",
                    tool_name,
                    hint
                );
            }
            Platform::Windows => {
                assert!(
                    hint.contains("winget") || hint.contains("scoop") || hint.contains("pip"),
                    "Windows hint for {} should mention winget, scoop, or pip: {}",
                    tool_name,
                    hint
                );
            }
            Platform::Linux => {
                assert!(
                    hint.contains("apt")
                        || hint.contains("dnf")
                        || hint.contains("pacman")
                        || hint.contains("pip"),
                    "Linux hint for {} should mention apt, dnf, pacman, or pip: {}",
                    tool_name,
                    hint
                );
            }
            Platform::Unknown => {
                // On unknown platforms, just verify we get some hint
                assert!(!hint.is_empty());
            }
        }
    }
}

#[test]
fn test_tool_install_hints_special_cases() {
    // Test special cases like Python packages and tool name variations

    // ocrmypdf should use pip
    let ocr_hint = ToolInstallHints::for_tool("ocrmypdf");
    assert!(ocr_hint.contains("pip") || ocr_hint.contains("ocrmypdf"));

    // tesseract should handle name variations (tesseract vs tesseract-ocr)
    let tesseract_hint = ToolInstallHints::for_tool("tesseract");
    assert!(tesseract_hint.contains("tesseract"));

    // libvips should handle name variations (libvips vs libvips-tools)
    let libvips_hint = ToolInstallHints::for_tool("libvips");
    assert!(libvips_hint.contains("libvips") || libvips_hint.contains("vips"));

    // exiftool should handle name variations (exiftool vs libimage-exiftool-perl)
    let exiftool_hint = ToolInstallHints::for_tool("exiftool");
    assert!(exiftool_hint.contains("exiftool") || exiftool_hint.contains("exif"));
}

#[test]
fn test_multiple_tools_checking() {
    // Test checking multiple tools (simulating check-deps behavior)
    let tools: Vec<(&str, Box<dyn Tool>)> = vec![("qpdf", Box::new(QpdfTool))];

    let mut found_count = 0;
    let mut missing_count = 0;

    for (name, tool) in tools {
        let config = ToolConfig::default();
        match tool.probe(&config) {
            Ok(info) => {
                if info.available {
                    found_count += 1;
                    assert!(
                        !info.version.is_empty(),
                        "Tool {} should have a version",
                        name
                    );
                    assert!(
                        !info.path.as_os_str().is_empty(),
                        "Tool {} should have a path",
                        name
                    );
                } else {
                    missing_count += 1;
                }
            }
            Err(ForgeKitError::ToolNotFound { hint, .. }) => {
                missing_count += 1;
                // Verify hint is provided for missing tools
                assert!(
                    !hint.is_empty(),
                    "Missing tool {} should have an install hint",
                    name
                );
            }
            Err(e) => {
                panic!("Unexpected error checking {}: {:?}", name, e);
            }
        }
    }

    // At least one tool should have been checked
    assert!(
        found_count + missing_count > 0,
        "Should have checked at least one tool"
    );
}

#[test]
fn test_tool_version_parsing() {
    // Test that version parsing works for qpdf
    let tool = QpdfTool;
    let config = ToolConfig::default();

    match tool.probe(&config) {
        Ok(info) => {
            // If qpdf is found, verify version is not empty
            assert!(!info.version.is_empty());

            // Verify we can call version() directly on the path
            let version_result = tool.version(&info.path);
            assert!(version_result.is_ok());
            let version = version_result.unwrap();
            assert!(!version.is_empty());
        }
        Err(ForgeKitError::ToolNotFound { .. }) => {
            // Tool not installed - skip version test
        }
        Err(e) => {
            panic!("Unexpected error: {:?}", e);
        }
    }
}

#[test]
fn test_platform_install_hints_bulk() {
    // Test bulk install hints for multiple tools
    let platform = detect_platform();
    let tools = vec!["qpdf", "gs", "tesseract"];

    let bulk_hint = platform.install_hints(&tools);
    assert!(!bulk_hint.is_empty());

    // Verify all tools are mentioned
    for tool in &tools {
        assert!(
            bulk_hint.contains(tool) || bulk_hint.contains("install"),
            "Bulk hint should mention {}: {}",
            tool,
            bulk_hint
        );
    }
}
