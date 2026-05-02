//! Public manifestation trait for prompt-context rendering adapters.

use serde_json::Value;

/// Interface for the Manifestation (Qianhuan) layer.
///
/// This trait handles the transformation of system state into
/// human-readable and LLM-ready contexts.
pub trait ManifestationInterface: Send + Sync {
    /// Renders a specific template with the given data.
    ///
    /// # Arguments
    /// * `template_name` - The name of the template (e.g., "`daily_agenda`").
    /// * `data` - The JSON data to fill into the template.
    ///
    /// # Errors
    /// Returns an error if the template cannot be found or if rendering fails.
    fn render_template(&self, template_name: &str, data: Value) -> anyhow::Result<String>;

    /// Generates a system prompt snippet based on the current context.
    ///
    /// This implements the "Instance-Adaptive Prompting" pattern.
    /// # Arguments
    /// * `state_context` - A summary of the current state (e.g., "`HIGH_STRESS`", "`STALE_TASKS`").
    fn inject_context(&self, state_context: &str) -> String;
}
