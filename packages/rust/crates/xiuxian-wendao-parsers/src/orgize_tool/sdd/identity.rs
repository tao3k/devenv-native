//! Shared helpers for Org-native SDD projections.

use std::collections::HashMap;
use std::path::PathBuf;

use orgize::ast::{SddNodeRecord, SddStatus};

pub(super) fn sdd_status_roots(paths: &[PathBuf]) -> Vec<PathBuf> {
    if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths.to_vec()
    }
}

pub(super) fn sdd_id_index(status: &SddStatus) -> HashMap<String, usize> {
    status
        .records
        .iter()
        .enumerate()
        .filter_map(|(index, record)| {
            non_blank(record.id.as_deref()).map(|id| (id.to_string(), index))
        })
        .collect()
}

pub(super) fn non_blank(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

pub(super) fn sdd_status_label(record: &SddNodeRecord) -> Option<&str> {
    non_blank(record.status.as_deref())
}
