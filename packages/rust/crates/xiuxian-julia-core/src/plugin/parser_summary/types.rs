//! Julia parser-summary decoded row and public source-id types.

use std::collections::BTreeMap;

use serde::Serialize;

/// Borrowed Julia repository source identifier accepted by public helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JuliaSourceId<'a>(&'a str);

impl<'a> JuliaSourceId<'a> {
    /// Return the raw repository source identifier.
    #[must_use]
    pub fn as_str(self) -> &'a str {
        self.0
    }
}

impl<'a> From<&'a str> for JuliaSourceId<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

/// Target kinds preserved by the native Julia parser docstring contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) enum JuliaParserDocTargetKind {
    /// A module-level docstring.
    Module,
    /// Any symbol-level docstring.
    Symbol,
}

/// Julia symbol kinds preserved by the native parser-summary contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) enum JuliaParserSymbolKind {
    /// A function-like declaration.
    Function,
    /// A type-like declaration.
    Type,
    /// A constant-like binding.
    Constant,
    /// Any other parser-owned symbol kind not yet projected downstream.
    Other,
}

/// One Julia symbol preserved from the native parser-summary contract.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) struct JuliaParserSymbol {
    /// Symbol display name.
    pub(crate) name: String,
    /// Normalized symbol kind for repo-intelligence mapping.
    pub(crate) kind: JuliaParserSymbolKind,
    /// Optional signature snippet emitted by the parser.
    pub(crate) signature: Option<String>,
    /// Optional 1-based source line where the symbol starts.
    pub(crate) line_start: Option<usize>,
    /// Optional 1-based source line where the symbol ends.
    pub(crate) line_end: Option<usize>,
    /// Parser-owned detail attributes preserved for downstream AST consumers.
    pub(crate) attributes: BTreeMap<String, String>,
}

/// One Julia import-like dependency preserved from the native parser-summary
/// contract.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) struct JuliaParserImport {
    /// Imported module or qualified target name.
    pub(crate) module: String,
    /// Whether the dependency was reexported into public scope.
    pub(crate) reexported: bool,
    /// Native dependency family, such as `using`, `import`, or `include`.
    pub(crate) dependency_kind: String,
    /// Native dependency syntax form, such as `path`, `member`, or `include`.
    pub(crate) dependency_form: String,
    /// Whether the dependency uses relative Julia module traversal.
    pub(crate) dependency_is_relative: bool,
    /// Count of leading relative-module separators.
    pub(crate) dependency_relative_level: i32,
    /// Local binding name introduced by the dependency when present.
    pub(crate) dependency_local_name: Option<String>,
    /// Parent path for selective imports when present.
    pub(crate) dependency_parent: Option<String>,
    /// Imported member name when present.
    pub(crate) dependency_member: Option<String>,
    /// Explicit alias when present.
    pub(crate) dependency_alias: Option<String>,
}

/// One Julia docstring attachment preserved from the native parser-summary
/// contract.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) struct JuliaParserDocAttachment {
    /// Target display name.
    pub(crate) target_name: String,
    /// Normalized target kind.
    pub(crate) target_kind: JuliaParserDocTargetKind,
    /// Optional parser-owned qualified path for the attached target.
    pub(crate) target_path: Option<String>,
    /// Optional 1-based target declaration start line.
    pub(crate) target_line_start: Option<usize>,
    /// Optional 1-based target declaration end line.
    pub(crate) target_line_end: Option<usize>,
    /// Trimmed docstring contents.
    pub(crate) content: String,
}

/// Julia file summary consumed by repo-intelligence after the native parser
/// cutover.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct JuliaParserFileSummary {
    /// Optional module declared in the parsed file.
    pub(crate) module_name: Option<String>,
    /// Names exported from the parsed file.
    pub(crate) exports: Vec<String>,
    /// Imported or reexported dependencies from the parsed file.
    pub(crate) imports: Vec<JuliaParserImport>,
    /// Directly declared parser-owned symbols.
    pub(crate) symbols: Vec<JuliaParserSymbol>,
    /// Parser-owned docstring attachments.
    pub(crate) docstrings: Vec<JuliaParserDocAttachment>,
    /// Literal include targets referenced by the file.
    pub(crate) includes: Vec<String>,
}

/// Julia root summary consumed by repo-intelligence after the native parser
/// cutover.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct JuliaParserSourceSummary {
    /// Root module name.
    pub(crate) module_name: String,
    /// Names exported from the root module.
    pub(crate) exports: Vec<String>,
    /// Imported or reexported dependencies from the root module.
    pub(crate) imports: Vec<JuliaParserImport>,
    /// Directly declared parser-owned symbols.
    pub(crate) symbols: Vec<JuliaParserSymbol>,
    /// Parser-owned docstring attachments.
    pub(crate) docstrings: Vec<JuliaParserDocAttachment>,
    /// Literal include targets referenced by the root file.
    pub(crate) includes: Vec<String>,
}
