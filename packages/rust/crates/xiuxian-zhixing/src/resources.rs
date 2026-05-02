//! Embedded resource tree for Xiuxian-Zhixing package assets.

/// Compile-time embedded resource tree rooted at `xiuxian-zhixing/resources`.
pub static RESOURCES: ::include_dir::Dir<'_> =
    ::include_dir::include_dir!("$CARGO_MANIFEST_DIR/resources");
