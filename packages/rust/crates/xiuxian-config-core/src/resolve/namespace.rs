//! Namespace extraction helpers for global config documents.

pub(super) fn extract_namespace_value(root: &toml::Value, namespace: &str) -> Option<toml::Value> {
    if namespace.trim().is_empty() {
        return Some(root.clone());
    }

    let mut cursor = root;
    for segment in namespace.split('.') {
        let key = segment.trim();
        if key.is_empty() {
            return None;
        }
        let table = cursor.as_table()?;
        cursor = table.get(key)?;
    }
    Some(cursor.clone())
}
