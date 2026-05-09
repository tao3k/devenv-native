use xiuxian_ast::{ChunkCodeRequest, CodeChunk, Lang, chunk_code};

pub(super) fn chunk_or_panic(
    content: &str,
    path: &str,
    lang: Lang,
    patterns: &[&str],
    min_lines: usize,
    max_lines: usize,
) -> Vec<CodeChunk> {
    match chunk_code(ChunkCodeRequest {
        content,
        file_path: path,
        lang,
        patterns,
        min_lines,
        max_lines,
    }) {
        Ok(chunks) => chunks,
        Err(error) => panic!("chunk_code failed: {error}"),
    }
}
