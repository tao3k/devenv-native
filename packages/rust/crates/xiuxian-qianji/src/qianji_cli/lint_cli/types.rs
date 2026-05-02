//! Shared lint command contracts.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum LintOutputFormat {
    Json,
    #[default]
    Llm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LintCliOutput {
    pub(crate) rendered: String,
    pub(crate) exit_code: i32,
}
