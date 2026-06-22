use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::json;
use tempfile::TempDir;

use super::{
    RepositorySnapshot, RepositorySurface, doc_format_hint, doc_sort_key, doc_title,
    documented_nested_users_guide_topics, documented_release_notes_topics, example_sort_key,
    is_supported_users_guide_doc_path, module_sort_key, repository_surface,
    synthetic_section_title,
};
use crate::julia_plugin_test_support::common::assert_sorted_json_snapshot;

fn surface_name(surface: RepositorySurface) -> &'static str {
    match surface {
        RepositorySurface::Api => "api",
        RepositorySurface::Example => "example",
        RepositorySurface::Documentation => "documentation",
        RepositorySurface::Support => "support",
    }
}

include!("discovery/repository_surfaces.rs");
include!("discovery/users_guide_docs.rs");
include!("discovery/doc_titles.rs");
