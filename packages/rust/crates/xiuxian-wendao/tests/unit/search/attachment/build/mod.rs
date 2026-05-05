use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Duration;

use crate::link_graph::LinkGraphAttachmentKind;
use crate::search::attachment::build::plan_attachment_build;
use crate::search::attachment::search_attachment_hits;
use crate::search::cache::SearchPlaneCache;
use crate::search::contracts::SearchProjectConfig;
use crate::search::{
    SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlanePhase,
    SearchPlaneService,
};

fn planning_service(project_root: &Path) -> SearchPlaneService {
    SearchPlaneService::with_paths(
        project_root.to_path_buf(),
        project_root.join(".data/search_plane"),
        SearchManifestKeyspace::new("xiuxian:test:search_plane:attachment-plan"),
        SearchMaintenancePolicy::default(),
    )
}

#[test]
fn plan_attachment_build_only_reparses_changed_notes() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("docs"))
        .unwrap_or_else(|error| panic!("create docs: {error}"));
    std::fs::write(
        project_root.join("docs/alpha.md"),
        "# Alpha\n\n![Topology](assets/topology.png)\n",
    )
    .unwrap_or_else(|error| panic!("write alpha note: {error}"));
    std::fs::write(
        project_root.join("docs/beta.md"),
        "# Beta\n\n![Avatar](images/avatar.jpg)\n",
    )
    .unwrap_or_else(|error| panic!("write beta note: {error}"));
    let projects = vec![SearchProjectConfig {
        name: "kernel".to_string(),
        root: ".".to_string(),
        dirs: vec!["docs".to_string()],
    }];
    let service = planning_service(project_root);

    let first = plan_attachment_build(
        &service,
        project_root,
        project_root,
        &projects,
        None,
        &BTreeMap::new(),
    );
    assert_eq!(first.base_epoch, None);
    assert!(
        first
            .changed_hits
            .iter()
            .any(|hit| hit.source_path == "docs/alpha.md" && hit.attachment_name == "topology.png")
    );
    assert!(
        first
            .changed_hits
            .iter()
            .any(|hit| hit.source_path == "docs/beta.md" && hit.attachment_name == "avatar.jpg")
    );

    std::thread::sleep(Duration::from_millis(5));
    std::fs::write(
        project_root.join("docs/alpha.md"),
        "# Alpha\n\n![Diagram](assets/diagram.png)\n",
    )
    .unwrap_or_else(|error| panic!("rewrite alpha note: {error}"));

    let second = plan_attachment_build(
        &service,
        project_root,
        project_root,
        &projects,
        Some(7),
        &first.file_fingerprints,
    );
    assert_eq!(second.base_epoch, Some(7));
    assert_eq!(
        second.replaced_paths,
        BTreeSet::from(["docs/alpha.md".to_string()])
    );
    assert!(
        second
            .changed_hits
            .iter()
            .all(|hit| hit.source_path == "docs/alpha.md")
    );
    assert!(
        second
            .changed_hits
            .iter()
            .any(|hit| hit.attachment_name == "diagram.png")
    );
    assert!(
        second
            .changed_hits
            .iter()
            .all(|hit| hit.attachment_name != "avatar.jpg"),
        "unchanged note attachments must not be reparsed into the changed set"
    );
}

#[test]
fn plan_attachment_build_ignores_metadata_only_edits_when_hits_are_unchanged() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path();
    std::fs::create_dir_all(project_root.join("docs"))
        .unwrap_or_else(|error| panic!("create docs: {error}"));
    std::fs::write(
        project_root.join("docs/alpha.md"),
        "# Alpha\n\n![Topology](assets/topology.png)\n",
    )
    .unwrap_or_else(|error| panic!("write alpha note: {error}"));
    let projects = vec![SearchProjectConfig {
        name: "kernel".to_string(),
        root: ".".to_string(),
        dirs: vec!["docs".to_string()],
    }];
    let service = planning_service(project_root);

    let first = plan_attachment_build(
        &service,
        project_root,
        project_root,
        &projects,
        None,
        &BTreeMap::new(),
    );
    let first_fingerprint = first
        .file_fingerprints
        .get("docs/alpha.md")
        .unwrap_or_else(|| panic!("initial attachment fingerprint"));

    std::thread::sleep(Duration::from_millis(5));
    std::fs::write(
        project_root.join("docs/alpha.md"),
        "# Alpha\n\n![Topology](assets/topology.png)\n\n",
    )
    .unwrap_or_else(|error| panic!("rewrite alpha note: {error}"));

    let second = plan_attachment_build(
        &service,
        project_root,
        project_root,
        &projects,
        Some(7),
        &first.file_fingerprints,
    );
    let second_fingerprint = second
        .file_fingerprints
        .get("docs/alpha.md")
        .unwrap_or_else(|| panic!("updated attachment fingerprint"));

    assert_eq!(second.base_epoch, Some(7));
    assert!(second.replaced_paths.is_empty());
    assert!(second.changed_hits.is_empty());
    assert_ne!(first_fingerprint.size_bytes, second_fingerprint.size_bytes);
    assert_eq!(first_fingerprint.blake3, second_fingerprint.blake3);
}

#[tokio::test]
async fn attachment_incremental_refresh_reuses_unchanged_rows() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(project_root.join("docs"))
        .unwrap_or_else(|error| panic!("create docs: {error}"));
    std::fs::write(
        project_root.join("docs/alpha.md"),
        "# Alpha\n\n![Topology](assets/topology.png)\n",
    )
    .unwrap_or_else(|error| panic!("write alpha note: {error}"));
    std::fs::write(
        project_root.join("docs/beta.md"),
        "# Beta\n\n![Avatar](images/avatar.jpg)\n",
    )
    .unwrap_or_else(|error| panic!("write beta note: {error}"));
    let projects = vec![SearchProjectConfig {
        name: "kernel".to_string(),
        root: ".".to_string(),
        dirs: vec!["docs".to_string()],
    }];
    let keyspace = SearchManifestKeyspace::new("xiuxian:test:search_plane:attachment-incremental");
    let cache = SearchPlaneCache::for_tests(keyspace.clone());
    let service = SearchPlaneService::with_runtime(
        project_root.clone(),
        storage_root,
        keyspace,
        SearchMaintenancePolicy::default(),
        cache,
    );

    super::ensure_attachment_index_started(
        &service,
        project_root.as_path(),
        project_root.as_path(),
        &projects,
    );
    wait_for_attachment_ready(&service, None).await;

    let initial_avatar = search_attachment_hits(&service, "avatar", 10, &[], &[], false)
        .await
        .unwrap_or_else(|error| panic!("query avatar: {error}"));
    assert_eq!(initial_avatar.len(), 1);
    let initial_topology = search_attachment_hits(&service, "topology", 10, &[], &[], false)
        .await
        .unwrap_or_else(|error| panic!("query topology: {error}"));
    assert_eq!(initial_topology.len(), 1);

    std::fs::write(
        project_root.join("docs/alpha.md"),
        "# Alpha\n\n![Diagram](assets/diagram.png)\n",
    )
    .unwrap_or_else(|error| panic!("rewrite alpha note: {error}"));
    super::ensure_attachment_index_started(
        &service,
        project_root.as_path(),
        project_root.as_path(),
        &projects,
    );
    wait_for_attachment_ready(&service, Some(1)).await;

    let avatar = search_attachment_hits(&service, "avatar", 10, &[], &[], false)
        .await
        .unwrap_or_else(|error| panic!("query avatar after refresh: {error}"));
    assert_eq!(avatar.len(), 1);
    let diagram = search_attachment_hits(&service, "diagram", 10, &[], &[], false)
        .await
        .unwrap_or_else(|error| panic!("query diagram after refresh: {error}"));
    assert_eq!(diagram.len(), 1);
    assert_eq!(diagram[0].kind, "image");
    let topology = search_attachment_hits(
        &service,
        "topology",
        10,
        &[],
        &[LinkGraphAttachmentKind::Image],
        false,
    )
    .await
    .unwrap_or_else(|error| panic!("query topology after refresh: {error}"));
    assert!(topology.is_empty());
    let active_epoch = service
        .coordinator()
        .status_for(SearchCorpusKind::Attachment)
        .active_epoch
        .unwrap_or_else(|| panic!("attachment active epoch"));
    assert!(
        service
            .local_epoch_parquet_path(SearchCorpusKind::Attachment, active_epoch)
            .exists(),
        "missing attachment parquet export"
    );
    assert_no_attachment_lance_tables(&service);
}

async fn wait_for_attachment_ready(service: &SearchPlaneService, previous_epoch: Option<u64>) {
    for _ in 0..100 {
        let status = service
            .coordinator()
            .status_for(SearchCorpusKind::Attachment);
        if status.phase == SearchPlanePhase::Ready
            && status.active_epoch.is_some()
            && previous_epoch.is_none_or(|epoch| status.active_epoch.unwrap_or_default() > epoch)
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("attachment build did not reach ready state");
}

fn assert_no_attachment_lance_tables(service: &SearchPlaneService) {
    let corpus_root = service.corpus_root(SearchCorpusKind::Attachment);
    let entries = std::fs::read_dir(corpus_root.as_path())
        .unwrap_or_else(|error| panic!("read attachment corpus root: {error}"));
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("read attachment corpus entry: {error}"));
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        assert!(
            !file_name.ends_with(".lance"),
            "unexpected Lance table left behind for attachment: {file_name}"
        );
    }
}
