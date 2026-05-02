use std::fs::File;
use std::io::Write;
use std::time::Instant;

use flate2::Compression;
use flate2::write::GzEncoder;
use tar::{Builder, Header};

use super::{audit_archive_attachment, is_supported_archive_path};

#[test]
fn archive_audit_reads_mets_gbs_member_manifest() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let archive_path = temp_dir.path().join("fixture.tar.gz");
    write_tar_gz_fixture(
        archive_path.as_path(),
        &[
            ("book/mets.xml", b"<mets/>".as_slice()),
            ("book/images/page0001.jp2", b"jp2-image".as_slice()),
            ("book/images/page0002.tif", b"tiff-image".as_slice()),
            ("book/readme.txt", b"readme".as_slice()),
        ],
    )?;

    let audit = audit_archive_attachment(archive_path.as_path())?;

    assert_eq!(audit.archive_format, "tar.gz");
    assert_eq!(audit.mime_type, "application/gzip");
    assert_eq!(audit.member_count, 4);
    assert_eq!(audit.regular_file_count, 4);
    assert_eq!(audit.directory_count, 0);
    assert_eq!(audit.xml_member_count, 1);
    assert_eq!(audit.image_member_count, 2);
    assert_eq!(audit.extension_counts.get("xml").copied(), Some(1));
    assert_eq!(audit.extension_counts.get("jp2").copied(), Some(1));
    assert_eq!(audit.extension_counts.get("tif").copied(), Some(1));
    assert_eq!(audit.extension_counts.get("txt").copied(), Some(1));
    assert_eq!(
        audit.likely_mets_member_path.as_deref(),
        Some("book/mets.xml")
    );
    assert_eq!(
        audit.rust_acceleration_candidate,
        "mets_gbs_member_manifest_candidate"
    );
    assert!(is_supported_archive_path(archive_path.as_path()));
    Ok(())
}

#[test]
fn archive_audit_marks_unsupported_non_archive() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source_path = temp_dir.path().join("fixture.bin");
    std::fs::write(source_path.as_path(), b"not an archive").map_err(|error| error.to_string())?;

    let audit = audit_archive_attachment(source_path.as_path())?;

    assert_eq!(audit.archive_format, "unsupported");
    assert_eq!(audit.mime_type, "application/octet-stream");
    assert_eq!(audit.member_count, 0);
    assert_eq!(audit.rust_acceleration_candidate, "unsupported_non_archive");
    assert!(!is_supported_archive_path(source_path.as_path()));
    Ok(())
}

#[test]
#[ignore = "requires WENDAO_ARCHIVE_AUDIT_SOURCE"]
fn archive_audit_real_fixture_from_env() -> Result<(), String> {
    let source = std::env::var("WENDAO_ARCHIVE_AUDIT_SOURCE")
        .map_err(|_| "WENDAO_ARCHIVE_AUDIT_SOURCE is required".to_owned())?;
    let started = Instant::now();
    let audit = audit_archive_attachment(source)?;
    let report = serde_json::json!({
        "schema": "xiuxian_wendao.archive_audit_probe.v1",
        "elapsedMs": started.elapsed().as_secs_f64() * 1000.0,
        "audit": audit,
    });
    let report = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    println!("{report}");
    Ok(())
}

fn write_tar_gz_fixture(path: &std::path::Path, files: &[(&str, &[u8])]) -> Result<(), String> {
    let file = File::create(path).map_err(|error| error.to_string())?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(encoder);
    for (member_path, bytes) in files {
        append_regular_file(&mut builder, member_path, bytes)?;
    }
    let encoder = builder.into_inner().map_err(|error| error.to_string())?;
    encoder.finish().map_err(|error| error.to_string())?;
    Ok(())
}

fn append_regular_file<W: Write>(
    builder: &mut Builder<W>,
    member_path: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let mut header = Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, member_path, bytes)
        .map_err(|error| error.to_string())
}
