// Test CDP command execution API
// These tests verify the ChromeDriver CDP execution capabilities

mod test_server;

use robert_webdriver::{CdpCommand, CdpScript, ChromeDriver, ConnectionMode};
use test_server::TestServer;

#[tokio::test]
async fn test_cdp_page_access() {
    // This test verifies we can get the Page for CDP commands
    let driver = ChromeDriver::new(ConnectionMode::Sandboxed {
        chrome_path: None,
        no_sandbox: true,
        headless: true,
    })
    .await
    .expect("Failed to launch Chrome");

    // Get the underlying Page which has execute() method
    let page = driver.current_page().await.expect("Failed to get page");

    // Page implements Command trait execution
    println!(
        "✅ Successfully got page: {}",
        std::any::type_name_of_val(&page)
    );

    driver.close().await.expect("Failed to close browser");
}

#[tokio::test]
async fn test_cdp_navigation() {
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

    // Use CDP commands for reliable navigation
    let script = CdpScript {
        name: "cdp-navigation-test".to_string(),
        description: "Test CDP navigation".to_string(),
        created: None,
        author: Some("Test".to_string()),
        tags: vec!["cdp".to_string()],
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
                    "expression": "window.location.href",
                    "returnByValue": true
                }),
                save_as: Some("test-cdp-url.json".to_string()),
                description: Some("Get current URL".to_string()),
            },
        ],
    };

    let report = driver
        .execute_cdp_script_direct(&script)
        .await
        .expect("Script execution failed");

    println!("📊 CDP Navigation Test:");
    println!(
        "   Commands: {}/{}",
        report.successful, report.total_commands
    );

    assert!(report.is_success(), "CDP navigation should succeed");

    // Verify URL was extracted
    let url_data = tokio::fs::read_to_string("test-cdp-url.json")
        .await
        .expect("Failed to read URL file");
    assert!(
        url_data.contains("127.0.0.1"),
        "URL should contain localhost IP"
    );

    println!("✅ CDP navigation check passed!");

    // Cleanup
    driver.close().await.expect("Failed to close browser");
    tokio::fs::remove_file("test-cdp-url.json").await.ok();
}

#[tokio::test]
async fn test_send_cdp_command_evaluate() {
    // Test the send_cdp_command API with Runtime.evaluate
    let driver = ChromeDriver::new(ConnectionMode::Sandboxed {
        chrome_path: None,
        no_sandbox: true,
        headless: true,
    })
    .await
    .expect("Failed to launch Chrome");

    // Navigate to a page first (send_cdp_command uses execute_script which needs a page)
    let script = CdpScript {
        name: "setup-page".to_string(),
        description: "Setup page for send_cdp_command test".to_string(),
        created: None,
        author: Some("Test".to_string()),
        tags: vec![],
        cdp_commands: vec![CdpCommand {
            method: "Page.navigate".to_string(),
            params: serde_json::json!({"url": "about:blank"}),
            save_as: None,
            description: Some("Navigate to blank page".to_string()),
        }],
    };
    driver
        .execute_cdp_script_direct(&script)
        .await
        .expect("Failed to setup page");

    // Test Runtime.evaluate (the only command currently supported by send_cdp_command)
    let params = serde_json::json!({
        "expression": "2 + 2"
    });

    let result = driver.send_cdp_command("Runtime.evaluate", params).await;

    // send_cdp_command only supports Runtime.evaluate currently
    assert!(
        result.is_ok(),
        "Runtime.evaluate should work via send_cdp_command: {:?}",
        result
    );
    let result_value = result.unwrap();
    println!("✅ send_cdp_command result: {}", result_value);

    driver.close().await.expect("Failed to close browser");
}

#[tokio::test]
async fn test_send_cdp_command_unsupported() {
    // Test that unsupported CDP commands return helpful error
    let driver = ChromeDriver::new(ConnectionMode::Sandboxed {
        chrome_path: None,
        no_sandbox: true,
        headless: true,
    })
    .await
    .expect("Failed to launch Chrome");

    // Try an unsupported command
    let params = serde_json::json!({
        "latitude": 37.7749,
        "longitude": -122.4194,
        "accuracy": 100
    });

    let result = driver
        .send_cdp_command("Emulation.setGeolocationOverride", params)
        .await;

    // Should fail with helpful error message
    assert!(result.is_err(), "Unsupported commands should fail");
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("not directly supported") || error_msg.contains("current_page"),
        "Error should mention using current_page()"
    );

    println!("✅ Unsupported command error: {}", error_msg);

    driver.close().await.expect("Failed to close browser");
}
