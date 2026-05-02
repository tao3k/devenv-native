//! Markdown file storage backend for journals and agenda tasks.

use crate::agenda::AgendaEntry;
use crate::journal::JournalEntry;
use crate::{Error, Result};
use chrono::Local;
use std::path::PathBuf;
use tokio::fs;

/// File-based storage for the Xiuxian-Zhixing system.
pub struct MarkdownStorage {
    /// Root directory where journals and agendas are stored.
    pub root_dir: PathBuf,
}

impl MarkdownStorage {
    /// Creates a new `MarkdownStorage` instance.
    #[must_use]
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    /// Records a journal entry into a date-based file (e.g., journal/2025-02-25.md).
    ///
    /// # Errors
    /// Returns an error if directory creation or file writing fails.
    pub async fn record_journal(&self, journal: &JournalEntry) -> Result<()> {
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        let dir_path = self.root_dir.join("journal");
        let file_path = dir_path.join(format!("{date_str}.md"));

        fs::create_dir_all(&dir_path)
            .await
            .map_err(|e| Error::Logic(format!("Failed to create journal directory: {e}")))?;

        let content = format!(
            "\n## [{}] Reflection\n{}\n<!-- id: {}, tags: {:?} -->\n",
            Local::now().format("%H:%M:%S"),
            journal.content,
            journal.id,
            journal.tags
        );

        let mut options = fs::OpenOptions::new();
        options.create(true).append(true);
        let mut _file = options
            .open(&file_path)
            .await
            .map_err(|e| Error::Logic(format!("Failed to open journal file: {e}")))?;

        fs::write(&file_path, content)
            .await
            .map_err(|e| Error::Logic(format!("Failed to write journal: {e}")))?;

        Ok(())
    }

    /// Appends a new task to the current day's agenda file.
    ///
    /// # Errors
    /// Returns an error if directory creation or file writing fails.
    pub async fn record_task(&self, task: &AgendaEntry) -> Result<()> {
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        let dir_path = self.root_dir.join("agenda");
        let file_path = dir_path.join(format!("{date_str}.md"));

        fs::create_dir_all(&dir_path)
            .await
            .map_err(|e| Error::Logic(format!("Failed to create agenda directory: {e}")))?;

        let content = format!(
            "- [ ] {} <!-- id: {}, priority: {:?}, journal:carryover: 0 -->\n",
            task.title, task.id, task.priority
        );

        let mut options = fs::OpenOptions::new();
        options.create(true).append(true);

        fs::write(&file_path, content)
            .await
            .map_err(|e| Error::Logic(format!("Failed to write task: {e}")))?;

        Ok(())
    }
}
