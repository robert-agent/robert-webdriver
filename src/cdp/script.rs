//! CDP Script Types
//!
//! Defines the JSON structure for CDP automation scripts.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

/// A CDP automation script containing a sequence of Chrome DevTools Protocol commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpScript {
    /// Unique script name (lowercase-hyphenated)
    pub name: String,

    /// Human-readable description of what this script does
    pub description: String,

    /// Script creation timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,

    /// Author (typically "Claude" for AI-generated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Tags for categorization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Sequence of CDP commands to execute
    pub cdp_commands: Vec<CdpCommand>,
}

/// A single CDP command with method name and parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpCommand {
    /// CDP method identifier (e.g., "Page.navigate", "Runtime.evaluate")
    pub method: String,

    /// JSON parameters for the command
    pub params: serde_json::Value,

    /// Optional: save command output to file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_as: Option<String>,

    /// Optional: description of this command step
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Result of executing a single CDP command
#[derive(Debug, Clone, Serialize)]
pub struct CommandResult {
    /// Step number (1-indexed)
    pub step: usize,

    /// CDP method that was executed
    pub method: String,

    /// Execution status
    pub status: CommandStatus,

    /// How long the command took to execute
    pub duration: Duration,

    /// Response from Chrome (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<serde_json::Value>,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Optional: file saved (if save_as was used)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saved_file: Option<String>,
}

/// Status of command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandStatus {
    Success,
    Failed,
    Skipped,
}

/// Complete report of script execution
#[derive(Debug, Clone, Serialize)]
pub struct ExecutionReport {
    /// Name of the script that was executed
    pub script_name: String,

    /// Total number of commands in the script
    pub total_commands: usize,

    /// Number of successfully executed commands
    pub successful: usize,

    /// Number of failed commands
    pub failed: usize,

    /// Number of skipped commands
    pub skipped: usize,

    /// Total execution time
    pub total_duration: Duration,

    /// Individual command results
    pub results: Vec<CommandResult>,
}

impl CdpScript {
    /// Load a CDP script from a JSON file
    pub async fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let script: CdpScript = serde_json::from_str(&content)?;
        Ok(script)
    }

    /// Save this script to a JSON file
    pub async fn to_file(&self, path: &Path) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    /// Validate script structure (basic checks)
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.name.is_empty() {
            anyhow::bail!("Script name cannot be empty");
        }

        if self.cdp_commands.is_empty() {
            anyhow::bail!("Script must contain at least one command");
        }

        for (i, cmd) in self.cdp_commands.iter().enumerate() {
            if cmd.method.is_empty() {
                anyhow::bail!("Command {} has empty method", i + 1);
            }

            if !cmd.method.contains('.') {
                anyhow::bail!(
                    "Command {} has invalid method '{}' (must be Domain.method format)",
                    i + 1,
                    cmd.method
                );
            }
        }

        Ok(())
    }
}

impl ExecutionReport {
    /// Create a new execution report
    pub fn new(script_name: String, total_commands: usize) -> Self {
        Self {
            script_name,
            total_commands,
            successful: 0,
            failed: 0,
            skipped: 0,
            total_duration: Duration::from_secs(0),
            results: Vec::with_capacity(total_commands),
        }
    }

    /// Add a command result and update counters
    pub fn add_result(&mut self, result: CommandResult) {
        self.total_duration += result.duration;

        match result.status {
            CommandStatus::Success => self.successful += 1,
            CommandStatus::Failed => self.failed += 1,
            CommandStatus::Skipped => self.skipped += 1,
        }

        self.results.push(result);
    }

    /// Check if the script execution was completely successful
    pub fn is_success(&self) -> bool {
        self.failed == 0 && self.successful == self.total_commands
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_commands == 0 {
            return 0.0;
        }
        (self.successful as f64 / self.total_commands as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_validation() {
        let mut script = CdpScript {
            name: "test".to_string(),
            description: "Test script".to_string(),
            created: None,
            author: None,
            tags: vec![],
            cdp_commands: vec![],
        };

        // Empty commands should fail
        assert!(script.validate().is_err());

        // Add valid command
        script.cdp_commands.push(CdpCommand {
            method: "Page.navigate".to_string(),
            params: serde_json::json!({"url": "https://example.com"}),
            save_as: None,
            description: None,
        });

        assert!(script.validate().is_ok());

        // Invalid method format should fail
        script.cdp_commands.push(CdpCommand {
            method: "InvalidMethod".to_string(),
            params: serde_json::json!({}),
            save_as: None,
            description: None,
        });

        assert!(script.validate().is_err());
    }

    #[test]
    fn test_execution_report() {
        let mut report = ExecutionReport::new("test".to_string(), 3);

        report.add_result(CommandResult {
            step: 1,
            method: "Page.navigate".to_string(),
            status: CommandStatus::Success,
            duration: Duration::from_millis(100),
            response: None,
            error: None,
            saved_file: None,
        });

        report.add_result(CommandResult {
            step: 2,
            method: "Runtime.evaluate".to_string(),
            status: CommandStatus::Failed,
            duration: Duration::from_millis(50),
            response: None,
            error: Some("Error".to_string()),
            saved_file: None,
        });

        assert_eq!(report.successful, 1);
        assert_eq!(report.failed, 1);
        assert!(!report.is_success());

        // Use approximate comparison for floating point
        let success_rate = report.success_rate();
        assert!((success_rate - 33.333333333333336).abs() < 0.0001);
    }
}
