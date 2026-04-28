use std::fs;
use std::path::Path;

use tempfile::{TempDir, tempdir};
use xiuxian_wendao::{LinkGraphIndex, LinkGraphPprSubgraphMode, LinkGraphRelatedPprOptions};

pub(crate) const RELATED_MAX_DISTANCE: usize = 4;
pub(crate) const RELATED_LIMIT: usize = 24;

pub(crate) fn env_f64(name: &str, default_value: f64) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|raw| raw.trim().parse::<f64>().ok())
        .filter(|value| *value > 0.0)
        .unwrap_or(default_value)
}

pub(crate) fn env_u64(name: &str, default_value: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default_value)
}

pub(crate) fn env_usize(name: &str, default_value: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default_value)
}

fn note_id(i: usize) -> String {
    format!("note-{i:05}")
}

fn hub_id(i: usize) -> String {
    format!("hub-{i:03}")
}

fn write_note(path: &Path, body: &str) -> Result<(), String> {
    fs::write(path, body).map_err(|error| format!("write fixture note {}: {error}", path.display()))
}

pub(crate) fn build_link_graph_fixture(
    root: &Path,
    node_count: usize,
    hub_count: usize,
) -> Result<(), String> {
    for i in 0..node_count {
        let current = note_id(i);
        let next = note_id((i + 1) % node_count);
        let jump = note_id((i + 97) % node_count);
        let hub = hub_id(i % hub_count);
        let body = format!(
            "# {current}\n\nSynthetic performance test note {i}.\n\nLinks: [[{next}]] [[{jump}]] [[{hub}]]\n"
        );
        write_note(&root.join(format!("{current}.md")), &body)?;
    }

    for i in 0..hub_count {
        let hub = hub_id(i);
        let mut links = String::new();
        let mut cursor = i;
        let stride = hub_count * 2;
        let mut emitted = 0_usize;
        while cursor < node_count && emitted < 160 {
            if !links.is_empty() {
                links.push(' ');
            }
            links.push_str("[[");
            links.push_str(&note_id(cursor));
            links.push_str("]]");
            emitted += 1;
            cursor += stride;
        }
        let body = format!("# {hub}\n\nSynthetic hub {i}.\n\nOutbound links: {links}\n");
        write_note(&root.join(format!("{hub}.md")), &body)?;
    }

    Ok(())
}

pub(crate) fn build_index(
    node_count: usize,
    hub_count: usize,
) -> Result<(TempDir, LinkGraphIndex), String> {
    let temp = tempdir().map_err(|error| format!("create fixture tempdir: {error}"))?;
    build_link_graph_fixture(temp.path(), node_count, hub_count)?;
    let index = LinkGraphIndex::build(temp.path())
        .map_err(|error| format!("build link graph fixture index: {error}"))?;
    Ok((temp, index))
}

pub(crate) fn default_ppr_options() -> LinkGraphRelatedPprOptions {
    LinkGraphRelatedPprOptions {
        alpha: Some(0.9),
        max_iter: Some(30),
        tol: Some(1e-6),
        subgraph_mode: Some(LinkGraphPprSubgraphMode::Auto),
    }
}

pub(crate) fn seed_set(node_count: usize, count: usize) -> Vec<String> {
    let mut seeds = Vec::with_capacity(count.max(1));
    for turn in 0..count.max(1) {
        seeds.push(note_id((turn * 211) % node_count));
    }
    seeds
}
