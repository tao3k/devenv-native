//! `zhenfa_router::native::semantic_check::docs_governance::rendering::landing` owns Wendao docs governance rendering landing behavior.

use std::fmt::Write as _;
use std::path::Path;

use crate::parsers::docs_governance::derive_opaque_doc_id;
use crate::zhenfa_router::native::semantic_check::docs_governance::rendering::shared::{
    render_section_prompt, render_section_summary,
};
use crate::zhenfa_router::native::semantic_check::docs_governance::types::SectionSpec;

/// Renders a section landing page template.
#[must_use]
pub fn render_section_landing_page(
    crate_name: &str,
    crate_dir: &Path,
    doc_path: &str,
    spec: &SectionSpec,
) -> String {
    let mut rendered = String::new();
    let _ = writeln!(rendered, "# {}\n", spec.title);
    rendered.push_str(":PROPERTIES:\n");
    let _ = writeln!(rendered, ":ID: {}", derive_opaque_doc_id(doc_path));
    let _ = writeln!(rendered, ":TYPE: {}", spec.doc_type);
    rendered.push_str(":STATUS: DRAFT\n");
    rendered.push_str(":END:\n\n");
    let _ = writeln!(
        rendered,
        "{}\n",
        render_section_summary(crate_name, crate_dir, spec)
    );
    let _ = writeln!(rendered, "{}", render_section_prompt(crate_name, spec));
    rendered
}
