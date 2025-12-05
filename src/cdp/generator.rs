//! CDP Script Generator using Claude CLI
//!
//! This module handles generating CDP scripts from natural language using Claude.

use super::claude_prompt::{generate_cdp_script_prompt, validate_generated_script};
use super::CdpScript;
use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// CDP Script Generator using Claude CLI
pub struct CdpScriptGenerator {
    claude_path: String,
    model: Option<String>,
}

impl CdpScriptGenerator {
    /// Create a new generator with default Claude CLI path
    pub fn new() -> Self {
        Self {
            claude_path: "claude".to_string(),
            model: None,
        }
    }

    /// Set custom Claude CLI path
    pub fn with_claude_path(mut self, path: String) -> Self {
        self.claude_path = path;
        self
    }

    /// Set Claude model to use (e.g., "sonnet", "opus")
    pub fn with_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    /// Generate a CDP script from a natural language description
    ///
    /// # Arguments
    /// * `description` - Natural language description of the automation task
    ///
    /// # Returns
    /// * `CdpScript` - Validated CDP script ready for execution
    ///
    /// # Errors
    /// * If Claude CLI is not available
    /// * If Claude generates invalid JSON
    /// * If generated script fails validation
    pub async fn generate(&self, description: &str) -> Result<CdpScript> {
        // Generate prompt
        let prompt = generate_cdp_script_prompt(description);

        // Call Claude CLI
        let response = self.call_claude(&prompt).await?;

        // Clean response (remove markdown code blocks if present)
        let json = self.clean_response(&response);

        // Validate and parse
        let script = validate_generated_script(&json)
            .map_err(|e| anyhow::anyhow!("Validation failed: {}", e))?;

        Ok(script)
    }

    /// Generate with retry on failure
    pub async fn generate_with_retry(
        &self,
        description: &str,
        max_retries: u32,
    ) -> Result<CdpScript> {
        let mut last_error = None;

        for attempt in 1..=max_retries {
            match self.generate(description).await {
                Ok(script) => return Ok(script),
                Err(e) => {
                    eprintln!(
                        "Generation attempt {}/{} failed: {}",
                        attempt, max_retries, e
                    );
                    last_error = Some(e);

                    if attempt < max_retries {
                        // Wait before retry
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Generation failed")))
    }

    /// Call Claude CLI with a prompt
    async fn call_claude(&self, prompt: &str) -> Result<String> {
        // Build command
        let mut cmd = Command::new(&self.claude_path);
        cmd.arg("--print") // Non-interactive mode
            .arg("--output-format")
            .arg("json") // JSON output
            .arg("--dangerously-skip-permissions"); // Skip permission prompts for automation

        // Add model if specified
        if let Some(model) = &self.model {
            cmd.arg("--model").arg(model);
        }

        // Pipe prompt to stdin
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .context("Failed to spawn Claude CLI. Is 'claude' installed?")?;

        // Write prompt to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(prompt.as_bytes())
                .await
                .context("Failed to write prompt to Claude")?;
            stdin.shutdown().await.context("Failed to close stdin")?;
        }

        // Wait for completion
        let output = child
            .wait_with_output()
            .await
            .context("Failed to wait for Claude CLI")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Claude CLI failed: {}", stderr);
        }

        // Parse Claude's JSON response
        let stdout = String::from_utf8_lossy(&output.stdout);
        let response: serde_json::Value =
            serde_json::from_str(&stdout).context("Failed to parse Claude CLI output as JSON")?;

        // Extract text from response
        let text = response
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Claude response missing 'text' field"))?;

        Ok(text.to_string())
    }

    /// Clean Claude's response (remove markdown formatting if present)
    fn clean_response(&self, response: &str) -> String {
        let trimmed = response.trim();

        // Remove markdown code blocks
        if trimmed.starts_with("```json") {
            trimmed
                .strip_prefix("```json")
                .and_then(|s| s.strip_suffix("```"))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| trimmed.to_string())
        } else if trimmed.starts_with("```") {
            trimmed
                .strip_prefix("```")
                .and_then(|s| s.strip_suffix("```"))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| trimmed.to_string())
        } else {
            trimmed.to_string()
        }
    }
}

impl Default for CdpScriptGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_response() {
        let gen = CdpScriptGenerator::new();

        // Test with markdown
        let response = r#"```json
{"name": "test"}
```"#;
        let cleaned = gen.clean_response(response);
        assert_eq!(cleaned, r#"{"name": "test"}"#);

        // Test without markdown
        let response = r#"{"name": "test"}"#;
        let cleaned = gen.clean_response(response);
        assert_eq!(cleaned, r#"{"name": "test"}"#);
    }

    // Note: Integration tests for generation are in tests/cdp_generator_test.rs
    // They require external Claude CLI and are excluded from CI
}
