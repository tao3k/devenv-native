//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
//! `zhenfa_router::native::semantic_check::docs_governance::rendering::shared` owns Wendao docs governance rendering shared behavior.

use std::fs;
use std::path::Path;

use crate::zhenfa_router::native::semantic_check::docs_governance::types::SectionSpec;

pub(crate) fn collect_section_links(docs_dir: &Path) -> Vec<(String, Vec<String>)> {
    let Ok(entries) = fs::read_dir(docs_dir) else {
        return Vec::new();
    };

    let mut section_links = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(section_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        let Ok(section_entries) = fs::read_dir(&path) else {
            continue;
        };

        let mut links = section_entries
            .flatten()
            .filter_map(|child| {
                let child_path = child.path();
                if !child_path.is_file() {
                    return None;
                }
                if child_path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                    return None;
                }
                let stem = child_path.file_stem()?.to_str()?;
                Some(format!("{section_name}/{stem}"))
            })
            .collect::<Vec<_>>();
        links.sort();

        if !links.is_empty() {
            section_links.push((section_name.to_string(), links));
        }
    }

    section_links.sort_by(|left, right| left.0.cmp(&right.0));
    section_links
}

/// Return the canonical package-doc section layout expected for one crate.
#[must_use]
pub fn standard_section_specs(crate_name: &str) -> Vec<SectionSpec> {
    let slug = crate_slug(crate_name);
    vec![
        SectionSpec {
            section_name: "01_core",
            relative_path: format!("01_core/101_{slug}_core_boundary.md"),
            title: "Core Boundary".to_string(),
            doc_type: "CORE",
        },
        SectionSpec {
            section_name: "03_features",
            relative_path: format!("03_features/201_{slug}_feature_ledger.md"),
            title: "Feature Ledger".to_string(),
            doc_type: "FEATURE",
        },
        SectionSpec {
            section_name: "05_research",
            relative_path: format!("05_research/301_{slug}_research_agenda.md"),
            title: "Research Agenda".to_string(),
            doc_type: "RESEARCH",
        },
        SectionSpec {
            section_name: "06_roadmap",
            relative_path: format!("06_roadmap/401_{slug}_roadmap.md"),
            title: "Roadmap".to_string(),
            doc_type: "ROADMAP",
        },
    ]
}

pub(crate) fn crate_slug(crate_name: &str) -> String {
    crate_name
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

pub(crate) fn render_section_summary(
    crate_name: &str,
    crate_dir: &Path,
    spec: &SectionSpec,
) -> String {
    let crate_kind = if crate_dir.join("src/lib.rs").is_file() {
        "library crate"
    } else if crate_dir.join("src/main.rs").is_file() {
        "binary crate"
    } else {
        "Rust crate"
    };

    match spec.section_name {
        "01_core" => format!(
            "Architecture boundary note for the `{crate_name}` {crate_kind}. Capture core responsibilities, integration edges, and invariants here."
        ),
        "03_features" => format!(
            "Feature ledger for the `{crate_name}` {crate_kind}. Track user-facing or system-facing capabilities implemented in this package."
        ),
        "05_research" => format!(
            "Research agenda for the `{crate_name}` {crate_kind}. Record external references, experiments, and design questions that still need hardening."
        ),
        "06_roadmap" => format!(
            "Roadmap tracker for the `{crate_name}` {crate_kind}. Use this page to pin the next implementation milestones and validation gates."
        ),
        _ => format!("Documentation placeholder for `{crate_name}`."),
    }
}

pub(crate) fn render_section_prompt(crate_name: &str, spec: &SectionSpec) -> String {
    match spec.section_name {
        "01_core" => format!(
            "Document the stable architectural boundary for `{crate_name}` before expanding deeper feature notes."
        ),
        "03_features" => format!(
            "Promote concrete `{crate_name}` capabilities into this ledger as feature slices land."
        ),
        "05_research" => format!(
            "Capture unresolved research questions and external references that inform `{crate_name}`."
        ),
        "06_roadmap" => format!(
            "List the next verified milestones for `{crate_name}` and keep them synchronized with GTD and ExecPlans."
        ),
        _ => "Extend this placeholder with package-specific detail.".to_string(),
    }
}

pub(crate) fn matches_section_heading(trimmed: &str, section_name: &str) -> bool {
    let heading = format!("## {section_name}");
    trimmed == heading || trimmed.starts_with(&format!("{heading}:"))
}
