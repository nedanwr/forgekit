// Integration tests for ForgeKit Core
// These tests verify that components work together correctly

use forgekit_core::job::progress::ProgressEvent;
use serde_json;

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
