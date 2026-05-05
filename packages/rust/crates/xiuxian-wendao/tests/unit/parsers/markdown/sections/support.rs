use crate::parsers::markdown::sections::ParsedSection;
use std::path::Path;

pub(super) fn extract_sections_from(body: &str) -> Vec<ParsedSection> {
    crate::parsers::markdown::sections::extract_sections(body, Path::new("test.md"), Path::new("/"))
}
