use super::support::{
    EpistemeFixture, EpistemeRegistryEntry, LoadedEpistemeSourceKind, Path, PathBuf,
    cleanup_managed_git_entry, init_git_repository, load_episteme_registry_entries,
};

#[test]
fn episteme_registry_loads_local_path_entry() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = EpistemeFixture::new()?;
    fixture.write_contract()?;

    let receipt = load_episteme_registry_entries(
        &[EpistemeRegistryEntry::local(
            "source_contract",
            fixture.episteme_root.clone(),
        )],
        Path::new("."),
    )?;

    assert_eq!(receipt.loaded_count, 1);
    assert_eq!(receipt.entries[0].id, "source_contract");
    assert_eq!(
        receipt.entries[0].source_kind,
        LoadedEpistemeSourceKind::Local
    );
    assert_eq!(receipt.entries[0].episteme_root, fixture.episteme_root);
    assert_eq!(receipt.entries[0].subdir, ".");
    assert!(receipt.entries[0].resolved_revision.is_none());
    Ok(())
}

#[test]
fn episteme_registry_filters_disabled_entries() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = load_episteme_registry_entries(
        &[EpistemeRegistryEntry {
            id: "disabled_entry".to_string(),
            path: None,
            url: None,
            enabled: false,
            subdir: PathBuf::from("."),
        }],
        Path::new("."),
    )?;

    assert_eq!(receipt.loaded_count, 0);
    assert!(receipt.entries.is_empty());
    Ok(())
}

#[test]
fn episteme_registry_rejects_mixed_path_and_url() {
    let result = load_episteme_registry_entries(
        &[EpistemeRegistryEntry {
            id: "mixed".to_string(),
            path: Some(PathBuf::from(".")),
            url: Some("https://github.com/example/example-episteme.git".to_string()),
            enabled: true,
            subdir: PathBuf::from("."),
        }],
        Path::new("."),
    );
    let Err(error) = result else {
        panic!("mixed path/url entry should fail");
    };

    assert!(error.to_string().contains("exactly one of `path` or `url`"));
}

#[test]
fn episteme_registry_rejects_unsafe_subdir() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = EpistemeFixture::new()?;
    fixture.write_contract()?;

    let result = load_episteme_registry_entries(
        &[EpistemeRegistryEntry::local("unsafe", fixture.episteme_root).with_subdir("../escape")],
        Path::new("."),
    );
    let Err(error) = result else {
        panic!("unsafe subdir should fail");
    };

    assert!(error.to_string().contains("unsafe subdir"));
    Ok(())
}

#[test]
fn episteme_registry_materializes_git_url_entry() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = EpistemeFixture::new()?;
    fixture.write_contract()?;
    init_git_repository(fixture.episteme_root.as_path())?;
    let url = fixture.episteme_root.display().to_string();

    let receipt = load_episteme_registry_entries(
        &[EpistemeRegistryEntry::git("remote_source", url.clone())],
        Path::new("."),
    )?;

    assert_eq!(receipt.loaded_count, 1);
    assert_eq!(receipt.entries[0].id, "remote_source");
    assert_eq!(
        receipt.entries[0].source_kind,
        LoadedEpistemeSourceKind::Git
    );
    assert_eq!(receipt.entries[0].url.as_deref(), Some(url.as_str()));
    assert!(
        receipt.entries[0]
            .episteme_root
            .join("ontology/manifest.toml")
            .is_file()
    );
    assert!(receipt.entries[0].resolved_revision.is_some());

    cleanup_managed_git_entry("remote_source", url.as_str())?;
    Ok(())
}
