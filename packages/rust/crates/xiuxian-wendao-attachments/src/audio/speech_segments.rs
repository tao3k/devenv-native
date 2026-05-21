//! Model-neutral speech timestamp sidecar parsing.

use serde_json::Value;

use super::types::AudioSpeechSegment;

/// Parse speech timestamp facts from JSONL or a JSON array.
///
/// Each row may use millisecond fields (`startMs`, `durationMs`, `endMs`) or
/// second fields (`startSeconds`, `durationSeconds`, `endSeconds`). The parser
/// returns stable timeline-ordered `AudioSpeechSegment` values and does not
/// attach any model or ASR identity.
///
/// # Errors
///
/// Returns an error when the sidecar is empty, malformed, or contains invalid
/// timing values.
pub fn parse_audio_speech_segments_sidecar(input: &str) -> Result<Vec<AudioSpeechSegment>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("audio speech segment sidecar is empty".to_owned());
    }
    let values = if trimmed.starts_with('[') {
        let Value::Array(values) = serde_json::from_str::<Value>(trimmed)
            .map_err(|error| format!("invalid audio speech segment JSON array: {error}"))?
        else {
            return Err("audio speech segment sidecar must be a JSON array or JSONL".to_owned());
        };
        values
    } else {
        trimmed
            .lines()
            .enumerate()
            .filter_map(|(line_index, line)| {
                let line = line.trim();
                (!line.is_empty()).then_some((line_index + 1, line))
            })
            .map(|(line_number, line)| {
                serde_json::from_str::<Value>(line).map_err(|error| {
                    format!("invalid audio speech segment JSONL line {line_number}: {error}")
                })
            })
            .collect::<Result<Vec<_>, _>>()?
    };
    parse_audio_speech_segment_values(values.as_slice())
}

fn parse_audio_speech_segment_values(values: &[Value]) -> Result<Vec<AudioSpeechSegment>, String> {
    if values.is_empty() {
        return Err("audio speech segment sidecar has no segments".to_owned());
    }
    let mut segments = values
        .iter()
        .enumerate()
        .map(|(offset, value)| parse_audio_speech_segment_value(offset, value))
        .collect::<Result<Vec<_>, _>>()?;
    segments.sort_by_key(|segment| (segment.start_ms, segment.index));
    for (index, segment) in segments.iter_mut().enumerate() {
        segment.index = u32::try_from(index)
            .map_err(|_| "audio speech segment index exceeds u32::MAX".to_owned())?;
    }
    Ok(segments)
}

fn parse_audio_speech_segment_value(
    offset: usize,
    value: &Value,
) -> Result<AudioSpeechSegment, String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("audio speech segment {offset} must be a JSON object"))?;
    let start_ms = millis_field(object, "startMs")
        .or_else(|| seconds_field(object, "startSeconds"))
        .ok_or_else(|| format!("audio speech segment {offset} missing start time"))??;
    let duration_ms = if let Some(duration_ms) =
        millis_field(object, "durationMs").or_else(|| seconds_field(object, "durationSeconds"))
    {
        duration_ms?
    } else {
        let end_ms = millis_field(object, "endMs")
            .or_else(|| seconds_field(object, "endSeconds"))
            .ok_or_else(|| format!("audio speech segment {offset} missing duration or end"))??;
        end_ms
            .checked_sub(start_ms)
            .ok_or_else(|| format!("audio speech segment {offset} end time is before start time"))?
    };
    if duration_ms == 0 {
        return Err(format!(
            "audio speech segment {offset} duration must be positive"
        ));
    }
    Ok(AudioSpeechSegment {
        index: u32::try_from(offset)
            .map_err(|_| "audio speech segment index exceeds u32::MAX".to_owned())?,
        start_ms,
        duration_ms,
    })
}

fn millis_field(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
) -> Option<Result<u64, String>> {
    object.get(key).map(|value| {
        value
            .as_u64()
            .ok_or_else(|| format!("{key} must be a non-negative integer millisecond value"))
    })
}

fn seconds_field(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
) -> Option<Result<u64, String>> {
    object.get(key).map(|value| {
        let seconds = value
            .as_f64()
            .ok_or_else(|| format!("{key} must be a non-negative second value"))?;
        if !seconds.is_finite() || seconds < 0.0 {
            return Err(format!("{key} must be a finite non-negative second value"));
        }
        let duration = std::time::Duration::try_from_secs_f64(seconds + 0.000_5)
            .map_err(|_| format!("{key} must fit in u64 milliseconds"))?;
        u64::try_from(duration.as_millis())
            .map_err(|_| format!("{key} must fit in u64 milliseconds"))
    })
}
