use crate::markdown::{MarkdownShowSection, render_show_surface};
use crate::template_catalog::EmbeddedTemplateCatalog;
use serde_json::json;

use crate::flowhub::{
    FlowhubModuleKind, FlowhubModuleShow, FlowhubModuleSummary, FlowhubRootShow, FlowhubShow,
};

pub(crate) const FLOWHUB_SCENARIO_CASE_TEMPLATE_NAME: &str = "flowhub_scenario_case.md.j2";
const FLOWHUB_SCENARIO_CASE_TEMPLATE_SOURCE: &str =
    include_str!("../../../resources/templates/control_plane/flowhub_scenario_case.md.j2");
pub(crate) const FLOWHUB_ROOT_MODULE_TEMPLATE_NAME: &str = "flowhub_root_module.md.j2";
const FLOWHUB_ROOT_MODULE_TEMPLATE_SOURCE: &str =
    include_str!("../../../resources/templates/control_plane/flowhub_root_module.md.j2");
const FLOWHUB_MODULE_EXPORTS_TEMPLATE_NAME: &str = "flowhub_module_exports.md.j2";
const FLOWHUB_MODULE_EXPORTS_TEMPLATE_SOURCE: &str =
    include_str!("../../../resources/templates/control_plane/flowhub_module_exports.md.j2");
const FLOWHUB_MODULE_CONTRACT_TEMPLATE_NAME: &str = "flowhub_module_contract.md.j2";
const FLOWHUB_MODULE_CONTRACT_TEMPLATE_SOURCE: &str =
    include_str!("../../../resources/templates/control_plane/flowhub_module_contract.md.j2");

static FLOWHUB_TEMPLATE_CATALOG: EmbeddedTemplateCatalog = EmbeddedTemplateCatalog::new(
    "Flowhub show template renderer",
    &[
        (
            FLOWHUB_SCENARIO_CASE_TEMPLATE_NAME,
            FLOWHUB_SCENARIO_CASE_TEMPLATE_SOURCE,
        ),
        (
            FLOWHUB_ROOT_MODULE_TEMPLATE_NAME,
            FLOWHUB_ROOT_MODULE_TEMPLATE_SOURCE,
        ),
        (
            FLOWHUB_MODULE_EXPORTS_TEMPLATE_NAME,
            FLOWHUB_MODULE_EXPORTS_TEMPLATE_SOURCE,
        ),
        (
            FLOWHUB_MODULE_CONTRACT_TEMPLATE_NAME,
            FLOWHUB_MODULE_CONTRACT_TEMPLATE_SOURCE,
        ),
    ],
);

pub(crate) fn render_flowhub_show_impl(show: &FlowhubShow) -> String {
    match show {
        FlowhubShow::Root(show) => render_flowhub_root_show(show),
        FlowhubShow::Module(show) => render_flowhub_module_show(show),
    }
}

fn render_flowhub_root_show(show: &FlowhubRootShow) -> String {
    let sections = show
        .modules
        .iter()
        .map(|module| {
            let lines = super::render_flowhub_root_module_section_lines(module);
            MarkdownShowSection {
                title: module.module_ref.as_str().into(),
                lines,
            }
        })
        .collect::<Vec<_>>();

    render_show_surface(
        "Flowhub",
        &[
            format!("Location: {}", show.root.display()),
            format!("Modules: {}", show.modules.len()),
        ],
        &sections,
    )
}

fn render_flowhub_module_show(show: &FlowhubModuleShow) -> String {
    let summary = &show.summary;
    let mut sections = vec![
        MarkdownShowSection {
            title: "Exports".into(),
            lines: render_flowhub_module_exports_section_lines(summary),
        },
        MarkdownShowSection {
            title: "Contract".into(),
            lines: render_flowhub_module_contract_section_lines(show),
        },
    ];

    if !summary.child_modules.is_empty() {
        sections.push(MarkdownShowSection {
            title: "Children".into(),
            lines: summary
                .child_modules
                .iter()
                .map(|child| format!("- {child}"))
                .collect(),
        });
    }
    if !show.scenario_cases.is_empty() {
        sections.push(MarkdownShowSection {
            title: "Scenario Cases".into(),
            lines: super::render_scenario_case_section_lines(
                &summary.module_ref,
                &show.scenario_cases,
            ),
        });
    }

    render_show_surface(
        "Flowhub Module",
        &[
            format!("Module: {}", summary.module_ref),
            format!("Name: {}", summary.module_name),
            format!("Location: {}", summary.module_dir.display()),
            format!("Kind: {}", module_kind_label(summary.kind)),
        ],
        &sections,
    )
}

fn render_flowhub_module_exports_section_lines(summary: &FlowhubModuleSummary) -> Vec<String> {
    render_embedded_flowhub_block(
        FLOWHUB_MODULE_EXPORTS_TEMPLATE_NAME,
        json!({
            "entry_export": summary.exports_entry,
            "ready_export": summary.exports_ready,
        }),
    )
    .unwrap_or_else(|error| {
        log::warn!(
            "failed to render Flowhub exports section through Qianji template catalog; falling back to inline format: {error}"
        );
        vec![
            format!("Entry export: {}", summary.exports_entry),
            format!("Ready export: {}", summary.exports_ready),
        ]
    })
}

pub(crate) fn render_flowhub_module_contract_section_lines(
    show: &FlowhubModuleShow,
) -> Vec<String> {
    render_embedded_flowhub_block(
        FLOWHUB_MODULE_CONTRACT_TEMPLATE_NAME,
        json!({
            "registered_children": show.registered_child_count,
            "required_contract_entries": show.required_contract_count,
        }),
    )
    .unwrap_or_else(|error| {
        log::warn!(
            "failed to render Flowhub contract section through Qianji template catalog; falling back to inline format: {error}"
        );
        vec![
            format!("Registered children: {}", show.registered_child_count),
            format!("Required contract entries: {}", show.required_contract_count),
        ]
    })
}

pub(crate) fn render_embedded_flowhub_block(
    template_name: &str,
    payload: serde_json::Value,
) -> Result<Vec<String>, String> {
    FLOWHUB_TEMPLATE_CATALOG.render_lines(template_name, payload)
}

pub(crate) fn module_kind_label(kind: FlowhubModuleKind) -> &'static str {
    match kind {
        FlowhubModuleKind::Composite => "composite",
        FlowhubModuleKind::Leaf => "node",
    }
}
