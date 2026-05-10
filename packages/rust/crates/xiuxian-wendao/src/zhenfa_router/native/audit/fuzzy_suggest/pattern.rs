/// Structural elements extracted from a sgrep pattern.
///
/// Used for fuzzy matching when exact patterns fail.
#[derive(Debug, Clone, Default)]
pub(crate) struct PatternSkeleton {
    /// Language keywords (fn, def, class, struct, etc.).
    pub(crate) keywords: Vec<String>,
    /// Structural punctuation ((), {}, <>, [], ->, :, ;).
    pub(crate) structure: Vec<String>,
    /// Metavariables ($NAME, $$$ARGS, etc.).
    pub(crate) metavariables: Vec<String>,
    /// Identifier-like terms (function names, type names).
    pub(crate) identifiers: Vec<String>,
}

impl PatternSkeleton {
    /// Extract structural skeleton from a sgrep pattern.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let skeleton = PatternSkeleton::extract("fn $NAME($$$ARGS) -> Result<$$$>");
    /// // skeleton.keywords = ["fn", "Result"]
    /// // skeleton.structure = ["(", ")", "->", "<", ">"]
    /// // skeleton.metavariables = ["$NAME", "$$$ARGS", "$$$"]
    /// ```
    pub(crate) fn extract(pattern: &str) -> Self {
        let mut skeleton = Self::default();

        // Tokenize the pattern
        let tokens = tokenize_pattern(pattern);

        for token in tokens {
            match token.as_str() {
                // Keywords (language constructs)
                "fn" | "def" | "class" | "struct" | "enum" | "impl" | "trait" | "interface"
                | "type" | "const" | "let" | "var" | "pub" | "private" | "async" | "await"
                | "return" | "if" | "else" | "for" | "while" | "match" | "case" | "func"
                | "function" | "public" | "protected" | "Result" | "Option" | "Vec" | "String"
                | "str" | "int" | "bool" | "void" | "null" | "None" | "Some" | "Ok" | "Err"
                    if !skeleton.keywords.contains(&token) =>
                {
                    skeleton.keywords.push(token);
                }
                // Structural elements
                "(" | ")" | "{" | "}" | "[" | "]" | "<" | ">" | "->" | "=>" | ":" | ";" | ","
                | "." => {
                    skeleton.structure.push(token);
                }
                // Metavariables
                t if t.starts_with('$') => {
                    skeleton.metavariables.push(token);
                }
                // Identifiers (everything else that looks like a name)
                t if is_identifier_like(t) && !skeleton.identifiers.contains(&token) => {
                    skeleton.identifiers.push(token);
                }
                _ => {}
            }
        }

        skeleton
    }
}

/// Tokenize a sgrep pattern into individual elements.
pub(crate) fn tokenize_pattern(pattern: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = pattern.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            // Two-character operators
            '-' if chars.peek() == Some(&'>') => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                chars.next();
                tokens.push("->".to_string());
            }
            '=' if chars.peek() == Some(&'>') => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                chars.next();
                tokens.push("=>".to_string());
            }
            // Single-character structural elements
            '(' | ')' | '{' | '}' | '[' | ']' | '<' | '>' | ':' | ';' | ',' | '.' => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                tokens.push(ch.to_string());
            }
            // Whitespace separates tokens
            ' ' | '\t' | '\n' | '\r' => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            // Accumulate identifier/keyword characters
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// Check if a string looks like an identifier.
pub(crate) fn is_identifier_like(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_alphanumeric() || c == '_')
        && s.chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
}

/// Extract the primary capture name from a pattern.
#[cfg(test)]
pub(crate) fn extract_capture_name(pattern: &str) -> Option<String> {
    let mut chars = pattern.chars().peekable();
    let mut in_capture = false;
    let mut capture = String::new();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            in_capture = true;
            capture.clear();

            // Check for $$$ (multi-match)
            if chars.peek() == Some(&'$') {
                chars.next();
                if chars.peek() == Some(&'$') {
                    chars.next();
                    // Skip $$$ captures
                    in_capture = false;
                }
            }
            continue;
        }

        if in_capture {
            if ch.is_alphanumeric() || ch == '_' {
                capture.push(ch);
            } else {
                if !capture.is_empty() {
                    return Some(capture.clone());
                }
                in_capture = false;
            }
        }
    }

    if capture.is_empty() {
        None
    } else {
        Some(capture)
    }
}
