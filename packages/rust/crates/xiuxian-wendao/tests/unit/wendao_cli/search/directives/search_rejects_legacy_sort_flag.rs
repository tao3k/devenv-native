use tempfile::TempDir;

use super::{wendao_cmd, write_file};

#[test]
fn test_wendao_search_rejects_legacy_sort_flag() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# A\n")?;

    let output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("search")
        .arg("a")
        .arg("--sort")
        .arg("path_asc")
        .output()?;

    assert!(
        !output.status.success(),
        "legacy --sort flag should be rejected, but command succeeded"
    );
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("unexpected argument '--sort'"));
    assert!(stderr.contains("--sort-term"));
    Ok(())
}
