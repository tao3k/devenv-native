use std::process::Command;

use super::{init_test_repository, must, open_bare_with_retry, open_checkout_with_retry, temp_dir};

#[test]
fn repository_handle_tracks_working_tree_presence() {
    let bare_dir = temp_dir();
    must(
        Command::new("git")
            .arg("init")
            .arg("--bare")
            .arg(bare_dir.path())
            .status(),
        "initialize bare repository",
    );
    let bare = must(
        open_bare_with_retry(bare_dir.path()),
        "open bare repository handle",
    );
    assert!(bare.workdir().is_none());

    let checkout_dir = temp_dir();
    init_test_repository(checkout_dir.path());
    let checkout = must(
        open_checkout_with_retry(checkout_dir.path()),
        "open checkout repository handle",
    );
    assert!(checkout.workdir().is_some());
}
