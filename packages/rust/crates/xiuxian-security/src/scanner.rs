//! Secret scanning primitives backed by compiled regex sets.

use regex::RegexSet;
use serde::Serialize;
use std::sync::LazyLock;

/// Security violation detected during scan.
#[derive(Debug, Clone, Serialize)]
pub struct SecurityViolation {
    /// Rule identifier (e.g., `AWS_ACCESS_KEY`).
    pub rule_id: String,
    /// Human-readable description of the violation.
    pub description: String,
    /// Redacted snippet showing context.
    pub snippet: String,
}

static SECRET_PATTERNS: LazyLock<RegexSet> = LazyLock::new(|| {
    match RegexSet::new([
        r"AKIA[0-9A-Z]{16}",                    // AWS Access Key ID
        r"(?i)sk_(test|live)_[0-9a-zA-Z]{24}",  // Stripe Secret Key (test or live)
        r"xox[baprs]-([0-9a-zA-Z\-]{10,48})",   // Slack Token (allows hyphens in token)
        r"-----BEGIN [A-Z ]+ PRIVATE KEY-----", // PEM Private Key
        r#"(?i)(api_key|access_token|secret)\s*[:=]\s*["'][A-Za-z0-9_=-]{16,}["']"#, // Generic API Key
    ]) {
        Ok(set) => set,
        Err(err) => panic!("invalid regex pattern in security scanner: {err}"),
    }
});

static PATTERN_NAMES: &[&str] = &[
    "AWS Access Key",
    "Stripe Secret Key",
    "Slack Token",
    "PEM Private Key",
    "Generic High-Entropy Secret",
];

/// `SecretScanner` - high-performance secret detection using `RegexSet`.
///
/// Uses `RegexSet` for O(n) scanning regardless of pattern count.
/// Patterns are compiled once at startup via lazy static initialization.
pub struct SecretScanner;

impl SecretScanner {
    /// Scan content for secrets (fail-fast on first match).
    ///
    /// Returns `None` if content is clean, `Some(SecurityViolation)` if secrets are found.
    #[must_use]
    pub fn scan(content: &str) -> Option<SecurityViolation> {
        let matches = SECRET_PATTERNS.matches(content);

        if let Some(idx) = matches.iter().next() {
            let description = PATTERN_NAMES
                .get(idx)
                .copied()
                .unwrap_or("Unknown Secret")
                .to_string();

            return Some(SecurityViolation {
                rule_id: format!("SEC-{:03}", idx + 1),
                description,
                snippet: "[REDACTED]".to_string(),
            });
        }

        None
    }

    /// Scan and return all violations (non-fail-fast).
    #[must_use]
    pub fn scan_all(content: &str) -> Vec<SecurityViolation> {
        let matches = SECRET_PATTERNS.matches(content);
        let mut violations = Vec::new();

        for idx in &matches {
            let description = PATTERN_NAMES
                .get(idx)
                .copied()
                .unwrap_or("Unknown Secret")
                .to_string();

            violations.push(SecurityViolation {
                rule_id: format!("SEC-{:03}", idx + 1),
                description,
                snippet: "[REDACTED]".to_string(),
            });
        }

        violations
    }

    /// Check whether content contains any secrets.
    #[must_use]
    pub fn contains_secrets(content: &str) -> bool {
        SECRET_PATTERNS.is_match(content)
    }
}
