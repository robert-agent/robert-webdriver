# robert-webdriver

Core browser automation library for the Robert project using Chrome DevTools Protocol (CDP).

## Overview

This library provides a high-level interface for browser automation using spider_chrome (maintained chromiumoxide fork) with Chrome DevTools Protocol. It serves as the foundation for both the CLI tool (`robert-cli`) and the future desktop application (`robert-app`).

**Key Feature**: Automatically downloads Chrome for Testing (~150MB) on first run, eliminating manual Chrome installation.

## Features

- **Auto-download Chrome**: Downloads Chrome for Testing automatically (cached at `~/.cache/robert/chrome`)
- **Browser Connection**: Launch Chrome or connect to existing Chrome via debug port
- **Headless Mode**: Run Chrome without visible window (for CI/CD)
- **CI/CD Support**: Auto-detects CI environments and configures appropriately
- **Navigation**: Navigate to URLs and track page state
- **Content Extraction**: Get page source, visible text, and element text
- **Error Handling**: Comprehensive error types with context

## Usage

### Basic Example

```rust
use robert_webdriver::{ChromeDriver, ConnectionMode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Auto-download Chrome and launch (visible window)
    let driver = ChromeDriver::launch_sandboxed().await?;

    // Navigate to a URL
    driver.navigate("https://example.com").await?;

    // Get page title
    let title = driver.title().await?;
    println!("Page title: {}", title);

    // Get page text
    let text = driver.get_page_text().await?;
    println!("Page text: {}", text);

    // Extract specific element text
    let h1 = driver.get_element_text("h1").await?;
    println!("H1 text: {}", h1);

    // Close browser
    driver.close().await?;

    Ok(())
}
```

### Advanced Examples

```rust
// Headless mode
let driver = ChromeDriver::new(ConnectionMode::Sandboxed {
    chrome_path: None,
    no_sandbox: false,
    headless: true,
}).await?;

// Custom Chrome path
let driver = ChromeDriver::launch_with_path(
    "/usr/bin/chromium".to_string(),
    false, // no_sandbox
    false, // headless
).await?;

// Connect to existing Chrome (advanced mode)
// First run: google-chrome --remote-debugging-port=9222
let driver = ChromeDriver::connect_debug_port(9222).await?;

// CI mode with auto-detection
let driver = ChromeDriver::launch_auto().await?;
```

## API Reference

### ChromeDriver

#### Connection Methods

- `launch_sandboxed() -> Result<Self>` - Auto-download and launch Chrome (visible window)
- `launch_with_path(path: String, no_sandbox: bool, headless: bool) -> Result<Self>` - Launch Chrome from specific path
- `launch_no_sandbox() -> Result<Self>` - Launch with `--no-sandbox` (Linux workaround)
- `launch_auto() -> Result<Self>` - Auto-detect CI and configure appropriately
- `connect_debug_port(port: u16) -> Result<Self>` - Connect to existing Chrome instance
- `new(mode: ConnectionMode) -> Result<Self>` - Low-level constructor with full control

#### Navigation Methods

- `navigate(&self, url: &str) -> Result<()>` - Navigate to a URL
- `current_url(&self) -> Result<String>` - Get the current page URL
- `title(&self) -> Result<String>` - Get the current page title

#### Content Extraction Methods

- `get_page_source(&self) -> Result<String>` - Get the full HTML source of the page
- `get_page_text(&self) -> Result<String>` - Get all visible text on the page
- `get_element_text(&self, selector: &str) -> Result<String>` - Get text from a specific element using CSS selector

#### Lifecycle Methods

- `close(self) -> Result<()>` - Close the browser connection

## ConnectionMode

```rust
pub enum ConnectionMode {
    Sandboxed {
        chrome_path: Option<String>,
        no_sandbox: bool,
        headless: bool,
    },
    DebugPort(u16),
}
```

## Error Types

```rust
pub enum BrowserError {
    ConnectionFailed(String),
    LaunchFailed(String),
    NavigationFailed(String),
    ElementNotFound(String),
    NoPage,
    CdpError(chromiumoxide::error::CdpError),
    Other(String),
}
```

## Testing

### Run Tests

```bash
# Run all tests (auto-downloads Chrome, visible window, 5 second delay)
cargo test --package robert-webdriver

# Run in CI mode (headless, no delay)
CI=true cargo test --package robert-webdriver

# Run with output visible
cargo test --package robert-webdriver -- --nocapture
```

Tests automatically:
- Download Chrome for Testing on first run
- Run with visible window locally (5 second delay to observe)
- Run headless in CI environments (auto-detected)
- Use `--no-sandbox` flag in CI for Linux compatibility

## Dependencies

- **spider_chrome**: Maintained chromiumoxide fork for CDP
- **spider_chromiumoxide_fetcher**: Auto-download Chrome for Testing
- **tokio**: Async runtime
- **anyhow**: Error handling
- **thiserror**: Custom error types
- **dirs**: Cache directory detection
- **futures**: Async utilities

### Dev Dependencies

- **warp**: HTTP server for integration tests (future)
- **serde_json**: JSON serialization for test data (future)

## CI/CD Integration

The library automatically detects CI environments and configures Chrome appropriately:

**Detected CI Variables:**
- `CI` (generic)
- `GITHUB_ACTIONS`
- `GITLAB_CI`
- `JENKINS_HOME`
- `CIRCLECI`

**CI Configuration:**
- Headless mode enabled
- `--no-sandbox` flag enabled (Linux AppArmor compatibility)
- No visible window or delays

## Troubleshooting

### Linux: "No usable sandbox" Error

Solution: Use `--no-sandbox` flag or run in CI mode:
```rust
let driver = ChromeDriver::launch_no_sandbox().await?;
// or
std::env::set_var("CI", "true");
let driver = ChromeDriver::launch_auto().await?;
```

### Chrome Auto-download Fails

- Check internet connection
- Verify cache directory is writable: `ls -la ~/.cache/robert/`
- Manually install Chrome and specify path via `launch_with_path()`

### macOS: Chrome Downloaded but Won't Launch

- Check Gatekeeper/security settings
- Try running Chrome manually first
- Use system Chrome via `launch_with_path()`

## Future Enhancements

- Screenshot capture
- Cookie management
- JavaScript execution
- Element interaction (click, type, etc.)
- Support for Firefox via CDP
- Custom Chrome flags configuration

## License

MIT OR Apache-2.0
