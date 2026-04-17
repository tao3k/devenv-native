use xiuxian_git_repo::{
    RevisionChangeKind, diff_checkout_revisions, read_checkout_file_bytes_at_revision,
};

use crate::support::{
    append_repo_file_and_commit, head_revision, init_test_repository, must,
    remove_repo_file_and_commit, rename_repo_file_and_commit, temp_dir,
};

#[test]
fn diff_checkout_revisions_reports_modify_delete_and_rename_paths() {
    let repository = temp_dir();
    init_test_repository(repository.path());
    append_repo_file_and_commit(
        repository.path(),
        "src/Alpha.jl",
        "module Alpha\nalpha() = 1\nend\n",
        "add alpha",
    );
    append_repo_file_and_commit(
        repository.path(),
        "examples/demo.jl",
        "using Alpha\nalpha()\n",
        "add example",
    );
    let previous_revision = head_revision(repository.path());

    append_repo_file_and_commit(
        repository.path(),
        "src/Alpha.jl",
        "module Alpha\nalpha() = 2\nend\n",
        "modify alpha",
    );
    remove_repo_file_and_commit(repository.path(), "examples/demo.jl", "remove example");
    rename_repo_file_and_commit(
        repository.path(),
        "src/Alpha.jl",
        "src/Beta.jl",
        "rename alpha",
    );
    let revision = head_revision(repository.path());

    let diff = must(
        diff_checkout_revisions(repository.path(), &previous_revision, &revision),
        "diff revisions",
    );

    assert_eq!(diff.previous_revision, previous_revision);
    assert_eq!(diff.revision, revision);
    assert!(diff.changes.iter().any(|change| {
        change.kind == RevisionChangeKind::Deleted
            && change.previous_path.as_deref() == Some("examples/demo.jl")
    }));
    assert!(diff.changes.iter().any(|change| {
        change.kind == RevisionChangeKind::Renamed
            && change.previous_path.as_deref() == Some("src/Alpha.jl")
            && change.path == "src/Beta.jl"
    }));
    assert!(diff.changed_paths().contains("src/Beta.jl"));
    assert!(diff.deleted_paths().contains("src/Alpha.jl"));
    assert!(diff.deleted_paths().contains("examples/demo.jl"));
}

#[test]
fn diff_checkout_revisions_returns_empty_summary_for_identical_revisions() {
    let repository = temp_dir();
    init_test_repository(repository.path());
    let revision = head_revision(repository.path());

    let diff = must(
        diff_checkout_revisions(repository.path(), &revision, &revision),
        "diff revisions",
    );

    assert!(diff.is_empty());
    assert!(diff.changed_paths().is_empty());
    assert!(diff.deleted_paths().is_empty());
}

#[test]
fn diff_checkout_revision_file_reads_previous_blob_contents() {
    let repository = temp_dir();
    init_test_repository(repository.path());
    append_repo_file_and_commit(
        repository.path(),
        "src/Alpha.jl",
        "module Alpha\nalpha() = 1\nend\n",
        "add alpha",
    );
    let previous_revision = head_revision(repository.path());

    append_repo_file_and_commit(
        repository.path(),
        "src/Alpha.jl",
        "module Alpha\nalpha() = 2\nend\n",
        "modify alpha",
    );
    let revision = head_revision(repository.path());

    let previous_blob = must(
        read_checkout_file_bytes_at_revision(repository.path(), &previous_revision, "src/Alpha.jl"),
        "read previous revision file",
    );
    let Some(previous_blob) = previous_blob else {
        panic!("previous revision file should exist");
    };
    let current_blob = must(
        read_checkout_file_bytes_at_revision(repository.path(), &revision, "src/Alpha.jl"),
        "read current revision file",
    );
    let Some(current_blob) = current_blob else {
        panic!("current revision file should exist");
    };

    assert_eq!(
        must(
            String::from_utf8(previous_blob),
            "previous blob should be utf8"
        ),
        "module Alpha\nalpha() = 1\nend\n"
    );
    assert_eq!(
        must(
            String::from_utf8(current_blob),
            "current blob should be utf8"
        ),
        "module Alpha\nalpha() = 2\nend\n"
    );
}
