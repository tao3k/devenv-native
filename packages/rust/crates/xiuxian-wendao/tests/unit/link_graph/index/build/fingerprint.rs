use super::{fingerprint_scan_roots, scan_note_fingerprint};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn write_note(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|error| panic!("create note parent: {error}"));
    }
    fs::write(path, content).unwrap_or_else(|error| panic!("write note: {error}"));
}

fn set(values: &[&str]) -> HashSet<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[test]
fn include_fingerprint_matches_root_scoped_filter() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("create temp dir: {error}"));
    let root = temp.path();
    write_note(root, "docs/a.md", "# Alpha\n");
    write_note(root, "docs/nested/b.md", "# Beta\n");
    write_note(root, "other/c.md", "# Outside\n");
    let include_dirs = set(&["docs"]);
    let excluded_dirs = HashSet::new();

    let include_scoped = scan_note_fingerprint(root, &include_dirs, &excluded_dirs);
    let all_docs = scan_note_fingerprint(root, &HashSet::new(), &excluded_dirs);

    assert_eq!(include_scoped.note_count, 2);
    assert!(include_scoped.total_size_bytes < all_docs.total_size_bytes);
}

#[test]
fn nested_include_roots_are_deduplicated() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("create temp dir: {error}"));
    let root = temp.path();
    fs::create_dir_all(root.join("docs/nested"))
        .unwrap_or_else(|error| panic!("create include dirs: {error}"));

    let roots = fingerprint_scan_roots(root, &set(&["docs", "docs/nested"]));

    assert_eq!(roots, vec![root.join("docs")]);
}
