//! CDP Script Validation
//!
//! This module provides comprehensive validation of CDP scripts before execution,
//! catching errors early and providing detailed error messages.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Detailed validation error with location information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationError {
    /// Error type/category
    pub error_type: ValidationErrorType,

    /// Human-readable error message
    pub message: String,

    /// Location of the error (command index, field name, etc.)
    pub location: ErrorLocation,

    /// Suggestion for fixing the error
    pub suggestion: Option<String>,
}

/// Types of validation errors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationErrorType {
    /// JSON syntax error
    JsonSyntax,

    /// Missing required field
    MissingField,

    /// Invalid field value
    InvalidValue,

    /// Unknown CDP command
    UnknownCommand,

    /// Invalid parameter for CDP command
    InvalidParameter,

    /// Missing required parameter
    MissingParameter,

    /// Invalid JSON structure
    InvalidStructure,

    /// Type mismatch (expected string, got number, etc.)
    TypeMismatch,
}

/// Location information for errors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorLocation {
    /// Command index (0-based) if error is in a specific command
    pub command_index: Option<usize>,

    /// Field path (e.g., "cdp_commands[0].params.url")
    pub field_path: String,

    /// Line number in JSON (if available)
    pub line: Option<usize>,

    /// Column number in JSON (if available)
    pub column: Option<usize>,
}

/// Result of validation with all errors found
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the script is valid
    pub is_valid: bool,

    /// List of all validation errors
    pub errors: Vec<ValidationError>,

    /// List of warnings (non-blocking issues)
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a failed validation result with errors
    pub fn failure(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    /// Add an error to the result
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
        self.is_valid = false;
    }

    /// Add a warning to the result
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Comprehensive CDP script validator
pub struct CdpValidator {
    /// Valid CDP commands (domain.method)
    valid_commands: Vec<&'static str>,

    /// Parameter schemas for each command
    parameter_schemas: HashMap<&'static str, CommandSchema>,
}

/// Schema for a CDP command's parameters
#[derive(Debug, Clone)]
pub struct CommandSchema {
    /// Required parameter names
    pub required_params: Vec<&'static str>,

    /// Optional parameter names
    pub optional_params: Vec<&'static str>,

    /// Parameter type expectations
    pub param_types: HashMap<&'static str, ParamType>,
}

/// Expected parameter types
#[derive(Debug, Clone, PartialEq)]
pub enum ParamType {
    String,
    Number,
    Boolean,
    Object,
    Array,
}

impl CdpValidator {
    /// Create a new validator with all supported CDP commands
    pub fn new() -> Self {
        let valid_commands = vec![
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
            "Emulation.clearGeolocationOverride",
        ];

        let mut parameter_schemas = HashMap::new();

        // Page.navigate schema
        parameter_schemas.insert(
            "Page.navigate",
            CommandSchema {
                required_params: vec!["url"],
                optional_params: vec!["referrer", "transitionType", "frameId"],
                param_types: [
                    ("url", ParamType::String),
                    ("referrer", ParamType::String),
                    ("transitionType", ParamType::String),
                    ("frameId", ParamType::String),
                ]
                .into_iter()
                .collect(),
            },
        );

        // Page.captureScreenshot schema
        parameter_schemas.insert(
            "Page.captureScreenshot",
            CommandSchema {
                required_params: vec![],
                optional_params: vec!["format", "quality", "clip", "captureBeyondViewport"],
                param_types: [
                    ("format", ParamType::String),
                    ("quality", ParamType::Number),
                    ("clip", ParamType::Object),
                    ("captureBeyondViewport", ParamType::Boolean),
                ]
                .into_iter()
                .collect(),
            },
        );

        // Runtime.evaluate schema
        parameter_schemas.insert(
            "Runtime.evaluate",
            CommandSchema {
                required_params: vec!["expression"],
                optional_params: vec!["returnByValue", "awaitPromise", "userGesture"],
                param_types: [
                    ("expression", ParamType::String),
                    ("returnByValue", ParamType::Boolean),
                    ("awaitPromise", ParamType::Boolean),
                    ("userGesture", ParamType::Boolean),
                ]
                .into_iter()
                .collect(),
            },
        );

        // Input.insertText schema
        parameter_schemas.insert(
            "Input.insertText",
            CommandSchema {
                required_params: vec!["text"],
                optional_params: vec![],
                param_types: [("text", ParamType::String)].into_iter().collect(),
            },
        );

        // Input.dispatchMouseEvent schema
        parameter_schemas.insert(
            "Input.dispatchMouseEvent",
            CommandSchema {
                required_params: vec!["type", "x", "y"],
                optional_params: vec!["button", "clickCount", "modifiers"],
                param_types: [
                    ("type", ParamType::String),
                    ("x", ParamType::Number),
                    ("y", ParamType::Number),
                    ("button", ParamType::String),
                    ("clickCount", ParamType::Number),
                    ("modifiers", ParamType::Number),
                ]
                .into_iter()
                .collect(),
            },
        );

        // Input.dispatchKeyEvent schema
        parameter_schemas.insert(
            "Input.dispatchKeyEvent",
            CommandSchema {
                required_params: vec!["type"],
                optional_params: vec!["key", "code", "text", "modifiers"],
                param_types: [
                    ("type", ParamType::String),
                    ("key", ParamType::String),
                    ("code", ParamType::String),
                    ("text", ParamType::String),
                    ("modifiers", ParamType::Number),
                ]
                .into_iter()
                .collect(),
            },
        );

        // Network.setCookie schema
        parameter_schemas.insert(
            "Network.setCookie",
            CommandSchema {
                required_params: vec!["name", "value"],
                optional_params: vec!["url", "domain", "path", "secure", "httpOnly", "expires"],
                param_types: [
                    ("name", ParamType::String),
                    ("value", ParamType::String),
                    ("url", ParamType::String),
                    ("domain", ParamType::String),
                    ("path", ParamType::String),
                    ("secure", ParamType::Boolean),
                    ("httpOnly", ParamType::Boolean),
                    ("expires", ParamType::Number),
                ]
                .into_iter()
                .collect(),
            },
        );

        // Network.deleteCookies schema
        parameter_schemas.insert(
            "Network.deleteCookies",
            CommandSchema {
                required_params: vec!["name"],
                optional_params: vec!["url", "domain", "path"],
                param_types: [
                    ("name", ParamType::String),
                    ("url", ParamType::String),
                    ("domain", ParamType::String),
                    ("path", ParamType::String),
                ]
                .into_iter()
                .collect(),
            },
        );

        // Emulation.setGeolocationOverride schema
        parameter_schemas.insert(
            "Emulation.setGeolocationOverride",
            CommandSchema {
                required_params: vec![],
                optional_params: vec!["latitude", "longitude", "accuracy"],
                param_types: [
                    ("latitude", ParamType::Number),
                    ("longitude", ParamType::Number),
                    ("accuracy", ParamType::Number),
                ]
                .into_iter()
                .collect(),
            },
        );

        // Emulation.setDeviceMetricsOverride schema
        parameter_schemas.insert(
            "Emulation.setDeviceMetricsOverride",
            CommandSchema {
                required_params: vec!["width", "height", "deviceScaleFactor", "mobile"],
                optional_params: vec!["screenWidth", "screenHeight", "screenOrientation"],
                param_types: [
                    ("width", ParamType::Number),
                    ("height", ParamType::Number),
                    ("deviceScaleFactor", ParamType::Number),
                    ("mobile", ParamType::Boolean),
                    ("screenWidth", ParamType::Number),
                    ("screenHeight", ParamType::Number),
                    ("screenOrientation", ParamType::Object),
                ]
                .into_iter()
                .collect(),
            },
        );

        Self {
            valid_commands,
            parameter_schemas,
        }
    }

    /// Validate a CDP script from JSON string
    pub fn validate_json(&self, json: &str) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Try to parse JSON
        let script: crate::cdp::CdpScript = match serde_json::from_str(json) {
            Ok(s) => s,
            Err(e) => {
                result.add_error(ValidationError {
                    error_type: ValidationErrorType::JsonSyntax,
                    message: format!("JSON syntax error: {}", e),
                    location: ErrorLocation {
                        command_index: None,
                        field_path: String::new(),
                        line: e.line().into(),
                        column: e.column().into(),
                    },
                    suggestion: Some("Check for missing commas, brackets, or quotes".to_string()),
                });
                return result;
            }
        };

        // Validate script structure
        self.validate_script(&script, &mut result);

        result
    }

    /// Validate a parsed CDP script
    pub fn validate_script(&self, script: &crate::cdp::CdpScript, result: &mut ValidationResult) {
        // Validate script name
        if script.name.is_empty() {
            result.add_error(ValidationError {
                error_type: ValidationErrorType::MissingField,
                message: "Script name is required and cannot be empty".to_string(),
                location: ErrorLocation {
                    command_index: None,
                    field_path: "name".to_string(),
                    line: None,
                    column: None,
                },
                suggestion: Some("Add a descriptive name for your script".to_string()),
            });
        } else if !script
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            result.add_warning(
                "Script name should only contain alphanumeric characters, hyphens, and underscores"
                    .to_string(),
            );
        }

        // Validate script description
        if script.description.is_empty() {
            result.add_warning("Script description is empty".to_string());
        }

        // Validate commands
        if script.cdp_commands.is_empty() {
            result.add_error(ValidationError {
                error_type: ValidationErrorType::MissingField,
                message: "Script must contain at least one command".to_string(),
                location: ErrorLocation {
                    command_index: None,
                    field_path: "cdp_commands".to_string(),
                    line: None,
                    column: None,
                },
                suggestion: Some("Add at least one CDP command to the script".to_string()),
            });
            return;
        }

        // Validate each command
        for (index, cmd) in script.cdp_commands.iter().enumerate() {
            self.validate_command(cmd, index, result);
        }
    }

    /// Validate a single CDP command
    fn validate_command(
        &self,
        cmd: &crate::cdp::CdpCommand,
        index: usize,
        result: &mut ValidationResult,
    ) {
        let field_prefix = format!("cdp_commands[{}]", index);

        // Validate method name format
        if cmd.method.is_empty() {
            result.add_error(ValidationError {
                error_type: ValidationErrorType::MissingField,
                message: format!("Command {} has empty method name", index + 1),
                location: ErrorLocation {
                    command_index: Some(index),
                    field_path: format!("{}.method", field_prefix),
                    line: None,
                    column: None,
                },
                suggestion: Some("Specify a CDP method in Domain.method format".to_string()),
            });
            return;
        }

        if !cmd.method.contains('.') {
            result.add_error(ValidationError {
                error_type: ValidationErrorType::InvalidValue,
                message: format!(
                    "Command {} has invalid method '{}' (must be Domain.method format)",
                    index + 1,
                    cmd.method
                ),
                location: ErrorLocation {
                    command_index: Some(index),
                    field_path: format!("{}.method", field_prefix),
                    line: None,
                    column: None,
                },
                suggestion: Some(
                    "Use format like 'Page.navigate' or 'Runtime.evaluate'".to_string(),
                ),
            });
            return;
        }

        // Check if command is supported
        if !self.valid_commands.contains(&cmd.method.as_str()) {
            result.add_error(ValidationError {
                error_type: ValidationErrorType::UnknownCommand,
                message: format!("Unknown CDP command: {}", cmd.method),
                location: ErrorLocation {
                    command_index: Some(index),
                    field_path: format!("{}.method", field_prefix),
                    line: None,
                    column: None,
                },
                suggestion: Some(format!(
                    "Supported commands: {}",
                    self.valid_commands.join(", ")
                )),
            });
            return;
        }

        // Validate parameters against schema
        if let Some(schema) = self.parameter_schemas.get(cmd.method.as_str()) {
            self.validate_parameters(cmd, schema, index, &field_prefix, result);
        }
    }

    /// Validate command parameters against schema
    fn validate_parameters(
        &self,
        cmd: &crate::cdp::CdpCommand,
        schema: &CommandSchema,
        index: usize,
        field_prefix: &str,
        result: &mut ValidationResult,
    ) {
        let params = cmd.params.as_object();

        if params.is_none() {
            if !schema.required_params.is_empty() {
                result.add_error(ValidationError {
                    error_type: ValidationErrorType::InvalidStructure,
                    message: format!("Command {} params must be an object", index + 1),
                    location: ErrorLocation {
                        command_index: Some(index),
                        field_path: format!("{}.params", field_prefix),
                        line: None,
                        column: None,
                    },
                    suggestion: Some(
                        "Params should be a JSON object with key-value pairs".to_string(),
                    ),
                });
            }
            return;
        }

        let params = params.unwrap();

        // Check required parameters
        for required_param in &schema.required_params {
            if !params.contains_key(*required_param) {
                result.add_error(ValidationError {
                    error_type: ValidationErrorType::MissingParameter,
                    message: format!(
                        "Command {} ({}) missing required parameter '{}'",
                        index + 1,
                        cmd.method,
                        required_param
                    ),
                    location: ErrorLocation {
                        command_index: Some(index),
                        field_path: format!("{}.params.{}", field_prefix, required_param),
                        line: None,
                        column: None,
                    },
                    suggestion: Some(format!("Add '{}' parameter", required_param)),
                });
            }
        }

        // Validate parameter types
        for (param_name, param_value) in params.iter() {
            if let Some(expected_type) = schema.param_types.get(param_name.as_str()) {
                let actual_type = match param_value {
                    serde_json::Value::String(_) => ParamType::String,
                    serde_json::Value::Number(_) => ParamType::Number,
                    serde_json::Value::Bool(_) => ParamType::Boolean,
                    serde_json::Value::Object(_) => ParamType::Object,
                    serde_json::Value::Array(_) => ParamType::Array,
                    serde_json::Value::Null => continue, // Null is acceptable
                };

                if &actual_type != expected_type {
                    result.add_error(ValidationError {
                        error_type: ValidationErrorType::TypeMismatch,
                        message: format!(
                            "Command {} ({}) parameter '{}' has wrong type (expected {:?}, got {:?})",
                            index + 1,
                            cmd.method,
                            param_name,
                            expected_type,
                            actual_type
                        ),
                        location: ErrorLocation {
                            command_index: Some(index),
                            field_path: format!("{}.params.{}", field_prefix, param_name),
                            line: None,
                            column: None,
                        },
                        suggestion: Some(format!("Change '{}' to be a {:?}", param_name, expected_type)),
                    });
                }
            } else if !schema.required_params.contains(&param_name.as_str())
                && !schema.optional_params.contains(&param_name.as_str())
            {
                result.add_warning(format!(
                    "Command {} ({}) has unknown parameter '{}' (will be passed through)",
                    index + 1,
                    cmd.method,
                    param_name
                ));
            }
        }
    }
}

impl Default for CdpValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_script() {
        let validator = CdpValidator::new();
        let json = r#"{
            "name": "test-script",
            "description": "Test script",
            "cdp_commands": [
                {
                    "method": "Page.navigate",
                    "params": {"url": "https://example.com"}
                }
            ]
        }"#;

        let result = validator.validate_json(json);
        assert!(result.is_valid, "Valid script should pass validation");
        assert!(result.errors.is_empty(), "Should have no errors");
    }

    #[test]
    fn test_json_syntax_error() {
        let validator = CdpValidator::new();
        let json = r#"{"name": "test", "description": "test"#; // Missing closing brace

        let result = validator.validate_json(json);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].error_type, ValidationErrorType::JsonSyntax);
    }

    #[test]
    fn test_missing_script_name() {
        let validator = CdpValidator::new();
        let json = r#"{
            "name": "",
            "description": "Test",
            "cdp_commands": [
                {"method": "Page.navigate", "params": {"url": "https://example.com"}}
            ]
        }"#;

        let result = validator.validate_json(json);
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == ValidationErrorType::MissingField
                && e.location.field_path == "name"));
    }

    #[test]
    fn test_unknown_command() {
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
        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == ValidationErrorType::UnknownCommand));
    }

    #[test]
    fn test_invalid_method_format() {
        let validator = CdpValidator::new();
        let json = r#"{
            "name": "test",
            "description": "Test",
            "cdp_commands": [
                {"method": "InvalidFormat", "params": {}}
            ]
        }"#;

        let result = validator.validate_json(json);
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == ValidationErrorType::InvalidValue));
    }

    #[test]
    fn test_missing_required_parameter() {
        let validator = CdpValidator::new();
        let json = r#"{
            "name": "test",
            "description": "Test",
            "cdp_commands": [
                {"method": "Page.navigate", "params": {}}
            ]
        }"#;

        let result = validator.validate_json(json);
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == ValidationErrorType::MissingParameter
                && e.message.contains("url")));
    }

    #[test]
    fn test_wrong_parameter_type() {
        let validator = CdpValidator::new();
        let json = r#"{
            "name": "test",
            "description": "Test",
            "cdp_commands": [
                {"method": "Page.navigate", "params": {"url": 123}}
            ]
        }"#;

        let result = validator.validate_json(json);
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == ValidationErrorType::TypeMismatch));
    }

    #[test]
    fn test_empty_commands() {
        let validator = CdpValidator::new();
        let json = r#"{
            "name": "test",
            "description": "Test",
            "cdp_commands": []
        }"#;

        let result = validator.validate_json(json);
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.error_type == ValidationErrorType::MissingField
                && e.location.field_path == "cdp_commands"));
    }

    #[test]
    fn test_multiple_errors() {
        let validator = CdpValidator::new();
        let json = r#"{
            "name": "",
            "description": "",
            "cdp_commands": [
                {"method": "Invalid", "params": {}},
                {"method": "Page.navigate", "params": {}},
                {"method": "Unknown.command", "params": {}}
            ]
        }"#;

        let result = validator.validate_json(json);
        assert!(!result.is_valid);
        assert!(result.errors.len() >= 3, "Should catch multiple errors");
    }
}
