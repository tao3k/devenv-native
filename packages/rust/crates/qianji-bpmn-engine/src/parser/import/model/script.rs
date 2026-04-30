#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawScriptTaskSpec {
    pub(crate) script_format: Option<String>,
    pub(crate) script_body: Option<String>,
}
