//! Inferred memory object classification.

/// Common first-class memory object kinds inferred from reflective records.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum InferredMemoryObjectKind {
    /// Final outcome, promotion, rejection, or supersession signal.
    Finality,
    /// Durable design, implementation, benchmark, or policy claim.
    Claim,
    /// Validation command, report path, benchmark receipt, or proof pointer.
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

/// A typed memory object inferred from a reflective question/value pair.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct InferredMemoryObject {
    /// Inferred object kind.
    pub kind: InferredMemoryObjectKind,
    /// Source reflection question.
    pub question: String,
    /// Source reflection answer.
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
