//! Studio-owned navigation contracts.

use serde::{Deserialize, Serialize};
use specta::Type;

use super::{StudioContractCategory, StudioContractPath};

/// Navigation target for opening files/symbols in the Studio editor.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StudioNavigationTarget {
    /// Full path or URI.
    pub path: StudioContractPath,
    /// Navigation category, such as `doc` or `symbol`.
    pub category: StudioContractCategory,
    /// Optional project label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    /// Optional root label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_label: Option<String>,
    /// 1-based line number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// 1-based end line number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_end: Option<usize>,
    /// 1-based column number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
}

#[cfg(feature = "local-runtime")]
impl From<xiuxian_wendao::search::contracts::StudioNavigationTarget> for StudioNavigationTarget {
    fn from(value: xiuxian_wendao::search::contracts::StudioNavigationTarget) -> Self {
        Self {
            path: value.path.into(),
            category: value.category.into(),
            project_name: value.project_name,
            root_label: value.root_label,
            line: value.line,
            line_end: value.line_end,
            column: value.column,
        }
    }
}

#[cfg(feature = "local-runtime")]
impl From<StudioNavigationTarget> for xiuxian_wendao::search::contracts::StudioNavigationTarget {
    fn from(value: StudioNavigationTarget) -> Self {
        Self {
            path: value.path.into_string(),
            category: value.category.into_string(),
            project_name: value.project_name,
            root_label: value.root_label,
            line: value.line,
            line_end: value.line_end,
            column: value.column,
        }
    }
}
