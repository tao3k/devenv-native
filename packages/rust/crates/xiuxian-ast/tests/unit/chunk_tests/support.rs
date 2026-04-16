use super::super::*;

pub(super) fn chunk_or_panic(
    content: &str,
    path: &str,
    lang: Lang,
    patterns: &[&str],
    min_lines: usize,
    max_lines: usize,
) -> Vec<CodeChunk> {
    match chunk_code(content, path, lang, patterns, min_lines, max_lines) {
        Ok(chunks) => chunks,
        Err(error) => panic!("chunk_code failed: {error}"),
    }
}
