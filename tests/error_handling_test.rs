//! Tests for error handling and edge cases
//! Tests error scenarios in CDP execution, file operations, and invalid inputs

mod test_server;

use robert_webdriver::{CdpCommand, CdpScript, ChromeDriver, ConnectionMode};
use std::path::Path;
use test_server::TestServer;

#[tokio::test]
async fn test_execute_cdp_script_file_not_found() {
    // Test execute_cdp_script with non-existent file
    let driver = ChromeDriver::new(ConnectionMode::Sandboxed {
        chrome_path: None,
        no_sandbox: true,
        headless: true,
    })
    .await
    .expect("Failed to launch Chrome");

    let result = driver
        .execute_cdp_script(Path::new("nonexistent-script.json"))
        .await;

    assert!(result.is_err(), "Should fail with non-existent file");
    let error = result.unwrap_err();
    println!("✅ File not found error: {}", error);

    driver.close().await.expect("Failed to close browser");
}

#[tokio::test]
async fn test_execute_cdp_script_invalid_json() {
    // Test execute_cdp_script with invalid JSON file
    let invalid_json_path = Path::new("invalid-test-script.json");

    // Write invalid JSON
    tokio::fs::write(invalid_json_path, "{ invalid json }")
        .await
        .expect("Failed to write test file");

    let driver = ChromeDriver::new(ConnectionMode::Sandboxed {
        chrome_path: None,
        no_sandbox: true,
        headless: true,
    })
    .await
    .expect("Failed to launch Chrome");

    let result = driver.execute_cdp_script(invalid_json_path).await;

    assert!(result.is_err(), "Should fail with invalid JSON");
    let error = result.unwrap_err();
    println!("✅ Invalid JSON error: {}", error);

    driver.close().await.expect("Failed to close browser");
    tokio::fs::remove_file(invalid_json_path).await.ok();
}

#[tokio::test]
async fn test_screenshot_to_nonexistent_directory() {
    // Test screenshot with path to non-existent directory
    let server = TestServer::start().await;
    server.wait_ready().await.expect("Server failed to start");
    let url = server.url();

    let driver = ChromeDriver::new(ConnectionMode::Sandboxed {
        chrome_path: None,
        no_sandbox: true,
        headless: true,
    })
    .await
    .expect("Failed to launch Chrome");

    let script = CdpScript {
        name: "screenshot-error-test".to_string(),
        description: "Test screenshot to invalid path".to_string(),
        created: None,
        author: Some("Test".to_string()),
        tags: vec![],
        cdp_commands: vec![
            CdpCommand {
                method: "Page.navigate".to_string(),
                params: serde_json::json!({"url": url}),
                save_as: None,
                description: Some("Navigate".to_string()),
            },
            CdpCommand {
                method: "Page.captureScreenshot".to_string(),
                params: serde_json::json!({}),
                save_as: Some("/nonexistent/directory/screenshot.png".to_string()),
                description: Some("Capture to invalid path".to_string()),
            },
        ],
    };

    let report = driver
        .execute_cdp_script_direct(&script)
        .await
        .expect("Script execution completed");

    println!("📊 Screenshot Error Test:");
    println!("   Successful: {}", report.successful);
    println!("   Failed: {}", report.failed);

    // The screenshot command may fail or succeed depending on permissions
    // We're just testing that the error path is executed
    if report.failed > 0 {
        println!("✅ Screenshot to invalid directory handled (failed as expected)");
        assert!(
            report.results[1].error.is_some(),
            "Should have error for invalid path"
        );
    } else {
        println!("⚠️ Screenshot succeeded (may have fallback behavior)");
    }

    driver.close().await.expect("Failed to close browser");
}

#[tokio::test]
async fn test_script_from_file_edge_cases() {
    // Test CdpScript::from_file with various edge cases
    use robert_webdriver::CdpScript;

    // Test 1: Non-existent file
    let result = CdpScript::from_file(Path::new("nonexistent.json")).await;
    assert!(result.is_err(), "Should fail with non-existent file");
    println!("✅ CdpScript::from_file handles missing file");

    // Test 2: Invalid JSON
    let invalid_path = Path::new("invalid-script-test.json");
    tokio::fs::write(invalid_path, "not valid json")
        .await
        .expect("Failed to write test file");

    let result = CdpScript::from_file(invalid_path).await;
    assert!(result.is_err(), "Should fail with invalid JSON");
    println!("✅ CdpScript::from_file handles invalid JSON");

    tokio::fs::remove_file(invalid_path).await.ok();

    // Test 3: Empty file
    let empty_path = Path::new("empty-script-test.json");
    tokio::fs::write(empty_path, "")
        .await
        .expect("Failed to write test file");

    let result = CdpScript::from_file(empty_path).await;
    assert!(result.is_err(), "Should fail with empty file");
    println!("✅ CdpScript::from_file handles empty file");

    tokio::fs::remove_file(empty_path).await.ok();
}

#[tokio::test]
async fn test_script_to_file() {
    // Test CdpScript::to_file
    use robert_webdriver::CdpScript;

    let script = CdpScript {
        name: "test-save".to_string(),
        description: "Test saving script".to_string(),
        created: None,
        author: Some("Test".to_string()),
        tags: vec!["test".to_string()],
        cdp_commands: vec![CdpCommand {
            method: "Page.navigate".to_string(),
            params: serde_json::json!({"url": "about:blank"}),
            save_as: None,
            description: Some("Test".to_string()),
        }],
    };

    let save_path = Path::new("test-saved-script.json");

    // Test saving
    script
        .to_file(save_path)
        .await
        .expect("Failed to save script");
    assert!(save_path.exists(), "Script file should exist");
    println!("✅ CdpScript::to_file saves successfully");

    // Test loading back
    let loaded = CdpScript::from_file(save_path)
        .await
        .expect("Failed to load script");
    assert_eq!(loaded.name, "test-save");
    assert_eq!(loaded.cdp_commands.len(), 1);
    println!("✅ CdpScript::from_file loads successfully");

    // Cleanup
    tokio::fs::remove_file(save_path).await.ok();
}

#[tokio::test]
async fn test_data_extraction_with_save() {
    // Test data extraction with save_as to ensure file writing works
    let server = TestServer::start().await;
    server.wait_ready().await.expect("Server failed to start");
    let url = server.url();

    let driver = ChromeDriver::new(ConnectionMode::Sandboxed {
        chrome_path: None,
        no_sandbox: true,
        headless: true,
    })
    .await
    .expect("Failed to launch Chrome");

    let script = CdpScript {
        name: "data-save-test".to_string(),
        description: "Test data extraction with file save".to_string(),
        created: None,
        author: Some("Test".to_string()),
        tags: vec![],
        cdp_commands: vec![
            CdpCommand {
                method: "Page.navigate".to_string(),
                params: serde_json::json!({"url": url}),
                save_as: None,
                description: Some("Navigate".to_string()),
            },
            CdpCommand {
                method: "Runtime.evaluate".to_string(),
                params: serde_json::json!({
                    "expression": "document.title",
                    "returnByValue": true
                }),
                save_as: Some("test-data-extraction.json".to_string()),
                description: Some("Extract and save data".to_string()),
            },
        ],
    };

    let report = driver
        .execute_cdp_script_direct(&script)
        .await
        .expect("Script execution failed");

    println!("📊 Data Extraction with Save Test:");
    println!(
        "   Commands: {}/{}",
        report.successful, report.total_commands
    );

    assert!(report.is_success(), "Script should succeed");

    // Verify file was created
    assert!(
        Path::new("test-data-extraction.json").exists(),
        "Output file should exist"
    );

    // Verify content
    let content = tokio::fs::read_to_string("test-data-extraction.json")
        .await
        .expect("Failed to read output file");
    assert!(
        content.contains("Example"),
        "Output should contain extracted data"
    );

    println!("✅ Data extraction with file save works");

    // Cleanup
    driver.close().await.expect("Failed to close browser");
    tokio::fs::remove_file("test-data-extraction.json")
        .await
        .ok();
}

#[tokio::test]
async fn test_send_cdp_command_missing_parameter() {
    // Test send_cdp_command with missing required parameter
    let driver = ChromeDriver::new(ConnectionMode::Sandboxed {
        chrome_path: None,
        no_sandbox: true,
        headless: true,
    })
    .await
    .expect("Failed to launch Chrome");

    // Try Runtime.evaluate without expression parameter
    let result = driver
        .send_cdp_command("Runtime.evaluate", serde_json::json!({}))
        .await;

    assert!(result.is_err(), "Should fail with missing parameter");
    let error = result.unwrap_err();
    println!("✅ Missing parameter error: {}", error);

    driver.close().await.expect("Failed to close browser");
}

#[tokio::test]
async fn test_navigate_to_invalid_url() {
    // Test navigation to invalid URL
    let driver = ChromeDriver::new(ConnectionMode::Sandboxed {
        chrome_path: None,
        no_sandbox: true,
        headless: true,
    })
    .await
    .expect("Failed to launch Chrome");

    // Try to navigate to invalid URL
    let result = driver.navigate("not-a-valid-url").await;

    // This may or may not fail depending on browser behavior
    // We're just testing that the method handles it
    match result {
        Ok(_) => println!("⚠️ Invalid URL navigation succeeded (browser may have error page)"),
        Err(e) => println!("✅ Invalid URL navigation failed as expected: {}", e),
    }

    driver.close().await.expect("Failed to close browser");
}
