use crate::modelica_plugin::types::{ParsedDeclaration, ParsedImport};
use serde::Serialize;

/// Native Modelica file summary consumed by the transitional Rust plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ModelicaParserFileSummary {
    /// Primary top-level class or package declared in the file.
    pub(crate) class_name: Option<String>,
    /// Import-like dependencies emitted by the native parser summary.
    pub(crate) imports: Vec<ParsedImport>,
    /// Symbol declarations emitted by the native parser summary.
    pub(crate) declarations: Vec<ParsedDeclaration>,
}
