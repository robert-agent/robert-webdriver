//! Chat UI Demo
//!
//! This example demonstrates the chat UI injection feature.
//! It navigates to a webpage and injects the chat interface,
//! showing how users can provide feedback to the agent.
//!
//! Run with:
//! ```bash
//! cargo run --example chat_ui_demo
//! ```

use robert_webdriver::{ChromeDriver, ConnectionMode};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Chat UI Injection Demo ===\n");

    // Launch Chrome in non-headless mode so we can see the UI
    println!("🚀 Launching Chrome...");
    let driver = ChromeDriver::new(ConnectionMode::Sandboxed {
        chrome_path: None,
        no_sandbox: false,
        headless: false, // Run with visible UI
    })
    .await?;
    println!("✅ Chrome launched successfully\n");

    // Navigate to example.com
    println!("🌐 Navigating to example.com...");
    driver.navigate("https://example.com").await?;
    println!("✅ Navigation complete\n");

    println!("📱 Chat UI has been injected on the right sidebar!");
    println!("   You should see a chat interface on the right side of the page.\n");

    // Send a welcome message from the agent
    println!("💬 Sending welcome message to chat...");
    driver
        .send_chat_message("Hello! I'm the agent. The chat UI is now active.")
        .await?;
    driver
        .send_chat_message("You can type messages to provide feedback as I work.")
        .await?;
    println!("✅ Messages sent\n");

    // Wait a bit for user interaction
    println!("⏳ Waiting 10 seconds for you to interact with the chat...");
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Check if the user sent any messages
    println!("\n📥 Checking for user messages...");
    let messages = driver.get_chat_messages().await?;

    println!("Total messages in chat: {}", messages.len());
    for (i, msg) in messages.iter().enumerate() {
        println!(
            "  {}. [{}] {}: {}",
            i + 1,
            msg.timestamp,
            msg.sender,
            msg.text
        );
    }

    if messages.iter().any(|m| m.sender == "user") {
        println!("\n✅ User sent feedback!");
        driver.send_chat_message("Thanks for the feedback!").await?;
    } else {
        println!("\n⏭️  No user messages received (that's okay!)");
    }

    // Navigate to another page to show chat UI persists
    println!("\n🌐 Navigating to another page (httpbin.org)...");
    driver.navigate("https://httpbin.org").await?;
    println!("✅ Navigation complete");
    println!("📱 Chat UI has been re-injected on the new page!\n");

    // Send a message on the new page
    driver
        .send_chat_message("Chat UI works across different pages!")
        .await?;

    println!("⏳ Waiting 10 more seconds...");
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Demonstrate collapse/expand
    println!("\n🔽 Collapsing chat UI...");
    driver.collapse_chat().await?;
    tokio::time::sleep(Duration::from_secs(3)).await;

    println!("🔼 Expanding chat UI...");
    driver.expand_chat().await?;
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Final message
    driver
        .send_chat_message("Demo complete! Closing browser in 5 seconds...")
        .await?;
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Get final message count
    let final_messages = driver.get_chat_messages().await?;
    println!("\n📊 Final Statistics:");
    println!("   Total messages: {}", final_messages.len());
    println!(
        "   User messages: {}",
        final_messages.iter().filter(|m| m.sender == "user").count()
    );
    println!(
        "   Agent messages: {}",
        final_messages
            .iter()
            .filter(|m| m.sender == "agent")
            .count()
    );

    // Close browser
    println!("\n🧹 Closing browser...");
    driver.close().await?;
    println!("✅ Demo complete!\n");

    Ok(())
}
