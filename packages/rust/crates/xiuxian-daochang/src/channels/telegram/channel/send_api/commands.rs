//! Telegram Bot API command synchronization.

use crate::channels::telegram::channel::TelegramChannel;

impl TelegramChannel {
    /// Synchronizes the built-in bot commands to the Telegram Bot Menu.
    ///
    /// This ensures that users see "/agenda" and "/journal" in the command list.
    ///
    /// # Errors
    ///
    /// Returns an error when the underlying Telegram HTTP request fails unexpectedly.
    pub async fn sync_bot_commands(&self) -> anyhow::Result<()> {
        let commands = serde_json::json!({
            "commands": [
                { "command": "agenda", "description": "View your current cultivation agenda" },
                { "command": "journal", "description": "Record a daily journal entry or reflection" },
                { "command": "help", "description": "Show help and available commands" }
            ]
        });

        tracing::info!("Synchronizing Telegram bot commands...");

        match self.send_api_request_once("setMyCommands", &commands).await {
            Ok(()) => {
                tracing::info!("Telegram bot commands synchronized successfully.");
                Ok(())
            }
            Err(e) => {
                tracing::warn!("Failed to synchronize Telegram bot commands: {e}");
                Err(e.into())
            }
        }
    }
}
