#[test]
fn handlers_surface_has_no_legacy_export_barrels() {
    let handlers_dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/studio/router/handlers");
    let entries = std::fs::read_dir(&handlers_dir)
        .unwrap_or_else(|error| panic!("read handlers dir {}: {error}", handlers_dir.display()));

    let mut legacy_barrels = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(std::ffi::OsStr::to_str) == Some("rs"))
        .filter_map(|path| {
            path.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .map(str::to_string)
        })
        .filter(|name| name.ends_with("_exports.rs"))
        .collect::<Vec<_>>();
    legacy_barrels.sort();

    assert!(
        legacy_barrels.is_empty(),
        "legacy handler export barrels should stay removed: {legacy_barrels:?}"
    );
}
