//! Telegram message chunking and caption preparation.

use super::constants::{
    CHUNK_CONTINUED_PREFIX, CHUNK_CONTINUES_SUFFIX, TELEGRAM_MAX_MESSAGE_LENGTH,
};
use pulldown_cmark::{Event, Options, Parser, TagEnd};

/// Split a message into chunks that respect Telegram's 4096 character limit.
/// Each chunk stays under the limit even after adding continuation markers.
#[doc(hidden)]
#[must_use]
pub fn split_message_for_telegram(message: &str) -> Vec<String> {
    let max_chunk_chars = TELEGRAM_MAX_MESSAGE_LENGTH.saturating_sub(chunk_marker_reserve_chars());
    if max_chunk_chars == 0 {
        return vec![message.to_string()];
    }

    if byte_index_after_n_chars(message, TELEGRAM_MAX_MESSAGE_LENGTH) == message.len() {
        return vec![message.to_string()];
    }

    if let Some(ast_chunks) = split_message_with_markdown_block_boundaries(message, max_chunk_chars)
    {
        return ast_chunks;
    }

    let mut chunks = Vec::new();
    let mut remaining = message;
    while !remaining.is_empty() {
        let max_chunk_boundary = byte_index_after_n_chars(remaining, max_chunk_chars);
        if max_chunk_boundary == remaining.len() {
            chunks.push(remaining.to_string());
            break;
        }

        let search_area = &remaining[..max_chunk_boundary];
        let chunk_end = choose_split_boundary(search_area, max_chunk_boundary, max_chunk_chars);
        chunks.push(remaining[..chunk_end].to_string());
        remaining = &remaining[chunk_end..];
    }
    chunks
}

fn split_message_with_markdown_block_boundaries(
    message: &str,
    max_chunk_chars: usize,
) -> Option<Vec<String>> {
    if !looks_like_markdown(message) {
        return None;
    }

    let mut boundaries = markdown_block_boundaries(message);
    boundaries.retain(|index| *index > 0 && *index < message.len());
    boundaries.sort_unstable();
    boundaries.dedup();
    if boundaries.is_empty() {
        return None;
    }

    let mut chunks = Vec::new();
    let mut start = 0usize;
    while start < message.len() {
        let relative_max_boundary = byte_index_after_n_chars(&message[start..], max_chunk_chars);
        let max_boundary = start + relative_max_boundary;
        if max_boundary >= message.len() {
            chunks.push(message[start..].to_string());
            break;
        }

        let chunk_end = boundaries
            .iter()
            .copied()
            .take_while(|boundary| *boundary <= max_boundary)
            .last()
            .filter(|boundary| *boundary > start)
            .unwrap_or_else(|| {
                let search_area = &message[start..max_boundary];
                start + choose_split_boundary(search_area, relative_max_boundary, max_chunk_chars)
            });

        if chunk_end <= start {
            return None;
        }

        chunks.push(message[start..chunk_end].to_string());
        start = chunk_end;
    }

    Some(chunks)
}

fn looks_like_markdown(message: &str) -> bool {
    message.contains('\n')
        || message.contains("```")
        || message.contains("# ")
        || message.contains("](")
        || message.contains("* ")
}

fn markdown_block_boundaries(message: &str) -> Vec<usize> {
    let parser = Parser::new_ext(message, Options::all()).into_offset_iter();
    let mut boundaries = Vec::new();
    for (event, range) in parser {
        let is_block_boundary = matches!(
            event,
            Event::End(
                TagEnd::Paragraph
                    | TagEnd::Heading(_)
                    | TagEnd::CodeBlock
                    | TagEnd::BlockQuote(_)
                    | TagEnd::List(_)
                    | TagEnd::Item
            ) | Event::Rule
        );
        if is_block_boundary {
            boundaries.push(range.end);
        }
    }
    boundaries
}

/// Reserve space for continuation markers when splitting.
/// Keep this as character-count based so future non-ASCII marker text remains safe.
#[doc(hidden)]
#[must_use]
pub fn chunk_marker_reserve_chars() -> usize {
    CHUNK_CONTINUED_PREFIX.chars().count() + CHUNK_CONTINUES_SUFFIX.chars().count()
}

#[doc(hidden)]
#[must_use]
pub fn decorate_chunk_for_telegram(chunk: &str, index: usize, total_chunks: usize) -> String {
    if total_chunks <= 1 {
        return chunk.to_string();
    }

    if index == 0 {
        format!("{chunk}{CHUNK_CONTINUES_SUFFIX}")
    } else if index == total_chunks - 1 {
        format!("{CHUNK_CONTINUED_PREFIX}{chunk}")
    } else {
        format!("{CHUNK_CONTINUED_PREFIX}{chunk}{CHUNK_CONTINUES_SUFFIX}")
    }
}

fn byte_index_after_n_chars(text: &str, n: usize) -> usize {
    if n == 0 {
        return 0;
    }

    text.char_indices()
        .nth(n)
        .map_or(text.len(), |(idx, _)| idx)
}

fn choose_split_boundary(search_area: &str, max_boundary: usize, max_chars: usize) -> usize {
    if let Some(pos) = search_area.rfind('\n') {
        let newline_char_pos = search_area[..pos].chars().count();
        if newline_char_pos >= max_chars / 2 {
            return pos + 1;
        }
    }

    if let Some(pos) = search_area.rfind(' ') {
        return pos + 1;
    }

    max_boundary
}
