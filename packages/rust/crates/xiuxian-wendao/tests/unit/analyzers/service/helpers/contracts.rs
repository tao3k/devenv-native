use crate::analyzers::service::{relation_kind_label, repo_hierarchical_uri};
use crate::analyzers::{hierarchy_segments_from_path, infer_ecosystem, record_hierarchical_uri};

#[test]
fn relation_labels_and_uris_remain_stable() {
    let expected = [
        (crate::analyzers::RelationKind::Contains, "contains"),
        (crate::analyzers::RelationKind::Calls, "calls"),
        (crate::analyzers::RelationKind::Uses, "uses"),
        (crate::analyzers::RelationKind::Documents, "documents"),
        (crate::analyzers::RelationKind::ExampleOf, "example_of"),
        (crate::analyzers::RelationKind::Declares, "declares"),
        (crate::analyzers::RelationKind::Implements, "implements"),
        (crate::analyzers::RelationKind::Imports, "imports"),
    ];

    for (kind, label) in expected {
        assert_eq!(relation_kind_label(kind), label);
    }

    assert_eq!(repo_hierarchical_uri("repo-a"), "repo://repo-a");
    assert_eq!(
        record_hierarchical_uri("repo-a", "sciml", "symbol", "/src/alpha/", "sym-a"),
        "wendao://repo/sciml/repo-a/symbol/src:alpha/sym-a"
    );
}

#[test]
fn ecosystem_and_path_helpers_cover_common_inputs() {
    assert_eq!(infer_ecosystem("Diffeq-Docs"), "sciml");
    assert_eq!(infer_ecosystem("MSL"), "msl");
    assert_eq!(infer_ecosystem("plain-repo"), "unknown");
    assert_eq!(
        hierarchy_segments_from_path("/alpha//beta/"),
        Some(vec!["alpha".to_string(), "beta".to_string()])
    );
}
