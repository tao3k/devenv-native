use std::borrow::Cow;

use crate::template_catalog::EmbeddedTemplateCatalog;
use serde_json::json;

const MARKDOWN_SHOW_SURFACE_TEMPLATE_NAME: &str = "qianji_show_surface.md.j2";
const MARKDOWN_SHOW_SURFACE_TEMPLATE_SOURCE: &str =
    include_str!("../../resources/templates/control_plane/qianji_show_surface.md.j2");

static SHOW_TEMPLATE_CATALOG: EmbeddedTemplateCatalog = EmbeddedTemplateCatalog::new(
    "Qianji markdown show surface renderer",
    &[(
        MARKDOWN_SHOW_SURFACE_TEMPLATE_NAME,
        MARKDOWN_SHOW_SURFACE_TEMPLATE_SOURCE,
    )],
);

/// One section in the shared markdown `qianji show` surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownShowSection<'a> {
    /// Stable section title rendered as an H2 heading.
    pub(crate) title: Cow<'a, str>,
    /// Section body lines rendered after the heading.
    pub(crate) lines: Vec<String>,
}

/// Render the shared markdown `qianji show` surface.
#[must_use]
pub(crate) fn render_show_surface(
    title: &str,
    header_lines: &[String],
    sections: &[MarkdownShowSection<'_>],
) -> String {
    let payload = json!({
        "title": title,
        "header_block": header_lines.join("\n"),
        "sections": sections
            .iter()
            .map(|section| {
                json!({
                    "title": section.title,
                    "body": section.lines.join("\n"),
                })
            })
            .collect::<Vec<_>>(),
    });

    match SHOW_TEMPLATE_CATALOG
        .render_lines(MARKDOWN_SHOW_SURFACE_TEMPLATE_NAME, payload)
        .map(|lines| lines.join("\n"))
    {
        Ok(rendered) => rendered,
        Err(error) => {
            log::warn!("{error}");
            render_show_surface_inline(title, header_lines, sections)
        }
    }
}

fn render_show_surface_inline(
    title: &str,
    header_lines: &[String],
    sections: &[MarkdownShowSection<'_>],
) -> String {
    let mut lines = vec![format!("# {title}")];
    if !header_lines.is_empty() {
        lines.push(String::new());
        lines.extend(header_lines.iter().cloned());
    }
    for section in sections {
        lines.push(String::new());
        lines.push(format!("## {}", section.title));
        lines.extend(section.lines.iter().cloned());
    }
    lines.join("\n")
}

#[cfg(test)]
#[path = "../../tests/unit/markdown/show.rs"]
mod tests;
