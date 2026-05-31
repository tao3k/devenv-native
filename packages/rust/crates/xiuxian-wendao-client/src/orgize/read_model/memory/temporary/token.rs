use std::collections::HashSet;

pub(super) fn normalized_text(value: &str) -> String {
    normalized_words(value).join(" ")
}

pub(super) fn normalized_words(value: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut ascii_run = String::new();
    let mut cjk_run = Vec::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            flush_cjk_run(&mut words, &mut cjk_run);
            ascii_run.push(character);
        } else if is_cjk_character(character) {
            flush_ascii_run(&mut words, &mut ascii_run);
            cjk_run.push(character);
        } else {
            flush_ascii_run(&mut words, &mut ascii_run);
            flush_cjk_run(&mut words, &mut cjk_run);
        }
    }
    flush_ascii_run(&mut words, &mut ascii_run);
    flush_cjk_run(&mut words, &mut cjk_run);
    words.into_iter().filter(|word| word.len() > 1).collect()
}

pub(super) fn flush_ascii_run(words: &mut Vec<String>, ascii_run: &mut String) {
    if !ascii_run.is_empty() {
        let run = std::mem::take(ascii_run);
        push_unique_word(words, run.to_ascii_lowercase());
        for segment in ascii_semantic_segments(run.as_str()) {
            push_unique_word(words, segment);
        }
    }
}

pub(super) fn ascii_semantic_segments(value: &str) -> Vec<String> {
    let characters = value.chars().collect::<Vec<_>>();
    let mut segments = Vec::new();
    let mut current = String::new();

    for (index, character) in characters.iter().copied().enumerate() {
        if !current.is_empty() && ascii_segment_boundary(&characters, index) {
            push_unique_word(&mut segments, current.to_ascii_lowercase());
            current.clear();
        }
        current.push(character);
    }
    if !current.is_empty() {
        push_unique_word(&mut segments, current.to_ascii_lowercase());
    }
    segments
}

pub(super) fn ascii_segment_boundary(characters: &[char], index: usize) -> bool {
    if index == 0 {
        return false;
    }
    let previous = characters[index - 1];
    let current = characters[index];
    let next = characters.get(index + 1).copied();
    (previous.is_ascii_lowercase() && current.is_ascii_uppercase())
        || (previous.is_ascii_alphabetic() && current.is_ascii_digit())
        || (previous.is_ascii_digit() && current.is_ascii_alphabetic())
        || (previous.is_ascii_uppercase()
            && current.is_ascii_uppercase()
            && next.is_some_and(|next| next.is_ascii_lowercase()))
}

pub(super) fn flush_cjk_run(words: &mut Vec<String>, cjk_run: &mut Vec<char>) {
    if cjk_run.is_empty() {
        return;
    }
    for character in cjk_run.iter() {
        push_unique_word(words, character.to_string());
    }
    for pair in cjk_run.windows(2) {
        push_unique_word(words, pair.iter().collect::<String>());
    }
    if cjk_run.len() > 2 {
        push_unique_word(words, cjk_run.iter().collect::<String>());
    }
    cjk_run.clear();
}

pub(super) fn push_unique_word(words: &mut Vec<String>, word: String) {
    if !words.iter().any(|existing| existing == &word) {
        words.push(word);
    }
}

pub(super) fn token_set_has_match(tokens: &HashSet<String>, query_token: &str) -> bool {
    tokens
        .iter()
        .any(|candidate_token| lexical_token_matches(query_token, candidate_token))
}

pub(super) fn lexical_token_matches(query_token: &str, candidate_token: &str) -> bool {
    if query_token == candidate_token {
        return true;
    }
    if token_is_cjk(query_token) || token_is_cjk(candidate_token) {
        return false;
    }
    let query_stem = lexical_stem(query_token);
    let candidate_stem = lexical_stem(candidate_token);
    query_stem == candidate_stem
        || (query_token.len() >= 5
            && candidate_token.len() >= 5
            && (candidate_token.contains(query_token) || query_token.contains(candidate_token)))
}

pub(super) fn lexical_stem(token: &str) -> &str {
    for suffix in ["ing", "ed", "es", "s"] {
        if token.len() > suffix.len() + 3
            && let Some(stem) = token.strip_suffix(suffix)
        {
            return stem;
        }
    }
    token
}

pub(super) fn token_is_cjk(token: &str) -> bool {
    token.chars().any(is_cjk_character)
}

pub(super) fn is_cjk_character(character: char) -> bool {
    matches!(
        character,
        '\u{3400}'..='\u{4dbf}'
            | '\u{4e00}'..='\u{9fff}'
            | '\u{f900}'..='\u{faff}'
            | '\u{20000}'..='\u{2a6df}'
            | '\u{2a700}'..='\u{2b73f}'
            | '\u{2b740}'..='\u{2b81f}'
            | '\u{2b820}'..='\u{2ceaf}'
    )
}

pub(super) fn ratio(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        return 0.0;
    }
    let numerator = u16::try_from(numerator.min(usize::from(u16::MAX))).unwrap_or(u16::MAX);
    let denominator = u16::try_from(denominator.min(usize::from(u16::MAX))).unwrap_or(u16::MAX);
    f32::from(numerator) / f32::from(denominator)
}
