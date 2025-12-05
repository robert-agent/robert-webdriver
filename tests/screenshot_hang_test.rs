//! Tests to diagnose and prevent screenshot hanging issues
//!
//! This test suite specifically tests scenarios that could cause
//! screenshot operations to hang or timeout.

mod test_server;

use robert_webdriver::{ChromeDriver, ConnectionMode};
use std::time::Duration;
use test_server::TestServer;
use tokio::time::timeout;

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

// ===== HANG PREVENTION TESTS =====

#[tokio::test]
async fn test_screenshot_with_timeout() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    driver.navigate(&url).await?;

    // Wrap screenshot in timeout to detect hangs
    let screenshot_result = timeout(Duration::from_secs(10), driver.screenshot()).await;

    match screenshot_result {
        Ok(Ok(data)) => {
            println!("✅ Screenshot completed within 10 seconds");
            assert!(data.len() > 1000, "Screenshot should have data");
        }
        Ok(Err(e)) => {
            driver.close().await?;
            anyhow::bail!("Screenshot failed with error: {}", e);
        }
        Err(_) => {
            driver.close().await?;
            anyhow::bail!("Screenshot TIMED OUT after 10 seconds - THIS IS THE HANG!");
        }
    }

    driver.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_screenshot_immediately_after_navigation() -> anyhow::Result<()> {
    // This tests the exact scenario that might be causing hangs
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    // Navigate and immediately take screenshot (no delay)
    driver.navigate(&url).await?;

    // Try to take screenshot with a timeout
    println!("🔍 Attempting screenshot immediately after navigation...");
    let start = std::time::Instant::now();

    let screenshot_result = timeout(Duration::from_secs(15), driver.screenshot()).await;

    let elapsed = start.elapsed();
    println!("⏱️  Screenshot operation took: {:?}", elapsed);

    match screenshot_result {
        Ok(Ok(data)) => {
            println!("✅ Screenshot succeeded in {:?}", elapsed);
            assert!(data.len() > 1000, "Screenshot should have data");

            if elapsed > Duration::from_secs(5) {
                println!(
                    "⚠️  WARNING: Screenshot took longer than expected ({:?})",
                    elapsed
                );
            }
        }
        Ok(Err(e)) => {
            driver.close().await?;
            anyhow::bail!("Screenshot failed: {}", e);
        }
        Err(_) => {
            driver.close().await?;
            anyhow::bail!("Screenshot HUNG for 15+ seconds - navigation may not be complete!");
        }
    }

    driver.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_screenshot_to_file_with_timeout() -> anyhow::Result<()> {
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    driver.navigate(&url).await?;

    let temp_dir = std::env::temp_dir();
    let screenshot_path = temp_dir.join("test-hang-screenshot.png");
    let _ = tokio::fs::remove_file(&screenshot_path).await;

    println!("🔍 Testing screenshot_to_file with timeout...");
    let start = std::time::Instant::now();

    let result = timeout(
        Duration::from_secs(10),
        driver.screenshot_to_file(&screenshot_path),
    )
    .await;

    let elapsed = start.elapsed();
    println!("⏱️  screenshot_to_file took: {:?}", elapsed);

    match result {
        Ok(Ok(())) => {
            println!("✅ screenshot_to_file succeeded in {:?}", elapsed);
            assert!(screenshot_path.exists(), "Screenshot file should exist");
            tokio::fs::remove_file(&screenshot_path).await?;
        }
        Ok(Err(e)) => {
            driver.close().await?;
            anyhow::bail!("screenshot_to_file failed: {}", e);
        }
        Err(_) => {
            driver.close().await?;
            anyhow::bail!("screenshot_to_file HUNG for 10+ seconds!");
        }
    }

    driver.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_multiple_screenshots_rapid_succession() -> anyhow::Result<()> {
    // Test for race conditions when taking multiple screenshots quickly
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    driver.navigate(&url).await?;

    println!("🔍 Taking 5 screenshots in rapid succession...");
    let start = std::time::Instant::now();

    for i in 1..=5 {
        println!("  📸 Screenshot {}/5...", i);
        let screenshot_start = std::time::Instant::now();

        let result = timeout(Duration::from_secs(5), driver.screenshot()).await;

        match result {
            Ok(Ok(data)) => {
                println!(
                    "    ✅ Screenshot {} completed in {:?}",
                    i,
                    screenshot_start.elapsed()
                );
                assert!(data.len() > 1000, "Screenshot {} should have data", i);
            }
            Ok(Err(e)) => {
                driver.close().await?;
                anyhow::bail!("Screenshot {} failed: {}", i, e);
            }
            Err(_) => {
                driver.close().await?;
                anyhow::bail!("Screenshot {} HUNG!", i);
            }
        }
    }

    let total_elapsed = start.elapsed();
    println!("✅ All 5 screenshots completed in {:?}", total_elapsed);

    driver.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_screenshot_after_javascript_execution() -> anyhow::Result<()> {
    // Test screenshot after DOM manipulation
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    driver.navigate(&url).await?;

    // Execute some JavaScript
    driver
        .execute_script("document.body.style.backgroundColor = 'red'")
        .await?;

    // Give a small delay for rendering
    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("🔍 Taking screenshot after JavaScript execution...");
    let result = timeout(Duration::from_secs(10), driver.screenshot()).await;

    match result {
        Ok(Ok(data)) => {
            println!("✅ Screenshot after JS execution succeeded");
            assert!(data.len() > 1000, "Screenshot should have data");
        }
        Ok(Err(e)) => {
            driver.close().await?;
            anyhow::bail!("Screenshot failed: {}", e);
        }
        Err(_) => {
            driver.close().await?;
            anyhow::bail!("Screenshot HUNG after JS execution!");
        }
    }

    driver.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_screenshot_on_slow_loading_page() -> anyhow::Result<()> {
    // Test screenshot when page might still be loading resources
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    // Start navigation but don't wait for complete load
    driver.navigate(&url).await?;

    // The navigate() function already waits for load event,
    // but let's test if screenshot still works
    println!("🔍 Taking screenshot on potentially slow page...");
    let result = timeout(Duration::from_secs(15), driver.screenshot()).await;

    match result {
        Ok(Ok(data)) => {
            println!("✅ Screenshot on slow page succeeded");
            assert!(data.len() > 1000, "Screenshot should have data");
        }
        Ok(Err(e)) => {
            driver.close().await?;
            anyhow::bail!("Screenshot failed: {}", e);
        }
        Err(_) => {
            driver.close().await?;
            anyhow::bail!("Screenshot HUNG on slow loading page!");
        }
    }

    driver.close().await?;
    Ok(())
}

// ===== DIAGNOSTIC TESTS =====

#[tokio::test]
async fn test_diagnose_screenshot_performance() -> anyhow::Result<()> {
    // Measure screenshot performance to identify slowness
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    driver.navigate(&url).await?;

    let mut timings = Vec::new();

    println!("📊 Measuring screenshot performance over 10 iterations...");

    for i in 1..=10 {
        let start = std::time::Instant::now();
        let _ = driver.screenshot().await?;
        let elapsed = start.elapsed();
        timings.push(elapsed);
        println!("  Screenshot {}: {:?}", i, elapsed);
    }

    // Calculate statistics
    let total: Duration = timings.iter().sum();
    let avg = total / timings.len() as u32;
    let max = timings.iter().max().unwrap();
    let min = timings.iter().min().unwrap();

    println!("\n📈 Performance Statistics:");
    println!("  Average: {:?}", avg);
    println!("  Min: {:?}", min);
    println!("  Max: {:?}", max);

    // Warn if screenshots are too slow
    if avg > Duration::from_secs(2) {
        println!(
            "⚠️  WARNING: Average screenshot time ({:?}) is slower than expected!",
            avg
        );
    }

    if *max > Duration::from_secs(5) {
        println!(
            "⚠️  WARNING: Max screenshot time ({:?}) indicates potential hang risk!",
            max
        );
    }

    driver.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_screenshot_with_explicit_page_ready_check() -> anyhow::Result<()> {
    // Test with explicit readyState checking before screenshot
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    driver.navigate(&url).await?;

    // Check if page is ready
    println!("🔍 Checking page ready state...");
    let ready_state: serde_json::Value = driver.execute_script("document.readyState").await?;
    println!("  Page readyState: {}", ready_state);

    // Take screenshot
    println!("📸 Taking screenshot after ready check...");
    let result = timeout(Duration::from_secs(10), driver.screenshot()).await;

    match result {
        Ok(Ok(data)) => {
            println!("✅ Screenshot succeeded");
            assert!(data.len() > 1000, "Screenshot should have data");
        }
        Ok(Err(e)) => {
            driver.close().await?;
            anyhow::bail!("Screenshot failed: {}", e);
        }
        Err(_) => {
            driver.close().await?;
            anyhow::bail!("Screenshot HUNG even with ready check!");
        }
    }

    driver.close().await?;
    Ok(())
}

// ===== STRESS TESTS =====

#[tokio::test]
async fn test_screenshot_stress_concurrent_operations() -> anyhow::Result<()> {
    // Test screenshot while other operations are happening
    let server = TestServer::start().await;
    server.wait_ready().await?;
    let url = server.url();
    let driver = create_headless_driver().await?;

    driver.navigate(&url).await?;

    println!("🔍 Taking screenshot while executing concurrent operations...");

    // Spawn multiple operations concurrently
    let screenshot_task = async { timeout(Duration::from_secs(10), driver.screenshot()).await };

    let js_task = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        driver.execute_script("2 + 2").await
    };

    let title_task = async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        driver.title().await
    };

    // Run all tasks concurrently
    let (screenshot_result, js_result, title_result) =
        tokio::join!(screenshot_task, js_task, title_task);

    // Check results
    match screenshot_result {
        Ok(Ok(data)) => {
            println!("✅ Screenshot completed during concurrent operations");
            assert!(data.len() > 1000);
        }
        Ok(Err(e)) => {
            driver.close().await?;
            anyhow::bail!("Screenshot failed: {}", e);
        }
        Err(_) => {
            driver.close().await?;
            anyhow::bail!("Screenshot timed out!");
        }
    }

    assert!(js_result.is_ok(), "JS execution should succeed");
    assert!(title_result.is_ok(), "Title retrieval should succeed");

    driver.close().await?;
    Ok(())
}
