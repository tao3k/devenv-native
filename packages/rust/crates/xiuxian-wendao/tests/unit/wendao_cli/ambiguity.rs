use serde_json::Value;
use tempfile::TempDir;

use super::{wendao_cmd, write_file};

#[test]
fn test_wendao_attachments_search_filters_by_ext_and_kind() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "# Alpha\n\n[Paper](files/paper.pdf)\n![Diagram](assets/diagram.png)\n[Key](security/signing.gpg)\n",
    )?;
    write_file(
        &tmp.path().join("docs/b.md"),
        "# Beta\n\n![Photo](assets/photo.jpg)\n",
    )?;

    let image_output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("attachments")
        .arg("--kind")
        .arg("image")
        .arg("--limit")
        .arg("10")
        .output()?;
    assert!(
        image_output.status.success(),
        "wendao attachments --kind image failed: {}",
        String::from_utf8_lossy(&image_output.stderr)
    );
    let image_payload: Value = serde_json::from_str(&String::from_utf8(image_output.stdout)?)?;
    let image_hits = image_payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("missing image attachment hits")?;
    assert!(
        image_hits
            .iter()
            .all(|row| row.get("kind").and_then(Value::as_str) == Some("image"))
    );
    assert!(
        image_hits
            .iter()
            .any(|row| { row.get("attachment_ext").and_then(Value::as_str) == Some("png") })
    );
    assert!(
        image_hits
            .iter()
            .any(|row| { row.get("attachment_ext").and_then(Value::as_str) == Some("jpg") })
    );

    let pdf_output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("attachments")
        .arg("--ext")
        .arg("pdf")
        .arg("--limit")
        .arg("10")
        .output()?;
    assert!(
        pdf_output.status.success(),
        "wendao attachments --ext pdf failed: {}",
        String::from_utf8_lossy(&pdf_output.stderr)
    );
    let pdf_payload: Value = serde_json::from_str(&String::from_utf8(pdf_output.stdout)?)?;
    let pdf_hits = pdf_payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("missing pdf attachment hits")?;
    assert_eq!(pdf_hits.len(), 1);
    assert_eq!(
        pdf_hits[0].get("attachment_ext").and_then(Value::as_str),
        Some("pdf")
    );
    assert_eq!(pdf_hits[0].get("kind").and_then(Value::as_str), Some("pdf"));
    Ok(())
}

#[test]
fn test_wendao_attachments_search_normalizes_file_scheme_targets()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "# Alpha\n\n[Absolute](/tmp/manual.pdf)\n[FileUri](file:///tmp/manual-2.pdf)\n",
    )?;

    let output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("attachments")
        .arg("--ext")
        .arg("pdf")
        .arg("--limit")
        .arg("10")
        .output()?;
    assert!(
        output.status.success(),
        "wendao attachments file targets failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: Value = serde_json::from_str(&String::from_utf8(output.stdout)?)?;
    let hits = payload
        .get("hits")
        .and_then(Value::as_array)
        .ok_or("missing attachment hits")?;
    assert!(hits.iter().any(|row| {
        row.get("attachment_path").and_then(Value::as_str) == Some("/tmp/manual.pdf")
    }));
    assert!(hits.iter().any(|row| {
        row.get("attachment_path").and_then(Value::as_str) == Some("/tmp/manual-2.pdf")
    }));
    Ok(())
}
