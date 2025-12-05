//! Integration tests designed to run headlessly in CI/CD environments
//!
//! Uses local HTTP server for fast, reliable, network-independent testing.
//! Each test uses its own server on a random port for perfect isolation.

mod test_server;

use robert_webdriver::{CdpCommand, CdpScript, ChromeDriver, ConnectionMode};
use test_server::TestServer;

/// Helper to create a headless driver for testing
async fn create_headless_driver() -> anyhow::Result<ChromeDriver> {
    ChromeDriver::new(ConnectionMode::Sandboxed {
        chrome_path: None,
        no_sandbox: true, // Required for CI environments
        headless: true,   // Always headless for these tests
    })
    .await
    .map_err(|e| anyhow::anyhow!("Failed to launch Chrome: {}", e))
}

#[tokio::test]
async fn test_basic_navigation_headless() -> anyhow::Result<()> {
    // Start server and verify it's ready
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();

    // Launch driver
    let driver = create_headless_driver().await?;

    // Use CDP script for reliable navigation (CDP commands work better than high-level navigate)
    let script = CdpScript {
        name: "basic-navigation-test".to_string(),
        description: "Navigate and verify title".to_string(),
        created: None,
        author: Some("Test".to_string()),
        tags: vec!["navigation".to_string()],
        cdp_commands: vec![
            CdpCommand {
                method: "Page.navigate".to_string(),
                params: serde_json::json!({
                    "url": url
                }),
                save_as: None,
                description: Some("Navigate to test server".to_string()),
            },
            CdpCommand {
                method: "Runtime.evaluate".to_string(),
                params: serde_json::json!({
                    "expression": "document.title",
                    "returnByValue": true
                }),
                save_as: Some("test-nav-title.json".to_string()),
                description: Some("Get page title".to_string()),
            },
        ],
    };

    // Execute the script
    println!("Navigating to: {}", url);
    let report = driver.execute_cdp_script_direct(&script).await?;

    println!("📊 Navigation Report:");
    println!(
        "   Commands executed: {}/{}",
        report.successful, report.total_commands
    );
    println!("   Success rate: {:.1}%", report.success_rate());

    // Verify execution succeeded
    if !report.is_success() {
        driver.close().await?;
        anyhow::bail!("CDP navigation script failed");
    }

    // Read the extracted title
    let title_data = tokio::fs::read_to_string("test-nav-title.json").await?;
    println!("✅ Extracted title data: {}", title_data);

    // Verify title contains "example"
    if !title_data.to_lowercase().contains("example") {
        driver.close().await?;
        tokio::fs::remove_file("test-nav-title.json").await.ok();
        anyhow::bail!("Title doesn't contain 'example': {}", title_data);
    }

    println!("✅ Title check passed!");

    // Cleanup
    driver.close().await?;
    tokio::fs::remove_file("test-nav-title.json").await.ok();

    Ok(())
}

#[tokio::test]
async fn test_cdp_script_execution_headless() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    let url = server.url();
    println!("Test server running on: {}", url);

    let driver = create_headless_driver().await?;

    // Create a simple CDP script
    let script = CdpScript {
        name: "headless-test".to_string(),
        description: "Test CDP script execution in headless mode".to_string(),
        created: None,
        author: Some("Test".to_string()),
        tags: vec!["test".to_string(), "headless".to_string()],
        cdp_commands: vec![
            CdpCommand {
                method: "Page.navigate".to_string(),
                params: serde_json::json!({
                    "url": url
                }),
                save_as: None,
                description: Some("Navigate to test server".to_string()),
            },
            CdpCommand {
                method: "Runtime.evaluate".to_string(),
                params: serde_json::json!({
                    "expression": "document.title",
                    "returnByValue": true
                }),
                save_as: Some("test-title.json".to_string()),
                description: Some("Extract page title".to_string()),
            },
        ],
    };

    // Execute the script
    let report = driver.execute_cdp_script_direct(&script).await?;

    println!("📊 Execution Report:");
    println!("   Script: {}", report.script_name);
    println!("   Total commands: {}", report.total_commands);
    println!("   Successful: {}", report.successful);
    println!("   Failed: {}", report.failed);
    println!("   Success rate: {:.1}%", report.success_rate());
    println!("   Duration: {:?}", report.total_duration);

    // Verify execution
    assert!(report.is_success(), "Script execution should succeed");
    assert_eq!(report.total_commands, 2, "Should have 2 commands");
    assert_eq!(report.successful, 2, "Both commands should succeed");

    // Cleanup
    driver.close().await?;
    if std::path::Path::new("test-title.json").exists() {
        tokio::fs::remove_file("test-title.json").await.ok();
    }

    Ok(())
}

#[tokio::test]
async fn test_screenshot_capture_headless() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    let url = server.url();
    println!("Test server running on: {}", url);

    let driver = create_headless_driver().await?;

    // Create screenshot script
    let script = CdpScript {
        name: "screenshot-test".to_string(),
        description: "Capture screenshot in headless mode".to_string(),
        created: None,
        author: Some("Test".to_string()),
        tags: vec!["screenshot".to_string()],
        cdp_commands: vec![
            CdpCommand {
                method: "Page.navigate".to_string(),
                params: serde_json::json!({
                    "url": url
                }),
                save_as: None,
                description: Some("Navigate to test server".to_string()),
            },
            CdpCommand {
                method: "Page.captureScreenshot".to_string(),
                params: serde_json::json!({
                    "format": "png",
                    "captureBeyondViewport": true
                }),
                save_as: Some("test-screenshot.png".to_string()),
                description: Some("Capture screenshot".to_string()),
            },
        ],
    };

    // Execute
    let report = driver.execute_cdp_script_direct(&script).await?;

    println!("📸 Screenshot Test:");
    println!("   Success: {}", report.is_success());
    println!(
        "   Commands: {}/{}",
        report.successful, report.total_commands
    );

    // Verify
    assert!(report.is_success(), "Screenshot script should succeed");

    // Verify file exists
    let screenshot_path = std::path::Path::new("test-screenshot.png");
    assert!(
        screenshot_path.exists(),
        "Screenshot file should be created"
    );

    // Check file size
    let metadata = std::fs::metadata(screenshot_path)?;
    println!("   Screenshot size: {} bytes", metadata.len());
    assert!(metadata.len() > 1000, "Screenshot should be at least 1KB");

    // Cleanup
    driver.close().await?;
    tokio::fs::remove_file("test-screenshot.png").await.ok();

    Ok(())
}

#[tokio::test]
async fn test_data_extraction_headless() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    let url = server.url();
    println!("Test server running on: {}", url);

    let driver = create_headless_driver().await?;

    // Create data extraction script
    let script = CdpScript {
        name: "extract-data-test".to_string(),
        description: "Extract data in headless mode".to_string(),
        created: None,
        author: Some("Test".to_string()),
        tags: vec!["extraction".to_string()],
        cdp_commands: vec![
            CdpCommand {
                method: "Page.navigate".to_string(),
                params: serde_json::json!({
                    "url": url
                }),
                save_as: None,
                description: Some("Navigate to test server".to_string()),
            },
            CdpCommand {
                method: "Runtime.evaluate".to_string(),
                params: serde_json::json!({
                    "expression": "JSON.stringify({title: document.title, heading: document.querySelector('h1').textContent})",
                    "returnByValue": true
                }),
                save_as: Some("test-extracted-data.json".to_string()),
                description: Some("Extract title and heading".to_string()),
            },
        ],
    };

    // Execute
    let report = driver.execute_cdp_script_direct(&script).await?;

    println!("📦 Data Extraction Test:");
    println!("   Success: {}", report.is_success());

    // Verify
    assert!(report.is_success(), "Extraction should succeed");

    // Verify extracted data file
    let data_path = std::path::Path::new("test-extracted-data.json");
    assert!(data_path.exists(), "Extracted data file should exist");

    let content = tokio::fs::read_to_string(data_path).await?;
    println!("   Extracted: {}", content);
    assert!(
        content.to_lowercase().contains("example"),
        "Data should contain 'example'"
    );

    // Cleanup
    driver.close().await?;
    tokio::fs::remove_file("test-extracted-data.json")
        .await
        .ok();

    Ok(())
}

#[tokio::test]
async fn test_multiple_commands_headless() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    let url = server.url();
    println!("Test server running on: {}", url);

    let driver = create_headless_driver().await?;

    // Create script with multiple diverse commands
    let script = CdpScript {
        name: "multi-command-test".to_string(),
        description: "Test multiple CDP commands".to_string(),
        created: None,
        author: Some("Test".to_string()),
        tags: vec!["multi".to_string()],
        cdp_commands: vec![
            CdpCommand {
                method: "Page.navigate".to_string(),
                params: serde_json::json!({"url": url}),
                save_as: None,
                description: Some("Navigate to test server".to_string()),
            },
            CdpCommand {
                method: "Runtime.evaluate".to_string(),
                params: serde_json::json!({
                    "expression": "document.title",
                    "returnByValue": true
                }),
                save_as: None,
                description: Some("Get title".to_string()),
            },
            CdpCommand {
                method: "Page.captureScreenshot".to_string(),
                params: serde_json::json!({
                    "format": "png",
                    "captureBeyondViewport": true
                }),
                save_as: Some("test-multi-screenshot.png".to_string()),
                description: Some("Screenshot".to_string()),
            },
        ],
    };

    // Execute
    let report = driver.execute_cdp_script_direct(&script).await?;

    println!("🔄 Multi-Command Test:");
    println!(
        "   Commands: {}/{}",
        report.successful, report.total_commands
    );
    println!("   Duration: {:?}", report.total_duration);

    // Verify all commands succeeded
    assert_eq!(report.total_commands, 3, "Should have 3 commands");
    assert_eq!(report.successful, 3, "All commands should succeed");
    assert_eq!(report.failed, 0, "No commands should fail");

    // Cleanup
    driver.close().await?;
    tokio::fs::remove_file("test-multi-screenshot.png")
        .await
        .ok();

    Ok(())
}
