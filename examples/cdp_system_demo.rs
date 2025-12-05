//! Complete CDP System Demonstration
//!
//! This example shows the full workflow:
//! 1. Generate a CDP script using Claude AI
//! 2. Validate the generated script
//! 3. Execute the script via ChromeDriver
//! 4. Display execution results
//!
//! Prerequisites:
//! - Claude CLI installed and configured
//! - Chrome/Chromium browser installed
//!
//! Run with:
//! ```bash
//! cargo run --example cdp_system_demo
//! ```

use robert_webdriver::{CdpScriptGenerator, ChromeDriver, ConnectionMode};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== CDP Script Generation System Demo ===\n");

    // Step 1: Generate CDP script from natural language
    println!("📝 Step 1: Generating CDP script with Claude AI...");
    println!("   Description: \"Take a screenshot of example.com\"");

    let generator = CdpScriptGenerator::new().with_model("sonnet".to_string()); // Use Claude Sonnet model

    let script = match generator
        .generate_with_retry("Take a screenshot of example.com", 3)
        .await
    {
        Ok(script) => {
            println!("   ✅ Script generated successfully!");
            println!("   Name: {}", script.name);
            println!("   Commands: {}", script.cdp_commands.len());
            script
        }
        Err(e) => {
            eprintln!("   ❌ Failed to generate script: {}", e);
            eprintln!("   Note: Make sure Claude CLI is installed and configured");
            return Ok(()); // Exit gracefully for demo
        }
    };

    println!();

    // Step 2: Validate the script
    println!("🔍 Step 2: Validating generated script...");
    match script.validate() {
        Ok(_) => println!("   ✅ Script validation passed"),
        Err(e) => {
            eprintln!("   ❌ Script validation failed: {}", e);
            return Ok(());
        }
    }

    println!("   Commands in script:");
    for (i, cmd) in script.cdp_commands.iter().enumerate() {
        println!("     {}. {} - {:?}", i + 1, cmd.method, cmd.description);
    }

    println!();

    // Step 3: Save the script
    println!("💾 Step 3: Saving script to file...");
    let script_path = Path::new("demo-screenshot.json");
    script.to_file(script_path).await?;
    println!("   ✅ Saved to: {}", script_path.display());

    println!();

    // Step 4: Execute the script
    println!("🚀 Step 4: Executing CDP script...");

    // Launch Chrome in headless mode
    let driver = match ChromeDriver::new(ConnectionMode::Sandboxed {
        chrome_path: None,
        no_sandbox: false,
        headless: true,
    })
    .await
    {
        Ok(driver) => {
            println!("   ✅ Chrome launched successfully");
            driver
        }
        Err(e) => {
            eprintln!("   ❌ Failed to launch Chrome: {}", e);
            eprintln!("   Note: Make sure Chrome/Chromium is installed");
            return Ok(());
        }
    };

    // Execute the script
    let report = driver.execute_cdp_script_direct(&script).await?;

    println!();

    // Step 5: Display results
    println!("📊 Step 5: Execution Report");
    println!("   Total commands: {}", report.total_commands);
    println!("   Successful: {}", report.successful);
    println!("   Failed: {}", report.failed);
    println!("   Success rate: {:.1}%", report.success_rate());
    println!("   Duration: {:?}", report.total_duration);

    println!();

    if report.is_success() {
        println!("✅ All commands executed successfully!");
    } else {
        println!("⚠️ Some commands failed:");
        for result in &report.results {
            if let Some(error) = &result.error {
                println!("   Step {}: {}", result.step, error);
            }
        }
    }

    println!();

    // Display individual command results
    println!("📋 Detailed Results:");
    for result in &report.results {
        use robert_webdriver::cdp::CommandStatus;
        let status = match result.status {
            CommandStatus::Success => "✅ Success",
            CommandStatus::Failed => "❌ Failed",
            CommandStatus::Skipped => "⏭️  Skipped",
        };

        println!("   Step {}: {} - {}", result.step, result.method, status);
        println!("      Duration: {:?}", result.duration);

        if let Some(saved_file) = &result.saved_file {
            println!("      Output: {}", saved_file);
        }

        if let Some(error) = &result.error {
            println!("      Error: {}", error);
        }
    }

    println!();

    // Cleanup
    driver.close().await?;
    println!("🧹 Chrome closed");

    println!();
    println!("=== Demo Complete ===");
    println!();
    println!("Generated files:");
    println!("  - {}", script_path.display());
    if report.is_success() {
        println!("  - example-screenshot.png (or similar)");
    }

    println!();
    println!("Next steps:");
    println!("  - Try modifying the natural language description");
    println!("  - Experiment with different CDP commands");
    println!("  - Create your own automation scripts");
    println!("  - Use the robert-generate CLI tool:");
    println!("    cargo run --bin robert-generate -- \"Your description\" -o output.json");

    Ok(())
}
