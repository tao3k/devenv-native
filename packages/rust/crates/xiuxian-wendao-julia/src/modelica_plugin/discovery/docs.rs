//! Modelica repository documentation record and semantic marker collection.

use std::collections::BTreeMap;
use std::path::Path;

use xiuxian_wendao_core::repo_intelligence::{DocRecord, ModuleRecord, SymbolRecord};

use super::snapshot::RepositorySnapshot;
use super::sorting::doc_sort_key;
use super::surface::{RepositorySurface, repository_surface};
use crate::modelica_plugin::parsing::contains_documentation_annotation;
use crate::modelica_plugin::pathing::path_components;
use crate::modelica_plugin::relations::{
    annotation_doc_title, doc_targets_for_annotation_doc, doc_targets_for_file_doc,
};
use crate::modelica_plugin::types::CollectedDoc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NestedUsersGuideTopic {
    pub(crate) title: &'static str,
    pub(crate) format: &'static str,
}

const CONVENTIONS_SECTION_TOPICS: [NestedUsersGuideTopic; 3] = [
    NestedUsersGuideTopic {
        title: "Documentation",
        format: "modelica_users_guide_documentation",
    },
    NestedUsersGuideTopic {
        title: "ModelicaCode",
        format: "modelica_users_guide_modelica_code",
    },
    NestedUsersGuideTopic {
        title: "Icons",
        format: "modelica_users_guide_icons",
    },
];

const RELEASE_NOTES_SECTION_TOPICS: [NestedUsersGuideTopic; 1] = [NestedUsersGuideTopic {
    title: "VersionManagement",
    format: "modelica_users_guide_release_notes_version_management",
}];

pub(crate) fn modelica_doc_surface_semantic_markers(
    relative_path: &str,
    contents: &str,
) -> Vec<String> {
    let mut markers = Vec::new();
    if contains_documentation_annotation(contents) {
        markers.push("annotation.documentation".to_string());
    }

    if repository_surface(relative_path) == RepositorySurface::Documentation
        && is_supported_users_guide_doc_path(Path::new(relative_path))
    {
        let file_stem = Path::new(relative_path)
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or_default();
        if file_stem.eq_ignore_ascii_case("Conventions") {
            markers.extend(
                documented_nested_users_guide_topics(contents)
                    .into_iter()
                    .map(|topic| format!("users_guide.section.{}", topic.title)),
            );
        } else if file_stem.eq_ignore_ascii_case("ReleaseNotes") {
            markers.extend(
                documented_release_notes_topics(contents)
                    .into_iter()
                    .map(|topic| format!("users_guide.section.{}", topic.title)),
            );
        }
    }

    markers.sort();
    markers
}

pub(crate) fn doc_format_hint(relative_path: &str, is_annotation: bool) -> Option<String> {
    if repository_surface(relative_path) == RepositorySurface::Documentation {
        return Some(users_guide_doc_format(relative_path, is_annotation));
    }
    if is_annotation {
        return Some("modelica_annotation".to_string());
    }
    Path::new(relative_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .map(str::to_string)
}

fn users_guide_doc_format(relative_path: &str, is_annotation: bool) -> String {
    let components = path_components(relative_path);
    let file_stem = Path::new(relative_path)
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default();
    let base = if components.contains(&"Tutorial") {
        "modelica_users_guide_tutorial"
    } else if file_stem.eq_ignore_ascii_case("Conventions") {
        "modelica_users_guide_conventions"
    } else if file_stem.eq_ignore_ascii_case("Connectors") {
        "modelica_users_guide_connectors"
    } else if file_stem.eq_ignore_ascii_case("Implementation") {
        "modelica_users_guide_implementation"
    } else if file_stem.eq_ignore_ascii_case("RevisionHistory") {
        "modelica_users_guide_revision_history"
    } else if file_stem.eq_ignore_ascii_case("VersionManagement") {
        "modelica_users_guide_version_management"
    } else if components.contains(&"Overview") || file_stem.eq_ignore_ascii_case("Overview") {
        "modelica_users_guide_overview"
    } else if components.contains(&"ReleaseNotes") || file_stem.eq_ignore_ascii_case("ReleaseNotes")
    {
        "modelica_users_guide_release_notes"
    } else if components.contains(&"References") || matches!(file_stem, "References" | "Literature")
    {
        "modelica_users_guide_reference"
    } else if file_stem.eq_ignore_ascii_case("Contact") {
        "modelica_users_guide_contact"
    } else if matches!(file_stem, "Glossar" | "Glossary") {
        "modelica_users_guide_glossary"
    } else if matches!(file_stem, "Parameters" | "Parameterization") {
        "modelica_users_guide_parameter"
    } else if file_stem.eq_ignore_ascii_case("Concept") || file_stem.ends_with("Concept") {
        "modelica_users_guide_concept"
    } else {
        "modelica_users_guide_page"
    };

    if is_annotation {
        format!("{base}_annotation")
    } else {
        base.to_string()
    }
}

pub(crate) fn is_supported_users_guide_doc_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(std::ffi::OsStr::to_str),
        Some("mo" | "md" | "rst" | "qmd")
    )
}

pub(crate) fn doc_title(path: &Path) -> String {
    if path.file_name().and_then(std::ffi::OsStr::to_str) == Some("package.mo") {
        return path
            .parent()
            .and_then(Path::file_name)
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("package")
            .to_string();
    }

    match path.extension().and_then(std::ffi::OsStr::to_str) {
        Some("mo" | "md" | "rst" | "qmd") => path
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("doc")
            .to_string(),
        _ => path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("doc")
            .to_string(),
    }
}

pub(crate) fn collect_doc_records(
    repo_id: &str,
    snapshot: &RepositorySnapshot,
    root_package_name: &str,
    module_lookup: &BTreeMap<String, ModuleRecord>,
    symbols: &[SymbolRecord],
    package_orders: &BTreeMap<String, Vec<String>>,
) -> Vec<CollectedDoc> {
    let root_module_id = module_lookup
        .get(root_package_name)
        .map(|module| module.module_id.clone());
    let mut docs = Vec::new();
    for entry in snapshot.entries() {
        let path = entry.absolute_path.as_path();
        let relative = entry.relative_path.as_str();
        let is_readme = path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .is_some_and(|name| name.to_ascii_lowercase().starts_with("readme"));
        let surface = entry.surface;
        let is_users_guide_doc =
            surface == RepositorySurface::Documentation && is_supported_users_guide_doc_path(path);
        let modelica_contents = entry.modelica_contents.as_deref();
        if is_readme || is_users_guide_doc {
            let title = doc_title(path);
            let format = doc_format_hint(relative, false);
            let target_ids = doc_targets_for_file_doc(
                relative,
                root_package_name,
                module_lookup,
                root_module_id.as_deref(),
            );
            docs.push(CollectedDoc {
                record: DocRecord {
                    repo_id: repo_id.to_string(),
                    doc_id: format!("repo:{repo_id}:doc:{relative}"),
                    title,
                    path: relative.to_string(),
                    format,
                    doc_target: None,
                },
                target_ids: target_ids.clone(),
            });
            docs.extend(collect_nested_users_guide_section_docs(
                repo_id,
                relative,
                modelica_contents,
                &target_ids,
            ));
        }

        let Some(contents) = modelica_contents else {
            continue;
        };
        if !contains_documentation_annotation(contents) {
            continue;
        }
        let target_ids = doc_targets_for_annotation_doc(
            relative,
            root_package_name,
            module_lookup,
            symbols,
            root_module_id.as_deref(),
        );
        if target_ids.is_empty() {
            continue;
        }
        docs.push(CollectedDoc {
            record: DocRecord {
                repo_id: repo_id.to_string(),
                doc_id: format!("repo:{repo_id}:doc:{relative}#annotation.documentation"),
                title: annotation_doc_title(relative, symbols),
                path: format!("{relative}#annotation.documentation"),
                format: doc_format_hint(relative, true),
                doc_target: None,
            },
            target_ids,
        });
    }
    docs.sort_by(|left, right| {
        doc_sort_key(left.record.path.as_str(), package_orders)
            .cmp(&doc_sort_key(right.record.path.as_str(), package_orders))
            .then_with(|| left.record.path.cmp(&right.record.path))
    });
    docs
}

fn collect_nested_users_guide_section_docs(
    repo_id: &str,
    relative_path: &str,
    contents: Option<&str>,
    target_ids: &[String],
) -> Vec<CollectedDoc> {
    if target_ids.is_empty() {
        return Vec::new();
    }
    let Some(contents) = contents else {
        return Vec::new();
    };
    let file_stem = Path::new(relative_path)
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default();
    let topics = if file_stem.eq_ignore_ascii_case("Conventions") {
        documented_nested_users_guide_topics(contents)
    } else if file_stem.eq_ignore_ascii_case("ReleaseNotes") {
        documented_release_notes_topics(contents)
    } else {
        Vec::new()
    };

    topics
        .into_iter()
        .map(|topic| CollectedDoc {
            record: DocRecord {
                repo_id: repo_id.to_string(),
                doc_id: format!("repo:{repo_id}:doc:{relative_path}#section.{}", topic.title),
                title: synthetic_section_title(topic.title),
                path: format!("{relative_path}#section.{}", topic.title),
                format: Some(topic.format.to_string()),
                doc_target: None,
            },
            target_ids: target_ids.to_vec(),
        })
        .collect()
}

pub(crate) fn synthetic_section_title(raw_title: &str) -> String {
    if let Some(version) = raw_title.strip_prefix("Version_") {
        return format!("Version {}", version.replace('_', "."));
    }

    let mut title = String::with_capacity(raw_title.len() + 4);
    let mut previous_is_lowercase = false;
    for ch in raw_title.chars() {
        if previous_is_lowercase && ch.is_ascii_uppercase() {
            title.push(' ');
        }
        previous_is_lowercase = ch.is_ascii_lowercase();
        title.push(ch);
    }
    title
}

pub(crate) fn documented_nested_users_guide_topics(contents: &str) -> Vec<NestedUsersGuideTopic> {
    CONVENTIONS_SECTION_TOPICS
        .into_iter()
        .filter(|topic| contains_documented_nested_topic(contents, topic.title))
        .collect()
}

pub(crate) fn documented_release_notes_topics(contents: &str) -> Vec<NestedUsersGuideTopic> {
    let mut topics = RELEASE_NOTES_SECTION_TOPICS
        .into_iter()
        .filter(|topic| contains_documented_nested_topic(contents, topic.title))
        .collect::<Vec<_>>();
    topics.extend(documented_release_notes_versions(contents));
    topics
}

fn documented_release_notes_versions(contents: &str) -> Vec<NestedUsersGuideTopic> {
    release_notes_version_names(contents)
        .into_iter()
        .filter(|version_name| contains_documented_nested_topic(contents, version_name.as_str()))
        .map(|version_name| NestedUsersGuideTopic {
            title: Box::leak(version_name.into_boxed_str()),
            format: "modelica_users_guide_release_notes_version",
        })
        .collect()
}

fn release_notes_version_names(contents: &str) -> Vec<String> {
    contents
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("class Version_") {
                return None;
            }
            trimmed
                .split_whitespace()
                .nth(1)
                .map(str::trim)
                .filter(|name| name.starts_with("Version_"))
                .map(str::to_string)
        })
        .collect()
}

fn contains_documented_nested_topic(contents: &str, topic_name: &str) -> bool {
    let Some((start, kind)) = topic_declaration_start(contents, topic_name) else {
        return false;
    };
    let end_marker = format!("end {topic_name};");
    let Some(relative_end) = contents[start..].find(end_marker.as_str()) else {
        return false;
    };
    let block = &contents[start..start + relative_end + end_marker.len()];
    block.contains("annotation (Documentation(")
        || block.contains("annotation(Documentation(")
        || (kind == "record" && block.contains("Documentation(info"))
}

fn topic_declaration_start<'a>(contents: &'a str, topic_name: &'a str) -> Option<(usize, &'a str)> {
    ["package", "class", "model", "record"]
        .into_iter()
        .find_map(|kind| {
            let marker = format!("{kind} {topic_name}");
            contents.find(marker.as_str()).map(|offset| (offset, kind))
        })
}
