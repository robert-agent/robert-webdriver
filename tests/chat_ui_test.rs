//! Chat UI Tests
//!
//! Tests for the chat UI injection and messaging functionality.
//!
//! NOTE: These tests are currently IGNORED because chat UI injection
//! has been disabled in favor of the Tauri app chat interface.
//! The chat UI code is kept for potential future use but is not
//! currently active in the application.

mod test_server;

use robert_webdriver::{ChromeDriver, ConnectionMode};
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
#[ignore = "Chat UI injection disabled - chat is now in Tauri app"]
async fn test_chat_ui_injection() -> anyhow::Result<()> {
    // Start local test server
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();

    // Launch Chrome in headless mode
    let driver = create_headless_driver().await?;

    // Navigate to test page
    driver.navigate(&url).await?;

    // Verify that the chat UI was injected
    let has_chat_ui = driver
        .execute_script(
            r#"
            document.getElementById('robert-chat-container') !== null
        "#,
        )
        .await?;

    assert_eq!(has_chat_ui, serde_json::json!(true));

    // Verify the chat API is available
    let has_chat_api = driver
        .execute_script(
            r#"
            typeof window.__ROBERT_CHAT_API__ !== 'undefined'
        "#,
        )
        .await?;

    assert_eq!(has_chat_api, serde_json::json!(true));

    driver.close().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "Chat UI injection disabled - chat is now in Tauri app"]
async fn test_send_agent_message() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();

    let driver = create_headless_driver().await?;
    driver.navigate(&url).await?;

    // Send a message from the agent
    driver.send_chat_message("Test message from agent").await?;

    // Verify the message was added to the chat
    let messages = driver.get_chat_messages().await?;

    // Should have at least 2 messages: welcome message + our test message
    assert!(messages.len() >= 2);

    // Check that our message is there
    let has_test_message = messages
        .iter()
        .any(|m| m.text == "Test message from agent" && m.sender == "agent");
    assert!(has_test_message);

    driver.close().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "Chat UI injection disabled - chat is now in Tauri app"]
async fn test_clear_chat_messages() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();

    let driver = create_headless_driver().await?;
    driver.navigate(&url).await?;

    // Send some messages
    driver.send_chat_message("Message 1").await?;
    driver.send_chat_message("Message 2").await?;

    // Verify messages exist
    let messages_before = driver.get_chat_messages().await?;
    assert!(messages_before.len() >= 3); // Welcome + 2 messages

    // Clear messages
    driver.clear_chat_messages().await?;

    // Verify messages are cleared
    let messages_after = driver.get_chat_messages().await?;
    assert_eq!(messages_after.len(), 0);

    driver.close().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "Chat UI injection disabled - chat is now in Tauri app"]
async fn test_chat_ui_persists_across_navigation() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();

    let driver = create_headless_driver().await?;

    // Navigate to first page
    driver.navigate(&url).await?;

    // Verify chat UI exists
    let has_chat_ui_1 = driver
        .execute_script(
            r#"
            document.getElementById('robert-chat-container') !== null
        "#,
        )
        .await?;
    assert_eq!(has_chat_ui_1, serde_json::json!(true));

    // Navigate to second page (using our local test server page2)
    let url2 = format!("{}/page2", url);
    driver.navigate(&url2).await?;

    // Verify chat UI is re-injected on new page
    let has_chat_ui_2 = driver
        .execute_script(
            r#"
            document.getElementById('robert-chat-container') !== null
        "#,
        )
        .await?;
    assert_eq!(has_chat_ui_2, serde_json::json!(true));

    driver.close().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "Chat UI injection disabled - chat is now in Tauri app"]
async fn test_chat_ui_collapse_expand() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();

    let driver = create_headless_driver().await?;
    driver.navigate(&url).await?;

    // Initially should not be collapsed
    let is_collapsed_initially = driver
        .execute_script(
            r#"
            document.getElementById('robert-chat-sidebar').classList.contains('collapsed')
        "#,
        )
        .await?;
    assert_eq!(is_collapsed_initially, serde_json::json!(false));

    // Collapse the chat
    driver.collapse_chat().await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let is_collapsed = driver
        .execute_script(
            r#"
            document.getElementById('robert-chat-sidebar').classList.contains('collapsed')
        "#,
        )
        .await?;
    assert_eq!(is_collapsed, serde_json::json!(true));

    // Expand the chat
    driver.expand_chat().await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let is_expanded = driver
        .execute_script(
            r#"
            !document.getElementById('robert-chat-sidebar').classList.contains('collapsed')
        "#,
        )
        .await?;
    assert_eq!(is_expanded, serde_json::json!(true));

    driver.close().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "Chat UI injection disabled - chat is now in Tauri app"]
async fn test_chat_ui_can_be_disabled() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();

    let mut driver = create_headless_driver().await?;

    // Disable chat UI
    driver.chat_ui_mut().disable();

    // Navigate - chat UI should not be injected
    driver.navigate(&url).await?;

    // Verify chat UI was not injected
    let has_chat_ui = driver
        .execute_script(
            r#"
            document.getElementById('robert-chat-container') !== null
        "#,
        )
        .await?;
    assert_eq!(has_chat_ui, serde_json::json!(false));

    driver.close().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "Chat UI injection disabled - chat is now in Tauri app"]
async fn test_manual_chat_ui_injection() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();

    let mut driver = create_headless_driver().await?;

    // Disable chat UI for automatic injection
    driver.chat_ui_mut().disable();

    // Navigate - chat UI should not be injected
    driver.navigate(&url).await?;

    // Verify no chat UI
    let has_chat_ui_before = driver
        .execute_script(
            r#"
            document.getElementById('robert-chat-container') !== null
        "#,
        )
        .await?;
    assert_eq!(has_chat_ui_before, serde_json::json!(false));

    // Enable chat UI and manually inject
    driver.chat_ui_mut().enable();
    driver.inject_chat_ui().await?;

    // Verify chat UI is now present
    let has_chat_ui_after = driver
        .execute_script(
            r#"
            document.getElementById('robert-chat-container') !== null
        "#,
        )
        .await?;
    assert_eq!(has_chat_ui_after, serde_json::json!(true));

    driver.close().await?;
    Ok(())
}
