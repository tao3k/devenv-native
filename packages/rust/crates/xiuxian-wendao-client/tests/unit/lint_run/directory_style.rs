use tempfile::TempDir;

use super::run_markdown_lint;

#[test]
fn lint_reports_directory_link_style_mismatch_with_precise_rewrite_guidance() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::create_dir_all(temp.path().join("docs")).expect("docs dir should exist");
    std::fs::write(
        temp.path().join("docs/index.md"),
        "---\ntitle: Documentation Index\n---\n# Documentation Index\n",
    )
    .expect("index should exist");
    std::fs::write(
        temp.path().join("docs/guide-one.md"),
        "# Guide One\nSee [[index|Documentation Index]].\n",
    )
    .expect("guide one should exist");
    std::fs::write(
        temp.path().join("docs/guide-two.md"),
        "# Guide Two\nSee [[index|Documentation Index]].\n",
    )
    .expect("guide two should exist");
    std::fs::write(
        temp.path().join("docs/guide-three.md"),
        "# Guide Three\nSee [Documentation Index](index.md).\n",
    )
    .expect("guide three should exist");

    let (status, stdout) = run_markdown_lint(&temp, Some("docs"));

    assert_eq!(status, Some(1));
    assert!(stdout.contains("directory_link_style_mismatch"), "{stdout}");
    assert!(
        stdout.contains(
            "Directory `docs` mixes explicit Obsidian wikilinks and Markdown note links."
        ),
        "{stdout}"
    );
    assert!(
        stdout.contains("found: [Documentation Index](index.md)"),
        "{stdout}"
    );
    assert!(
        stdout.contains(
            "expected: Rewrite note links in this file to `[[index.md|Documentation Index]]` to match directory `docs`."
        ),
        "{stdout}"
    );
    assert!(
        stdout.contains("tip: Neighbor files already using `[[target|label]]` style include: docs/guide-one.md, docs/guide-two.md."),
        "{stdout}"
    );
}

#[test]
fn lint_reports_directory_link_style_ambiguity_when_no_local_style_dominates() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::create_dir_all(temp.path().join("notes")).expect("notes dir should exist");
    std::fs::write(
        temp.path().join("notes/index.md"),
        "---\ntitle: Notes Index\n---\n# Notes Index\n",
    )
    .expect("index should exist");
    std::fs::write(
        temp.path().join("notes/obsidian.md"),
        "# Obsidian\nSee [[index|Notes Index]].\n",
    )
    .expect("obsidian note should exist");
    std::fs::write(
        temp.path().join("notes/markdown.md"),
        "# Markdown\nSee [Notes Index](index.md).\n",
    )
    .expect("markdown note should exist");

    let (status, stdout) = run_markdown_lint(&temp, Some("notes"));

    assert_eq!(status, Some(1));
    assert!(
        stdout.contains("directory_link_style_ambiguous"),
        "{stdout}"
    );
    assert!(
        stdout.contains("Directory `notes` mixes explicit Obsidian wikilinks and Markdown note links without a clear local contract."),
        "{stdout}"
    );
    assert!(
        stdout.contains("expected: Choose either `[[target|label]]` or `[label](target)` for directory `notes`, then rewrite files consistently."),
        "{stdout}"
    );
    assert!(
        stdout.contains(
            "tip: Obsidian-style files: notes/obsidian.md. Markdown-style files: notes/markdown.md."
        ),
        "{stdout}"
    );
}
