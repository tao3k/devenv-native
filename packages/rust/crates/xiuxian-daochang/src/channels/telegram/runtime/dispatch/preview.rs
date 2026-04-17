const LOG_PREVIEW_LEN: usize = 120;
const LOG_PREVIEW_TAIL_LEN: usize = 24;
const LOG_PREVIEW_ANCHORS: [&str; 2] = ["Trigger", "Recall Result"];

fn strip_think_sections(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut cursor = input;

    loop {
        let Some(start) = cursor.find("<think>") else {
            out.push_str(cursor);
            break;
        };

        out.push_str(&cursor[..start]);
        let after_start = &cursor[start + "<think>".len()..];
        let Some(end) = after_start.find("</think>") else {
            break;
        };
        cursor = &after_start[end + "</think>".len()..];
    }

    out
}

pub(in crate::channels::telegram::runtime) fn log_preview(s: &str) -> String {
    let sanitized = strip_think_sections(s);
    let one_line: String = sanitized
        .chars()
        .map(|c| if c == '\n' { ' ' } else { c })
        .collect();
    let total_chars = one_line.chars().count();
    if total_chars <= LOG_PREVIEW_LEN {
        one_line
    } else {
        let head_len = LOG_PREVIEW_LEN.saturating_sub(LOG_PREVIEW_TAIL_LEN + 3);
        let head = one_line
            .chars()
            .take(head_len)
            .collect::<String>()
            .trim_end()
            .to_string();
        let tail = one_line
            .chars()
            .rev()
            .take(LOG_PREVIEW_TAIL_LEN)
            .collect::<Vec<char>>()
            .into_iter()
            .rev()
            .collect::<String>();
        let anchors = LOG_PREVIEW_ANCHORS
            .iter()
            .copied()
            .filter(|anchor| {
                one_line.contains(anchor) && !head.contains(anchor) && !tail.contains(anchor)
            })
            .collect::<Vec<_>>()
            .join(" | ");
        if anchors.is_empty() {
            format!("{head}...{tail}")
        } else {
            format!("{head}...{anchors}...{tail}")
        }
    }
}

pub(in crate::channels::telegram::runtime) fn sanitize_reply_for_send(s: &str) -> String {
    strip_think_sections(s).trim().to_string()
}

#[cfg(test)]
#[path = "../../../../../tests/unit/channels/telegram/runtime/dispatch/preview.rs"]
mod tests;
