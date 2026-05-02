//! Embedded resource tree exposed by the crate facade.

/// Compile-time embedded resource tree rooted at `omni-agent/resources`.
pub static RESOURCES: ::include_dir::Dir<'_> =
    ::include_dir::include_dir!("$CARGO_MANIFEST_DIR/resources");
