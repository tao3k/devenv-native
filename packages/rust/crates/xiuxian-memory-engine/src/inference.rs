//! Inferred memory object classification.

/// Common first-class memory object kinds inferred from reflective records and
/// structured source properties.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum InferredMemoryObjectKind {
    /// Final outcome, promotion, rejection, or supersession signal.
    Finality,
    /// Durable design, implementation, benchmark, or policy claim.
    Claim,
    /// Report path, artifact, benchmark receipt, commit, or proof pointer.
    Evidence,
    /// Symptom, cause, fix, or avoidance rule.
    Failure,
    /// User correction, naming rule, command rule, or architecture preference.
    Preference,
}

impl InferredMemoryObjectKind {
    /// Return the stable graph-facing kind name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Finality => "finality",
            Self::Claim => "claim",
            Self::Evidence => "evidence",
            Self::Failure => "failure",
            Self::Preference => "preference",
        }
    }

    /// Return the ranking-facet label used by memory consumers.
    #[must_use]
    pub const fn facet_label(self) -> &'static str {
        match self {
            Self::Finality => "memory-finality",
            Self::Claim => "memory-claim",
            Self::Evidence => "memory-evidence",
            Self::Failure => "memory-failure",
            Self::Preference => "memory-preference",
        }
    }
}

/// A typed memory object inferred from a source question/property and value.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct InferredMemoryObject {
    /// Inferred object kind.
    pub kind: InferredMemoryObjectKind,
    /// Source reflection question or property key.
    pub question: String,
    /// Source reflection answer or property value.
    pub value: String,
}

/// Infer a first-class memory object from a reflection question/value pair.
#[must_use]
pub fn infer_memory_object_from_reflection(
    question: impl AsRef<str>,
    value: impl AsRef<str>,
) -> Option<InferredMemoryObject> {
    let question = question.as_ref().trim();
    let value = value.as_ref().trim();
    if question.is_empty() || value.is_empty() {
        return None;
    }
    let kind = infer_memory_object_kind_from_question(question)?;
    Some(InferredMemoryObject {
        kind,
        question: question.to_string(),
        value: value.to_string(),
    })
}

/// Infer first-class memory objects from structured source properties.
#[must_use]
pub fn infer_memory_objects_from_properties<'a>(
    properties: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Vec<InferredMemoryObject> {
    properties
        .into_iter()
        .filter_map(|(key, value)| infer_memory_object_from_property(key, value))
        .collect()
}

/// Infer a first-class memory object from one structured source property.
#[must_use]
pub fn infer_memory_object_from_property(
    key: impl AsRef<str>,
    value: impl AsRef<str>,
) -> Option<InferredMemoryObject> {
    let key = key.as_ref().trim();
    let value = value.as_ref().trim();
    if key.is_empty() || value.is_empty() {
        return None;
    }
    let kind = infer_memory_object_kind_from_property_key(key)?;
    if !property_value_matches_memory_kind(kind, value) {
        return None;
    }
    Some(InferredMemoryObject {
        kind,
        question: key.to_string(),
        value: value.to_string(),
    })
}

/// Infer the memory object kind from a reflection question.
#[must_use]
pub fn infer_memory_object_kind_from_question(
    question: impl AsRef<str>,
) -> Option<InferredMemoryObjectKind> {
    let normalized = question.as_ref().to_ascii_lowercase();
    if normalized.contains("finality") || normalized.contains("outcome") {
        return Some(InferredMemoryObjectKind::Finality);
    }
    if normalized.contains("evidence") || normalized.contains("proof") {
        return Some(InferredMemoryObjectKind::Evidence);
    }
    if normalized.contains("failure") || normalized.contains("avoid") {
        return Some(InferredMemoryObjectKind::Failure);
    }
    if normalized.contains("preference")
        || normalized.contains("naming")
        || normalized.contains("correction")
    {
        return Some(InferredMemoryObjectKind::Preference);
    }
    normalized
        .contains("claim")
        .then_some(InferredMemoryObjectKind::Claim)
}

/// Infer the memory object kind from a structured property key.
#[must_use]
pub fn infer_memory_object_kind_from_property_key(
    key: impl AsRef<str>,
) -> Option<InferredMemoryObjectKind> {
    use InferredMemoryObjectKind::{Claim, Evidence, Failure, Finality, Preference};

    match normalize_property_key(key.as_ref()).as_str() {
        "OUTCOME" | "RESULT" | "SIGNAL" | "TASK_OUTCOME" => Some(Finality),
        "CLAIM" | "REUSABLE_KNOWLEDGE" => Some(Claim),
        "EVIDENCE" | "EVIDENCE_REF" | "PROOF" | "REFERENCE" | "REFERENCES" => Some(Evidence),
        "SYMPTOM" | "CAUSE" | "FIX" | "FAILURE_NOTE" | "FAILURE_MODE" => Some(Failure),
        "PREFERENCE" | "PREFERENCE_SIGNAL" | "REUSE_RULE" | "NAMING_RULE" | "CORRECTION" => {
            Some(Preference)
        }
        _ => None,
    }
}

fn property_value_matches_memory_kind(kind: InferredMemoryObjectKind, value: &str) -> bool {
    match kind {
        InferredMemoryObjectKind::Evidence => is_evidence_reference(value),
        InferredMemoryObjectKind::Finality
        | InferredMemoryObjectKind::Claim
        | InferredMemoryObjectKind::Failure
        | InferredMemoryObjectKind::Preference => value.chars().any(char::is_alphabetic),
    }
}

fn is_evidence_reference(value: &str) -> bool {
    let value = value.trim();
    value.starts_with("http://")
        || value.starts_with("https://")
        || value.starts_with("id:")
        || value.starts_with("orgid:")
        || value.starts_with("commit:")
        || value.starts_with("artifact:")
        || value.contains('/')
        || value.contains('#')
        || value_has_evidence_extension(value)
        || value_is_commit_like(value)
}

fn value_has_evidence_extension(value: &str) -> bool {
    [
        ".arrow", ".csv", ".duckdb", ".html", ".json", ".jsonl", ".log", ".md", ".org", ".parquet",
        ".pdf", ".png", ".svg", ".tsv", ".txt",
    ]
    .iter()
    .any(|extension| value.ends_with(extension))
}

fn value_is_commit_like(value: &str) -> bool {
    (7..=64).contains(&value.len()) && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn normalize_property_key(key: &str) -> String {
    key.trim()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
}
