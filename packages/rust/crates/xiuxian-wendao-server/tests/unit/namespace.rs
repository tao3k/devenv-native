const SERVER_MANIFEST: &str = include_str!("../../Cargo.toml");

#[test]
fn server_manifest_owns_only_transport_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = SERVER_MANIFEST.parse::<toml::Value>()?;
    let Some(dependencies) = manifest.get("dependencies").and_then(toml::Value::as_table) else {
        return Err(std::io::Error::other("server Cargo.toml should define dependencies").into());
    };
    let actual = dependencies.keys().map(String::as_str).collect::<Vec<_>>();
    let expected = [
        "arrow-array",
        "arrow-flight",
        "arrow-schema",
        "async-trait",
        "base64",
        "futures",
        "serde",
        "serde_json",
        "tokio",
        "tokio-stream",
        "tonic",
    ];

    assert!(
        actual == expected,
        "xiuxian-wendao-server must own only transport dependencies; expected {expected:?}, got {actual:?}"
    );

    Ok(())
}
