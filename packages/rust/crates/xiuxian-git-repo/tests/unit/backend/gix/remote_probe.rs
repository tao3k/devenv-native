use super::{
    Ref, RevisionSelector, clone_bare_with_retry, create_annotated_tag, create_branch_and_commit,
    default_remote_head_revision, describe_remote_refs, head_revision, init_test_repository, must,
    object_id, probe_remote_target_revision_with_retry, remote_probe_options,
    remote_ref_target_revision, rev_parse, temp_dir,
};

#[test]
fn default_remote_head_revision_uses_symbolic_head_object() {
    let remote_refs = vec![
        Ref::Symbolic {
            full_ref_name: "HEAD".into(),
            target: "refs/heads/main".into(),
            tag: None,
            object: object_id(b"0123456789012345678901234567890123456789"),
        },
        Ref::Direct {
            full_ref_name: "refs/heads/main".into(),
            object: object_id(b"0123456789012345678901234567890123456789"),
        },
    ];

    assert_eq!(
        default_remote_head_revision(&remote_refs).as_deref(),
        Some("0123456789012345678901234567890123456789")
    );
}

#[test]
fn remote_ref_target_revision_prefers_peeled_target_object_for_tags() {
    let remote_refs = vec![Ref::Peeled {
        full_ref_name: "refs/tags/v1.0.0".into(),
        tag: object_id(b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        object: object_id(b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
    }];

    assert_eq!(
        remote_ref_target_revision(&remote_refs, "refs/tags/v1.0.0").as_deref(),
        Some("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
    );
}

#[test]
fn probe_remote_target_revision_resolves_default_head_for_local_mirror() {
    let source = temp_dir();
    init_test_repository(source.path());
    let mirror = temp_dir();
    let repository = must(
        clone_bare_with_retry(source.path().display().to_string().as_str(), mirror.path()),
        "clone bare mirror",
    );
    let expected = head_revision(source.path());

    let probed = must(
        probe_remote_target_revision_with_retry(&repository, None),
        "probe default head",
    );

    assert_eq!(
        probed.as_deref(),
        Some(expected.as_str()),
        "remote refs: {}",
        describe_remote_refs(&repository)
    );
}

#[test]
fn probe_remote_target_revision_resolves_branch_for_local_mirror() {
    let source = temp_dir();
    init_test_repository(source.path());
    create_branch_and_commit(
        source.path(),
        "release",
        "src/release.jl",
        "const RELEASE = true\n",
        "release branch commit",
    );
    let mirror = temp_dir();
    let repository = must(
        clone_bare_with_retry(source.path().display().to_string().as_str(), mirror.path()),
        "clone bare mirror",
    );
    let expected = rev_parse(source.path(), "release");

    let probed = must(
        probe_remote_target_revision_with_retry(
            &repository,
            Some(&RevisionSelector::Branch("release".to_string())),
        ),
        "probe branch",
    );

    assert_eq!(
        probed.as_deref(),
        Some(expected.as_str()),
        "remote refs: {}",
        describe_remote_refs(&repository)
    );
}

#[test]
fn probe_remote_target_revision_resolves_annotated_tag_for_local_mirror() {
    let source = temp_dir();
    init_test_repository(source.path());
    create_annotated_tag(source.path(), "v1.0.0", "release tag");
    let mirror = temp_dir();
    let repository = must(
        clone_bare_with_retry(source.path().display().to_string().as_str(), mirror.path()),
        "clone bare mirror",
    );
    let expected = rev_parse(source.path(), "refs/tags/v1.0.0^{}");

    let probed = must(
        probe_remote_target_revision_with_retry(
            &repository,
            Some(&RevisionSelector::Tag("v1.0.0".to_string())),
        ),
        "probe tag",
    );

    assert_eq!(
        probed.as_deref(),
        Some(expected.as_str()),
        "remote refs: {}",
        describe_remote_refs(&repository)
    );
}

#[test]
fn remote_probe_options_include_expected_refspecs() {
    let default = must(remote_probe_options(None), "build default probe options");
    assert_eq!(default.extra_refspecs.len(), 1);
    assert!(format!("{:?}", default.extra_refspecs[0]).contains("HEAD"));

    let branch = must(
        remote_probe_options(Some(&RevisionSelector::Branch("main".to_string()))),
        "build branch probe options",
    );
    assert_eq!(branch.extra_refspecs.len(), 1);
    assert!(format!("{:?}", branch.extra_refspecs[0]).contains("refs/heads/main"));

    let tag = must(
        remote_probe_options(Some(&RevisionSelector::Tag("v1.0.0".to_string()))),
        "build tag probe options",
    );
    assert_eq!(tag.extra_refspecs.len(), 1);
    assert!(format!("{:?}", tag.extra_refspecs[0]).contains("refs/tags/v1.0.0"));
}
