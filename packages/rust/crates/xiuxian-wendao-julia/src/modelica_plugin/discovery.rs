//! Modelica repository discovery facade.

mod docs;
mod overlay;
mod records;
mod snapshot;
mod sorting;
mod surface;

pub(crate) use docs::{collect_doc_records, modelica_doc_surface_semantic_markers};
pub(crate) use overlay::safe_package_overlay_metadata_for_relative_path;
pub(crate) use records::{
    collect_example_records, collect_import_records, collect_module_records, collect_symbol_records,
};
pub(crate) use snapshot::RepositorySnapshot;
pub(crate) use surface::is_api_surface_path;

#[cfg(test)]
pub(crate) use docs::{
    doc_format_hint, doc_title, documented_nested_users_guide_topics,
    documented_release_notes_topics, is_supported_users_guide_doc_path, synthetic_section_title,
};
#[cfg(test)]
pub(crate) use sorting::{doc_sort_key, example_sort_key, module_sort_key};
#[cfg(test)]
pub(crate) use surface::{RepositorySurface, repository_surface};

#[cfg(test)]
#[path = "../../tests/unit/plugin/discovery.rs"]
mod tests;
