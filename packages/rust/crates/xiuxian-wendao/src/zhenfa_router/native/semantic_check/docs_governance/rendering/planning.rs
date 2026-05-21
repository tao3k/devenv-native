//! `zhenfa_router::native::semantic_check::docs_governance::rendering::planning` owns Wendao docs governance rendering planning behavior.

use crate::parsers::docs_governance::collect_lines;
use crate::zhenfa_router::native::semantic_check::IssueLocation;
use crate::zhenfa_router::native::semantic_check::docs_governance::rendering::footer::render_index_footer_with_values;
use crate::zhenfa_router::native::semantic_check::docs_governance::rendering::shared::matches_section_heading;
use crate::zhenfa_router::native::semantic_check::docs_governance::types::SectionSpec;

/// Plan the insertion patch for a missing `:RELATIONS:` block in an index document.
#[must_use]
pub fn plan_index_relations_block_insertion(
    index_content: &str,
    body_links: &[String],
) -> (IssueLocation, String) {
    let lines = collect_lines(index_content);
    let insertion_line = lines
        .iter()
        .find(|line| line.trimmed == "---" || line.trimmed == ":FOOTER:");
    let insert_offset = insertion_line.map_or(index_content.len(), |line| line.start_offset);
    let prefix = if insert_offset == 0 || index_content[..insert_offset].ends_with("\n\n") {
        ""
    } else if index_content[..insert_offset].ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    let suffix = if insertion_line.is_some() { "\n" } else { "" };

    (
        IssueLocation {
            line: insertion_line
                .or_else(|| lines.last())
                .map_or(1, |line| line.line_number),
            heading_path: "Index Relations".to_string(),
            byte_range: Some((insert_offset, insert_offset)),
        },
        format!(
            "{prefix}:RELATIONS:\n:LINKS: {}\n:END:\n{suffix}",
            body_links
                .iter()
                .map(|link| format!("[[{link}]]"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    )
}

/// Plan the insertion patch for a missing index footer block.
#[must_use]
pub fn plan_index_footer_block_insertion(index_content: &str) -> (IssueLocation, String) {
    let lines = collect_lines(index_content);
    let insert_offset = index_content.len();
    let prefix = if index_content.is_empty() || index_content.ends_with("\n\n") {
        ""
    } else if index_content.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };

    (
        IssueLocation {
            line: lines.last().map_or(1, |line| line.line_number),
            heading_path: "Index Footer".to_string(),
            byte_range: Some((insert_offset, insert_offset)),
        },
        format!(
            "{prefix}---\n\n{}",
            render_index_footer_with_values("v2.0", "pending")
        ),
    )
}

/// Plan the insertion patch for a missing section landing-page link in the package index.
#[must_use]
pub fn plan_index_section_link_insertion(
    index_content: &str,
    spec: &SectionSpec,
    link_target: &str,
) -> (IssueLocation, String) {
    let lines = collect_lines(index_content);

    if let Some((heading_idx, heading_line)) = lines
        .iter()
        .enumerate()
        .find(|(_, line)| matches_section_heading(line.trimmed, spec.section_name))
    {
        let next_heading_idx = lines
            .iter()
            .enumerate()
            .skip(heading_idx + 1)
            .find(|(_, line)| line.trimmed.starts_with("## "))
            .map_or(lines.len(), |(idx, _)| idx);

        let section_lines = &lines[heading_idx + 1..next_heading_idx];
        if let Some(anchor) = section_lines
            .iter()
            .rev()
            .find(|line| !line.trimmed.is_empty())
        {
            let prefix = if anchor.newline.is_empty() { "\n" } else { "" };
            return (
                IssueLocation {
                    line: anchor.line_number,
                    heading_path: spec.section_name.to_string(),
                    byte_range: Some((anchor.end_offset, anchor.end_offset)),
                },
                format!("{prefix}- [[{link_target}]]\n"),
            );
        }

        let insert_offset = section_lines
            .iter()
            .take_while(|line| line.trimmed.is_empty())
            .last()
            .map_or(heading_line.end_offset, |line| line.end_offset);
        let prefix = if insert_offset == heading_line.end_offset {
            "\n"
        } else {
            ""
        };
        return (
            IssueLocation {
                line: heading_line.line_number,
                heading_path: spec.section_name.to_string(),
                byte_range: Some((insert_offset, insert_offset)),
            },
            format!("{prefix}- [[{link_target}]]\n"),
        );
    }

    let insertion_line = lines.iter().find(|line| {
        line.trimmed == ":RELATIONS:" || line.trimmed == "---" || line.trimmed == ":FOOTER:"
    });
    let insert_offset = insertion_line.map_or(index_content.len(), |line| line.start_offset);
    let prefix = if index_content.is_empty()
        || (insert_offset > 0 && index_content[..insert_offset].ends_with("\n\n"))
    {
        ""
    } else if insert_offset > 0 && index_content[..insert_offset].ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    let suffix = if insertion_line.is_some() { "\n" } else { "" };

    (
        IssueLocation {
            line: insertion_line
                .or_else(|| lines.last())
                .map_or(1, |line| line.line_number),
            heading_path: "Docs Index".to_string(),
            byte_range: Some((insert_offset, insert_offset)),
        },
        format!(
            "{prefix}## {}\n\n- [[{link_target}]]\n{suffix}",
            spec.section_name
        ),
    )
}
