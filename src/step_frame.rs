//! Step Frame Capture for Browser Workflows
//!
//! This module provides functionality to capture detailed "step frames" during browser automation.
//! Each frame represents a moment in time with a screenshot, DOM state, and action context.
//!
//! Based on the Step Frame Schema specification in agent-formats/specs/STEP_FRAME_SCHEMA.md

use crate::error::{BrowserError, Result};
use crate::ChromeDriver;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ===== STEP FRAME STRUCTS =====

/// A complete step frame capturing a moment in a browser workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepFrame {
    /// Unique frame identifier (sequential starting from 0)
    pub frame_id: usize,

    /// ISO 8601 timestamp when frame was captured
    pub timestamp: String,

    /// Milliseconds elapsed since workflow start
    pub elapsed_ms: u64,

    /// Visual state (screenshot)
    pub screenshot: ScreenshotInfo,

    /// DOM state
    pub dom: DomInfo,

    /// VisualDom state (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visual_dom: Option<VisualDomInfo>,

    /// User/Agent action being performed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<ActionInfo>,

    /// Natural language transcript
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<TranscriptInfo>,
}

/// Screenshot information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotInfo {
    /// Relative or absolute path to screenshot file
    pub path: String,

    /// Image format (png, jpeg, webp)
    pub format: String,

    /// File size in bytes
    pub size_bytes: usize,

    /// Image dimensions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<Dimensions>,

    /// SHA-256 hash for deduplication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

/// Image or viewport dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

/// DOM state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomInfo {
    /// Current page URL
    pub url: String,

    /// Page title
    pub title: String,

    /// Path to saved HTML file (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_path: Option<String>,

    /// SHA-256 hash of HTML content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_hash: Option<String>,

    /// Interactive elements on the page (optional, can be expensive to collect)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interactive_elements: Option<Vec<InteractiveElement>>,
}

/// An interactive element on the page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveElement {
    pub selector: String,
    pub tag: String,
    pub text: String,
    pub is_visible: bool,
    pub is_enabled: bool,
}

// ===== VISUALDOM STRUCTS =====

/// VisualDom snapshot information
///
/// VisualDom is a custom format we created that provides a structured representation
/// of the DOM with layout and visual information. It combines data from Chrome DevTools
/// Protocol's DOMSnapshot.captureSnapshot with embedded base64 images.
///
/// This format allows AI agents to understand page structure, layout, and content
/// without requiring expensive OCR on screenshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualDomInfo {
    /// Path to saved VisualDom JSON file
    pub path: String,

    /// File size in bytes
    pub size_bytes: usize,

    /// Number of DOM nodes in the snapshot
    pub node_count: usize,

    /// SHA-256 hash for deduplication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

/// CDP DOMSnapshot captureSnapshot response
///
/// This is the raw response from CDP's DOMSnapshot.captureSnapshot command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSnapshotResponse {
    /// Array of document snapshots (usually one, but can include iframes)
    pub documents: Vec<DocumentSnapshot>,

    /// String table - all strings are stored as indexes into this array for efficiency
    pub strings: Vec<String>,
}

/// A snapshot of a single document (page or iframe)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSnapshot {
    /// Index of document URL in string table
    #[serde(rename = "documentURL")]
    pub document_url: i64,

    /// Index of title in string table
    pub title: i64,

    /// Index of base URL in string table
    #[serde(rename = "baseURL")]
    pub base_url: i64,

    /// Index of content language in string table
    #[serde(rename = "contentLanguage")]
    pub content_language: i64,

    /// Index of encoding name in string table
    #[serde(rename = "encodingName")]
    pub encoding_name: i64,

    /// Index of public ID in string table
    #[serde(rename = "publicId")]
    pub public_id: i64,

    /// Index of system ID in string table
    #[serde(rename = "systemId")]
    pub system_id: i64,

    /// Index of frame ID in string table
    #[serde(rename = "frameId")]
    pub frame_id: i64,

    /// DOM node tree
    pub nodes: NodeTreeSnapshot,

    /// Layout tree (positions, styles, text)
    pub layout: LayoutTreeSnapshot,

    /// Text boxes
    #[serde(rename = "textBoxes")]
    pub text_boxes: TextBoxSnapshot,

    /// Scroll offset X
    #[serde(rename = "scrollOffsetX")]
    pub scroll_offset_x: Option<f64>,

    /// Scroll offset Y
    #[serde(rename = "scrollOffsetY")]
    pub scroll_offset_y: Option<f64>,

    /// Content width
    #[serde(rename = "contentWidth")]
    pub content_width: Option<f64>,

    /// Content height
    #[serde(rename = "contentHeight")]
    pub content_height: Option<f64>,
}

/// Snapshot of the DOM node tree structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTreeSnapshot {
    /// Parent node index (parallel array)
    #[serde(rename = "parentIndex", default)]
    pub parent_index: Option<Vec<i64>>,

    /// Node type (parallel array)
    #[serde(rename = "nodeType", default)]
    pub node_type: Option<Vec<i64>>,

    /// Node name (string index, parallel array)
    #[serde(rename = "nodeName", default)]
    pub node_name: Option<Vec<i64>>,

    /// Node value (string index, parallel array)
    #[serde(rename = "nodeValue", default)]
    pub node_value: Option<Vec<i64>>,

    /// Backend node ID (parallel array)
    #[serde(rename = "backendNodeId", default)]
    pub backend_node_id: Option<Vec<i64>>,

    /// Attributes (array of string index arrays)
    #[serde(default)]
    pub attributes: Option<Vec<Vec<i64>>>,

    /// Text value for text nodes
    #[serde(rename = "textValue", default)]
    pub text_value: Option<RareStringData>,

    /// Input value for input elements
    #[serde(rename = "inputValue", default)]
    pub input_value: Option<RareStringData>,

    /// Current source URL for images/media
    #[serde(rename = "currentSourceURL", default)]
    pub current_source_url: Option<RareStringData>,

    /// Origin URL
    #[serde(rename = "originURL", default)]
    pub origin_url: Option<RareStringData>,

    /// Is clickable
    #[serde(rename = "isClickable", default)]
    pub is_clickable: Option<RareBooleanData>,
}

/// Layout tree snapshot with visual information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutTreeSnapshot {
    /// Node index (maps to NodeTreeSnapshot, parallel array)
    #[serde(rename = "nodeIndex")]
    pub node_index: Vec<i64>,

    /// Computed styles (array of string index arrays)
    pub styles: Vec<Vec<i64>>,

    /// Bounding rectangles (parallel array)
    pub bounds: Vec<Rectangle>,

    /// Text content (string index, parallel array)
    pub text: Vec<i64>,

    /// Stacking contexts
    #[serde(rename = "stackingContexts", default)]
    pub stacking_contexts: Option<RareBooleanData>,

    /// Paint orders (parallel array)
    #[serde(rename = "paintOrders", default)]
    pub paint_orders: Option<Vec<i64>>,

    /// Offset rectangles
    #[serde(rename = "offsetRects", default)]
    pub offset_rects: Option<Vec<Rectangle>>,

    /// Scroll rectangles
    #[serde(rename = "scrollRects", default)]
    pub scroll_rects: Option<Vec<Rectangle>>,

    /// Client rectangles
    #[serde(rename = "clientRects", default)]
    pub client_rects: Option<Vec<Rectangle>>,

    /// Blended background colors (string index array)
    #[serde(rename = "blendedBackgroundColors", default)]
    pub blended_background_colors: Option<Vec<i64>>,

    /// Text color opacities
    #[serde(rename = "textColorOpacities", default)]
    pub text_color_opacities: Option<Vec<f64>>,
}

/// Text box snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBoxSnapshot {
    /// Layout index (maps to LayoutTreeSnapshot)
    #[serde(rename = "layoutIndex")]
    pub layout_index: Vec<i64>,

    /// Start position in text
    pub start: Vec<i64>,

    /// Length of text
    pub length: Vec<i64>,

    /// Bounding rectangles
    pub bounds: Vec<Rectangle>,
}

/// Rectangle [x, y, width, height]
pub type Rectangle = Vec<f64>;

/// Sparse boolean data (only stores true values with their indexes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RareBooleanData {
    /// Indexes where the value is true
    pub index: Vec<i64>,
}

/// Sparse integer data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RareIntegerData {
    /// Indexes
    pub index: Vec<i64>,

    /// Values at those indexes
    pub value: Vec<i64>,
}

/// Sparse string data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RareStringData {
    /// Indexes
    pub index: Vec<i64>,

    /// String table indexes at those indexes
    pub value: Vec<i64>,
}

/// Action being performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInfo {
    /// Action type
    pub action_type: String,

    /// High-level description of intent
    pub intent: String,

    /// CSS selector or description of target
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

/// Natural language transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptInfo {
    /// Description of what is happening
    pub action_description: String,

    /// Why this action was chosen
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,

    /// What should happen next
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_outcome: Option<String>,
}

// ===== CAPTURE OPTIONS =====

/// Options for capturing a step frame
#[derive(Debug, Clone)]
pub struct CaptureOptions {
    /// Directory to save screenshots
    pub screenshot_dir: PathBuf,

    /// Directory to save DOM HTML files (optional)
    pub dom_dir: Option<PathBuf>,

    /// Directory to save VisualDom JSON files (optional)
    pub visual_dom_dir: Option<PathBuf>,

    /// Screenshot format (png, jpeg)
    pub screenshot_format: ScreenshotFormat,

    /// Whether to save the HTML DOM
    pub save_html: bool,

    /// Whether to capture VisualDom (opt-in)
    pub capture_visual_dom: bool,

    /// Computed styles to include in VisualDom (empty = none, specific props = filter)
    pub visual_dom_computed_styles: Vec<String>,

    /// Whether to include DOM rectangles in VisualDom
    pub visual_dom_include_dom_rects: bool,

    /// Whether to include paint order in VisualDom
    pub visual_dom_include_paint_order: bool,

    /// Whether to include images as base64 in VisualDom
    pub visual_dom_include_images: bool,

    /// Whether to compute SHA-256 hashes
    pub compute_hashes: bool,

    /// Whether to extract interactive elements (expensive)
    pub extract_interactive_elements: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum ScreenshotFormat {
    Png,
    Jpeg,
}

impl Default for CaptureOptions {
    fn default() -> Self {
        Self {
            screenshot_dir: PathBuf::from("./screenshots"),
            dom_dir: Some(PathBuf::from("./dom")),
            visual_dom_dir: Some(PathBuf::from("./visualdom")),
            screenshot_format: ScreenshotFormat::Png,
            save_html: true,
            capture_visual_dom: false, // Opt-in only
            visual_dom_computed_styles: Self::balanced_computed_styles(),
            visual_dom_include_dom_rects: true,
            visual_dom_include_paint_order: true,
            visual_dom_include_images: true,
            compute_hashes: true,
            extract_interactive_elements: false,
        }
    }
}

impl CaptureOptions {
    /// Returns a balanced set of computed styles for VisualDom capture
    ///
    /// Includes styles that are useful for understanding layout and visibility
    /// without capturing every CSS property (which would be expensive)
    pub fn balanced_computed_styles() -> Vec<String> {
        vec![
            // Position and layout
            "display".to_string(),
            "position".to_string(),
            "top".to_string(),
            "left".to_string(),
            "right".to_string(),
            "bottom".to_string(),
            "width".to_string(),
            "height".to_string(),
            "z-index".to_string(),
            // Visibility
            "visibility".to_string(),
            "opacity".to_string(),
            // Typography
            "font-size".to_string(),
            "font-weight".to_string(),
            "font-family".to_string(),
            "color".to_string(),
            // Spacing
            "padding".to_string(),
            "margin".to_string(),
            // Background
            "background-color".to_string(),
            "background-image".to_string(),
        ]
    }

    /// Returns a minimal set of computed styles (just positioning)
    pub fn minimal_computed_styles() -> Vec<String> {
        vec![
            "display".to_string(),
            "position".to_string(),
            "visibility".to_string(),
        ]
    }

    /// Returns all computed styles (empty vec = capture all)
    pub fn all_computed_styles() -> Vec<String> {
        vec![]
    }
}

// ===== CAPTURE FUNCTION =====

/// Captures a step frame from the current browser state
///
/// # Arguments
///
/// * `driver` - Reference to the ChromeDriver
/// * `frame_id` - Sequential frame identifier
/// * `elapsed_ms` - Milliseconds since workflow start
/// * `options` - Capture options
/// * `user_instruction` - Optional user instruction text
/// * `action_info` - Optional action being performed
///
/// # Returns
///
/// A `StepFrame` with all captured information
///
/// # Errors
///
/// Returns error if:
/// - Cannot access current browser page (fail fast)
/// - Screenshot capture fails
/// - DOM retrieval fails
/// - File I/O fails
///
/// # Example
///
/// ```no_run
/// use robert_webdriver::{ChromeDriver, ConnectionMode};
/// use robert_webdriver::step_frame::{capture_step_frame, CaptureOptions, ActionInfo};
///
/// # async fn example() -> anyhow::Result<()> {
/// let driver = ChromeDriver::new(ConnectionMode::Sandboxed {
///     chrome_path: None,
///     no_sandbox: true,
///     headless: true,
/// }).await?;
///
/// driver.navigate("https://example.com").await?;
///
/// let options = CaptureOptions::default();
/// let action = Some(ActionInfo {
///     action_type: "navigate".to_string(),
///     intent: "Navigate to example.com".to_string(),
///     target: None,
/// });
///
/// let frame = capture_step_frame(&driver, 0, 0, &options, None, action).await?;
/// println!("Captured frame: {:?}", frame);
/// # Ok(())
/// # }
/// ```
pub async fn capture_step_frame(
    driver: &ChromeDriver,
    frame_id: usize,
    elapsed_ms: u64,
    options: &CaptureOptions,
    user_instruction: Option<String>,
    action_info: Option<ActionInfo>,
) -> Result<StepFrame> {
    log::info!("╔═══════════════════════════════════════════════════════════╗");
    log::info!(
        "║  📸 CAPTURING STEP FRAME {}                              ║",
        frame_id
    );
    log::info!("╚═══════════════════════════════════════════════════════════╝");

    if let Some(ref instruction) = user_instruction {
        log::info!("📝 User instruction: {}", instruction);
    }
    if let Some(ref action) = action_info {
        log::info!("🎯 Action: {} - {}", action.action_type, action.intent);
    }
    log::info!("⏱️  Elapsed: {}ms", elapsed_ms);

    // 1. FAIL FAST: Access current page to verify connection
    log::debug!("🔍 Verifying browser connection...");
    let page = driver.current_page().await.map_err(|e| {
        log::error!("❌ Failed to access browser page: {}", e);
        BrowserError::Other(format!(
            "Failed to access browser page (connection failed): {}",
            e
        ))
    })?;

    // Verify page is accessible by getting URL
    let _ = page.url().await.map_err(|e| {
        log::error!("❌ Failed to get page URL: {}", e);
        BrowserError::Other(format!(
            "Failed to get page URL (browser not responding): {}",
            e
        ))
    })?;

    log::debug!("✓ Browser connection verified");

    // 2. TAKE SCREENSHOT
    log::info!("📸 Capturing screenshot...");
    let screenshot_filename = format!(
        "frame_{:04}.{}",
        frame_id,
        format_extension(options.screenshot_format)
    );
    let screenshot_path = options.screenshot_dir.join(&screenshot_filename);
    log::debug!("Screenshot path: {:?}", screenshot_path);

    // Ensure screenshot directory exists
    tokio::fs::create_dir_all(&options.screenshot_dir)
        .await
        .map_err(|e| {
            log::error!("❌ Failed to create screenshot directory: {}", e);
            BrowserError::Other(format!("Failed to create screenshot directory: {}", e))
        })?;

    // Capture screenshot
    driver.screenshot_to_file(&screenshot_path).await?;
    log::info!("✓ Screenshot captured: {}", screenshot_filename);

    // Get screenshot file size
    let screenshot_metadata = tokio::fs::metadata(&screenshot_path)
        .await
        .map_err(|e| BrowserError::Other(format!("Failed to read screenshot metadata: {}", e)))?;

    let screenshot_size = screenshot_metadata.len() as usize;

    // Optionally compute screenshot hash
    let screenshot_hash = if options.compute_hashes {
        Some(compute_file_hash(&screenshot_path).await?)
    } else {
        None
    };

    // 3. SAVE DOM
    log::info!("📄 Extracting DOM...");
    let url = driver.current_url().await?;
    let title = driver.title().await?;
    log::debug!("URL: {}", url);
    log::debug!("Title: {}", title);
    let html_content = driver.get_page_source().await?;
    log::info!("✓ DOM extracted ({} KB)", html_content.len() / 1024);

    let (html_path, html_hash) = if options.save_html {
        if let Some(dom_dir) = &options.dom_dir {
            // Ensure DOM directory exists
            tokio::fs::create_dir_all(dom_dir).await.map_err(|e| {
                BrowserError::Other(format!("Failed to create DOM directory: {}", e))
            })?;

            let html_filename = format!("frame_{:04}.html", frame_id);
            let html_file_path = dom_dir.join(&html_filename);

            // Save HTML to file
            tokio::fs::write(&html_file_path, &html_content)
                .await
                .map_err(|e| BrowserError::Other(format!("Failed to write HTML file: {}", e)))?;

            // Compute hash if requested
            let hash = if options.compute_hashes {
                Some(compute_string_hash(&html_content))
            } else {
                None
            };

            (Some(html_file_path.to_string_lossy().to_string()), hash)
        } else {
            // No DOM directory specified, just compute hash if requested
            let hash = if options.compute_hashes {
                Some(compute_string_hash(&html_content))
            } else {
                None
            };
            (None, hash)
        }
    } else {
        (None, None)
    };

    // 4. EXTRACT INTERACTIVE ELEMENTS (optional, expensive)
    let interactive_elements = if options.extract_interactive_elements {
        log::info!("🔍 Extracting interactive elements...");
        let elements = extract_interactive_elements_from_page(driver).await?;
        log::info!("✓ Found {} interactive elements", elements.len());
        Some(elements)
    } else {
        None
    };

    // 5. CAPTURE VISUALDOM (optional)
    let visual_dom_info = if options.capture_visual_dom {
        log::info!("🗺️  Capturing VisualDom...");

        // Capture the VisualDom data
        let visual_dom_data = driver
            .capture_visual_dom(
                &options.visual_dom_computed_styles,
                options.visual_dom_include_dom_rects,
                options.visual_dom_include_paint_order,
                options.visual_dom_include_images,
            )
            .await?;

        // Parse to get node count
        let node_count = visual_dom_data
            .get("documents")
            .and_then(|docs| docs.as_array())
            .and_then(|arr| arr.first())
            .and_then(|doc| doc.get("nodes"))
            .and_then(|nodes| nodes.get("nodeIndex"))
            .and_then(|idx| idx.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        log::info!("✓ VisualDom captured ({} nodes)", node_count);

        // Save to file if directory specified
        if let Some(visual_dom_dir) = &options.visual_dom_dir {
            // Ensure VisualDom directory exists
            tokio::fs::create_dir_all(visual_dom_dir)
                .await
                .map_err(|e| {
                    BrowserError::Other(format!("Failed to create VisualDom directory: {}", e))
                })?;

            let visual_dom_filename = format!("frame_{:04}.visualdom.json", frame_id);
            let visual_dom_file_path = visual_dom_dir.join(&visual_dom_filename);

            // Save VisualDom to file
            let visual_dom_json = serde_json::to_string_pretty(&visual_dom_data).map_err(|e| {
                BrowserError::Other(format!("Failed to serialize VisualDom: {}", e))
            })?;

            tokio::fs::write(&visual_dom_file_path, &visual_dom_json)
                .await
                .map_err(|e| {
                    BrowserError::Other(format!("Failed to write VisualDom file: {}", e))
                })?;

            // Get file size
            let visual_dom_metadata =
                tokio::fs::metadata(&visual_dom_file_path)
                    .await
                    .map_err(|e| {
                        BrowserError::Other(format!("Failed to read VisualDom metadata: {}", e))
                    })?;
            let visual_dom_size = visual_dom_metadata.len() as usize;

            // Compute hash if requested
            let visual_dom_hash = if options.compute_hashes {
                Some(compute_string_hash(&visual_dom_json))
            } else {
                None
            };

            log::info!("   VisualDom: {} KB", visual_dom_size / 1024);

            Some(VisualDomInfo {
                path: visual_dom_file_path.to_string_lossy().to_string(),
                size_bytes: visual_dom_size,
                node_count,
                hash: visual_dom_hash,
            })
        } else {
            // No directory specified, skip saving
            None
        }
    } else {
        None
    };

    // 6. BUILD TRANSCRIPT
    let transcript = if let Some(instruction) = user_instruction {
        Some(TranscriptInfo {
            action_description: instruction.clone(),
            reasoning: None,
            expected_outcome: None,
        })
    } else {
        action_info.as_ref().map(|action| TranscriptInfo {
            action_description: action.intent.clone(),
            reasoning: None,
            expected_outcome: None,
        })
    };

    // 7. CONSTRUCT STEP FRAME
    log::info!("✅ Step frame {} captured successfully", frame_id);
    log::info!("   Screenshot: {} KB", screenshot_size / 1024);
    log::info!("   DOM: {} KB", html_content.len() / 1024);
    if let Some(ref vd) = visual_dom_info {
        log::info!(
            "   VisualDom: {} KB ({} nodes)",
            vd.size_bytes / 1024,
            vd.node_count
        );
    }
    log::info!("   URL: {}", url);

    Ok(StepFrame {
        frame_id,
        timestamp: chrono::Utc::now().to_rfc3339(),
        elapsed_ms,
        screenshot: ScreenshotInfo {
            path: screenshot_path.to_string_lossy().to_string(),
            format: format_string(options.screenshot_format),
            size_bytes: screenshot_size,
            dimensions: None, // Could be extracted from image metadata
            hash: screenshot_hash,
        },
        dom: DomInfo {
            url,
            title,
            html_path,
            html_hash,
            interactive_elements,
        },
        visual_dom: visual_dom_info,
        action: action_info,
        transcript,
    })
}

// ===== HELPER FUNCTIONS =====

fn format_extension(format: ScreenshotFormat) -> &'static str {
    match format {
        ScreenshotFormat::Png => "png",
        ScreenshotFormat::Jpeg => "jpg",
    }
}

fn format_string(format: ScreenshotFormat) -> String {
    match format {
        ScreenshotFormat::Png => "png".to_string(),
        ScreenshotFormat::Jpeg => "jpeg".to_string(),
    }
}

/// Compute SHA-256 hash of a file
async fn compute_file_hash(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};

    let contents = tokio::fs::read(path)
        .await
        .map_err(|e| BrowserError::Other(format!("Failed to read file for hashing: {}", e)))?;

    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let hash = hasher.finalize();

    Ok(format!("{:x}", hash))
}

/// Compute SHA-256 hash of a string
fn compute_string_hash(content: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let hash = hasher.finalize();

    format!("{:x}", hash)
}

/// Extract interactive elements from the current page
async fn extract_interactive_elements_from_page(
    driver: &ChromeDriver,
) -> Result<Vec<InteractiveElement>> {
    // JavaScript to extract interactive elements
    let js_code = r#"
        (() => {
            const selectors = ['button', 'a', 'input', 'select', 'textarea'];
            const elements = [];

            selectors.forEach(tag => {
                const nodes = document.querySelectorAll(tag);
                nodes.forEach((el, idx) => {
                    if (idx < 50) { // Limit to first 50 of each type
                        const rect = el.getBoundingClientRect();
                        const isVisible = rect.width > 0 && rect.height > 0;
                        elements.push({
                            selector: `${tag}:nth-of-type(${idx + 1})`,
                            tag: tag,
                            text: el.textContent ? el.textContent.trim().substring(0, 100) : '',
                            is_visible: isVisible,
                            is_enabled: !el.disabled
                        });
                    }
                });
            });

            return elements;
        })()
    "#;

    let result = driver.execute_script(js_code).await?;

    // Parse the result
    let elements: Vec<InteractiveElement> = serde_json::from_value(result).unwrap_or_default();

    Ok(elements)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_capture_options() {
        let options = CaptureOptions::default();
        assert_eq!(options.screenshot_dir, PathBuf::from("./screenshots"));
        assert_eq!(options.dom_dir, Some(PathBuf::from("./dom")));
        assert!(options.save_html);
        assert!(options.compute_hashes);
        assert!(!options.extract_interactive_elements);
    }

    #[test]
    fn test_format_extension() {
        assert_eq!(format_extension(ScreenshotFormat::Png), "png");
        assert_eq!(format_extension(ScreenshotFormat::Jpeg), "jpg");
    }

    #[test]
    fn test_format_string() {
        assert_eq!(format_string(ScreenshotFormat::Png), "png");
        assert_eq!(format_string(ScreenshotFormat::Jpeg), "jpeg");
    }

    #[test]
    fn test_compute_string_hash() {
        let hash1 = compute_string_hash("hello world");
        let hash2 = compute_string_hash("hello world");
        let hash3 = compute_string_hash("different");

        // Same input should produce same hash
        assert_eq!(hash1, hash2);

        // Different input should produce different hash
        assert_ne!(hash1, hash3);

        // Hash should be 64 hex characters (SHA-256)
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_step_frame_serialization() {
        let frame = StepFrame {
            frame_id: 0,
            timestamp: "2025-10-11T12:00:00Z".to_string(),
            elapsed_ms: 0,
            screenshot: ScreenshotInfo {
                path: "./screenshots/frame_0000.png".to_string(),
                format: "png".to_string(),
                size_bytes: 12345,
                dimensions: Some(Dimensions {
                    width: 1920,
                    height: 1080,
                }),
                hash: Some("abc123".to_string()),
            },
            dom: DomInfo {
                url: "https://example.com".to_string(),
                title: "Example".to_string(),
                html_path: Some("./dom/frame_0000.html".to_string()),
                html_hash: Some("def456".to_string()),
                interactive_elements: None,
            },
            visual_dom: Some(VisualDomInfo {
                path: "./visualdom/frame_0000.visualdom.json".to_string(),
                size_bytes: 54321,
                node_count: 150,
                hash: Some("ghi789".to_string()),
            }),
            action: Some(ActionInfo {
                action_type: "navigate".to_string(),
                intent: "Navigate to example.com".to_string(),
                target: None,
            }),
            transcript: Some(TranscriptInfo {
                action_description: "Navigating to example.com".to_string(),
                reasoning: Some("User requested navigation".to_string()),
                expected_outcome: Some("Page should load".to_string()),
            }),
        };

        // Test serialization
        let json = serde_json::to_string_pretty(&frame).unwrap();
        assert!(json.contains("frame_id"));
        assert!(json.contains("screenshot"));
        assert!(json.contains("dom"));
        assert!(json.contains("visual_dom"));

        // Test deserialization
        let deserialized: StepFrame = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.frame_id, 0);
        assert_eq!(deserialized.screenshot.size_bytes, 12345);
        assert!(deserialized.visual_dom.is_some());
        assert_eq!(deserialized.visual_dom.unwrap().node_count, 150);
    }

    #[test]
    fn test_default_capture_options_visual_dom() {
        let options = CaptureOptions::default();
        assert_eq!(options.visual_dom_dir, Some(PathBuf::from("./visualdom")));
        assert!(!options.capture_visual_dom); // Opt-in by default
        assert!(options.visual_dom_include_dom_rects);
        assert!(options.visual_dom_include_paint_order);
        assert!(options.visual_dom_include_images);
        assert!(!options.visual_dom_computed_styles.is_empty());
    }

    #[test]
    fn test_computed_styles_presets() {
        let balanced = CaptureOptions::balanced_computed_styles();
        assert!(!balanced.is_empty());
        assert!(balanced.contains(&"display".to_string()));
        assert!(balanced.contains(&"position".to_string()));

        let minimal = CaptureOptions::minimal_computed_styles();
        assert!(minimal.len() < balanced.len());
        assert!(minimal.contains(&"display".to_string()));

        let all = CaptureOptions::all_computed_styles();
        assert!(all.is_empty()); // Empty vec means capture all
    }
}
