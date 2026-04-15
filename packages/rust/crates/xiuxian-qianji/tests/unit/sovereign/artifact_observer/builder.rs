use super::*;

#[test]
fn builder_creates_default_observer() {
    let observer = ArtifactObserverBuilder::new().build_noop();
    assert!(observer.config().enabled);
}

#[test]
fn builder_disabled() {
    let observer = ArtifactObserverBuilder::new().enabled(false).build_noop();
    assert!(!observer.config().enabled);
}

#[test]
fn builder_custom_trace_path() {
    let observer = ArtifactObserverBuilder::new()
        .trace_base_path("custom/traces")
        .build_noop();
    assert_eq!(observer.config().trace_base_path, "custom/traces");
}

#[test]
fn builder_ingest_on_exit_false() {
    let observer = ArtifactObserverBuilder::new()
        .ingest_on_exit(false)
        .build_noop();
    assert!(!observer.config().ingest_on_exit);
}

#[test]
fn builder_ingest_on_early_halt_false() {
    let observer = ArtifactObserverBuilder::new()
        .ingest_on_early_halt(false)
        .build_noop();
    assert!(!observer.config().ingest_on_early_halt);
}

#[test]
fn builder_chained_config() {
    let observer = ArtifactObserverBuilder::new()
        .enabled(false)
        .trace_base_path("my/path")
        .ingest_on_exit(false)
        .ingest_on_early_halt(false)
        .build_noop();
    let config = observer.config();
    assert!(!config.enabled);
    assert_eq!(config.trace_base_path, "my/path");
    assert!(!config.ingest_on_exit);
    assert!(!config.ingest_on_early_halt);
}
