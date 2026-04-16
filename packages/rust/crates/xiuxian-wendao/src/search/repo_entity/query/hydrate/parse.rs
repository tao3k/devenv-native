use std::collections::BTreeMap;

use crate::analyzers::RepoBacklinkItem;
use crate::analyzers::{ImportKind, RepoSymbolKind};
use crate::search::repo_entity::query::lookup::RepoEntitySearchError;

pub(crate) fn non_empty_vec(values: Vec<String>) -> Option<Vec<String>> {
    (!values.is_empty()).then_some(values)
}

pub(crate) fn parse_backlink_items(
    value: Option<&str>,
) -> Result<Option<Vec<RepoBacklinkItem>>, RepoEntitySearchError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let items = serde_json::from_str::<Vec<RepoBacklinkItem>>(value)
        .map_err(|error| RepoEntitySearchError::Decode(error.to_string()))?;
    Ok((!items.is_empty()).then_some(items))
}

pub(crate) fn parse_attributes_map(
    value: Option<&str>,
) -> Result<BTreeMap<String, String>, RepoEntitySearchError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(BTreeMap::new());
    };
    serde_json::from_str::<BTreeMap<String, String>>(value)
        .map_err(|error| RepoEntitySearchError::Decode(error.to_string()))
}

pub(crate) fn parse_symbol_kind(kind: &str) -> RepoSymbolKind {
    match kind {
        "function" => RepoSymbolKind::Function,
        "type" => RepoSymbolKind::Type,
        "constant" => RepoSymbolKind::Constant,
        "module_export" => RepoSymbolKind::ModuleExport,
        _ => RepoSymbolKind::Other,
    }
}

pub(crate) fn parse_import_kind(kind: &str) -> ImportKind {
    match kind {
        "module" => ImportKind::Module,
        "reexport" => ImportKind::Reexport,
        _ => ImportKind::Symbol,
    }
}
