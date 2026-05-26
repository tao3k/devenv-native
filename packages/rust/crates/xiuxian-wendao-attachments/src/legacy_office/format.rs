use std::path::Path;

/// Legacy Office formats handled without LibreOffice or Java.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyOfficeFormat {
    /// Word 97-2003 binary document.
    Doc,
    /// Excel 97-2003 binary workbook.
    Xls,
    /// PowerPoint 97-2003 binary presentation.
    Ppt,
}

impl LegacyOfficeFormat {
    /// Returns the extension associated with this legacy Office format.
    #[must_use]
    pub fn extension(self) -> &'static str {
        match self {
            Self::Doc => "doc",
            Self::Xls => "xls",
            Self::Ppt => "ppt",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Doc => "Word 97-2003 document",
            Self::Xls => "Excel 97-2003 workbook",
            Self::Ppt => "PowerPoint 97-2003 presentation",
        }
    }
}

/// Returns whether the path is a legacy Office source handled by this module.
#[must_use]
pub fn is_supported_legacy_office_path(path: &Path) -> bool {
    legacy_office_format(path).is_some()
}

/// Detects a legacy Office format from the source path extension.
#[must_use]
pub fn legacy_office_format(path: &Path) -> Option<LegacyOfficeFormat> {
    match path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("doc") => Some(LegacyOfficeFormat::Doc),
        Some("xls") => Some(LegacyOfficeFormat::Xls),
        Some("ppt") => Some(LegacyOfficeFormat::Ppt),
        _ => None,
    }
}
