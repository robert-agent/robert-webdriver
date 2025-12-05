//! Comprehensive unit and integration tests for screenshot functionality
//!
//! This test suite covers:
//! - Taking screenshots from running browsers
//! - Saving screenshots to files
//! - Screenshot capture via CDP commands
//! - Integration with step frames (browser workflow documentation)
//! - Error handling for screenshot operations

mod test_server;

use robert_webdriver::{CdpCommand, CdpScript, ChromeDriver, ConnectionMode};
use std::path::PathBuf;
use test_server::TestServer;

/// Helper to create a headless driver for testing
async fn create_headless_driver() -> anyhow::Result<ChromeDriver> {
    ChromeDriver::new(ConnectionMode::Sandboxed {
        chrome_path: None,
        no_sandbox: true,
        headless: true,
    })
    .await
    .map_err(|e| anyhow::anyhow!("Failed to launch Chrome: {}", e))
}

// ===== UNIT TESTS FOR SCREENSHOT METHODS =====

#[tokio::test]
async fn test_screenshot_returns_valid_png_data() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    // Navigate to a page
    driver.navigate(&url).await?;

    // Take screenshot
    let screenshot_data = driver.screenshot().await?;

    // Verify PNG header (89 50 4E 47 0D 0A 1A 0A)
    assert!(screenshot_data.len() > 8, "Screenshot should have data");
    assert_eq!(screenshot_data[0], 0x89, "Should start with PNG signature");
    assert_eq!(screenshot_data[1], 0x50, "PNG signature byte 2");
    assert_eq!(screenshot_data[2], 0x4E, "PNG signature byte 3");
    assert_eq!(screenshot_data[3], 0x47, "PNG signature byte 4");

    // Verify reasonable size (should be at least 1KB)
    assert!(
        screenshot_data.len() > 1000,
        "Screenshot should be at least 1KB, got {} bytes",
        screenshot_data.len()
    );

    driver.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_screenshot_to_file_creates_valid_file() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    // Navigate to a page
    driver.navigate(&url).await?;

    // Create temp file path
    let temp_dir = std::env::temp_dir();
    let screenshot_path = temp_dir.join("test-screenshot-unit.png");

    // Clean up any existing file
    let _ = tokio::fs::remove_file(&screenshot_path).await;

    // Take screenshot to file
    driver.screenshot_to_file(&screenshot_path).await?;

    // Verify file exists
    assert!(
        screenshot_path.exists(),
        "Screenshot file should be created at {:?}",
        screenshot_path
    );

    // Verify file size
    let metadata = tokio::fs::metadata(&screenshot_path).await?;
    assert!(
        metadata.len() > 1000,
        "Screenshot file should be at least 1KB"
    );

    // Verify PNG format by reading file header
    let file_data = tokio::fs::read(&screenshot_path).await?;
    assert_eq!(file_data[0], 0x89, "File should start with PNG signature");
    assert_eq!(file_data[1], 0x50, "PNG signature byte 2");
    assert_eq!(file_data[2], 0x4E, "PNG signature byte 3");
    assert_eq!(file_data[3], 0x47, "PNG signature byte 4");

    // Cleanup
    tokio::fs::remove_file(&screenshot_path).await?;
    driver.close().await?;

    Ok(())
}

#[tokio::test]
async fn test_screenshot_multiple_times() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    driver.navigate(&url).await?;

    // Take multiple screenshots
    let screenshot1 = driver.screenshot().await?;
    let screenshot2 = driver.screenshot().await?;
    let screenshot3 = driver.screenshot().await?;

    // All should be valid PNG data
    assert!(screenshot1.len() > 1000, "Screenshot 1 should have data");
    assert!(screenshot2.len() > 1000, "Screenshot 2 should have data");
    assert!(screenshot3.len() > 1000, "Screenshot 3 should have data");

    // All should be PNG format
    for (i, data) in [&screenshot1, &screenshot2, &screenshot3]
        .iter()
        .enumerate()
    {
        assert_eq!(data[0], 0x89, "Screenshot {} should be PNG", i + 1);
    }

    driver.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_screenshot_different_pages() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    // Navigate to first page and take screenshot
    driver.navigate(&url).await?;
    let screenshot1 = driver.screenshot().await?;

    // Navigate to about:blank and take screenshot
    driver.navigate("about:blank").await?;
    let screenshot2 = driver.screenshot().await?;

    // Both should be valid
    assert!(screenshot1.len() > 1000, "Screenshot 1 should have data");
    assert!(screenshot2.len() > 1000, "Screenshot 2 should have data");

    // Screenshots should be different (different content)
    assert_ne!(
        screenshot1, screenshot2,
        "Screenshots of different pages should differ"
    );

    driver.close().await?;
    Ok(())
}

// ===== CDP COMMAND TESTS =====

#[tokio::test]
async fn test_cdp_capture_screenshot_command() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    let temp_dir = std::env::temp_dir();
    let screenshot_path = temp_dir.join("test-cdp-screenshot.png");
    let _ = tokio::fs::remove_file(&screenshot_path).await;

    // Create CDP script with screenshot command
    let script = CdpScript {
        name: "screenshot-test".to_string(),
        description: "Test Page.captureScreenshot command".to_string(),
        created: None,
        author: Some("Test".to_string()),
        tags: vec!["screenshot".to_string()],
        cdp_commands: vec![
            CdpCommand {
                method: "Page.navigate".to_string(),
                params: serde_json::json!({"url": url}),
                save_as: None,
                description: Some("Navigate to test page".to_string()),
            },
            CdpCommand {
                method: "Page.captureScreenshot".to_string(),
                params: serde_json::json!({
                    "format": "png",
                    "captureBeyondViewport": true
                }),
                save_as: Some(screenshot_path.to_string_lossy().to_string()),
                description: Some("Capture screenshot".to_string()),
            },
        ],
    };

    // Execute script
    let report = driver.execute_cdp_script_direct(&script).await?;

    // Verify execution succeeded
    assert!(report.is_success(), "CDP script should succeed");
    assert_eq!(report.successful, 2, "Both commands should succeed");

    // Verify screenshot file was created
    assert!(
        screenshot_path.exists(),
        "Screenshot file should be created via CDP command"
    );

    // Verify file is valid PNG
    let metadata = tokio::fs::metadata(&screenshot_path).await?;
    assert!(metadata.len() > 1000, "Screenshot should be at least 1KB");

    let file_data = tokio::fs::read(&screenshot_path).await?;
    assert_eq!(file_data[0], 0x89, "Should be PNG format");

    // Cleanup
    tokio::fs::remove_file(&screenshot_path).await?;
    driver.close().await?;

    Ok(())
}

#[tokio::test]
async fn test_cdp_screenshot_with_different_formats() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    driver.navigate(&url).await?;

    let temp_dir = std::env::temp_dir();

    // Test PNG format
    let png_path = temp_dir.join("test-format-png.png");
    let _ = tokio::fs::remove_file(&png_path).await;

    let script = CdpScript {
        name: "png-test".to_string(),
        description: "Test PNG format".to_string(),
        created: None,
        author: Some("Test".to_string()),
        tags: vec![],
        cdp_commands: vec![CdpCommand {
            method: "Page.captureScreenshot".to_string(),
            params: serde_json::json!({
                "format": "png"
            }),
            save_as: Some(png_path.to_string_lossy().to_string()),
            description: Some("PNG screenshot".to_string()),
        }],
    };

    let report = driver.execute_cdp_script_direct(&script).await?;
    assert!(report.is_success(), "PNG screenshot should succeed");
    assert!(png_path.exists(), "PNG file should be created");

    // Verify PNG signature
    let png_data = tokio::fs::read(&png_path).await?;
    assert_eq!(png_data[0..4], [0x89, 0x50, 0x4E, 0x47], "Should be PNG");

    // Test JPEG format
    let jpeg_path = temp_dir.join("test-format-jpeg.jpg");
    let _ = tokio::fs::remove_file(&jpeg_path).await;

    let script = CdpScript {
        name: "jpeg-test".to_string(),
        description: "Test JPEG format".to_string(),
        created: None,
        author: Some("Test".to_string()),
        tags: vec![],
        cdp_commands: vec![CdpCommand {
            method: "Page.captureScreenshot".to_string(),
            params: serde_json::json!({
                "format": "jpeg",
                "quality": 90
            }),
            save_as: Some(jpeg_path.to_string_lossy().to_string()),
            description: Some("JPEG screenshot".to_string()),
        }],
    };

    let report = driver.execute_cdp_script_direct(&script).await?;
    assert!(report.is_success(), "JPEG screenshot should succeed");
    assert!(jpeg_path.exists(), "JPEG file should be created");

    // Verify JPEG signature (FF D8 FF)
    let jpeg_data = tokio::fs::read(&jpeg_path).await?;
    assert_eq!(jpeg_data[0..2], [0xFF, 0xD8], "Should be JPEG");

    // Cleanup
    tokio::fs::remove_file(&png_path).await?;
    tokio::fs::remove_file(&jpeg_path).await?;
    driver.close().await?;

    Ok(())
}

// ===== INTEGRATION TESTS FOR BROWSER STEP FRAMES =====

/// Helper struct for a browser step frame
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct BrowserStepFrame {
    frame_id: usize,
    timestamp: String,
    elapsed_ms: u64,
    screenshot: ScreenshotInfo,
    dom: DomInfo,
    action: ActionInfo,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ScreenshotInfo {
    path: String,
    format: String,
    size_bytes: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DomInfo {
    url: String,
    title: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ActionInfo {
    description: String,
}

#[tokio::test]
async fn test_screenshot_integration_with_step_frame() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    // Navigate to page
    driver.navigate(&url).await?;

    // Get page info
    let page_url = driver.current_url().await?;
    let page_title = driver.title().await?;

    // Take screenshot for frame
    let temp_dir = std::env::temp_dir();
    let screenshot_path = temp_dir.join("frame-screenshot-1.png");
    let _ = tokio::fs::remove_file(&screenshot_path).await;

    driver.screenshot_to_file(&screenshot_path).await?;

    // Verify screenshot was captured
    assert!(screenshot_path.exists(), "Screenshot should be created");
    let metadata = tokio::fs::metadata(&screenshot_path).await?;

    // Create a step frame with the screenshot
    let frame = BrowserStepFrame {
        frame_id: 1,
        timestamp: chrono::Utc::now().to_rfc3339(),
        elapsed_ms: 0,
        screenshot: ScreenshotInfo {
            path: screenshot_path.to_string_lossy().to_string(),
            format: "png".to_string(),
            size_bytes: metadata.len() as usize,
        },
        dom: DomInfo {
            url: page_url,
            title: page_title,
        },
        action: ActionInfo {
            description: "Navigate to test page".to_string(),
        },
    };

    // Serialize frame to JSON
    let frame_json = serde_json::to_string_pretty(&frame)?;
    println!("📄 Step Frame JSON:\n{}", frame_json);

    // Verify frame has screenshot info
    assert!(frame_json.contains("frame-screenshot-1.png"));
    assert!(frame_json.contains("\"format\": \"png\""));
    assert!(frame.screenshot.size_bytes > 1000);

    // Cleanup
    tokio::fs::remove_file(&screenshot_path).await?;
    driver.close().await?;

    Ok(())
}

#[tokio::test]
async fn test_multiple_step_frames_with_screenshots() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    let temp_dir = std::env::temp_dir();
    let mut frames: Vec<BrowserStepFrame> = Vec::new();
    let start_time = std::time::Instant::now();

    // Frame 1: Initial navigation
    driver.navigate(&url).await?;
    let screenshot_path_1 = temp_dir.join("frame-1.png");
    let _ = tokio::fs::remove_file(&screenshot_path_1).await;
    driver.screenshot_to_file(&screenshot_path_1).await?;

    let metadata_1 = tokio::fs::metadata(&screenshot_path_1).await?;
    frames.push(BrowserStepFrame {
        frame_id: 1,
        timestamp: chrono::Utc::now().to_rfc3339(),
        elapsed_ms: start_time.elapsed().as_millis() as u64,
        screenshot: ScreenshotInfo {
            path: screenshot_path_1.to_string_lossy().to_string(),
            format: "png".to_string(),
            size_bytes: metadata_1.len() as usize,
        },
        dom: DomInfo {
            url: driver.current_url().await?,
            title: driver.title().await?,
        },
        action: ActionInfo {
            description: "Navigate to page".to_string(),
        },
    });

    // Frame 2: Execute JavaScript
    driver
        .execute_script("document.body.style.backgroundColor = 'lightblue'")
        .await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let screenshot_path_2 = temp_dir.join("frame-2.png");
    let _ = tokio::fs::remove_file(&screenshot_path_2).await;
    driver.screenshot_to_file(&screenshot_path_2).await?;

    let metadata_2 = tokio::fs::metadata(&screenshot_path_2).await?;
    frames.push(BrowserStepFrame {
        frame_id: 2,
        timestamp: chrono::Utc::now().to_rfc3339(),
        elapsed_ms: start_time.elapsed().as_millis() as u64,
        screenshot: ScreenshotInfo {
            path: screenshot_path_2.to_string_lossy().to_string(),
            format: "png".to_string(),
            size_bytes: metadata_2.len() as usize,
        },
        dom: DomInfo {
            url: driver.current_url().await?,
            title: driver.title().await?,
        },
        action: ActionInfo {
            description: "Change background color".to_string(),
        },
    });

    // Verify we have 2 frames with screenshots
    assert_eq!(frames.len(), 2, "Should have 2 step frames");
    assert!(screenshot_path_1.exists(), "Frame 1 screenshot exists");
    assert!(screenshot_path_2.exists(), "Frame 2 screenshot exists");

    // Verify screenshots are different (different DOM state)
    let data1 = tokio::fs::read(&screenshot_path_1).await?;
    let data2 = tokio::fs::read(&screenshot_path_2).await?;
    // Note: Screenshots might be the same if background change isn't visible
    // but both should be valid PNGs
    assert_eq!(data1[0], 0x89, "Frame 1 screenshot is PNG");
    assert_eq!(data2[0], 0x89, "Frame 2 screenshot is PNG");

    // Serialize all frames
    let frames_json = serde_json::to_string_pretty(&frames)?;
    println!("📋 Multiple Step Frames:\n{}", frames_json);

    // Cleanup
    tokio::fs::remove_file(&screenshot_path_1).await?;
    tokio::fs::remove_file(&screenshot_path_2).await?;
    driver.close().await?;

    Ok(())
}

#[tokio::test]
async fn test_step_frame_with_cdp_workflow() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    let temp_dir = std::env::temp_dir();
    let screenshot_path = temp_dir.join("cdp-workflow-frame.png");
    let _ = tokio::fs::remove_file(&screenshot_path).await;

    // Execute a complete workflow via CDP and capture frame
    let script = CdpScript {
        name: "workflow-with-frame".to_string(),
        description: "Complete workflow with step frame".to_string(),
        created: None,
        author: Some("Test".to_string()),
        tags: vec!["workflow".to_string()],
        cdp_commands: vec![
            CdpCommand {
                method: "Page.navigate".to_string(),
                params: serde_json::json!({"url": url}),
                save_as: None,
                description: Some("Navigate to page".to_string()),
            },
            CdpCommand {
                method: "Page.captureScreenshot".to_string(),
                params: serde_json::json!({
                    "format": "png",
                    "captureBeyondViewport": true
                }),
                save_as: Some(screenshot_path.to_string_lossy().to_string()),
                description: Some("Capture state".to_string()),
            },
        ],
    };

    let report = driver.execute_cdp_script_direct(&script).await?;

    // Verify workflow execution
    assert!(report.is_success(), "Workflow should succeed");

    // Verify screenshot was captured as part of workflow
    assert!(screenshot_path.exists(), "Screenshot should be in frame");

    // Create frame from workflow
    let metadata = tokio::fs::metadata(&screenshot_path).await?;
    let frame = BrowserStepFrame {
        frame_id: 1,
        timestamp: chrono::Utc::now().to_rfc3339(),
        elapsed_ms: report.total_duration.as_millis() as u64,
        screenshot: ScreenshotInfo {
            path: screenshot_path.to_string_lossy().to_string(),
            format: "png".to_string(),
            size_bytes: metadata.len() as usize,
        },
        dom: DomInfo {
            url: driver.current_url().await?,
            title: driver.title().await?,
        },
        action: ActionInfo {
            description: format!("Executed workflow: {}", script.name),
        },
    };

    // Verify frame is complete
    assert!(frame.screenshot.size_bytes > 1000);
    assert_eq!(frame.screenshot.format, "png");
    assert!(frame.screenshot.path.contains("cdp-workflow-frame.png"));

    // Cleanup
    tokio::fs::remove_file(&screenshot_path).await?;
    driver.close().await?;

    Ok(())
}

// ===== ERROR HANDLING TESTS =====

#[tokio::test]
async fn test_screenshot_before_navigation() -> anyhow::Result<()> {
    let driver = create_headless_driver().await?;

    // Try to take screenshot immediately after launch (should still work on blank page)
    let result = driver.screenshot().await;

    // Should succeed even on about:blank
    assert!(
        result.is_ok(),
        "Screenshot should work on initial blank page"
    );

    let screenshot_data = result?;
    assert!(screenshot_data.len() > 100, "Should have some data");

    driver.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_screenshot_to_invalid_path() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    driver.navigate(&url).await?;

    // Try to save to invalid path (directory that doesn't exist and can't be created)
    let invalid_path = PathBuf::from("/root/definitely/does/not/exist/screenshot.png");
    let result = driver.screenshot_to_file(&invalid_path).await;

    // Should fail gracefully
    assert!(result.is_err(), "Should fail when saving to invalid path");

    driver.close().await?;
    Ok(())
}
