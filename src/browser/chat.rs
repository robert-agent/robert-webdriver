//! Chat UI Injection Module
//!
//! Provides functionality to inject a chat interface into web pages
//! for real-time user feedback during agent operations.

use crate::error::{BrowserError, Result};

/// The JavaScript code for the chat UI
/// This is embedded at compile time from chat_ui.js
const CHAT_UI_SCRIPT: &str = include_str!("../chat_ui.js");

/// Chat UI manager for injecting and interacting with the chat interface
pub struct ChatUI {
    enabled: bool,
}

impl ChatUI {
    /// Create a new ChatUI instance
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// Create a ChatUI instance with enabled/disabled state
    pub fn with_enabled(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Check if chat UI injection is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable chat UI injection
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable chat UI injection
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Get the chat UI injection script
    pub fn get_injection_script(&self) -> &str {
        CHAT_UI_SCRIPT
    }

    /// Inject the chat UI into a page
    pub async fn inject(&self, page: &chromiumoxide::page::Page) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        page.evaluate(CHAT_UI_SCRIPT)
            .await
            .map_err(|e| BrowserError::Other(format!("Failed to inject chat UI: {}", e)))?;

        Ok(())
    }

    /// Send a message from the agent to the chat UI
    pub async fn send_agent_message(
        &self,
        page: &chromiumoxide::page::Page,
        message: &str,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Escape the message for JavaScript
        let escaped_message = message
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");

        let script = format!(
            r#"
            if (window.__ROBERT_CHAT_API__) {{
                window.__ROBERT_CHAT_API__.sendMessage("{}");
            }}
            "#,
            escaped_message
        );

        page.evaluate(script.as_str())
            .await
            .map_err(|e| BrowserError::Other(format!("Failed to send agent message: {}", e)))?;

        Ok(())
    }

    /// Retrieve all messages from the chat UI
    pub async fn get_messages(&self, page: &chromiumoxide::page::Page) -> Result<Vec<ChatMessage>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let script = r#"
            window.__ROBERT_CHAT_MESSAGES__ || []
        "#;

        let result = page
            .evaluate(script)
            .await
            .map_err(|e| BrowserError::Other(format!("Failed to get chat messages: {}", e)))?;

        let messages: Vec<ChatMessage> = result
            .into_value()
            .map_err(|e| BrowserError::Other(format!("Failed to parse chat messages: {}", e)))?;

        Ok(messages)
    }

    /// Clear all messages from the chat UI
    pub async fn clear_messages(&self, page: &chromiumoxide::page::Page) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let script = r#"
            if (window.__ROBERT_CHAT_API__) {
                window.__ROBERT_CHAT_API__.clearMessages();
            }
        "#;

        page.evaluate(script)
            .await
            .map_err(|e| BrowserError::Other(format!("Failed to clear chat messages: {}", e)))?;

        Ok(())
    }

    /// Collapse the chat sidebar
    pub async fn collapse(&self, page: &chromiumoxide::page::Page) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let script = r#"
            if (window.__ROBERT_CHAT_API__) {
                window.__ROBERT_CHAT_API__.collapse();
            }
        "#;

        page.evaluate(script)
            .await
            .map_err(|e| BrowserError::Other(format!("Failed to collapse chat: {}", e)))?;

        Ok(())
    }

    /// Expand the chat sidebar
    pub async fn expand(&self, page: &chromiumoxide::page::Page) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let script = r#"
            if (window.__ROBERT_CHAT_API__) {
                window.__ROBERT_CHAT_API__.expand();
            }
        "#;

        page.evaluate(script)
            .await
            .map_err(|e| BrowserError::Other(format!("Failed to expand chat: {}", e)))?;

        Ok(())
    }

    /// Get unprocessed messages from the chat (messages waiting for agent response)
    pub async fn get_unprocessed_messages(
        &self,
        page: &chromiumoxide::page::Page,
    ) -> Result<Vec<ChatMessage>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let script = r#"
            window.__ROBERT_UNPROCESSED_MESSAGES__ || []
        "#;

        let result = page.evaluate(script).await.map_err(|e| {
            BrowserError::Other(format!("Failed to get unprocessed messages: {}", e))
        })?;

        let messages: Vec<ChatMessage> = result.into_value().map_err(|e| {
            BrowserError::Other(format!("Failed to parse unprocessed messages: {}", e))
        })?;

        Ok(messages)
    }

    /// Clear unprocessed messages after they have been handled
    pub async fn clear_unprocessed_messages(&self, page: &chromiumoxide::page::Page) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let script = r#"
            window.__ROBERT_UNPROCESSED_MESSAGES__ = [];
        "#;

        page.evaluate(script).await.map_err(|e| {
            BrowserError::Other(format!("Failed to clear unprocessed messages: {}", e))
        })?;

        Ok(())
    }

    /// Get feedback submissions from users
    pub async fn get_feedback(
        &self,
        page: &chromiumoxide::page::Page,
    ) -> Result<Vec<UserFeedback>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let script = r#"
            window.__ROBERT_FEEDBACK__ || []
        "#;

        let result = page
            .evaluate(script)
            .await
            .map_err(|e| BrowserError::Other(format!("Failed to get feedback: {}", e)))?;

        let feedback: Vec<UserFeedback> = result
            .into_value()
            .map_err(|e| BrowserError::Other(format!("Failed to parse feedback: {}", e)))?;

        Ok(feedback)
    }

    /// Clear feedback after it has been processed
    pub async fn clear_feedback(&self, page: &chromiumoxide::page::Page) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let script = r#"
            window.__ROBERT_FEEDBACK__ = [];
        "#;

        page.evaluate(script)
            .await
            .map_err(|e| BrowserError::Other(format!("Failed to clear feedback: {}", e)))?;

        Ok(())
    }
}

impl Default for ChatUI {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a message in the chat
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub text: String,
    pub sender: String,
    pub timestamp: u64,
}

/// Represents user feedback on an agent action
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserFeedback {
    pub action_id: String,
    pub positive: bool,
    pub comment: Option<String>,
    pub agent_name: String,
    pub original_request: String,
    pub error_description: Option<String>,
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_ui_creation() {
        let chat_ui = ChatUI::new();
        assert!(chat_ui.is_enabled());

        let chat_ui_disabled = ChatUI::with_enabled(false);
        assert!(!chat_ui_disabled.is_enabled());
    }

    #[test]
    fn test_chat_ui_enable_disable() {
        let mut chat_ui = ChatUI::new();
        assert!(chat_ui.is_enabled());

        chat_ui.disable();
        assert!(!chat_ui.is_enabled());

        chat_ui.enable();
        assert!(chat_ui.is_enabled());
    }

    #[test]
    fn test_injection_script_available() {
        let chat_ui = ChatUI::new();
        let script = chat_ui.get_injection_script();
        assert!(!script.is_empty());
        assert!(script.contains("robert-chat-container"));
    }
}
