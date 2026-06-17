use std::collections::HashSet;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct Lang(&'static str);

impl Lang {
    pub(super) fn as_str(self) -> &'static str {
        self.0
    }
}

impl TryFrom<&str> for Lang {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        source_language_id_from_identifier(value)
            .map(Self)
            .ok_or(())
    }
}

pub(super) fn code_language_from_path(path: &Path) -> Option<Lang> {
    source_language_id_from_path(path).map(Lang)
}

pub(super) fn normalize_code_language_identifier(identifier: &str) -> Option<Lang> {
    source_language_id_from_identifier(identifier).map(Lang)
}

pub(super) fn supported_ast_lang(
    path: &Path,
    excluded_languages: &HashSet<String>,
) -> Option<Lang> {
    let lang = code_language_from_path(path)?;
    (!excluded_languages.contains(lang.as_str())).then_some(lang)
}

fn source_language_id_from_path(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(std::ffi::OsStr::to_str) {
        Some(ext) if ext.eq_ignore_ascii_case("rs") => Some("rust"),
        Some(ext) if ext.eq_ignore_ascii_case("py") => Some("python"),
        Some(ext) if ext.eq_ignore_ascii_case("ts") || ext.eq_ignore_ascii_case("tsx") => {
            Some("typescript")
        }
        Some(ext) if ext.eq_ignore_ascii_case("js") || ext.eq_ignore_ascii_case("jsx") => {
            Some("javascript")
        }
        Some(ext) if ext.eq_ignore_ascii_case("jl") => Some("julia"),
        Some(ext) if ext.eq_ignore_ascii_case("mo") => Some("modelica"),
        Some(ext) if ext.eq_ignore_ascii_case("sql") => Some("sql"),
        Some(ext) if ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml") => {
            Some("yaml")
        }
        Some(ext) if ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown") => {
            Some("markdown")
        }
        Some(ext) if ext.eq_ignore_ascii_case("toml") => Some("toml"),
        _ => None,
    }
}

fn source_language_id_from_identifier(identifier: &str) -> Option<&'static str> {
    match identifier.trim().to_ascii_lowercase().as_str() {
        "rust" | "rs" => Some("rust"),
        "python" | "py" => Some("python"),
        "typescript" | "ts" | "tsx" => Some("typescript"),
        "javascript" | "js" | "jsx" => Some("javascript"),
        "julia" | "jl" | "julia-code-parser" => Some("julia"),
        "modelica" | "mo" => Some("modelica"),
        "sql" => Some("sql"),
        "yaml" | "yml" => Some("yaml"),
        "markdown" | "md" => Some("markdown"),
        "toml" => Some("toml"),
        _ => None,
    }
}
