use include_dir::{Dir, File};
use std::path::Path;
use xiuxian_wendao_core::WendaoResourceUri;

use xiuxian_wendao_parsers::{parse_skill_frontmatter, uses_skill_frontmatter};

pub(crate) fn is_markdown_file(path: &str) -> bool {
    matches!(
        path.rsplit('.').next().map(str::to_ascii_lowercase),
        Some(ext) if ext == "md" || ext == "markdown"
    )
}

pub(crate) fn normalize_registry_key(path: &str) -> String {
    path.trim().trim_start_matches("./").replace('\\', "/")
}

pub(crate) fn is_wendao_uri(target: &str) -> bool {
    WendaoResourceUri::parse(target).is_ok()
}

pub(crate) fn collect_embedded_markdown_files<'a>(dir: &'a Dir<'a>, out: &mut Vec<&'a File<'a>>) {
    for file in dir.files() {
        let path = file.path().to_string_lossy().replace('\\', "/");
        if is_markdown_file(path.as_str()) {
            out.push(file);
        }
    }
    for child in dir.dirs() {
        collect_embedded_markdown_files(child, out);
    }
}

pub(crate) fn semantic_skill_name_from_descriptor(path: &str, markdown: &str) -> Option<String> {
    if !uses_skill_frontmatter(Some(Path::new(path)), markdown) {
        return None;
    }
    parse_skill_frontmatter(markdown)
        .ok()
        .and_then(|frontmatter| frontmatter.name)
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
}
