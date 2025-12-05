//! Integration tests for CDP script validation

use robert_webdriver::{CdpValidator, ValidationErrorType};

#[test]
fn test_complete_valid_script() {
    let validator = CdpValidator::new();

    let json = r#"{
        "name": "complete-test-script",
        "description": "A complete valid CDP script for testing",
        "author": "Test Suite",
        "tags": ["test", "validation"],
        "cdp_commands": [
            {
                "method": "Page.navigate",
                "params": {"url": "https://example.com"},
                "description": "Navigate to example.com"
            },
            {
                "method": "Runtime.evaluate",
                "params": {
                    "expression": "document.title",
                    "returnByValue": true
                },
                "save_as": "title.json",
                "description": "Extract page title"
            },
            {
                "method": "Page.captureScreenshot",
                "params": {
                    "format": "png",
                    "captureBeyondViewport": true
                },
                "save_as": "screenshot.png",
                "description": "Take screenshot"
            }
        ]
    }"#;

    let result = validator.validate_json(json);

    assert!(result.is_valid, "Complete valid script should pass");
    assert!(result.errors.is_empty(), "Should have no errors");
    println!("✅ Complete valid script passed validation");
}

#[test]
fn test_malformed_json() {
    let validator = CdpValidator::new();

    let test_cases = vec![
        (
            r#"{"name": "test""#, // Missing closing brace
            "Missing closing brace",
        ),
        (
            r#"{"name": "test", "description": "test",}"#, // Trailing comma
            "Trailing comma",
        ),
        (
            r#"{'name': 'test'}"#, // Single quotes
            "Single quotes instead of double",
        ),
        (
            r#"{"name": "test" "description": "test"}"#, // Missing comma
            "Missing comma between fields",
        ),
    ];

    for (json, description) in test_cases {
        let result = validator.validate_json(json);
        assert!(!result.is_valid, "{} should fail validation", description);
        assert!(
            !result.errors.is_empty(),
            "{} should have errors",
            description
        );
        assert!(
            result.errors[0].error_type == ValidationErrorType::JsonSyntax,
            "{} should be JSON syntax error",
            description
        );
        println!("✅ Caught malformed JSON: {}", description);
    }
}

#[test]
fn test_missing_required_fields() {
    let validator = CdpValidator::new();

    // Missing name
    let json1 = r#"{
        "name": "",
        "description": "Test",
        "cdp_commands": [
            {"method": "Page.navigate", "params": {"url": "https://example.com"}}
        ]
    }"#;

    let result1 = validator.validate_json(json1);
    assert!(!result1.is_valid, "Empty name should fail");
    assert!(
        result1
            .errors
            .iter()
            .any(|e| e.location.field_path == "name"),
        "Should error on name field"
    );
    println!("✅ Caught empty script name");

    // No commands
    let json2 = r#"{
        "name": "test",
        "description": "Test",
        "cdp_commands": []
    }"#;

    let result2 = validator.validate_json(json2);
    assert!(!result2.is_valid, "Empty commands should fail");
    assert!(
        result2
            .errors
            .iter()
            .any(|e| e.location.field_path == "cdp_commands"),
        "Should error on cdp_commands field"
    );
    println!("✅ Caught empty commands array");

    // Missing required command parameter
    let json3 = r#"{
        "name": "test",
        "description": "Test",
        "cdp_commands": [
            {"method": "Page.navigate", "params": {}}
        ]
    }"#;

    let result3 = validator.validate_json(json3);
    assert!(!result3.is_valid, "Missing url parameter should fail");
    assert!(
        result3
            .errors
            .iter()
            .any(|e| e.error_type == ValidationErrorType::MissingParameter),
        "Should be missing parameter error"
    );
    println!("✅ Caught missing required parameter (url)");
}

#[test]
fn test_invalid_command_methods() {
    let validator = CdpValidator::new();

    // Invalid format (no dot)
    let json1 = r#"{
        "name": "test",
        "description": "Test",
        "cdp_commands": [
            {"method": "InvalidFormat", "params": {}}
        ]
    }"#;

    let result1 = validator.validate_json(json1);
    assert!(!result1.is_valid);
    assert!(result1
        .errors
        .iter()
        .any(|e| e.error_type == ValidationErrorType::InvalidValue
            && e.message.contains("Domain.method")));
    println!("✅ Caught invalid method format");

    // Unknown command
    let json2 = r#"{
        "name": "test",
        "description": "Test",
        "cdp_commands": [
            {"method": "Unknown.command", "params": {}}
        ]
    }"#;

    let result2 = validator.validate_json(json2);
    assert!(!result2.is_valid);
    assert!(result2
        .errors
        .iter()
        .any(|e| e.error_type == ValidationErrorType::UnknownCommand));
    println!("✅ Caught unknown CDP command");

    // Empty method
    let json3 = r#"{
        "name": "test",
        "description": "Test",
        "cdp_commands": [
            {"method": "", "params": {}}
        ]
    }"#;

    let result3 = validator.validate_json(json3);
    assert!(!result3.is_valid);
    assert!(result3
        .errors
        .iter()
        .any(|e| e.error_type == ValidationErrorType::MissingField));
    println!("✅ Caught empty method name");
}

#[test]
fn test_parameter_type_validation() {
    let validator = CdpValidator::new();

    // url should be string, not number
    let json1 = r#"{
        "name": "test",
        "description": "Test",
        "cdp_commands": [
            {"method": "Page.navigate", "params": {"url": 123}}
        ]
    }"#;

    let result1 = validator.validate_json(json1);
    assert!(!result1.is_valid);
    assert!(
        result1
            .errors
            .iter()
            .any(|e| e.error_type == ValidationErrorType::TypeMismatch),
        "Should catch type mismatch"
    );
    println!("✅ Caught parameter type mismatch (number instead of string)");

    // captureBeyondViewport should be boolean, not string
    let json2 = r#"{
        "name": "test",
        "description": "Test",
        "cdp_commands": [
            {
                "method": "Page.captureScreenshot",
                "params": {"format": "png", "captureBeyondViewport": "true"}
            }
        ]
    }"#;

    let result2 = validator.validate_json(json2);
    assert!(!result2.is_valid);
    assert!(result2
        .errors
        .iter()
        .any(|e| e.error_type == ValidationErrorType::TypeMismatch
            && e.message.contains("captureBeyondViewport")));
    println!("✅ Caught parameter type mismatch (string instead of boolean)");

    // Mouse event coordinates should be numbers
    let json3 = r#"{
        "name": "test",
        "description": "Test",
        "cdp_commands": [
            {
                "method": "Input.dispatchMouseEvent",
                "params": {"type": "mousePressed", "x": "100", "y": "200"}
            }
        ]
    }"#;

    let result3 = validator.validate_json(json3);
    assert!(!result3.is_valid);
    assert!(result3
        .errors
        .iter()
        .any(|e| e.error_type == ValidationErrorType::TypeMismatch));
    println!("✅ Caught parameter type mismatch (string instead of number for coordinates)");
}

#[test]
fn test_command_specific_validation() {
    let validator = CdpValidator::new();

    // Page.navigate requires url
    let json1 = r#"{
        "name": "test",
        "description": "Test",
        "cdp_commands": [
            {"method": "Page.navigate", "params": {"referrer": "https://google.com"}}
        ]
    }"#;

    let result1 = validator.validate_json(json1);
    assert!(!result1.is_valid);
    assert!(result1.errors.iter().any(|e| e.message.contains("url")));
    println!("✅ Page.navigate validation works");

    // Runtime.evaluate requires expression
    let json2 = r#"{
        "name": "test",
        "description": "Test",
        "cdp_commands": [
            {"method": "Runtime.evaluate", "params": {"returnByValue": true}}
        ]
    }"#;

    let result2 = validator.validate_json(json2);
    assert!(!result2.is_valid);
    assert!(result2
        .errors
        .iter()
        .any(|e| e.message.contains("expression")));
    println!("✅ Runtime.evaluate validation works");

    // Input.dispatchMouseEvent requires type, x, y
    let json3 = r#"{
        "name": "test",
        "description": "Test",
        "cdp_commands": [
            {"method": "Input.dispatchMouseEvent", "params": {"type": "mousePressed"}}
        ]
    }"#;

    let result3 = validator.validate_json(json3);
    assert!(!result3.is_valid);
    assert!(result3.errors.len() >= 2, "Should require both x and y");
    println!("✅ Input.dispatchMouseEvent validation works");

    // Network.setCookie requires name and value
    let json4 = r#"{
        "name": "test",
        "description": "Test",
        "cdp_commands": [
            {"method": "Network.setCookie", "params": {"domain": "example.com"}}
        ]
    }"#;

    let result4 = validator.validate_json(json4);
    assert!(!result4.is_valid);
    assert!(result4.errors.len() >= 2, "Should require name and value");
    println!("✅ Network.setCookie validation works");
}

#[test]
fn test_multiple_commands_with_mixed_errors() {
    let validator = CdpValidator::new();

    let json = r#"{
        "name": "test",
        "description": "Test with multiple errors",
        "cdp_commands": [
            {
                "method": "Page.navigate",
                "params": {"url": "https://example.com"}
            },
            {
                "method": "Invalid.command",
                "params": {}
            },
            {
                "method": "Runtime.evaluate",
                "params": {"returnByValue": "not-a-boolean"}
            },
            {
                "method": "Page.captureScreenshot",
                "params": {"format": 123}
            }
        ]
    }"#;

    let result = validator.validate_json(json);
    assert!(!result.is_valid);

    // Should catch:
    // 1. Unknown command (Invalid.command)
    // 2. Missing required parameter (expression in Runtime.evaluate)
    // 3. Type mismatch (returnByValue should be boolean)
    // 4. Type mismatch (format should be string)

    assert!(
        result.errors.len() >= 3,
        "Should catch multiple errors across commands"
    );

    let has_unknown_command = result
        .errors
        .iter()
        .any(|e| e.error_type == ValidationErrorType::UnknownCommand);
    let has_missing_param = result
        .errors
        .iter()
        .any(|e| e.error_type == ValidationErrorType::MissingParameter);
    let has_type_mismatch = result
        .errors
        .iter()
        .any(|e| e.error_type == ValidationErrorType::TypeMismatch);

    assert!(has_unknown_command, "Should catch unknown command");
    assert!(has_missing_param, "Should catch missing parameter");
    assert!(has_type_mismatch, "Should catch type mismatch");

    println!(
        "✅ Caught {} errors across multiple commands",
        result.errors.len()
    );
}

#[test]
fn test_validation_error_location_information() {
    let validator = CdpValidator::new();

    let json = r#"{
        "name": "test",
        "description": "Test",
        "cdp_commands": [
            {"method": "Page.navigate", "params": {"url": 123}},
            {"method": "Unknown.command", "params": {}}
        ]
    }"#;

    let result = validator.validate_json(json);
    assert!(!result.is_valid);

    // Check that errors have location information
    for error in &result.errors {
        assert!(
            !error.location.field_path.is_empty(),
            "Error should have field path"
        );

        if let Some(cmd_index) = error.location.command_index {
            assert!(cmd_index < 2, "Command index should be valid");
        }

        assert!(!error.message.is_empty(), "Error should have message");
    }

    // Check specific error has correct command index
    let unknown_cmd_error = result
        .errors
        .iter()
        .find(|e| e.error_type == ValidationErrorType::UnknownCommand);

    assert!(
        unknown_cmd_error.is_some(),
        "Should have unknown command error"
    );
    assert_eq!(
        unknown_cmd_error.unwrap().location.command_index,
        Some(1),
        "Unknown command error should point to command index 1"
    );

    println!("✅ Validation errors include proper location information");
}

#[test]
fn test_validation_suggestions() {
    let validator = CdpValidator::new();

    let json = r#"{
        "name": "test",
        "description": "Test",
        "cdp_commands": [
            {"method": "Unknown.command", "params": {}}
        ]
    }"#;

    let result = validator.validate_json(json);
    assert!(!result.is_valid);

    // Check that errors have helpful suggestions
    for error in &result.errors {
        if error.error_type == ValidationErrorType::UnknownCommand {
            assert!(
                error.suggestion.is_some(),
                "Unknown command error should have suggestion"
            );
            let suggestion = error.suggestion.as_ref().unwrap();
            assert!(
                suggestion.contains("Page.navigate") || suggestion.contains("Supported commands"),
                "Suggestion should mention supported commands"
            );
        }
    }

    println!("✅ Validation errors include helpful suggestions");
}

#[test]
fn test_warnings_for_non_critical_issues() {
    let validator = CdpValidator::new();

    let json = r#"{
        "name": "test-script-with-special-chars!",
        "description": "",
        "cdp_commands": [
            {
                "method": "Page.navigate",
                "params": {
                    "url": "https://example.com",
                    "unknownParameter": "value"
                }
            }
        ]
    }"#;

    let result = validator.validate_json(json);

    // Script should still be valid despite warnings
    assert!(
        result.is_valid,
        "Script with warnings should still be valid"
    );

    // But should have warnings
    assert!(
        !result.warnings.is_empty(),
        "Should have warnings for non-critical issues"
    );

    println!(
        "✅ Validation generates {} warnings for non-critical issues",
        result.warnings.len()
    );
}

#[test]
fn test_all_supported_commands() {
    let validator = CdpValidator::new();

    let supported_commands = vec![
        ("Page.navigate", r#"{"url": "https://example.com"}"#),
        ("Page.captureScreenshot", r#"{"format": "png"}"#),
        ("Page.reload", r#"{}"#),
        ("Page.goBack", r#"{}"#),
        ("Page.goForward", r#"{}"#),
        (
            "Runtime.evaluate",
            r#"{"expression": "document.title", "returnByValue": true}"#,
        ),
        ("Input.insertText", r#"{"text": "hello"}"#),
        (
            "Input.dispatchMouseEvent",
            r#"{"type": "mousePressed", "x": 100, "y": 200}"#,
        ),
        ("Input.dispatchKeyEvent", r#"{"type": "keyDown"}"#),
        ("Network.getCookies", r#"{}"#),
        ("Network.setCookie", r#"{"name": "test", "value": "value"}"#),
        ("Network.deleteCookies", r#"{"name": "test"}"#),
        (
            "Emulation.setGeolocationOverride",
            r#"{"latitude": 37.7749, "longitude": -122.4194}"#,
        ),
        (
            "Emulation.setDeviceMetricsOverride",
            r#"{"width": 1920, "height": 1080, "deviceScaleFactor": 1.0, "mobile": false}"#,
        ),
        ("Emulation.clearGeolocationOverride", r#"{}"#),
    ];

    for (method, params) in supported_commands {
        let json = format!(
            r#"{{
                "name": "test",
                "description": "Test {}",
                "cdp_commands": [
                    {{"method": "{}", "params": {}}}
                ]
            }}"#,
            method, method, params
        );

        let result = validator.validate_json(&json);
        assert!(result.is_valid, "{} should be recognized and valid", method);
        println!("✅ {} command validated successfully", method);
    }
}
