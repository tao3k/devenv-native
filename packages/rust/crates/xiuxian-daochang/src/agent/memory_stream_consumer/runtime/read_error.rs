use crate::agent::memory_stream_consumer::types::StreamReadErrorKind;

pub(in crate::agent::memory_stream_consumer) fn classify_stream_read_error(
    error: &anyhow::Error,
) -> StreamReadErrorKind {
    let message = error_chain_message(error).to_ascii_uppercase();
    if message.contains("NOGROUP") {
        return StreamReadErrorKind::MissingConsumerGroup;
    }
    if [
        "CONNECTION",
        "BROKEN PIPE",
        "RESET BY PEER",
        "TIMED OUT",
        "TIMEOUT",
        "IO ERROR",
        "SOCKET",
        "EOF",
    ]
    .iter()
    .any(|needle| message.contains(needle))
    {
        return StreamReadErrorKind::Transport;
    }
    StreamReadErrorKind::Other
}

fn error_chain_message(error: &anyhow::Error) -> String {
    let mut parts = Vec::new();
    for cause in error.chain() {
        let cause_text = cause.to_string();
        if cause_text.is_empty() {
            continue;
        }
        parts.push(cause_text);
    }
    if parts.is_empty() {
        error.to_string()
    } else {
        parts.join(": ")
    }
}
