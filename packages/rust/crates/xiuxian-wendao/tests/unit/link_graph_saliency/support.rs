use redis::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) const TEST_VALKEY_URL: &str = "redis://127.0.0.1:6379/0";

pub(super) fn unique_prefix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or(0);
    format!("omni:test:saliency:{nanos}")
}

pub(super) fn valkey_connection() -> Result<Connection, String> {
    let client = redis::Client::open(TEST_VALKEY_URL).map_err(|err| err.to_string())?;
    client.get_connection().map_err(|err| err.to_string())
}

pub(super) fn clear_prefix(prefix: &str) -> Result<(), String> {
    let mut conn = valkey_connection()?;
    let pattern = format!("{prefix}:*");
    let keys: Vec<String> = redis::cmd("KEYS")
        .arg(&pattern)
        .query(&mut conn)
        .map_err(|err| err.to_string())?;
    if !keys.is_empty() {
        redis::cmd("DEL")
            .arg(keys)
            .query::<()>(&mut conn)
            .map_err(|err| err.to_string())?;
    }
    Ok(())
}

fn snapshot_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(relative)
}

fn read_snapshot(relative: &str) -> String {
    let path = snapshot_path(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read snapshot {}: {error}", path.display()))
}

pub(super) fn assert_snapshot_eq(relative: &str, actual: &str) {
    let expected = read_snapshot(relative);
    assert_eq!(
        expected, actual,
        "snapshot mismatch: {relative}\n--- expected ---\n{expected}\n--- actual ---\n{actual}"
    );
}
