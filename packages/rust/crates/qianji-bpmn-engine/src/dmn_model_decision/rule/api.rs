use super::{Arc, DmnInputEntry, DmnOutputEntry};

/// One bounded DMN rule.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DmnRule {
    /// Stable rule identifier.
    pub rule_id: Arc<str>,
    /// Optional free-form rule description.
    pub description: Option<Arc<str>>,
    /// Input predicates for this rule.
    pub input_entries: Vec<DmnInputEntry>,
    /// Output expressions for this rule.
    pub output_entries: Vec<DmnOutputEntry>,
}

impl DmnRule {
    /// Creates one bounded DMN rule.
    #[must_use]
    pub fn new(
        rule_id: impl AsRef<str>,
        description: Option<impl AsRef<str>>,
        input_entries: Vec<DmnInputEntry>,
        output_entries: Vec<DmnOutputEntry>,
    ) -> Self {
        Self {
            rule_id: Arc::<str>::from(rule_id.as_ref()),
            description: description.map(|value| Arc::<str>::from(value.as_ref())),
            input_entries,
            output_entries,
        }
    }
}
