//! Shared deterministic workflow-source admission contracts.

pub(super) const MARKDOWN_WORKFLOW_COMPILER: &str = "qianji-server-markdown-step-compiler-v1";
pub(super) const MARKDOWN_MEDIA_TYPE: &str = "text/markdown";

pub(super) struct WorkflowSourceCompilation {
    pub bpmn_xml: String,
    pub compiler: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WorkflowSourceCompileError {
    MarkdownStepsMissing,
}

#[derive(Debug, Clone)]
pub(super) struct WorkflowTask {
    pub id: String,
    pub name: String,
    pub documentation: String,
}

pub(super) fn xml_id(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-' | '.' => character,
            _ => '_',
        })
        .collect()
}

pub(super) fn xml_attr(value: &str) -> String {
    xml_escape(value)
}

pub(super) fn xml_text(value: &str) -> String {
    xml_escape(value)
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
