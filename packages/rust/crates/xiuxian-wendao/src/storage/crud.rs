//! Valkey-backed CRUD operations for `KnowledgeStorage`.

use chrono::Utc;
use serde_yaml::Value;

use crate::settings::{get_setting_string, merged_wendao_settings};
use crate::types::KnowledgeEntry;
use crate::valkey_common::open_client;
use xiuxian_config_core::toml_first_named_string;

use super::KnowledgeStorage;

const KNOWLEDGE_VALKEY_URL_SETTING: &str = "storage.knowledge.valkey_url";
const KNOWLEDGE_VALKEY_URL_ENV: &str = "XIUXIAN_WENDAO_KNOWLEDGE_VALKEY_URL";
const DEFAULT_KNOWLEDGE_VALKEY_URL: &str = "redis://127.0.0.1/";

/// Error returned by `KnowledgeStorage` CRUD operations.
#[derive(Debug, thiserror::Error)]
pub enum KnowledgeStorageError {
    /// Valkey command, connection, or client creation failed.
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
    /// Stored knowledge entry JSON could not be serialized or deserialized.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub(super) type StorageResult<T> = Result<T, KnowledgeStorageError>;

impl KnowledgeStorage {
    /// Initialize the storage (validate Valkey connectivity).
    ///
    /// # Errors
    ///
    /// Returns an error when the Valkey client cannot be created, the
    /// connection cannot be established, or the connectivity/key checks fail.
    pub fn init(&self) -> StorageResult<()> {
        let client = Self::redis_client()?;
        let mut conn = client.get_connection()?;
        let _pong: String = redis::cmd("PING").query(&mut conn)?;
        let _exists: i64 = redis::cmd("EXISTS")
            .arg(self.entries_key())
            .query(&mut conn)?;
        Ok(())
    }

    pub(super) fn entries_key(&self) -> String {
        format!("{}:entries", self.table_name())
    }

    fn resolve_knowledge_valkey_url() -> String {
        let settings = merged_wendao_settings();
        resolve_knowledge_valkey_url_with_settings_and_lookup(&settings, &|name| {
            std::env::var(name).ok()
        })
    }

    pub(super) fn redis_client() -> StorageResult<redis::Client> {
        let url = Self::resolve_knowledge_valkey_url();
        Ok(open_client(url.as_str())?)
    }

    /// Upsert a knowledge entry.
    ///
    /// # Errors
    ///
    /// Returns an error when storage initialization fails, the Valkey
    /// connection fails, or JSON serialization/deserialization fails.
    pub fn upsert(&self, entry: &KnowledgeEntry) -> StorageResult<()> {
        self.init()?;
        let mut conn = Self::storage_connection()?;
        let existing = self.load_existing_entry(&mut conn, &entry.id)?;
        let to_store = version_entry_for_upsert(entry, existing);
        self.write_entry_payload(&mut conn, &to_store)?;
        Ok(())
    }

    fn storage_connection() -> StorageResult<redis::Connection> {
        let client = Self::redis_client()?;
        Ok(client.get_connection()?)
    }

    fn load_existing_entry(
        &self,
        conn: &mut redis::Connection,
        entry_id: &str,
    ) -> StorageResult<Option<KnowledgeEntry>> {
        let existing_raw: Option<String> = redis::cmd("HGET")
            .arg(self.entries_key())
            .arg(entry_id)
            .query(conn)?;
        Ok(existing_raw
            .as_deref()
            .map(serde_json::from_str::<KnowledgeEntry>)
            .transpose()?)
    }

    fn write_entry_payload(
        &self,
        conn: &mut redis::Connection,
        entry: &KnowledgeEntry,
    ) -> StorageResult<()> {
        let payload = serde_json::to_string(entry)?;
        let _: i64 = redis::cmd("HSET")
            .arg(self.entries_key())
            .arg(&entry.id)
            .arg(payload)
            .query(conn)?;
        Ok(())
    }

    /// Count total entries.
    ///
    /// # Errors
    ///
    /// Returns an error when the Valkey client or connection cannot be
    /// created, or when the `HLEN` command fails.
    pub fn count(&self) -> StorageResult<i64> {
        let client = Self::redis_client()?;
        let mut conn = client.get_connection()?;
        let total: i64 = redis::cmd("HLEN")
            .arg(self.entries_key())
            .query(&mut conn)?;
        Ok(total)
    }

    /// Delete an entry by ID.
    ///
    /// # Errors
    ///
    /// Returns an error when the Valkey client or connection cannot be
    /// created, or when the `HDEL` command fails.
    pub fn delete(&self, entry_id: impl AsRef<str>) -> StorageResult<()> {
        let client = Self::redis_client()?;
        let mut conn = client.get_connection()?;
        let _: i64 = redis::cmd("HDEL")
            .arg(self.entries_key())
            .arg(entry_id.as_ref())
            .query(&mut conn)?;
        Ok(())
    }

    /// Retrieve one knowledge entry by ID.
    ///
    /// # Errors
    /// Returns an error if Valkey connection or deserialization fails.
    pub fn get_entry(&self, entry_id: impl AsRef<str>) -> StorageResult<Option<KnowledgeEntry>> {
        let client = Self::redis_client()?;
        let mut conn = client.get_connection()?;
        let entries_key = self.entries_key();
        let raw: Option<String> = redis::cmd("HGET")
            .arg(&entries_key)
            .arg(entry_id.as_ref())
            .query(&mut conn)?;

        match raw {
            Some(s) => Ok(Some(serde_json::from_str::<KnowledgeEntry>(&s)?)),
            None => Ok(None),
        }
    }

    /// Load all knowledge entries from the table.
    ///
    /// # Errors
    /// Returns an error if Valkey connection or deserialization fails.
    pub fn load_all_entries(&self) -> StorageResult<Vec<KnowledgeEntry>> {
        let client = Self::redis_client()?;
        let mut conn = client.get_connection()?;
        let entries_key = self.entries_key();
        let raws: std::collections::HashMap<String, String> =
            redis::cmd("HGETALL").arg(&entries_key).query(&mut conn)?;

        let mut out = Vec::new();
        for s in raws.values() {
            out.push(serde_json::from_str::<KnowledgeEntry>(s)?);
        }
        Ok(out)
    }

    /// Clear all entries.
    ///
    /// # Errors
    ///
    /// Returns an error when the Valkey client or connection cannot be
    /// created, or when the `DEL` command fails.
    pub fn clear(&self) -> StorageResult<()> {
        let client = Self::redis_client()?;
        let mut conn = client.get_connection()?;
        let _: i64 = redis::cmd("DEL").arg(self.entries_key()).query(&mut conn)?;
        Ok(())
    }
}

fn version_entry_for_upsert(
    entry: &KnowledgeEntry,
    existing: Option<KnowledgeEntry>,
) -> KnowledgeEntry {
    let now = Utc::now();
    let (created_at, version) = match existing {
        Some(found) => (found.created_at, found.version + 1),
        None => (now, entry.version.max(1)),
    };

    let mut to_store = entry.clone();
    to_store.created_at = created_at;
    to_store.updated_at = now;
    to_store.version = version;
    to_store
}

fn resolve_knowledge_valkey_url_with_fallback(candidate: Option<String>) -> String {
    candidate.unwrap_or_else(|| DEFAULT_KNOWLEDGE_VALKEY_URL.to_string())
}

fn resolve_knowledge_valkey_url_with_settings_and_lookup(
    settings: &Value,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> String {
    resolve_knowledge_valkey_url_with_fallback(
        toml_first_named_string(
            KNOWLEDGE_VALKEY_URL_SETTING,
            get_setting_string(settings, KNOWLEDGE_VALKEY_URL_SETTING),
            lookup,
            &[KNOWLEDGE_VALKEY_URL_ENV, "VALKEY_URL", "REDIS_URL"],
        )
        .map(|candidate| candidate.value),
    )
}

impl KnowledgeStorage {
    #[cfg(test)]
    fn redis_client_from_url(valkey_url: &str) -> StorageResult<redis::Client> {
        Ok(open_client(valkey_url)?)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/storage/crud.rs"]
mod tests;
