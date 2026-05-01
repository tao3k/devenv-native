use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ArchiveFormat {
    pub(crate) name: &'static str,
    pub(crate) mime_type: &'static str,
}

const TAR: ArchiveFormat = ArchiveFormat {
    name: "tar",
    mime_type: "application/x-tar",
};

const TAR_GZ: ArchiveFormat = ArchiveFormat {
    name: "tar.gz",
    mime_type: "application/gzip",
};

pub(crate) fn detect_archive_format(path: &Path) -> Option<ArchiveFormat> {
    if has_extension(path, "tgz") || has_compound_extension(path, "tar", "gz") {
        return Some(TAR_GZ);
    }
    if has_extension(path, "tar") {
        return Some(TAR);
    }
    None
}

pub(crate) fn unsupported_format() -> ArchiveFormat {
    ArchiveFormat {
        name: "unsupported",
        mime_type: "application/octet-stream",
    }
}

fn has_extension(path: &Path, expected: &str) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(expected))
}

fn has_compound_extension(path: &Path, stem_extension: &str, extension: &str) -> bool {
    has_extension(path, extension)
        && path
            .file_stem()
            .map(Path::new)
            .is_some_and(|stem| has_extension(stem, stem_extension))
}
