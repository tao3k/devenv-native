use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::UNIX_EPOCH;

use duckdb::params;
use sha2::{Digest, Sha256};

use super::queries::lookup_content_hash;
use super::types::{DOCUMENT_EXTRACT_SCHEMA_VERSION, DocumentExtractJobRegistry};

impl DocumentExtractJobRegistry {
    pub(super) fn content_hash_for(&self, source_path: &Path) -> Result<String, String> {
        let metadata = source_path.metadata().map_err(|error| {
            format!(
                "read document extract source metadata `{}`: {error}",
                source_path.display()
            )
        })?;
        let size_bytes = i64::try_from(metadata.len()).unwrap_or(i64::MAX);
        let mtime_ns = metadata_modified_ns(&metadata)?;
        let conn = self.connection()?;
        if let Some(hash) = lookup_content_hash(&conn, source_path, size_bytes, mtime_ns)? {
            return Ok(hash);
        }

        let hash = streaming_sha256(source_path)?;
        conn.execute(
            r"
            INSERT INTO document_extract_source_hashes
            (source_path, size_bytes, mtime_ns, content_hash)
            VALUES (?, ?, ?, ?)
            ",
            params![
                source_path.to_string_lossy().to_string(),
                size_bytes,
                mtime_ns,
                hash
            ],
        )
        .map_err(|error| format!("cache document extract content hash: {error}"))?;
        Ok(hash)
    }

    pub(super) fn job_id_for(&self, source_path: &Path, content_hash: &str) -> String {
        let suffix = source_path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .map_or_else(String::new, |suffix| {
                format!(".{}", suffix.to_ascii_lowercase())
            });
        let key = format!(
            "{content_hash}|{suffix}|{DOCUMENT_EXTRACT_SCHEMA_VERSION}|{}",
            self.converter_profile
        );
        hex_sha256(key.as_bytes())
    }
}

fn metadata_modified_ns(metadata: &fs::Metadata) -> Result<i64, String> {
    let duration = metadata
        .modified()
        .map_err(|error| format!("read document extract source mtime: {error}"))?
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("document extract source mtime is before epoch: {error}"))?;
    i64::try_from(duration.as_nanos()).map_err(|_| "source mtime_ns overflowed i64".to_string())
}

fn streaming_sha256(source_path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(source_path).map_err(|error| {
        format!(
            "open document extract source for hashing `{}`: {error}",
            source_path.display()
        )
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| {
            format!(
                "read document extract source for hashing `{}`: {error}",
                source_path.display()
            )
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn hex_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
