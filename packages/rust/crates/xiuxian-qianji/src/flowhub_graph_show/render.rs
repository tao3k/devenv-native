use crate::markdown::{MarkdownShowSection, render_show_surface};

use super::FlowhubGraphShow;
use super::render_execution::{
    render_declared_topology_line, render_execution_section_lines, render_mermaid_section_lines,
    render_node_section_lines,
};
use super::render_surface::{
    display_graph_path, render_check_surface_section_lines, render_done_gate_section_lines,
    render_persistent_target_surface_section_lines,
};

pub(super) fn render_flowhub_graph_show_impl(show: &FlowhubGraphShow) -> String {
    render_show_surface(
        "Graph",
        &graph_show_metadata_lines(show),
        &graph_show_sections(show),
    )
}

pub(crate) fn graph_show_sections(show: &FlowhubGraphShow) -> Vec<MarkdownShowSection<'_>> {
    let mut sections = vec![
        MarkdownShowSection {
            title: "Execution".into(),
            lines: render_execution_section_lines(show),
        },
        MarkdownShowSection {
            title: "Nodes".into(),
            lines: render_node_section_lines(show),
        },
        MarkdownShowSection {
            title: "Check Surface".into(),
            lines: render_check_surface_section_lines(show),
        },
    ];

    if !show
        .declared_check_surface
        .persistent_target_surface_tree
        .is_empty()
    {
        sections.push(MarkdownShowSection {
            title: "Persistent Target Surface".into(),
            lines: render_persistent_target_surface_section_lines(show),
        });
    }

    if !show.declared_check_surface.done_gate_require.is_empty() {
        sections.push(MarkdownShowSection {
            title: "Done Gate".into(),
            lines: render_done_gate_section_lines(show),
        });
    }
    sections.push(MarkdownShowSection {
        title: "Mermaid".into(),
        lines: render_mermaid_section_lines(show),
    });

    sections
}

fn graph_show_metadata_lines(show: &FlowhubGraphShow) -> Vec<String> {
    vec![
        format!("Name: {}", show.merimind_graph_name),
        format!("Path: {}", display_graph_path(&show.graph_path)),
        format!("Owning module: {}", show.owning_module_ref),
        format!("Direction: {}", show.direction),
        format!("Topology: {}", show.topology.as_str()),
        render_declared_topology_line(show.declared_topology),
    ]
}
