use super::BootcampVfsMount;
use crate::contracts::QianjiManifest;
use crate::error::QianjiError;
use xiuxian_wendao_core::WendaoResourceUri;
use xiuxian_wendao_runtime::artifacts::zhixing::embedded_resource_text_from_wendao_uri;

pub(super) fn parse_manifest(manifest_toml: &str) -> Result<QianjiManifest, QianjiError> {
    toml::from_str(manifest_toml)
        .map_err(|error| QianjiError::Topology(format!("Failed to parse TOML: {error}")))
}

fn normalize_relative_path(path: &str) -> String {
    path.trim().trim_start_matches("./").replace('\\', "/")
}

fn resolve_flow_manifest_from_mounts(
    flow_uri: &str,
    vfs_mounts: &[BootcampVfsMount],
) -> Option<String> {
    let parsed = WendaoResourceUri::parse(flow_uri).ok()?;
    let semantic_name = parsed.semantic_name();
    let entity_relative_path =
        normalize_relative_path(parsed.entity_relative_path().to_string_lossy().as_ref());

    for mount in vfs_mounts {
        if !semantic_name.eq_ignore_ascii_case(mount.semantic_name) {
            continue;
        }
        let references_dir = normalize_relative_path(mount.references_dir);
        if references_dir.is_empty() {
            continue;
        }
        let candidate_path = format!("{references_dir}/{entity_relative_path}");
        let Some(content) = mount
            .dir
            .get_file(candidate_path.as_str())
            .and_then(include_dir::File::contents_utf8)
        else {
            continue;
        };
        return Some(content.to_string());
    }
    None
}

pub(super) fn resolve_flow_manifest_toml(
    flow_uri: &str,
    vfs_mounts: &[BootcampVfsMount],
) -> Result<String, QianjiError> {
    if let Some(content) = resolve_flow_manifest_from_mounts(flow_uri, vfs_mounts) {
        return Ok(content);
    }
    if let Some(content) = embedded_resource_text_from_wendao_uri(flow_uri) {
        return Ok(content.to_string());
    }
    Err(QianjiError::Topology(format!(
        "semantic flow manifest not found for URI `{flow_uri}`"
    )))
}

pub(super) fn parsed_manifest_requires_llm(manifest: &QianjiManifest) -> bool {
    manifest.nodes.iter().any(|node| {
        if node.task_type.trim().eq_ignore_ascii_case("llm") {
            return true;
        }
        node.task_type.trim().eq_ignore_ascii_case("formal_audit")
            && node.qianhuan.is_some()
            && node.llm.is_some()
    })
}

pub(super) fn parsed_manifest_requires_link_graph(manifest: &QianjiManifest) -> bool {
    manifest
        .nodes
        .iter()
        .any(|node| node.task_type.trim().eq_ignore_ascii_case("knowledge"))
}
