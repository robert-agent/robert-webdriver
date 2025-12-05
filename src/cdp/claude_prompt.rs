//! Claude prompt templates for CDP script generation

/// Generate a prompt for Claude to create a CDP automation script
pub fn generate_cdp_script_prompt(user_request: &str) -> String {
    format!(
        r#"You are a browser automation expert generating Chrome DevTools Protocol (CDP) scripts.

USER REQUEST: {user_request}

Generate a JSON script that accomplishes this task using CDP commands.

AVAILABLE CDP COMMANDS:

1. Page.navigate - Navigate to URL
   {{"method": "Page.navigate", "params": {{"url": "https://example.com"}}}}

2. Page.captureScreenshot - Take screenshot (saves automatically if save_as provided)
   {{"method": "Page.captureScreenshot", "params": {{"format": "png", "captureBeyondViewport": true}}, "save_as": "screenshot.png"}}

3. Page.reload - Refresh page
   {{"method": "Page.reload", "params": {{"ignoreCache": true}}}}

4. Page.goBack - Browser back
   {{"method": "Page.goBack", "params": {{"entryId": -1}}}}

5. Page.goForward - Browser forward
   {{"method": "Page.goForward", "params": {{"entryId": 1}}}}

6. Runtime.evaluate - Execute JavaScript (saves result if save_as provided)
   {{"method": "Runtime.evaluate", "params": {{"expression": "document.title", "returnByValue": true}}, "save_as": "result.json"}}

7. Input.insertText - Type text
   {{"method": "Input.insertText", "params": {{"text": "Hello World"}}}}

8. Input.dispatchMouseEvent - Click/mouse action
   {{"method": "Input.dispatchMouseEvent", "params": {{"type": "mousePressed", "x": 100, "y": 200, "button": "left", "clickCount": 1}}}}

9. Input.dispatchKeyEvent - Press keys
   {{"method": "Input.dispatchKeyEvent", "params": {{"type": "keyDown", "key": "Enter"}}}}

10. Network.getCookies - Get all cookies
    {{"method": "Network.getCookies", "params": {{}}}}

11. Network.setCookie - Set a cookie
    {{"method": "Network.setCookie", "params": {{"name": "session", "value": "abc123", "domain": "example.com"}}}}

12. Network.deleteCookies - Delete cookies
    {{"method": "Network.deleteCookies", "params": {{"name": "session"}}}}

13. Emulation.setGeolocationOverride - Set location
    {{"method": "Emulation.setGeolocationOverride", "params": {{"latitude": 37.7749, "longitude": -122.4194, "accuracy": 100}}}}

14. Emulation.setDeviceMetricsOverride - Mobile emulation
    {{"method": "Emulation.setDeviceMetricsOverride", "params": {{"width": 375, "height": 667, "deviceScaleFactor": 2, "mobile": true}}}}

IMPORTANT RULES:

1. ONLY use commands from the list above
2. Always navigate to a page first before interacting with it
3. For clicking elements, use Runtime.evaluate with JavaScript like: document.querySelector('button').click()
4. For extracting data, use Runtime.evaluate with JavaScript
5. For screenshots, always set "captureBeyondViewport": true for full page
6. Use save_as field when you want to save screenshots or extracted data
7. Sequence commands logically (navigate before interact, wait for page load)
8. Use descriptive names and descriptions

OUTPUT FORMAT (JSON only, no markdown):

{{
  "name": "descriptive-name-with-hyphens",
  "description": "Clear description of what this automation does",
  "created": "{current_timestamp}",
  "author": "Claude",
  "tags": ["tag1", "tag2"],
  "cdp_commands": [
    {{
      "method": "Page.navigate",
      "params": {{"url": "..."}},
      "description": "Navigate to the target page"
    }},
    {{
      "method": "Runtime.evaluate",
      "params": {{"expression": "...", "returnByValue": true}},
      "save_as": "optional-output.json",
      "description": "Extract or manipulate data"
    }}
  ]
}}

EXAMPLES:

Example 1 - Screenshot a page:
{{
  "name": "screenshot-example-com",
  "description": "Take a screenshot of example.com",
  "created": "2025-10-09T00:00:00Z",
  "author": "Claude",
  "tags": ["screenshot"],
  "cdp_commands": [
    {{
      "method": "Page.navigate",
      "params": {{"url": "https://example.com"}},
      "description": "Navigate to example.com"
    }},
    {{
      "method": "Page.captureScreenshot",
      "params": {{"format": "png", "captureBeyondViewport": true}},
      "save_as": "example.png",
      "description": "Capture full page screenshot"
    }}
  ]
}}

Example 2 - Extract data:
{{
  "name": "extract-product-info",
  "description": "Extract product information from a page",
  "created": "2025-10-09T00:00:00Z",
  "author": "Claude",
  "tags": ["data-extraction"],
  "cdp_commands": [
    {{
      "method": "Page.navigate",
      "params": {{"url": "https://shop.example.com/product/123"}},
      "description": "Navigate to product page"
    }},
    {{
      "method": "Runtime.evaluate",
      "params": {{
        "expression": "JSON.stringify({{title: document.querySelector('h1').textContent, price: document.querySelector('.price').textContent}})",
        "returnByValue": true
      }},
      "save_as": "product-info.json",
      "description": "Extract product title and price"
    }}
  ]
}}

Example 3 - Click and interact:
{{
  "name": "login-automation",
  "description": "Automate login to a website",
  "created": "2025-10-09T00:00:00Z",
  "author": "Claude",
  "tags": ["login", "automation"],
  "cdp_commands": [
    {{
      "method": "Page.navigate",
      "params": {{"url": "https://example.com/login"}},
      "description": "Navigate to login page"
    }},
    {{
      "method": "Runtime.evaluate",
      "params": {{
        "expression": "document.querySelector('input[name=email]').value = 'user@example.com'; document.querySelector('input[name=password]').value = 'password123'",
        "returnByValue": false
      }},
      "description": "Fill login form"
    }},
    {{
      "method": "Runtime.evaluate",
      "params": {{
        "expression": "document.querySelector('button[type=submit]').click()",
        "returnByValue": false
      }},
      "description": "Click submit button"
    }}
  ]
}}

Now generate the CDP script for the user's request. Output ONLY valid JSON, no markdown code blocks."#,
        user_request = user_request,
        current_timestamp = chrono::Utc::now().to_rfc3339()
    )
}

/// Validate that a JSON string is a valid CDP script
pub fn validate_generated_script(json: &str) -> Result<crate::cdp::CdpScript, String> {
    // Parse JSON
    let script: crate::cdp::CdpScript =
        serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {}", e))?;

    // Validate using built-in validation
    script
        .validate()
        .map_err(|e| format!("Script validation failed: {}", e))?;

    // Additional validation for CDP commands
    let valid_methods = [
        "Page.navigate",
        "Page.captureScreenshot",
        "Page.reload",
        "Page.goBack",
        "Page.goForward",
        "Runtime.evaluate",
        "Input.insertText",
        "Input.dispatchMouseEvent",
        "Input.dispatchKeyEvent",
        "Network.getCookies",
        "Network.setCookie",
        "Network.deleteCookies",
        "Emulation.setGeolocationOverride",
        "Emulation.setDeviceMetricsOverride",
    ];

    for cmd in &script.cdp_commands {
        if !valid_methods.contains(&cmd.method.as_str()) {
            return Err(format!(
                "Unknown CDP command: {}. Only the listed commands are supported.",
                cmd.method
            ));
        }
    }

    Ok(script)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_prompt() {
        let prompt = generate_cdp_script_prompt("Take a screenshot of google.com");
        assert!(prompt.contains("USER REQUEST"));
        assert!(prompt.contains("Page.navigate"));
        assert!(prompt.contains("Page.captureScreenshot"));
    }

    #[test]
    fn test_validate_valid_script() {
        let json = r#"{
            "name": "test",
            "description": "Test script",
            "cdp_commands": [
                {
                    "method": "Page.navigate",
                    "params": {"url": "https://example.com"}
                }
            ]
        }"#;

        let result = validate_generated_script(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_method() {
        let json = r#"{
            "name": "test",
            "description": "Test script",
            "cdp_commands": [
                {
                    "method": "Invalid.method",
                    "params": {}
                }
            ]
        }"#;

        let result = validate_generated_script(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown CDP command"));
    }
}
