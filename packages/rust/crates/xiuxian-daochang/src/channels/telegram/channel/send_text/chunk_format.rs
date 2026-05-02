use pulldown_cmark::{Event, Options, Parser, Tag};

use crate::channels::telegram::channel::chunking::{
    decorate_chunk_for_telegram, split_message_for_telegram,
};
use crate::channels::telegram::channel::constants::{
    TELEGRAM_MAX_AUTO_TEXT_CHUNKS, TELEGRAM_MAX_MESSAGE_LENGTH,
};
use crate::channels::telegram::channel::markdown::{
    markdown_to_telegram_html, markdown_to_telegram_markdown_v2,
};

pub(super) struct PreparedChunk {
    pub(super) plain_text: String,
    pub(super) markdown_v2_text: String,
    pub(super) markdown_chars: usize,
    pub(super) html_text: String,
    pub(super) html_chars: usize,
}

pub(super) fn should_prefer_html_chunk(
    plain_text: &str,
    markdown_chars: usize,
    html_chars: usize,
) -> bool {
    markdown_chars.saturating_sub(html_chars) >= 256 || markdown_ast_contains_images(plain_text)
}

pub(super) fn split_chunks_with_guard(message: &str) -> (Vec<String>, Option<(usize, usize)>) {
    let mut chunks = split_message_for_telegram(message);
    let truncated = if chunks.len() > TELEGRAM_MAX_AUTO_TEXT_CHUNKS {
        let total_chunks = chunks.len();
        chunks.truncate(TELEGRAM_MAX_AUTO_TEXT_CHUNKS);
        let kept_chunks = chunks.len();
        tracing::warn!(
            total_chunks,
            kept_chunks,
            "Telegram message exceeds auto-chunk guard; truncating output to prevent flood"
        );
        Some((total_chunks, kept_chunks))
    } else {
        None
    };
    (chunks, truncated)
}

pub(super) fn prepare_chunks(chunks: &[String]) -> Vec<PreparedChunk> {
    chunks
        .iter()
        .enumerate()
        .map(|(index, chunk)| {
            let plain_text = decorate_chunk_for_telegram(chunk, index, chunks.len());
            let markdown_v2_text = markdown_to_telegram_markdown_v2(&plain_text);
            let html_text = markdown_to_telegram_html(&plain_text);
            let markdown_chars = markdown_v2_text.chars().count();
            let html_chars = html_text.chars().count();
            PreparedChunk {
                plain_text,
                markdown_v2_text,
                markdown_chars,
                html_text,
                html_chars,
            }
        })
        .collect()
}

pub(super) fn warn_chunk_payload_fallbacks(prepared_chunks: &[PreparedChunk]) {
    let markdown_overflow_chunks = prepared_chunks
        .iter()
        .filter(|chunk| chunk.markdown_chars > TELEGRAM_MAX_MESSAGE_LENGTH)
        .count();
    let html_overflow_chunks = prepared_chunks
        .iter()
        .filter(|chunk| {
            chunk.markdown_chars > TELEGRAM_MAX_MESSAGE_LENGTH
                && chunk.html_chars > TELEGRAM_MAX_MESSAGE_LENGTH
        })
        .count();
    let prefer_html_chunks = prepared_chunks
        .iter()
        .filter(|chunk| {
            should_prefer_html_chunk(&chunk.plain_text, chunk.markdown_chars, chunk.html_chars)
        })
        .count();

    if markdown_overflow_chunks > 0 {
        tracing::warn!(
            chunks = prepared_chunks.len(),
            markdown_overflow_chunks,
            html_overflow_chunks,
            prefer_html_chunks,
            "Telegram MarkdownV2 payload exceeds limit for some chunks; using per-chunk fallback"
        );
    }
}

pub(super) fn assert_chunk_limit(chunk: &PreparedChunk, index: usize) {
    debug_assert!(
        chunk.plain_text.chars().count() <= TELEGRAM_MAX_MESSAGE_LENGTH,
        "chunk {} exceeds limit: {} > {}",
        index,
        chunk.plain_text.chars().count(),
        TELEGRAM_MAX_MESSAGE_LENGTH
    );
}

fn markdown_ast_contains_images(markdown: &str) -> bool {
    Parser::new_ext(markdown, Options::all())
        .any(|event| matches!(event, Event::Start(Tag::Image { .. })))
}
