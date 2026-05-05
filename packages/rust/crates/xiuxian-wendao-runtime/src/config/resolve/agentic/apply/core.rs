//! Agentic settings application boundary for typed runtime config updates.

use super::{execution, expansion, search, suggested};
use crate::config::LinkGraphAgenticRuntimeConfig;
use serde_yaml::Value;

pub(in crate::config::resolve::agentic) fn apply_suggested_link_settings(
    settings: &Value,
    resolved: &mut LinkGraphAgenticRuntimeConfig,
) {
    suggested::apply_suggested_link_settings(settings, resolved);
}

pub(in crate::config::resolve::agentic) fn apply_search_settings(
    settings: &Value,
    resolved: &mut LinkGraphAgenticRuntimeConfig,
) {
    search::apply_search_settings(settings, resolved);
}

pub(in crate::config::resolve::agentic) fn apply_expansion_settings(
    settings: &Value,
    resolved: &mut LinkGraphAgenticRuntimeConfig,
) {
    expansion::apply_expansion_settings(settings, resolved);
}

pub(in crate::config::resolve::agentic) fn apply_execution_settings(
    settings: &Value,
    resolved: &mut LinkGraphAgenticRuntimeConfig,
) {
    execution::apply_execution_settings(settings, resolved);
}
