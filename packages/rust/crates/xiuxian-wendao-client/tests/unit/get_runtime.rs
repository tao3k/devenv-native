use std::path::PathBuf;
use std::process::Command;

use tempfile::TempDir;
use xiuxian_wendao_client::{ClientContext, OutputFormat};

#[test]
fn standalone_get_command_runs_without_embedded_runtime() {
    let temp = tempdir_or_panic();
    let readme = temp.path().join("README.md");
    std::fs::write(&readme, "# Demo\n\n## Usage\n\nRun it.\n")
        .unwrap_or_else(|error| panic!("write README: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("get")
        .arg("toc")
        .arg("./README.md")
        .output()
        .unwrap_or_else(|error| panic!("run standalone get: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    assert_eq!(output.status.code(), Some(0));
    assert!(
        stdout.starts_with(format!("path: {}", display_test_path(&readme)).as_str()),
        "unexpected stdout: {stdout}"
    );
    assert!(
        stdout.contains("# Demo -> [L1 1-2]"),
        "unexpected stdout: {stdout}"
    );
    assert!(
        stdout.contains("## Usage -> [L2 3-5]"),
        "unexpected stdout: {stdout}"
    );
    assert!(!stdout.contains("# TOC"), "unexpected stdout: {stdout}");
}

#[test]
fn standalone_page_index_renders_links_and_embeds_with_source_syntax() {
    let temp = tempdir_or_panic();
    let readme = temp.path().join("README.md");
    std::fs::write(
        &readme,
        "# Demo\n\n[Guide](docs/guide.md)\n![Diagram](assets/diagram.png)\n![[notes/overview#Section]]\n",
    )
    .unwrap_or_else(|error| panic!("write README: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("get")
        .arg("page-index")
        .arg("./README.md")
        .output()
        .unwrap_or_else(|error| panic!("run standalone page-index: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    assert_eq!(output.status.code(), Some(0));
    assert!(
        stdout.starts_with(format!("path: {}", display_test_path(&readme)).as_str()),
        "unexpected stdout: {stdout}"
    );
    assert!(
        stdout.contains("kind: Explanation | roots: 1 | nodes: 1 | links: 1 | embeds: 2"),
        "unexpected stdout: {stdout}"
    );
    assert!(
        stdout.contains("links: [Guide](docs/guide.md)"),
        "unexpected stdout: {stdout}"
    );
    assert!(
        stdout.contains("embeds: ![Diagram](assets/diagram.png), ![[notes/overview#Section]]"),
        "unexpected stdout: {stdout}"
    );
    assert!(
        !stdout.contains("markdown_link:"),
        "unexpected stdout: {stdout}"
    );
}

#[test]
fn client_context_tracks_optional_config_path() {
    let context = ClientContext::new(PathBuf::from(".").as_path(), OutputFormat::Text)
        .with_config_file(Some(PathBuf::from("wendao.toml")));

    assert!(context.config_file().is_some());
}

#[test]
fn standalone_get_command_honors_cli_ignore_dirs() {
    let temp = tempdir_or_panic();
    std::fs::create_dir_all(temp.path().join("docs"))
        .unwrap_or_else(|error| panic!("create docs dir: {error}"));
    std::fs::create_dir_all(temp.path().join("generated"))
        .unwrap_or_else(|error| panic!("create generated dir: {error}"));
    std::fs::write(
        temp.path().join("docs/guide.md"),
        "# Guide\n\n## Usage\n\nVisible.\n",
    )
    .unwrap_or_else(|error| panic!("write guide: {error}"));
    std::fs::write(
        temp.path().join("generated/cache.md"),
        "# Cache\n\n## Generated\n\nHidden.\n",
    )
    .unwrap_or_else(|error| panic!("write generated doc: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("get")
        .arg("toc")
        .arg(".")
        .arg("--ignore")
        .arg("generated")
        .output()
        .unwrap_or_else(|error| panic!("run standalone get with ignore: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    assert_eq!(output.status.code(), Some(0));
    assert!(
        stdout.contains("docs/guide.md"),
        "unexpected stdout: {stdout}"
    );
    assert!(
        !stdout.contains("generated/cache.md"),
        "unexpected stdout: {stdout}"
    );
}

#[test]
fn standalone_get_command_merges_root_wendao_toml_and_cli_ignore_dirs() {
    let temp = tempdir_or_panic();
    std::fs::create_dir_all(temp.path().join("docs"))
        .unwrap_or_else(|error| panic!("create docs dir: {error}"));
    std::fs::create_dir_all(temp.path().join("generated"))
        .unwrap_or_else(|error| panic!("create generated dir: {error}"));
    std::fs::create_dir_all(temp.path().join("scratch"))
        .unwrap_or_else(|error| panic!("create scratch dir: {error}"));
    std::fs::write(
        temp.path().join("docs/guide.md"),
        "# Guide\n\n## Usage\n\nVisible.\n",
    )
    .unwrap_or_else(|error| panic!("write guide: {error}"));
    std::fs::write(
        temp.path().join("generated/cache.md"),
        "# Cache\n\n## Generated\n\nHidden by config.\n",
    )
    .unwrap_or_else(|error| panic!("write generated doc: {error}"));
    std::fs::write(
        temp.path().join("scratch/note.md"),
        "# Scratch\n\n## Draft\n\nHidden by CLI.\n",
    )
    .unwrap_or_else(|error| panic!("write scratch doc: {error}"));
    std::fs::write(
        temp.path().join("wendao.toml"),
        "[sources]\nexclude_dirs = [\"generated\"]\n",
    )
    .unwrap_or_else(|error| panic!("write wendao.toml: {error}"));

    let output = Command::new(env!("CARGO_BIN_EXE_wendao-client"))
        .arg("--root")
        .arg(temp.path())
        .arg("get")
        .arg("toc")
        .arg(".")
        .arg("--ignore")
        .arg("scratch")
        .output()
        .unwrap_or_else(|error| panic!("run standalone get with merged ignore: {error}"));

    let stdout =
        String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("stdout utf8: {error}"));
    assert_eq!(output.status.code(), Some(0));
    assert!(
        stdout.contains("docs/guide.md"),
        "unexpected stdout: {stdout}"
    );
    assert!(
        !stdout.contains("generated/cache.md"),
        "unexpected stdout: {stdout}"
    );
    assert!(
        !stdout.contains("scratch/note.md"),
        "unexpected stdout: {stdout}"
    );
}

fn tempdir_or_panic() -> TempDir {
    tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"))
}

fn display_test_path(path: &std::path::Path) -> String {
    std::fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/")
}
