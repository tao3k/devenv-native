use std::sync::Arc;

/// Bounded script-task metadata preserved for host dispatch.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnScriptTaskSpec {
    /// Optional source-level `scriptFormat` attribute.
    #[serde(default)]
    pub script_format: Option<Arc<str>>,
    /// Optional nested `<bpmn:script>` body trimmed during parse-time capture.
    #[serde(default)]
    pub script_body: Option<Arc<str>>,
}

impl BpmnScriptTaskSpec {
    /// Creates one bounded script-task metadata snapshot.
    #[must_use]
    pub fn new(
        script_format: Option<impl AsRef<str>>,
        script_body: Option<impl AsRef<str>>,
    ) -> Self {
        Self {
            script_format: script_format.map(|format| Arc::<str>::from(format.as_ref())),
            script_body: script_body.map(|body| Arc::<str>::from(body.as_ref())),
        }
    }
}
