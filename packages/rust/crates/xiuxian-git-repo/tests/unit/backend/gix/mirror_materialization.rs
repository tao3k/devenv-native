use std::process::Command;

use super::{
    checkout_detached_to_revision, clone_bare_with_retry, clone_checkout_from_mirror,
    create_annotated_tag, create_branch_and_commit, fetch_origin_with_retry, head_revision,
    init_test_repository, must, temp_dir,
};

#[test]
fn clone_bare_with_retry_preserves_mirror_branch_and_tag_refs() {
    let source = temp_dir();
    init_test_repository(source.path());
    create_branch_and_commit(
        source.path(),
        "release",
        "src/release.jl",
        "const RELEASE = true\n",
        "release branch commit",
    );
    create_annotated_tag(source.path(), "v1.0.0", "release tag");
    let mirror = temp_dir();
    let _repository = must(
        clone_bare_with_retry(source.path().display().to_string().as_str(), mirror.path()),
        "clone bare mirror",
    );
    let refs_output = must(
        Command::new("git")
            .arg("-C")
            .arg(mirror.path())
            .arg("show-ref")
            .output(),
        "list mirror refs",
    );
    let refs = String::from_utf8_lossy(&refs_output.stdout);

    assert!(refs.contains("refs/heads/release"), "refs: {refs}");
    assert!(refs.contains("refs/tags/v1.0.0"), "refs: {refs}");
}

#[test]
fn fetch_origin_with_retry_refreshes_existing_mirror() {
    let source = temp_dir();
    init_test_repository(source.path());
    let mirror = temp_dir();
    let repository = must(
        clone_bare_with_retry(source.path().display().to_string().as_str(), mirror.path()),
        "clone bare mirror",
    );

    create_branch_and_commit(
        source.path(),
        "release",
        "src/release.jl",
        "const RELEASE = true\n",
        "release branch commit",
    );

    must(fetch_origin_with_retry(&repository), "fetch mirror");

    let refs_output = must(
        Command::new("git")
            .arg("-C")
            .arg(mirror.path())
            .arg("show-ref")
            .output(),
        "list mirror refs",
    );
    let refs = String::from_utf8_lossy(&refs_output.stdout);

    assert!(refs.contains("refs/heads/release"), "refs: {refs}");
}

#[test]
fn clone_checkout_from_mirror_materializes_requested_revision() {
    let source = temp_dir();
    init_test_repository(source.path());
    let mirror = temp_dir();
    let _mirror_repository = must(
        clone_bare_with_retry(source.path().display().to_string().as_str(), mirror.path()),
        "clone bare mirror",
    );
    let checkout = temp_dir();
    let expected = head_revision(source.path());
    let mirror_origin = mirror.path().display().to_string();

    let materialized = must(
        clone_checkout_from_mirror(mirror_origin.as_str(), checkout.path()),
        "materialize checkout",
    );

    assert_eq!(head_revision(checkout.path()), expected);
    assert!(materialized.workdir().is_some());
    assert!(checkout.path().join(".git").exists());
}

#[test]
fn checkout_detached_to_revision_resets_existing_checkout() {
    let source = temp_dir();
    init_test_repository(source.path());
    let mirror = temp_dir();
    let mirror_repository = must(
        clone_bare_with_retry(source.path().display().to_string().as_str(), mirror.path()),
        "clone bare mirror",
    );
    let checkout = temp_dir();
    let initial = head_revision(source.path());
    let mirror_origin = mirror.path().display().to_string();
    let mut checkout_repository = must(
        clone_checkout_from_mirror(mirror_origin.as_str(), checkout.path()),
        "materialize checkout",
    );
    assert_eq!(head_revision(checkout.path()), initial);

    create_branch_and_commit(
        source.path(),
        "main",
        "src/runtime.jl",
        "const UPDATED = true\n",
        "update main",
    );
    must(
        fetch_origin_with_retry(&mirror_repository),
        "refresh mirror",
    );
    must(
        fetch_origin_with_retry(&checkout_repository),
        "refresh checkout from mirror",
    );
    let updated = head_revision(source.path());

    must(
        checkout_detached_to_revision(&mut checkout_repository, &updated),
        "reset checkout",
    );

    assert_eq!(head_revision(checkout.path()), updated);
}
