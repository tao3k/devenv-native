use super::resolve_resource_file;

#[tokio::test]
async fn resolve_resource_file_accepts_paths_inside_extraction_root() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let resource = temp.path().join("image.png");
    std::fs::write(resource.as_path(), b"image")
        .unwrap_or_else(|error| panic!("write resource: {error}"));

    let resolved = resolve_resource_file(temp.path(), "image.png")
        .await
        .unwrap_or_else(|error| panic!("resource should resolve: {error:?}"));

    assert_eq!(
        resolved,
        resource
            .canonicalize()
            .unwrap_or_else(|error| panic!("canonicalize resource: {error}"))
    );
}

#[tokio::test]
async fn resolve_resource_file_rejects_paths_outside_extraction_root() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
    let outside = temp.path().join("outside.txt");
    let root = temp.path().join("extract");
    std::fs::create_dir_all(root.as_path()).unwrap_or_else(|error| panic!("create root: {error}"));
    std::fs::write(outside.as_path(), b"secret")
        .unwrap_or_else(|error| panic!("write outside: {error}"));

    let Err(error) =
        resolve_resource_file(root.as_path(), outside.to_string_lossy().as_ref()).await
    else {
        panic!("outside resource path should be rejected");
    };

    assert!(format!("{error:?}").contains("RESOURCE_OUTSIDE_EXTRACTION_ROOT"));
}
