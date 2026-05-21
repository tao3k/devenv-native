//! Rendering helpers for package documentation index pages.

use std::fmt::Write as _;
use std::path::Path;

use crate::parsers::docs_governance::derive_opaque_doc_id;
use crate::zhenfa_router::native::semantic_check::docs_governance::rendering::footer::render_index_footer_with_values;
use crate::zhenfa_router::native::semantic_check::docs_governance::rendering::shared::collect_section_links;

/// Renders a package docs index template.
#[must_use]
pub fn render_package_docs_index(crate_name: &str, doc_path: &str, docs_dir: &Path) -> String {
    let section_links = collect_section_links(docs_dir);
    let mut rendered = String::new();
    render_index_header(&mut rendered, crate_name, doc_path);

    if section_links.is_empty() {
        render_empty_index_body(&mut rendered);
    } else {
        render_section_link_body(&mut rendered, &section_links);
        render_section_relations(&mut rendered, &section_links);
    }

    render_index_footer(&mut rendered);
    rendered
}

fn render_index_header(rendered: &mut String, crate_name: &str, doc_path: &str) {
    let _ = writeln!(rendered, "# {crate_name}: Map of Content");
    rendered.push('\n');
    rendered.push_str(":PROPERTIES:\n");
    let _ = writeln!(rendered, ":ID: {}", derive_opaque_doc_id(doc_path));
    rendered.push_str(":TYPE: INDEX\n");
    rendered.push_str(":STATUS: ACTIVE\n");
    rendered.push_str(":END:\n\n");
    let _ = writeln!(
        rendered,
        "Standardized documentation index for the `{crate_name}` package.\n"
    );
}

fn render_empty_index_body(rendered: &mut String) {
    rendered.push_str(
        "Populate package-local documentation sections under this directory and extend this index as the package surface evolves.\n",
    );
}

fn render_section_link_body(rendered: &mut String, section_links: &[(String, Vec<String>)]) {
    for (section, links) in section_links {
        let _ = writeln!(rendered, "## {section}\n");
        for link in links {
            let _ = writeln!(rendered, "- [[{link}]]");
        }
        rendered.push('\n');
    }
}

fn render_section_relations(rendered: &mut String, section_links: &[(String, Vec<String>)]) {
    rendered.push_str(":RELATIONS:\n");
    rendered.push_str(":LINKS: ");
    rendered.push_str(
        &section_links
            .iter()
            .flat_map(|(_, links)| links.iter())
            .map(|link| format!("[[{link}]]"))
            .collect::<Vec<_>>()
            .join(", "),
    );
    rendered.push_str("\n:END:\n");
}

fn render_index_footer(rendered: &mut String) {
    rendered.push_str("\n---\n\n");
    rendered.push_str(&render_index_footer_with_values("v2.0", "pending"));
}
