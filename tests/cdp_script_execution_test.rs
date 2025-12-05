//! Integration tests for CDP script execution
//! Tests programmatic CDP script execution without file dependencies

mod test_server;

use robert_webdriver::{CdpCommand, CdpScript, ChromeDriver, ConnectionMode};
use test_server::TestServer;

#[tokio::test]
async fn test_execute_navigation_and_screenshot() {
    // Start local test server
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

    // Create script programmatically
    let script = CdpScript {
        name: "navigation-screenshot-test".to_string(),
        description: "Navigate and take screenshot".to_string(),
        created: None,
        author: Some("Test".to_string()),
        tags: vec!["screenshot".to_string()],
        cdp_commands: vec![
            CdpCommand {
                method: "Page.navigate".to_string(),
                params: serde_json::json!({"url": url}),
                save_as: None,
                description: Some("Navigate to test server".to_string()),
            },
            CdpCommand {
                method: "Page.captureScreenshot".to_string(),
                params: serde_json::json!({}),
                save_as: Some("test-execution-screenshot.png".to_string()),
                description: Some("Capture screenshot".to_string()),
            },
        ],
    };

    let report = driver
        .execute_cdp_script_direct(&script)
        .await
        .expect("Script execution failed");

    println!("📊 Navigation + Screenshot Test:");
    println!("   Script: {}", report.script_name);
    println!(
        "   Commands: {}/{}",
        report.successful, report.total_commands
    );
    println!("   Success rate: {:.1}%", report.success_rate());

    assert!(report.is_success(), "Script execution should succeed");
    assert_eq!(report.successful, 2, "Should execute 2 commands");

    // Verify screenshot was saved
    assert!(
        std::path::Path::new("test-execution-screenshot.png").exists(),
        "Screenshot should be saved"
    );

    println!("✅ Navigation + screenshot test passed!");

    // Cleanup
    driver.close().await.expect("Failed to close browser");
    tokio::fs::remove_file("test-execution-screenshot.png")
        .await
        .ok();
}

#[tokio::test]
async fn test_execute_data_extraction() {
    // Start local test server
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

    // Create script to extract title and heading
    let script = CdpScript {
        name: "data-extraction-test".to_string(),
        description: "Extract page data".to_string(),
        created: None,
        author: Some("Test".to_string()),
        tags: vec!["extraction".to_string()],
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
                save_as: Some("test-exec-title.json".to_string()),
                description: Some("Get title".to_string()),
            },
            CdpCommand {
                method: "Runtime.evaluate".to_string(),
                params: serde_json::json!({
                    "expression": "document.querySelector('h1').textContent",
                    "returnByValue": true
                }),
                save_as: Some("test-exec-heading.json".to_string()),
                description: Some("Get heading".to_string()),
            },
        ],
    };

    let report = driver
        .execute_cdp_script_direct(&script)
        .await
        .expect("Script execution failed");

    println!("📊 Data Extraction Test:");
    println!("   Total commands: {}", report.total_commands);
    println!("   Successful: {}", report.successful);
    println!("   Duration: {:?}", report.total_duration);

    assert!(report.is_success(), "Script execution should succeed");
    assert_eq!(report.successful, 3, "Should execute 3 commands");

    // Verify extracted data was saved
    let title_content = tokio::fs::read_to_string("test-exec-title.json")
        .await
        .expect("Title file should exist");
    println!("Extracted title: {}", title_content);
    assert!(
        title_content.contains("Example"),
        "Title should contain 'Example'"
    );

    let heading_content = tokio::fs::read_to_string("test-exec-heading.json")
        .await
        .expect("Heading file should exist");
    assert!(
        heading_content.contains("Example Domain"),
        "Heading should contain 'Example Domain'"
    );

    println!("✅ Data extraction test passed!");

    // Cleanup
    driver.close().await.expect("Failed to close browser");
    tokio::fs::remove_file("test-exec-title.json").await.ok();
    tokio::fs::remove_file("test-exec-heading.json").await.ok();
}

#[tokio::test]
async fn test_execute_programmatic_script() {
    // Start local test server
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

    // Create script programmatically
    let script = CdpScript {
        name: "programmatic-test".to_string(),
        description: "Test script created in code".to_string(),
        created: None,
        author: Some("Test".to_string()),
        tags: vec!["test".to_string()],
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
        ],
    };

    // Execute the script
    let report = driver
        .execute_cdp_script_direct(&script)
        .await
        .expect("Script execution failed");

    println!("📊 Programmatic Script Execution:");
    println!("   Total commands: {}", report.total_commands);
    println!("   Successful: {}", report.successful);

    assert!(report.is_success(), "Script should succeed");
    assert_eq!(report.total_commands, 2, "Should have 2 commands");

    println!("✅ Programmatic script test passed!");

    driver.close().await.expect("Failed to close browser");
}

#[tokio::test]
async fn test_invalid_cdp_command() {
    let driver = ChromeDriver::new(ConnectionMode::Sandboxed {
        chrome_path: None,
        no_sandbox: true,
        headless: true,
    })
    .await
    .expect("Failed to launch Chrome");

    let script = CdpScript {
        name: "invalid-command-test".to_string(),
        description: "Test with invalid command".to_string(),
        created: None,
        author: None,
        tags: vec![],
        cdp_commands: vec![CdpCommand {
            method: "Invalid.command".to_string(),
            params: serde_json::json!({}),
            save_as: None,
            description: None,
        }],
    };

    let report = driver
        .execute_cdp_script_direct(&script)
        .await
        .expect("Script execution completed");

    println!("📊 Invalid Command Test:");
    println!("   Failed: {}", report.failed);
    println!("   Error: {:?}", report.results[0].error);

    // Should fail with unsupported command error
    assert!(!report.is_success(), "Invalid command should fail");
    assert_eq!(report.failed, 1, "Should have 1 failed command");
    assert!(
        report.results[0]
            .error
            .as_ref()
            .unwrap()
            .contains("Unsupported"),
        "Error should mention unsupported command"
    );

    println!("✅ Invalid command test passed!");

    driver.close().await.expect("Failed to close browser");
}

#[tokio::test]
async fn test_execute_cdp_script_from_file() {
    // Test file-based CDP script execution
    // Create a temporary script file
    let script_content = serde_json::json!({
        "name": "file-based-test",
        "description": "Test loading script from file",
        "author": "Test",
        "tags": ["file", "test"],
        "cdp_commands": [
            {
                "method": "Page.navigate",
                "params": {"url": "about:blank"},
                "description": "Navigate to blank page"
            }
        ]
    });

    // Write script to temp file
    let script_path = std::path::Path::new("test-script.json");
    tokio::fs::write(
        script_path,
        serde_json::to_string_pretty(&script_content).unwrap(),
    )
    .await
    .expect("Failed to write script file");

    // Launch driver
    let driver = ChromeDriver::new(ConnectionMode::Sandboxed {
        chrome_path: None,
        no_sandbox: true,
        headless: true,
    })
    .await
    .expect("Failed to launch Chrome");

    // Execute script from file
    let report = driver
        .execute_cdp_script(script_path)
        .await
        .expect("Failed to execute script from file");

    println!("📊 File-based Script Execution:");
    println!("   Script: {}", report.script_name);
    println!(
        "   Commands: {}/{}",
        report.successful, report.total_commands
    );

    assert!(report.is_success(), "File-based script should succeed");
    assert_eq!(report.script_name, "file-based-test");
    assert_eq!(report.total_commands, 1);

    println!("✅ File-based script test passed!");

    // Cleanup
    driver.close().await.expect("Failed to close browser");
    tokio::fs::remove_file(script_path).await.ok();
}
