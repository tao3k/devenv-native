//! Top-level integration harness for `config::xiuxian`.

mod config {
    mod tests {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/unit/config/loading.rs"
        ));
    }
}
