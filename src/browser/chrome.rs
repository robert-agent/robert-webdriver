// spider_chrome re-exports chromiumoxide API
use crate::error::{BrowserError, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide_fetcher::{BrowserFetcher, BrowserFetcherOptions};
use futures::StreamExt;
use std::path::{Path, PathBuf};

pub struct ChromeDriver {
    browser: Browser,
    temp_dir: Option<PathBuf>,
    chat_ui: super::chat::ChatUI,
}

/// Connection mode for Chrome browser
pub enum ConnectionMode {
    /// Sandboxed mode - launches Chrome using system installation
    Sandboxed {
        chrome_path: Option<String>,
        no_sandbox: bool,
        headless: bool,
    },
    /// Advanced mode - connects to existing Chrome on debug port
    DebugPort(u16),
}

impl ChromeDriver {
    /// Helper method to get the current active page, excluding Chrome's new-tab-page
    async fn get_active_page(&self) -> Result<chromiumoxide::page::Page> {
        let pages = self.browser.pages().await?;

        // Filter out chrome://new-tab-page/ and return the first real page
        // If no real pages exist, return the last page (most recently created)
        for page in pages.iter() {
            if let Ok(Some(url)) = page.url().await {
                if !url.starts_with("chrome://") {
                    return Ok(page.clone());
                }
            }
        }

        // No non-chrome page found, try to use any existing page
        if let Some(page) = pages.last() {
            return Ok(page.clone());
        }

        // No pages at all, create one
        self.browser
            .new_page("about:blank")
            .await
            .map_err(|e| BrowserError::Other(format!("Failed to create page: {}", e)))
    }

    /// Launch Chrome in sandboxed mode (uses system Chrome)
    pub async fn launch_sandboxed() -> Result<Self> {
        Self::new(ConnectionMode::Sandboxed {
            chrome_path: None,
            no_sandbox: false,
            headless: false,
        })
        .await
    }

    /// Launch Chrome in sandboxed mode with custom path
    pub async fn launch_with_path(
        chrome_path: String,
        no_sandbox: bool,
        headless: bool,
    ) -> Result<Self> {
        Self::new(ConnectionMode::Sandboxed {
            chrome_path: Some(chrome_path),
            no_sandbox,
            headless,
        })
        .await
    }

    /// Launch Chrome with no-sandbox flag (Linux workaround for AppArmor restrictions)
    pub async fn launch_no_sandbox() -> Result<Self> {
        Self::new(ConnectionMode::Sandboxed {
            chrome_path: None,
            no_sandbox: true,
            headless: false,
        })
        .await
    }

    /// Launch Chrome with auto-detection for CI environments
    pub async fn launch_auto() -> Result<Self> {
        let is_ci = std::env::var("CI").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
            || std::env::var("GITLAB_CI").is_ok()
            || std::env::var("JENKINS_HOME").is_ok()
            || std::env::var("CIRCLECI").is_ok();

        Self::new(ConnectionMode::Sandboxed {
            chrome_path: None,
            no_sandbox: is_ci, // CI environments typically need --no-sandbox
            headless: is_ci,   // CI environments should run headless
        })
        .await
    }

    /// Connect to existing Chrome on debug port (advanced mode)
    pub async fn connect_debug_port(port: u16) -> Result<Self> {
        Self::new(ConnectionMode::DebugPort(port)).await
    }

    /// Create new ChromeDriver with specified connection mode
    pub async fn new(mode: ConnectionMode) -> Result<Self> {
        let (browser, temp_dir) = match mode {
            ConnectionMode::Sandboxed {
                chrome_path,
                no_sandbox,
                headless,
            } => {
                // Create a unique temporary directory for this browser instance
                // This ensures parallel tests don't share profile data
                // Using timestamp in nanoseconds ensures uniqueness across threads
                let unique_id = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                let temp_dir = std::env::temp_dir().join(format!("chromiumoxide-{}", unique_id));
                std::fs::create_dir_all(&temp_dir).map_err(|e| {
                    BrowserError::LaunchFailed(format!("Failed to create temp directory: {}", e))
                })?;

                // Launch Chrome with visible UI or headless
                let mut config = if headless {
                    BrowserConfig::builder()
                } else {
                    BrowserConfig::builder().with_head()
                };

                // Set unique user data directory for test isolation
                config = config.user_data_dir(&temp_dir);

                // Add no-sandbox flag if requested (Linux AppArmor workaround)
                if no_sandbox {
                    config = config.arg("--no-sandbox");
                }

                // Use custom Chrome path if provided, otherwise try auto-download
                if let Some(path) = chrome_path {
                    config = config.chrome_executable(path);
                } else {
                    // Try to auto-download Chrome if not found
                    match Self::ensure_chrome_installed().await {
                        Ok(path) => {
                            config = config.chrome_executable(path);
                        }
                        Err(e) => {
                            // If auto-download fails, let chromiumoxide try to find system Chrome
                            eprintln!(
                                "Note: Auto-download failed ({}), trying system Chrome...",
                                e
                            );
                        }
                    }
                }

                let (browser, mut handler) = Browser::launch(config.build().map_err(|e| {
                    BrowserError::LaunchFailed(format!(
                        "{}. \n\n\
                                 Chrome not found. You can:\n\
                                 - Install Chrome: https://www.google.com/chrome/\n\
                                 - Ubuntu/Debian: sudo apt install chromium-browser\n\
                                 - Fedora: sudo dnf install chromium\n\
                                 - macOS: brew install --cask google-chrome\n\
                                 - Or specify path: --chrome-path /path/to/chrome\n\
                                 - Linux sandbox issue? Try: --no-sandbox",
                        e
                    ))
                })?)
                .await
                .map_err(|e| {
                    BrowserError::LaunchFailed(format!(
                        "{}. \n\n\
                         Chrome not found. You can:\n\
                         - Install Chrome: https://www.google.com/chrome/\n\
                         - Ubuntu/Debian: sudo apt install chromium-browser\n\
                         - Fedora: sudo dnf install chromium\n\
                         - macOS: brew install --cask google-chrome\n\
                         - Or specify path: --chrome-path /path/to/chrome\n\
                         - Linux sandbox issue? Try: --no-sandbox",
                        e
                    ))
                })?;

                // Spawn handler task
                tokio::spawn(async move {
                    while (handler.next().await).is_some() {
                        // Handle browser events
                    }
                });

                (browser, Some(temp_dir))
            }
            ConnectionMode::DebugPort(port) => {
                let url = format!("http://localhost:{}", port);
                let (browser, mut handler) = Browser::connect(&url).await.map_err(|e| {
                    BrowserError::ConnectionFailed(format!(
                        "Failed to connect to Chrome on port {}. \
                             Make sure Chrome is running with --remote-debugging-port={}: {}",
                        port, port, e
                    ))
                })?;

                // Spawn handler task
                tokio::spawn(async move {
                    while (handler.next().await).is_some() {
                        // Handle browser events
                    }
                });

                (browser, None)
            }
        };

        Ok(Self {
            browser,
            temp_dir,
            chat_ui: super::chat::ChatUI::new(),
        })
    }

    /// Navigate to a URL
    pub async fn navigate(&self, url: &str) -> Result<()> {
        use chromiumoxide::cdp::browser_protocol::page::NavigateParams;

        // Normalize URL - add https:// if no protocol specified
        let normalized_url = if !url.starts_with("http://")
            && !url.starts_with("https://")
            && !url.starts_with("file://")
            && !url.starts_with("about:")
            && !url.starts_with("data:")
        {
            eprintln!("🔧 Normalizing URL: {} -> https://{}", url, url);
            format!("https://{}", url)
        } else {
            url.to_string()
        };

        eprintln!("🌐 Starting navigation to: {}", normalized_url);

        // Always get all pages and work with the first one (or create if none exist)
        let mut pages = self.browser.pages().await?;
        eprintln!("📄 Found {} browser page(s)", pages.len());

        // Close all but the first page to ensure we only have one page
        for (i, p) in pages.iter().enumerate() {
            if i > 0 {
                eprintln!("🗑️  Closing extra page {}", i);
                let _ = p
                    .execute(
                        chromiumoxide::cdp::browser_protocol::target::CloseTargetParams::new(
                            p.target_id().clone(),
                        ),
                    )
                    .await;
            }
        }

        // Refresh page list after closing
        pages = self.browser.pages().await?;

        let page = if let Some(page) = pages.first() {
            eprintln!("✓ Using existing page");
            // Use the first (and now only) page
            page.clone()
        } else {
            eprintln!("➕ Creating new page");
            // No page exists, create a new one
            self.browser
                .new_page("about:blank")
                .await
                .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?
        };

        // Use CDP Page.navigate command directly (more reliable than goto())
        // This is what the working headless_integration tests use
        eprintln!("🚀 Executing CDP Navigate command...");
        let params = NavigateParams::builder()
            .url(&normalized_url)
            .build()
            .map_err(|e| {
                BrowserError::NavigationFailed(format!("Invalid URL {}: {}", normalized_url, e))
            })?;

        let response = page.execute(params).await.map_err(|e| {
            eprintln!("❌ CDP Navigate failed: {}", e);
            let error_str = e.to_string();

            // Detect "oneshot canceled" error which indicates browser connection is dead
            if error_str.contains("oneshot canceled") {
                BrowserError::NavigationFailed(
                    "Browser connection lost. The browser may have been closed or crashed. Please launch the browser again.".to_string()
                )
            } else {
                BrowserError::NavigationFailed(format!(
                    "Failed to navigate to {}: {}",
                    normalized_url, e
                ))
            }
        })?;

        // Check if navigation was successful
        let nav_result = response.result;
        if let Some(error_text) = nav_result.error_text {
            eprintln!("❌ Navigation error from browser: {}", error_text);
            return Err(BrowserError::NavigationFailed(format!(
                "Navigation error: {}",
                error_text
            )));
        }

        eprintln!("📡 Frame ID: {:?}", nav_result.frame_id);
        if let Some(loader_id) = &nav_result.loader_id {
            eprintln!("📦 Loader ID: {:?}", loader_id);
        }

        // Wait for the page to load using Page.loadEventFired with timeout
        // This is more reliable than arbitrary sleeps
        eprintln!("⏳ Waiting for page load event (30s timeout)...");
        use chromiumoxide::cdp::browser_protocol::page::EventLoadEventFired;

        let load_result = tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            page.event_listener::<EventLoadEventFired>(),
        )
        .await;

        match load_result {
            Ok(Ok(_)) => {
                eprintln!("✓ Page load event fired successfully");
            }
            Ok(Err(e)) => {
                eprintln!("⚠️  Warning: Could not wait for load event: {}", e);
            }
            Err(_) => {
                eprintln!("❌ Timeout waiting for page load event after 30s");
                return Err(BrowserError::NavigationFailed(format!(
                    "Request timed out. \n\
                    Possible causes:\n\
                    - Network connectivity issues\n\
                    - URL is unreachable: {}\n\
                    - Firewall or proxy blocking the connection\n\
                    - Browser unable to resolve DNS\n\
                    \n\
                    Debug: Check if you can access {} in your regular browser.",
                    normalized_url, normalized_url
                )));
            }
        }

        // Additional small delay for page state to stabilize
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        eprintln!("✓ Navigation completed successfully");

        // NOTE: Chat UI injection disabled - chat is now in the Tauri app

        Ok(())
    }

    /// Get current URL
    pub async fn current_url(&self) -> Result<String> {
        let page = self.get_active_page().await?;

        let url = page
            .url()
            .await
            .map_err(|e| BrowserError::Other(e.to_string()))?
            .ok_or(BrowserError::NoPage)?;

        Ok(url)
    }

    /// Get page title
    pub async fn title(&self) -> Result<String> {
        let page = self.get_active_page().await?;

        let title = page
            .get_title()
            .await
            .map_err(|e| BrowserError::Other(e.to_string()))?
            .ok_or(BrowserError::NoPage)?;

        Ok(title)
    }

    /// Get page HTML source
    pub async fn get_page_source(&self) -> Result<String> {
        let page = self.get_active_page().await?;

        let html = page
            .content()
            .await
            .map_err(|e| BrowserError::Other(e.to_string()))?;

        Ok(html)
    }

    /// Get visible page text
    pub async fn get_page_text(&self) -> Result<String> {
        let page = self.get_active_page().await?;

        let text = page
            .find_element("body")
            .await
            .map_err(|_e| BrowserError::ElementNotFound("body".to_string()))?
            .inner_text()
            .await
            .map_err(|_e| BrowserError::ElementNotFound("body".to_string()))?
            .ok_or(BrowserError::ElementNotFound("body".to_string()))?;

        Ok(text)
    }

    /// Get text from specific element
    pub async fn get_element_text(&self, selector: &str) -> Result<String> {
        let page = self.get_active_page().await?;

        let text = page
            .find_element(selector)
            .await
            .map_err(|_e| BrowserError::ElementNotFound(selector.to_string()))?
            .inner_text()
            .await
            .map_err(|_e| BrowserError::ElementNotFound(selector.to_string()))?
            .ok_or(BrowserError::ElementNotFound(selector.to_string()))?;

        Ok(text)
    }

    /// Take a screenshot of the current page
    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        let page = self.get_active_page().await?;

        let screenshot = page
            .screenshot(chromiumoxide::page::ScreenshotParams::default())
            .await
            .map_err(|e| BrowserError::Other(format!("Failed to take screenshot: {}", e)))?;

        Ok(screenshot)
    }

    /// Take a screenshot and save to file
    pub async fn screenshot_to_file(&self, path: &Path) -> Result<()> {
        let screenshot_data = self.screenshot().await?;

        tokio::fs::write(path, screenshot_data)
            .await
            .map_err(|e| BrowserError::Other(format!("Failed to write screenshot: {}", e)))?;

        Ok(())
    }

    /// Capture a VisualDom snapshot with layout, style, and image information
    ///
    /// VisualDom is a custom format we created that combines Chrome DevTools Protocol's
    /// DOMSnapshot.captureSnapshot with embedded base64 images. This provides a structured
    /// representation of the DOM including computed styles, layout bounds, text content,
    /// and images that AI agents can analyze without expensive OCR.
    ///
    /// # Arguments
    /// * `computed_styles` - Specific CSS properties to capture (empty = all styles)
    /// * `include_dom_rects` - Whether to include offsetRects, scrollRects, clientRects
    /// * `include_paint_order` - Whether to include paint order information
    /// * `include_images` - Whether to include image data as base64
    ///
    /// # Returns
    /// JSON response containing:
    /// - documents: Array of document snapshots with DOM tree, layout, and text
    /// - strings: String table (all strings indexed for efficiency)
    /// - images: (if include_images=true) Array of {src, data, width, height} for all images
    pub async fn capture_visual_dom(
        &self,
        computed_styles: &[String],
        include_dom_rects: bool,
        include_paint_order: bool,
        include_images: bool,
    ) -> Result<serde_json::Value> {
        let page = self.get_active_page().await?;

        // Execute CDP DOMSnapshot.captureSnapshot command
        let result = page
            .execute(
                chromiumoxide::cdp::browser_protocol::dom_snapshot::CaptureSnapshotParams {
                    computed_styles: computed_styles.to_vec(),
                    include_dom_rects: Some(include_dom_rects),
                    include_paint_order: Some(include_paint_order),
                    include_blended_background_colors: Some(false),
                    include_text_color_opacities: Some(false),
                },
            )
            .await
            .map_err(|e| BrowserError::Other(format!("Failed to capture DOM snapshot: {}", e)))?;

        // Extract the inner result and serialize to JSON
        let mut snapshot = serde_json::to_value(result.result)
            .map_err(|e| BrowserError::Other(format!("Failed to serialize snapshot: {}", e)))?;

        // If images requested, extract and embed them as base64
        if include_images {
            let images = self.extract_images_as_base64().await?;
            if let Some(obj) = snapshot.as_object_mut() {
                obj.insert("images".to_string(), images);
            }
        }

        Ok(snapshot)
    }

    /// Extract all images from the page and convert to base64
    ///
    /// Returns an array of objects with {src, data, width, height, alt}
    async fn extract_images_as_base64(&self) -> Result<serde_json::Value> {
        let js_code = r#"
            (async () => {
                const images = Array.from(document.querySelectorAll('img'));
                const results = [];

                for (const img of images) {
                    try {
                        // Skip invisible images
                        const rect = img.getBoundingClientRect();
                        if (rect.width === 0 || rect.height === 0) continue;

                        // Create a canvas to convert image to base64
                        const canvas = document.createElement('canvas');
                        canvas.width = img.naturalWidth || img.width;
                        canvas.height = img.naturalHeight || img.height;

                        const ctx = canvas.getContext('2d');
                        ctx.drawImage(img, 0, 0);

                        // Convert to base64 (will be data URI format)
                        const dataUrl = canvas.toDataURL('image/png');

                        results.push({
                            src: img.src || img.currentSrc,
                            data: dataUrl,
                            width: canvas.width,
                            height: canvas.height,
                            alt: img.alt || '',
                            x: rect.x,
                            y: rect.y,
                            displayWidth: rect.width,
                            displayHeight: rect.height,
                        });
                    } catch (e) {
                        // Skip images that can't be converted (CORS, etc.)
                        // But still record their metadata
                        const rect = img.getBoundingClientRect();
                        results.push({
                            src: img.src || img.currentSrc,
                            data: null,
                            width: img.naturalWidth || img.width,
                            height: img.naturalHeight || img.height,
                            alt: img.alt || '',
                            x: rect.x,
                            y: rect.y,
                            displayWidth: rect.width,
                            displayHeight: rect.height,
                            error: 'CORS or load error',
                        });
                    }
                }

                return results;
            })()
        "#;

        self.execute_script(js_code).await
    }

    /// Execute arbitrary JavaScript in the page context
    pub async fn execute_script(&self, script: &str) -> Result<serde_json::Value> {
        let page = self.get_active_page().await?;

        let result = page
            .evaluate(script)
            .await
            .map_err(|e| BrowserError::Other(format!("Script execution failed: {}", e)))?;

        Ok(result.into_value().unwrap_or(serde_json::Value::Null))
    }

    /// Execute JavaScript and return a specific type
    pub async fn execute_script_typed<T: serde::de::DeserializeOwned>(
        &self,
        script: &str,
    ) -> Result<T> {
        let page = self.get_active_page().await?;

        let result = page
            .evaluate(script)
            .await
            .map_err(|e| BrowserError::Other(format!("Script execution failed: {}", e)))?;

        result
            .into_value()
            .map_err(|e| BrowserError::Other(format!("Failed to deserialize result: {}", e)))
    }

    /// Send a raw CDP (Chrome DevTools Protocol) command using JSON
    ///
    /// This is a convenience wrapper for sending arbitrary CDP commands.
    /// The method should be in the format "Domain.method" (e.g., "Page.captureScreenshot", "Network.getCookies")
    ///
    /// For typed/safe CDP usage, use `driver.current_page()` to get the Page and use chromiumoxide's typed CDP methods.
    ///
    /// # Note on JavaScript Execution
    /// For executing JavaScript, use `execute_script()` instead - it's simpler and more reliable.
    ///
    /// # Common CDP Commands
    /// - `Page.captureScreenshot` - Take screenshots with custom options
    /// - `Emulation.setDeviceMetricsOverride` - Mobile device emulation
    /// - `Network.getCookies` - Get all cookies
    /// - `Performance.getMetrics` - Get performance metrics
    /// - `DOM.getDocument` - Get DOM tree
    /// - `Input.dispatchMouseEvent` - Simulate mouse events
    /// - `Input.dispatchKeyEvent` - Simulate keyboard events
    ///
    /// # Example - Runtime.evaluate (Supported)
    /// ```no_run
    /// use serde_json::json;
    /// use robert_webdriver::{ChromeDriver, ConnectionMode};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let driver = ChromeDriver::new(ConnectionMode::Sandboxed {
    ///     chrome_path: None,
    ///     no_sandbox: true,
    ///     headless: true,
    /// }).await?;
    ///
    /// let params = json!({"expression": "2 + 2"});
    /// let result = driver.send_cdp_command("Runtime.evaluate", params).await?;
    /// println!("Result: {}", result);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Note
    /// For other CDP commands (Emulation, Network, etc.), use `driver.current_page()` to access
    /// chromiumoxide's typed CDP API. See tests in `tests/cdp_execution_test.rs` for examples.
    pub async fn send_cdp_command(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // For now, we'll implement common use cases via JavaScript
        // This is a limitation of chromiumoxide's typed API
        // TODO: Implement proper CDP command execution when chromiumoxide supports it

        // Special handling for common commands
        match method {
            "Runtime.evaluate" => {
                // Use our built-in execute_script for this
                if let Some(expression) = params.get("expression").and_then(|v| v.as_str()) {
                    let result = self.execute_script(expression).await?;
                    Ok(serde_json::json!({
                        "result": {
                            "type": "object",
                            "value": result
                        }
                    }))
                } else {
                    Err(BrowserError::Other(
                        "Runtime.evaluate requires 'expression' parameter".to_string(),
                    ))
                }
            }
            _ => {
                // For other CDP commands, user should use current_page() and chromiumoxide types
                Err(BrowserError::Other(format!(
                    "CDP command '{}' not directly supported. Use driver.current_page() and chromiumoxide::cdp types for typed CDP access. \
                    For JavaScript execution, use driver.execute_script(). \
                    See documentation for examples.",
                    method
                )))
            }
        }
    }

    /// Get access to the underlying Browser for advanced CDP usage
    pub fn browser(&self) -> &Browser {
        &self.browser
    }

    /// Get access to the current page for advanced operations
    /// Returns the active page (excluding Chrome's new-tab-page)
    pub async fn current_page(&self) -> Result<chromiumoxide::page::Page> {
        self.get_active_page().await
    }

    /// Check if the browser is still alive and responsive
    /// Returns true if the browser connection is healthy, false otherwise
    pub async fn is_alive(&self) -> bool {
        // Try to get pages - if this fails, the browser is dead
        match self.browser.pages().await {
            Ok(pages) => {
                // If we can get pages, try a simple operation to verify connection
                if let Some(page) = pages.first() {
                    // Try to get the URL - if this times out or fails, browser is dead
                    matches!(
                        tokio::time::timeout(tokio::time::Duration::from_secs(2), page.url()).await,
                        Ok(Ok(_))
                    )
                } else {
                    // No pages but browser responded - still alive
                    true
                }
            }
            Err(_) => false,
        }
    }

    /// Close the browser connection
    pub async fn close(self) -> Result<()> {
        self.browser
            .close()
            .await
            .map_err(|e| BrowserError::Other(e.to_string()))?;
        Ok(())
    }

    /// Ensure Chrome is installed, downloading if necessary
    async fn ensure_chrome_installed() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| BrowserError::Other("Cannot determine cache directory".to_string()))?
            .join("robert")
            .join("chrome");

        // Create cache directory if it doesn't exist
        tokio::fs::create_dir_all(&cache_dir)
            .await
            .map_err(|e| BrowserError::Other(format!("Failed to create cache dir: {}", e)))?;

        // Check if Chrome already downloaded
        let revision_info_path = cache_dir.join(".downloaded");
        if revision_info_path.exists() {
            // Chrome already downloaded, find the executable
            if let Some(executable) = Self::find_chrome_in_cache(&cache_dir).await {
                return Ok(executable);
            }
        }

        // Download Chrome
        eprintln!("📥 Downloading Chrome for Testing (first time only, ~150MB)...");
        let fetcher = BrowserFetcher::new(
            BrowserFetcherOptions::builder()
                .with_path(&cache_dir)
                .build()
                .map_err(|e| BrowserError::Other(format!("Fetcher config failed: {}", e)))?,
        );

        let info = fetcher
            .fetch()
            .await
            .map_err(|e| BrowserError::Other(format!("Chrome download failed: {}", e)))?;

        // Mark as downloaded
        tokio::fs::write(&revision_info_path, "downloaded")
            .await
            .map_err(|e| BrowserError::Other(format!("Failed to write marker: {}", e)))?;

        eprintln!("✅ Chrome downloaded successfully!");

        Ok(info.executable_path)
    }

    /// Find Chrome executable in cache directory
    async fn find_chrome_in_cache(cache_dir: &Path) -> Option<PathBuf> {
        // Look for Chrome executable in various possible locations
        let possible_paths = vec![
            cache_dir.join("chrome"),
            cache_dir.join("chrome.exe"),
            cache_dir.join("Google Chrome.app/Contents/MacOS/Google Chrome"),
            cache_dir.join("chrome-linux/chrome"),
            cache_dir.join("chrome-mac/Chromium.app/Contents/MacOS/Chromium"),
            cache_dir.join("chrome-win/chrome.exe"),
        ];

        for path in possible_paths {
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    /// Execute a CDP script from a JSON file
    ///
    /// This method loads a CDP script and executes it via the CDP executor.
    /// Scripts are JSON files containing Chrome DevTools Protocol commands.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use robert_webdriver::{ChromeDriver, ConnectionMode};
    /// use std::path::Path;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let driver = ChromeDriver::new(ConnectionMode::Sandboxed {
    ///     chrome_path: None,
    ///     no_sandbox: true,
    ///     headless: true,
    /// }).await?;
    ///
    /// let report = driver.execute_cdp_script(Path::new("script.json")).await?;
    /// println!("Executed {} commands, {} successful",
    ///     report.total_commands, report.successful);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See `tests/cdp_script_execution_test.rs::test_execute_cdp_script_from_file` for a complete example.
    pub async fn execute_cdp_script(
        &self,
        script_path: &std::path::Path,
    ) -> Result<crate::cdp::ExecutionReport> {
        // Load script from file
        let script = crate::cdp::CdpScript::from_file(script_path)
            .await
            .map_err(|e| BrowserError::Other(format!("Failed to load script: {}", e)))?;

        // Get current page
        let page = self.current_page().await?;

        // Create executor and run script
        let executor = crate::cdp::CdpExecutor::new(page);
        executor
            .execute_script(&script)
            .await
            .map_err(|e| BrowserError::Other(format!("Script execution failed: {}", e)))
    }

    /// Execute a CDP script from an in-memory CdpScript struct
    ///
    /// Useful when scripts are generated programmatically (e.g., by Claude)
    /// rather than loaded from files.
    pub async fn execute_cdp_script_direct(
        &self,
        script: &crate::cdp::CdpScript,
    ) -> Result<crate::cdp::ExecutionReport> {
        let page = self.current_page().await?;
        let executor = crate::cdp::CdpExecutor::new(page);
        executor
            .execute_script(script)
            .await
            .map_err(|e| BrowserError::Other(format!("Script execution failed: {}", e)))
    }

    // ===== CHAT UI METHODS =====

    /// Get a reference to the ChatUI manager
    pub fn chat_ui(&self) -> &super::chat::ChatUI {
        &self.chat_ui
    }

    /// Get a mutable reference to the ChatUI manager
    pub fn chat_ui_mut(&mut self) -> &mut super::chat::ChatUI {
        &mut self.chat_ui
    }

    /// Send a message from the agent to the chat UI
    pub async fn send_chat_message(&self, message: &str) -> Result<()> {
        let page = self.current_page().await?;
        self.chat_ui.send_agent_message(&page, message).await
    }

    /// Get all messages from the chat UI
    pub async fn get_chat_messages(&self) -> Result<Vec<super::chat::ChatMessage>> {
        let page = self.current_page().await?;
        self.chat_ui.get_messages(&page).await
    }

    /// Clear all messages from the chat UI
    pub async fn clear_chat_messages(&self) -> Result<()> {
        let page = self.current_page().await?;
        self.chat_ui.clear_messages(&page).await
    }

    /// Manually inject the chat UI (useful if it was disabled during construction)
    pub async fn inject_chat_ui(&self) -> Result<()> {
        let page = self.current_page().await?;
        self.chat_ui.inject(&page).await
    }

    /// Collapse the chat UI sidebar
    pub async fn collapse_chat(&self) -> Result<()> {
        let page = self.current_page().await?;
        self.chat_ui.collapse(&page).await
    }

    /// Expand the chat UI sidebar
    pub async fn expand_chat(&self) -> Result<()> {
        let page = self.current_page().await?;
        self.chat_ui.expand(&page).await
    }

    /// Position the browser window
    ///
    /// Places the browser window on the left 3/4 of the screen (Robert app takes right 1/4)
    pub async fn position_window(&self, screen_width: u32, screen_height: u32) -> Result<()> {
        use chromiumoxide::cdp::browser_protocol::browser::{
            Bounds, GetWindowForTargetParams, SetWindowBoundsParams,
        };

        let page = self.current_page().await?;
        let target_id = page.target_id();

        // Get the window ID for this target
        let window_result = page
            .execute(GetWindowForTargetParams {
                target_id: Some(target_id.clone()),
            })
            .await
            .map_err(|e| BrowserError::Other(format!("Failed to get window: {}", e)))?;

        let window_id = window_result.window_id;

        // Calculate dimensions: left 3/4 of screen (Robert app takes right 1/4)
        let browser_width = (screen_width * 3) / 4;
        let browser_height = screen_height;
        let browser_x = 0;
        let browser_y = 0;

        // Set window bounds
        let bounds = Bounds {
            left: Some(browser_x as i64),
            top: Some(browser_y as i64),
            width: Some(browser_width as i64),
            height: Some(browser_height as i64),
            window_state: Some(chromiumoxide::cdp::browser_protocol::browser::WindowState::Normal),
        };

        page.execute(SetWindowBoundsParams { window_id, bounds })
            .await
            .map_err(|e| BrowserError::Other(format!("Failed to set window bounds: {}", e)))?;

        eprintln!(
            "✓ Browser window positioned: {}x{} at ({}, {})",
            browser_width, browser_height, browser_x, browser_y
        );

        Ok(())
    }
}

impl Drop for ChromeDriver {
    fn drop(&mut self) {
        // Clean up temporary directory if it exists
        if let Some(temp_dir) = &self.temp_dir {
            if temp_dir.exists() {
                let _ = std::fs::remove_dir_all(temp_dir);
            }
        }
    }
}
