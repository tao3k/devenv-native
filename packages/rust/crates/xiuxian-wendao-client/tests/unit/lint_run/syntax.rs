use tempfile::TempDir;

use super::run_markdown_lint;

#[test]
fn lint_reports_invalid_yaml_frontmatter() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::write(temp.path().join("demo.md"), "---\ntags: [demo\n---\n")
        .expect("demo should exist");

    let (status, stdout) = run_markdown_lint(&temp, None);

    assert_eq!(status, Some(1));
    assert!(stdout.contains("invalid_frontmatter_yaml"));
    assert!(stdout.contains("kind: official_syntax"));
    assert!(stdout.contains("problem: YAML frontmatter is syntactically invalid."));
    assert!(stdout.contains("demo.md"));
}

#[test]
fn lint_reports_unclosed_frontmatter() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::write(temp.path().join("demo.md"), "---\ntitle: demo\nbody\n")
        .expect("demo should exist");

    let (status, stdout) = run_markdown_lint(&temp, None);

    assert_eq!(status, Some(1));
    assert!(stdout.contains("unclosed_frontmatter"));
    assert!(stdout.contains("kind: official_syntax"));
    assert!(stdout.contains("problem: YAML frontmatter opens but never closes."));
    assert!(stdout.contains(
        "expected: Close the frontmatter with `---` or `...` before the document body begins."
    ));
}

#[test]
fn lint_reports_invalid_utf8_as_official_syntax() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::write(temp.path().join("demo.md"), vec![0xff, 0xfe, 0xfd]).expect("demo should exist");

    let (status, stdout) = run_markdown_lint(&temp, None);

    assert_eq!(status, Some(1));
    assert!(stdout.contains("invalid_utf8"));
    assert!(stdout.contains("kind: official_syntax"));
    assert!(stdout.contains("problem: Markdown file is not valid UTF-8."));
}

#[test]
fn lint_reports_unclosed_fence() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::write(
        temp.path().join("demo.md"),
        "# Demo\n```rust\nfn main() {}\n",
    )
    .expect("demo should exist");

    let (status, stdout) = run_markdown_lint(&temp, None);

    assert_eq!(status, Some(1));
    assert!(stdout.contains("unclosed_fence"));
    assert!(stdout.contains("kind: official_syntax"));
    assert!(stdout.contains("problem: Fenced code block opens but never closes."));
    assert!(stdout.contains(
        "expected: Add a closing fence with the same marker type and at least the same width."
    ));
}

#[test]
fn lint_succeeds_for_clean_markdown() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::write(
        temp.path().join("guide.md"),
        "---\ntitle: Demo\n---\n# Heading\n```rust\nfn main() {}\n```\n",
    )
    .expect("guide should exist");

    let (status, stdout) = run_markdown_lint(&temp, None);

    assert_eq!(status, Some(0));
    assert!(
        stdout.contains("Markdown lint passed: checked 1 file(s), 0 issue(s)."),
        "{stdout}"
    );
}

#[test]
fn lint_uses_wendao_configured_project_roots_when_no_paths_are_given() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::create_dir_all(temp.path().join("frontend")).expect("frontend dir should exist");
    std::fs::write(
        temp.path().join("wendao.toml"),
        "[link_graph.projects.frontend]\nroot = \"frontend\"\n",
    )
    .expect("config should exist");
    std::fs::write(
        temp.path().join("frontend/guide.md"),
        "---\ntitle: Frontend Guide\n---\n# Frontend Guide\n",
    )
    .expect("frontend guide should exist");
    std::fs::write(temp.path().join("loose.md"), "---\ntags: [broken\n---\n")
        .expect("loose file should exist");

    let (status, stdout) = run_markdown_lint(&temp, None);

    assert_eq!(status, Some(0));
    assert!(
        stdout.contains("Markdown lint passed: checked 1 file(s), 0 issue(s)."),
        "{stdout}"
    );
}

#[test]
fn lint_skips_managed_remote_project_roots_when_paths_are_omitted() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::create_dir_all(temp.path().join("frontend")).expect("frontend dir should exist");
    std::fs::create_dir_all(temp.path().join("readonly-mirror"))
        .expect("readonly mirror dir should exist");
    std::fs::write(
        temp.path().join("wendao.toml"),
        concat!(
            "[link_graph.projects.frontend]\n",
            "root = \"frontend\"\n\n",
            "[link_graph.projects.readonly]\n",
            "root = \"readonly-mirror\"\n",
            "url = \"https://example.com/repo.git\"\n",
        ),
    )
    .expect("config should exist");
    std::fs::write(
        temp.path().join("frontend/guide.md"),
        "---\ntitle: Frontend Guide\n---\n# Frontend Guide\n",
    )
    .expect("frontend guide should exist");
    std::fs::write(
        temp.path().join("readonly-mirror/broken.md"),
        "---\ntags: [broken\n---\n",
    )
    .expect("readonly file should exist");

    let (status, stdout) = run_markdown_lint(&temp, None);

    assert_eq!(status, Some(0));
    assert!(
        stdout.contains("Markdown lint passed: checked 1 file(s), 0 issue(s)."),
        "{stdout}"
    );
}

#[test]
fn lint_skips_explicit_read_only_project_roots_when_paths_are_omitted() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::create_dir_all(temp.path().join("frontend")).expect("frontend dir should exist");
    std::fs::create_dir_all(temp.path().join("readonly-local"))
        .expect("readonly local dir should exist");
    std::fs::write(
        temp.path().join("wendao.toml"),
        concat!(
            "[link_graph.projects.frontend]\n",
            "root = \"frontend\"\n\n",
            "[link_graph.projects.readonly]\n",
            "root = \"readonly-local\"\n",
            "read_only = true\n",
        ),
    )
    .expect("config should exist");
    std::fs::write(
        temp.path().join("frontend/guide.md"),
        "---\ntitle: Frontend Guide\n---\n# Frontend Guide\n",
    )
    .expect("frontend guide should exist");
    std::fs::write(
        temp.path().join("readonly-local/broken.md"),
        "---\ntags: [broken\n---\n",
    )
    .expect("readonly file should exist");

    let (status, stdout) = run_markdown_lint(&temp, None);

    assert_eq!(status, Some(0));
    assert!(
        stdout.contains("Markdown lint passed: checked 1 file(s), 0 issue(s)."),
        "{stdout}"
    );
}

#[test]
fn lint_honors_explicit_read_only_false_for_managed_remote_projects() {
    let temp = TempDir::new().expect("tempdir should exist");
    std::fs::create_dir_all(temp.path().join("frontend")).expect("frontend dir should exist");
    std::fs::create_dir_all(temp.path().join("mirror")).expect("mirror dir should exist");
    std::fs::write(
        temp.path().join("wendao.toml"),
        concat!(
            "[link_graph.projects.frontend]\n",
            "root = \"frontend\"\n\n",
            "[link_graph.projects.mirror]\n",
            "root = \"mirror\"\n",
            "url = \"https://example.com/repo.git\"\n",
            "read_only = false\n",
        ),
    )
    .expect("config should exist");
    std::fs::write(
        temp.path().join("frontend/guide.md"),
        "---\ntitle: Frontend Guide\n---\n# Frontend Guide\n",
    )
    .expect("frontend guide should exist");
    std::fs::write(
        temp.path().join("mirror/broken.md"),
        "---\ntags: [broken\n---\n",
    )
    .expect("mirror file should exist");

    let (status, stdout) = run_markdown_lint(&temp, None);

    assert_eq!(status, Some(1));
    assert!(stdout.contains("mirror/broken.md"), "{stdout}");
    assert!(stdout.contains("invalid_frontmatter_yaml"), "{stdout}");
}
