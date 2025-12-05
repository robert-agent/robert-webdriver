//! Unit tests for CDP script generation and validation
//!
//! Note: AI-based generation tests are excluded because they require external Claude CLI
//! and may have non-deterministic results. The validation tests verify the structure
//! and correctness of generated scripts.

use robert_webdriver::cdp::validate_generated_script;

#[test]
fn test_script_validation_valid() {
    // Test validation logic with valid script
    let valid_json = r#"{
        "name": "test-script",
        "description": "Test script",
        "cdp_commands": [
            {
                "method": "Page.navigate",
                "params": {"url": "https://example.com"}
            }
        ]
    }"#;

    let result = validate_generated_script(valid_json);
    assert!(result.is_ok(), "Valid script should pass validation");
}

#[test]
fn test_script_validation_unknown_command() {
    // Test with unknown command
    let invalid_json = r#"{
        "name": "test",
        "description": "Test script",
        "cdp_commands": [
            {
                "method": "Unknown.command",
                "params": {}
            }
        ]
    }"#;

    let result = validate_generated_script(invalid_json);
    assert!(result.is_err(), "Unknown command should fail validation");

    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("unknown") || error.to_string().contains("Unknown"),
        "Error should mention unknown command"
    );
}

#[test]
fn test_script_validation_malformed_json() {
    // Test with malformed JSON
    let malformed_json = r#"{
        "name": "test",
        "cdp_commands": [
            {
                "method": "Page.navigate"
                // Missing comma
                "params": {}
            }
        ]
    }"#;

    let result = validate_generated_script(malformed_json);
    assert!(result.is_err(), "Malformed JSON should fail validation");
}

#[test]
fn test_script_validation_missing_required_fields() {
    // Test with missing required fields
    let missing_fields_json = r#"{
        "name": "test",
        "cdp_commands": [
            {
                "method": "Page.navigate"
            }
        ]
    }"#;

    let result = validate_generated_script(missing_fields_json);
    assert!(
        result.is_err(),
        "Missing required params should fail validation"
    );
}

#[test]
fn test_script_validation_multiple_commands() {
    // Test with multiple valid commands
    let multi_command_json = r#"{
        "name": "multi-command-test",
        "description": "Test multiple commands",
        "cdp_commands": [
            {
                "method": "Page.navigate",
                "params": {"url": "https://example.com"}
            },
            {
                "method": "Runtime.evaluate",
                "params": {
                    "expression": "document.title",
                    "returnByValue": true
                }
            },
            {
                "method": "Page.captureScreenshot",
                "params": {}
            }
        ]
    }"#;

    let result = validate_generated_script(multi_command_json);
    assert!(
        result.is_ok(),
        "Multiple valid commands should pass validation"
    );
}

#[test]
fn test_script_validation_with_save_as() {
    // Test with save_as field
    let json_with_save = r#"{
        "name": "save-test",
        "description": "Test save_as functionality",
        "cdp_commands": [
            {
                "method": "Page.navigate",
                "params": {"url": "https://example.com"}
            },
            {
                "method": "Runtime.evaluate",
                "params": {
                    "expression": "document.title",
                    "returnByValue": true
                },
                "save_as": "output.json"
            }
        ]
    }"#;

    let result = validate_generated_script(json_with_save);
    assert!(result.is_ok(), "Script with save_as should pass validation");
}
