//! CDP Command Executor
//!
//! Runtime interpreter that executes CDP commands via spider_chrome's Page API.

use super::script::{CdpCommand, CdpScript, CommandResult, CommandStatus, ExecutionReport};
use anyhow::{Context, Result};
use serde_json::Value;
use std::time::Instant;

// Import spider_chrome types
// Note: We use chromiumoxide module names because spider_chrome re-exports them
use chromiumoxide::cdp::browser_protocol::emulation;
use chromiumoxide::cdp::browser_protocol::input;
use chromiumoxide::cdp::browser_protocol::network;
use chromiumoxide::cdp::browser_protocol::page;
use chromiumoxide::cdp::js_protocol::runtime;
use chromiumoxide::page::Page;

/// CDP Script Executor
///
/// Executes CDP scripts by dispatching JSON commands to typed CDP command structs
/// and executing them via spider_chrome's Page API.
pub struct CdpExecutor {
    page: Page,
}

impl CdpExecutor {
    /// Create a new executor with the given Page
    pub fn new(page: Page) -> Self {
        Self { page }
    }

    /// Execute a complete CDP script
    pub async fn execute_script(&self, script: &CdpScript) -> Result<ExecutionReport> {
        // Validate script before execution
        script.validate()?;

        let mut report = ExecutionReport::new(script.name.clone(), script.cdp_commands.len());

        for (i, cmd) in script.cdp_commands.iter().enumerate() {
            let step = i + 1;
            let start = Instant::now();

            match self.execute_command(cmd).await {
                Ok((response, saved_file)) => {
                    report.add_result(CommandResult {
                        step,
                        method: cmd.method.clone(),
                        status: CommandStatus::Success,
                        duration: start.elapsed(),
                        response: Some(response),
                        error: None,
                        saved_file,
                    });
                }
                Err(e) => {
                    report.add_result(CommandResult {
                        step,
                        method: cmd.method.clone(),
                        status: CommandStatus::Failed,
                        duration: start.elapsed(),
                        response: None,
                        error: Some(e.to_string()),
                        saved_file: None,
                    });

                    // Stop execution on first error
                    // TODO: Make this configurable (continue_on_error flag)
                    break;
                }
            }
        }

        Ok(report)
    }

    /// Execute a single CDP command
    ///
    /// Returns (response_json, optional_saved_file_path)
    async fn execute_command(&self, cmd: &CdpCommand) -> Result<(Value, Option<String>)> {
        match cmd.method.as_str() {
            // ===== PAGE DOMAIN =====
            "Page.navigate" => self.execute_page_navigate(cmd).await,
            "Page.captureScreenshot" => self.execute_page_capture_screenshot(cmd).await,
            "Page.reload" => self.execute_page_reload(cmd).await,
            "Page.goBack" => self.execute_page_go_back(cmd).await,
            "Page.goForward" => self.execute_page_go_forward(cmd).await,

            // ===== RUNTIME DOMAIN =====
            "Runtime.evaluate" => self.execute_runtime_evaluate(cmd).await,

            // ===== INPUT DOMAIN =====
            "Input.insertText" => self.execute_input_insert_text(cmd).await,
            "Input.dispatchMouseEvent" => self.execute_input_dispatch_mouse_event(cmd).await,
            "Input.dispatchKeyEvent" => self.execute_input_dispatch_key_event(cmd).await,

            // ===== NETWORK DOMAIN =====
            "Network.getCookies" => self.execute_network_get_cookies(cmd).await,
            "Network.setCookie" => self.execute_network_set_cookie(cmd).await,
            "Network.deleteCookies" => self.execute_network_delete_cookies(cmd).await,

            // ===== EMULATION DOMAIN =====
            "Emulation.setGeolocationOverride" => self.execute_emulation_set_geolocation(cmd).await,
            "Emulation.setDeviceMetricsOverride" => {
                self.execute_emulation_set_device_metrics(cmd).await
            }

            // Unsupported method
            _ => {
                anyhow::bail!("Unsupported CDP method: {}", cmd.method);
            }
        }
    }

    // ===== PAGE DOMAIN IMPLEMENTATIONS =====

    async fn execute_page_navigate(&self, cmd: &CdpCommand) -> Result<(Value, Option<String>)> {
        let params: page::NavigateParams = serde_json::from_value(cmd.params.clone())
            .context("Failed to parse Page.navigate parameters")?;

        let response = self
            .page
            .execute(params)
            .await
            .context("Page.navigate failed")?;

        Ok((serde_json::to_value(&*response)?, None))
    }

    async fn execute_page_capture_screenshot(
        &self,
        cmd: &CdpCommand,
    ) -> Result<(Value, Option<String>)> {
        let params: page::CaptureScreenshotParams = serde_json::from_value(cmd.params.clone())
            .context("Failed to parse Page.captureScreenshot parameters")?;

        let response = self
            .page
            .execute(params)
            .await
            .context("Page.captureScreenshot failed")?;

        // Handle saving screenshot to file
        let saved_file = if let Some(filename) = &cmd.save_as {
            // Decode base64 image data
            use base64::{engine::general_purpose, Engine as _};
            let image_data = general_purpose::STANDARD
                .decode(&response.data)
                .context("Failed to decode screenshot base64 data")?;

            // Save to file
            tokio::fs::write(filename, image_data)
                .await
                .context("Failed to write screenshot to file")?;

            Some(filename.clone())
        } else {
            None
        };

        Ok((serde_json::to_value(&*response)?, saved_file))
    }

    async fn execute_page_reload(&self, cmd: &CdpCommand) -> Result<(Value, Option<String>)> {
        let params: page::ReloadParams = serde_json::from_value(cmd.params.clone())
            .context("Failed to parse Page.reload parameters")?;

        let response = self
            .page
            .execute(params)
            .await
            .context("Page.reload failed")?;

        Ok((serde_json::to_value(&*response)?, None))
    }

    async fn execute_page_go_back(&self, cmd: &CdpCommand) -> Result<(Value, Option<String>)> {
        let params: page::NavigateToHistoryEntryParams = serde_json::from_value(cmd.params.clone())
            .context("Failed to parse Page.goBack parameters")?;

        let response = self
            .page
            .execute(params)
            .await
            .context("Page.goBack failed")?;

        Ok((serde_json::to_value(&*response)?, None))
    }

    async fn execute_page_go_forward(&self, cmd: &CdpCommand) -> Result<(Value, Option<String>)> {
        let params: page::NavigateToHistoryEntryParams = serde_json::from_value(cmd.params.clone())
            .context("Failed to parse Page.goForward parameters")?;

        let response = self
            .page
            .execute(params)
            .await
            .context("Page.goForward failed")?;

        Ok((serde_json::to_value(&*response)?, None))
    }

    // ===== RUNTIME DOMAIN IMPLEMENTATIONS =====

    async fn execute_runtime_evaluate(&self, cmd: &CdpCommand) -> Result<(Value, Option<String>)> {
        let params: runtime::EvaluateParams = serde_json::from_value(cmd.params.clone())
            .context("Failed to parse Runtime.evaluate parameters")?;

        let response = self
            .page
            .execute(params)
            .await
            .context("Runtime.evaluate failed")?;

        // Handle saving result to file
        let saved_file = if let Some(filename) = &cmd.save_as {
            // Serialize the result value to JSON string
            let content = serde_json::to_string_pretty(&response.result)?;
            tokio::fs::write(filename, content)
                .await
                .context("Failed to write evaluate result to file")?;
            Some(filename.clone())
        } else {
            None
        };

        Ok((serde_json::to_value(&*response)?, saved_file))
    }

    // ===== INPUT DOMAIN IMPLEMENTATIONS =====

    async fn execute_input_insert_text(&self, cmd: &CdpCommand) -> Result<(Value, Option<String>)> {
        let params: input::InsertTextParams = serde_json::from_value(cmd.params.clone())
            .context("Failed to parse Input.insertText parameters")?;

        let response = self
            .page
            .execute(params)
            .await
            .context("Input.insertText failed")?;

        Ok((serde_json::to_value(&*response)?, None))
    }

    async fn execute_input_dispatch_mouse_event(
        &self,
        cmd: &CdpCommand,
    ) -> Result<(Value, Option<String>)> {
        let params: input::DispatchMouseEventParams = serde_json::from_value(cmd.params.clone())
            .context("Failed to parse Input.dispatchMouseEvent parameters")?;

        let response = self
            .page
            .execute(params)
            .await
            .context("Input.dispatchMouseEvent failed")?;

        Ok((serde_json::to_value(&*response)?, None))
    }

    async fn execute_input_dispatch_key_event(
        &self,
        cmd: &CdpCommand,
    ) -> Result<(Value, Option<String>)> {
        let params: input::DispatchKeyEventParams = serde_json::from_value(cmd.params.clone())
            .context("Failed to parse Input.dispatchKeyEvent parameters")?;

        let response = self
            .page
            .execute(params)
            .await
            .context("Input.dispatchKeyEvent failed")?;

        Ok((serde_json::to_value(&*response)?, None))
    }

    // ===== NETWORK DOMAIN IMPLEMENTATIONS =====

    async fn execute_network_get_cookies(
        &self,
        cmd: &CdpCommand,
    ) -> Result<(Value, Option<String>)> {
        let params: network::GetCookiesParams = serde_json::from_value(cmd.params.clone())
            .context("Failed to parse Network.getCookies parameters")?;

        let response = self
            .page
            .execute(params)
            .await
            .context("Network.getCookies failed")?;

        // Optionally save cookies to file
        let saved_file = if let Some(filename) = &cmd.save_as {
            let json = serde_json::to_string_pretty(&response.cookies)?;
            tokio::fs::write(filename, json)
                .await
                .context("Failed to write cookies to file")?;
            Some(filename.clone())
        } else {
            None
        };

        Ok((serde_json::to_value(&*response)?, saved_file))
    }

    async fn execute_network_set_cookie(
        &self,
        cmd: &CdpCommand,
    ) -> Result<(Value, Option<String>)> {
        let params: network::SetCookiesParams = serde_json::from_value(cmd.params.clone())
            .context("Failed to parse Network.setCookie parameters")?;

        let response = self
            .page
            .execute(params)
            .await
            .context("Network.setCookie failed")?;

        Ok((serde_json::to_value(&*response)?, None))
    }

    async fn execute_network_delete_cookies(
        &self,
        cmd: &CdpCommand,
    ) -> Result<(Value, Option<String>)> {
        let params: network::DeleteCookiesParams = serde_json::from_value(cmd.params.clone())
            .context("Failed to parse Network.deleteCookies parameters")?;

        let response = self
            .page
            .execute(params)
            .await
            .context("Network.deleteCookies failed")?;

        Ok((serde_json::to_value(&*response)?, None))
    }

    // ===== EMULATION DOMAIN IMPLEMENTATIONS =====

    async fn execute_emulation_set_geolocation(
        &self,
        cmd: &CdpCommand,
    ) -> Result<(Value, Option<String>)> {
        let params: emulation::SetGeolocationOverrideParams =
            serde_json::from_value(cmd.params.clone())
                .context("Failed to parse Emulation.setGeolocationOverride parameters")?;

        let response = self
            .page
            .execute(params)
            .await
            .context("Emulation.setGeolocationOverride failed")?;

        Ok((serde_json::to_value(&*response)?, None))
    }

    async fn execute_emulation_set_device_metrics(
        &self,
        cmd: &CdpCommand,
    ) -> Result<(Value, Option<String>)> {
        let params: emulation::SetDeviceMetricsOverrideParams =
            serde_json::from_value(cmd.params.clone())
                .context("Failed to parse Emulation.setDeviceMetricsOverride parameters")?;

        let response = self
            .page
            .execute(params)
            .await
            .context("Emulation.setDeviceMetricsOverride failed")?;

        Ok((serde_json::to_value(&*response)?, None))
    }
}
