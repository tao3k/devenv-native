use tempfile::TempDir;

use super::run_markdown_lint;

#[test]
fn lint_reports_non_canonical_obsidian_wikilinks_as_text() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::create_dir_all(temp.path().join("01_core")).expect("target dir should exist");
    std::fs::write(
        temp.path().join("01_core/106_docs_maintenance_playbook.md"),
        "---\ntitle: Docs Maintenance Playbook\n---\n# Docs Maintenance Playbook\n",
    )
    .expect("target note should exist");
    std::fs::write(
        temp.path().join("guide.md"),
        "# Heading\nSee [[Docs Maintenance Playbook|01_core/106_docs_maintenance_playbook]].\n",
    )
    .expect("guide should exist");

    let (status, stdout) = run_markdown_lint(&temp, None);

    assert_eq!(status, Some(1));
    assert!(stdout.contains("non_canonical_obsidian_alias_order"));
    assert!(stdout.contains("kind: repo_authoring_policy"));
    assert!(stdout.contains("target_title: Docs Maintenance Playbook"));
    assert!(stdout.contains(
        "rewrite as `[[01_core/106_docs_maintenance_playbook|Docs Maintenance Playbook]]`."
    ));
}

#[test]
fn lint_reports_bare_obsidian_wikilinks_as_text() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::create_dir_all(temp.path().join("docs")).expect("docs dir should exist");
    std::fs::write(
        temp.path().join("docs/index.md"),
        "---\ntitle: Documentation Index\n---\n# Documentation Index\n",
    )
    .expect("target note should exist");
    std::fs::write(
        temp.path().join("guide.md"),
        "# Heading\nSee [[docs/index]].\n",
    )
    .expect("guide should exist");

    let (status, stdout) = run_markdown_lint(&temp, None);

    assert_eq!(status, Some(1));
    assert!(stdout.contains("bare_obsidian_wikilink"));
    assert!(stdout.contains("kind: repo_authoring_policy"));
    assert!(stdout.contains("Obsidian officially allows bare wikilinks"));
    assert!(stdout.contains("target_title: Documentation Index"));
    assert!(stdout.contains(
        "rewrite as `[[docs/index|Documentation Index]]` or `[Documentation Index](docs/index)`."
    ));
}

#[test]
fn lint_reports_redundant_heading_labels_with_namespace_guidance() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::create_dir_all(temp.path().join("02_parser")).expect("parser dir should exist");
    std::fs::write(
        temp.path().join("02_parser/index.md"),
        "---\ntitle: Wendao Parser Docs\n---\n# Wendao Parser Docs\n",
    )
    .expect("target note should exist");
    std::fs::write(
        temp.path().join("guide.md"),
        "# Heading\nSee [[02_parser/index#Semantic Check|Semantic Check]].\n",
    )
    .expect("guide should exist");

    let (status, stdout) = run_markdown_lint(&temp, None);

    assert_eq!(status, Some(1));
    assert!(stdout.contains("redundant_obsidian_label"));
    assert!(stdout.contains("kind: repo_authoring_policy"));
    assert!(stdout.contains("target_title: Wendao Parser Docs"));
    assert!(stdout.contains("target_heading: Semantic Check"));
    assert!(stdout.contains("rewrite as `[[02_parser/index#Semantic Check|Wendao Parser Docs / Semantic Check]]` or `[Wendao Parser Docs / Semantic Check](02_parser/index#Semantic Check)`."));
}

#[test]
fn lint_reports_mixed_link_syntax_as_official_syntax_failure() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::create_dir_all(temp.path().join("docs")).expect("docs dir should exist");
    std::fs::write(
        temp.path().join("docs/index.md"),
        "---\ntitle: Documentation Index\n---\n# Documentation Index\n",
    )
    .expect("target note should exist");
    std::fs::write(
        temp.path().join("guide.md"),
        "# Heading\nSee [[docs/index]](Documentation Index).\n",
    )
    .expect("guide should exist");

    let (status, stdout) = run_markdown_lint(&temp, None);

    assert_eq!(status, Some(1));
    assert!(stdout.contains("mixed_wikilink_markdown_link"));
    assert!(stdout.contains("kind: official_syntax"));
    assert!(stdout.contains(
        "Choose either official Obsidian wikilink syntax or standard Markdown link syntax."
    ));
}

#[test]
fn lint_accepts_official_obsidian_embeds_and_addressed_targets() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::create_dir_all(temp.path().join("docs")).expect("docs dir should exist");
    std::fs::write(
        temp.path().join("docs/index.md"),
        "---\ntitle: Documentation Index\n---\n# Documentation Index\n## Parser Contracts\nParagraph.\n^block-id\n",
    )
    .expect("target note should exist");
    std::fs::write(
        temp.path().join("guide.md"),
        concat!(
            "# Root\n",
            "## Local Heading\n",
            "Paragraph.\n",
            "^local-block-id\n",
            "See [[Three laws of motion|Overview]].\n",
            "And [[Three laws of motion.md|Laws Note]].\n",
            "See [[docs/index#Parser Contracts|Contracts Overview]].\n",
            "And [[#Local Heading|Local Heading Context]].\n",
            "And [[#^local-block-id|Local Block Context]].\n",
            "And [[Help and support#Questions and advice#Report bugs and request features|Bug Reports]].\n",
            "And [[docs/index#^block-id|Block Context]].\n",
            "And ![[docs/index]].\n",
            "And ![[docs/index#Parser Contracts]].\n",
            "And ![[docs/index#^block-id]].\n",
        ),
    )
    .expect("guide should exist");

    let (status, stdout) = run_markdown_lint(&temp, None);

    assert_eq!(status, Some(0));
    assert!(
        stdout.contains("Markdown lint passed: checked 2 file(s), 0 issue(s)."),
        "{stdout}"
    );
}
