use super::{pack_artifact_directory, unpack_artifact_directory};

#[test]
fn artifact_directory_bundle_roundtrips_regular_files() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    std::fs::create_dir_all(source.join("resources"))?;
    std::fs::write(source.join("_resources.arrow"), b"arrow")?;
    std::fs::write(source.join("resources").join("page.md"), b"# Page\n")?;

    let bytes = pack_artifact_directory(source.as_path())?;
    unpack_artifact_directory(bytes.as_slice(), target.as_path())?;

    assert_eq!(std::fs::read(target.join("_resources.arrow"))?, b"arrow");
    assert_eq!(
        std::fs::read(target.join("resources").join("page.md"))?,
        b"# Page\n"
    );
    Ok(())
}

#[test]
#[cfg(unix)]
fn artifact_directory_bundle_rejects_symlinks() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    let source = temp.path().join("source");
    std::fs::create_dir_all(source.as_path())?;
    std::os::unix::fs::symlink("/tmp/escape", source.join("escape"))?;

    let Err(error) = pack_artifact_directory(source.as_path()) else {
        return Err(std::io::Error::other("symlink bundle should fail").into());
    };

    assert!(error.to_string().contains("symlink"));
    Ok(())
}

#[test]
fn artifact_directory_bundle_rejects_path_traversal_entries()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    let bytes = tar_bytes_with_raw_path("../escape.txt", b"escape");

    let Err(error) = unpack_artifact_directory(bytes.as_slice(), temp.path()) else {
        return Err(std::io::Error::other("path traversal bundle should fail").into());
    };

    assert!(error.to_string().contains("must be relative"));
    Ok(())
}

fn tar_bytes_with_raw_path(path: &str, payload: &[u8]) -> Vec<u8> {
    let mut header = [0_u8; 512];
    let path_bytes = path.as_bytes();
    header[..path_bytes.len()].copy_from_slice(path_bytes);
    write_tar_octal(&mut header[100..108], 0o644);
    write_tar_octal(&mut header[108..116], 0);
    write_tar_octal(&mut header[116..124], 0);
    write_tar_octal(&mut header[124..136], payload.len() as u64);
    write_tar_octal(&mut header[136..148], 0);
    header[148..156].fill(b' ');
    header[156] = b'0';
    header[257..263].copy_from_slice(b"ustar\0");
    header[263..265].copy_from_slice(b"00");
    let checksum = header.iter().map(|byte| u32::from(*byte)).sum::<u32>();
    let checksum_text = format!("{checksum:06o}\0 ");
    header[148..156].copy_from_slice(checksum_text.as_bytes());
    let mut bytes = header.to_vec();
    bytes.extend_from_slice(payload);
    let padding = (512 - (payload.len() % 512)) % 512;
    bytes.extend(std::iter::repeat_n(0, padding));
    bytes.extend([0_u8; 1024]);
    bytes
}

fn write_tar_octal(target: &mut [u8], value: u64) {
    let text = format!("{value:0width$o}\0", width = target.len() - 1);
    target.copy_from_slice(text.as_bytes());
}
