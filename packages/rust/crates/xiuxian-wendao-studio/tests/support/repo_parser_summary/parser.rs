use super::rows::{ParserSummaryRequest, ParserSummaryRow};

const FILE_SUMMARY_KIND: &str = "julia_file_summary";
const ROOT_SUMMARY_KIND: &str = "julia_root_summary";

#[derive(Debug, Clone)]
struct ParsedJuliaFile {
    module_name: Option<String>,
    exports: Vec<String>,
    imports: Vec<ParsedImport>,
    includes: Vec<String>,
    symbols: Vec<ParsedSymbol>,
    docstrings: Vec<ParsedDocstring>,
}

#[derive(Debug, Clone)]
struct ParsedImport {
    module: String,
    dependency_kind: String,
    reexported: bool,
}

#[derive(Debug, Clone)]
struct ParsedSymbol {
    name: String,
    kind: SymbolKind,
    signature: String,
    line_start: i64,
    line_end: i64,
}

#[derive(Debug, Clone, Copy)]
enum SymbolKind {
    Function,
    Type,
}

#[derive(Debug, Clone)]
struct ParsedDocstring {
    target_name: String,
    content: String,
    target_path: String,
    line_start: i64,
    line_end: i64,
}

#[derive(Debug, Clone)]
struct PendingDocstring {
    content: String,
}

pub(crate) fn build_response_rows(request: &ParserSummaryRequest) -> Vec<ParserSummaryRow> {
    let summary_kind = summary_kind_for_request(request.request_id.as_str());
    let parsed = parse_julia_source(request.source_text.as_str());
    let module_name = parsed.module_name.as_deref();
    let mut rows = Vec::new();

    rows.extend(export_rows(
        request,
        summary_kind,
        module_name,
        &parsed.exports,
    ));
    rows.extend(import_rows(
        request,
        summary_kind,
        module_name,
        &parsed.imports,
    ));
    rows.extend(include_rows(
        request,
        summary_kind,
        module_name,
        &parsed.includes,
    ));
    rows.extend(symbol_rows(
        request,
        summary_kind,
        module_name,
        &parsed.symbols,
    ));
    rows.extend(docstring_rows(
        request,
        summary_kind,
        module_name,
        &parsed.docstrings,
    ));

    if rows.is_empty() {
        rows.push(ParserSummaryRow::base(request, summary_kind, module_name));
    }
    rows
}

fn summary_kind_for_request(request_id: &str) -> &'static str {
    if request_id.starts_with("julia-root-summary:") {
        ROOT_SUMMARY_KIND
    } else {
        FILE_SUMMARY_KIND
    }
}

fn export_rows(
    request: &ParserSummaryRequest,
    summary_kind: &str,
    module_name: Option<&str>,
    exports: &[String],
) -> Vec<ParserSummaryRow> {
    exports
        .iter()
        .map(|name| {
            let mut row = ParserSummaryRow::base(request, summary_kind, module_name);
            row.item_group = Some("export".to_string());
            row.item_name = Some(name.clone());
            row
        })
        .collect()
}

fn import_rows(
    request: &ParserSummaryRequest,
    summary_kind: &str,
    module_name: Option<&str>,
    imports: &[ParsedImport],
) -> Vec<ParserSummaryRow> {
    imports
        .iter()
        .map(|import| {
            let mut row = ParserSummaryRow::base(request, summary_kind, module_name);
            row.item_group = Some("import".to_string());
            row.item_name = Some(import.module.clone());
            row.item_reexported = Some(import.reexported);
            row.item_dependency_kind = Some(import.dependency_kind.clone());
            row.item_dependency_form = Some("path".to_string());
            row.item_dependency_target = Some(import.module.clone());
            row.item_dependency_is_relative = Some(false);
            row.item_dependency_relative_level = Some(0);
            row
        })
        .collect()
}

fn include_rows(
    request: &ParserSummaryRequest,
    summary_kind: &str,
    module_name: Option<&str>,
    includes: &[String],
) -> Vec<ParserSummaryRow> {
    includes
        .iter()
        .map(|include| {
            let mut row = ParserSummaryRow::base(request, summary_kind, module_name);
            row.item_group = Some("include".to_string());
            row.item_path = Some(include.clone());
            row.item_dependency_kind = Some("include".to_string());
            row.item_dependency_form = Some("path".to_string());
            row.item_dependency_target = Some(include.clone());
            row.item_dependency_is_relative = Some(true);
            row.item_dependency_relative_level = Some(0);
            row
        })
        .collect()
}

fn symbol_rows(
    request: &ParserSummaryRequest,
    summary_kind: &str,
    module_name: Option<&str>,
    symbols: &[ParsedSymbol],
) -> Vec<ParserSummaryRow> {
    symbols
        .iter()
        .map(|symbol| {
            let mut row = ParserSummaryRow::base(request, summary_kind, module_name);
            row.item_group = Some("symbol".to_string());
            row.item_name = Some(symbol.name.clone());
            row.item_signature = Some(symbol.signature.clone());
            row.item_top_level = Some(true);
            row.item_line_start = Some(symbol.line_start);
            row.item_line_end = Some(symbol.line_end);
            match symbol.kind {
                SymbolKind::Function => {
                    row.item_kind = Some("function".to_string());
                    row.item_function_positional_arity = Some(1);
                    row.item_function_keyword_arity = Some(0);
                    row.item_function_has_varargs = Some(false);
                }
                SymbolKind::Type => {
                    row.item_kind = Some("type".to_string());
                    row.item_type_kind = Some("struct".to_string());
                }
            }
            attach_module_owner(&mut row, module_name);
            row
        })
        .collect()
}

fn docstring_rows(
    request: &ParserSummaryRequest,
    summary_kind: &str,
    module_name: Option<&str>,
    docstrings: &[ParsedDocstring],
) -> Vec<ParserSummaryRow> {
    docstrings
        .iter()
        .map(|docstring| {
            let mut row = ParserSummaryRow::base(request, summary_kind, module_name);
            row.item_group = Some("docstring".to_string());
            row.item_target_kind = Some("symbol".to_string());
            row.item_target_name = Some(docstring.target_name.clone());
            row.item_target_path = Some(docstring.target_path.clone());
            row.item_target_line_start = Some(docstring.line_start);
            row.item_target_line_end = Some(docstring.line_end);
            row.item_content = Some(docstring.content.clone());
            row
        })
        .collect()
}

fn attach_module_owner(row: &mut ParserSummaryRow, module_name: Option<&str>) {
    let Some(module_name) = module_name else {
        return;
    };
    row.item_module_name = Some(module_name.to_string());
    row.item_module_path = Some(module_name.to_string());
    row.item_owner_name = Some(module_name.to_string());
    row.item_owner_kind = Some("module".to_string());
    row.item_owner_path = Some(module_name.to_string());
}

fn parse_julia_source(source: &str) -> ParsedJuliaFile {
    let lines = source.lines().collect::<Vec<_>>();
    let mut parsed = ParsedJuliaFile {
        module_name: None,
        exports: Vec::new(),
        imports: Vec::new(),
        includes: Vec::new(),
        symbols: Vec::new(),
        docstrings: Vec::new(),
    };
    let mut pending_docstring = None;
    let mut index = 0usize;

    while index < lines.len() {
        let line = lines[index].trim();
        let line_number = i64::try_from(index + 1).unwrap_or(i64::MAX);
        if let Some(docstring) = parse_inline_docstring(line) {
            pending_docstring = Some(docstring);
            index += 1;
            continue;
        }
        if line == "\"\"\"" {
            let (docstring, next_index) = collect_docstring(&lines, index + 1);
            pending_docstring = Some(docstring);
            index = next_index;
            continue;
        }
        if line.is_empty() {
            index += 1;
            continue;
        }
        if let Some(module_name) = parse_module_name(line) {
            parsed.module_name = Some(module_name);
        } else if let Some(exports) = parse_exports(line) {
            parsed.exports.extend(exports);
        } else if let Some(import) = parse_import(line) {
            parsed.imports.push(import);
        } else if let Some(include) = parse_include(line) {
            parsed.includes.push(include);
        } else if let Some((symbol, next_index)) = parse_struct(&lines, index, line_number) {
            push_symbol_and_docstring(&mut parsed, symbol, pending_docstring.take());
            index = next_index;
            continue;
        } else if let Some((symbol, next_index)) = parse_block_function(&lines, index, line_number)
        {
            push_symbol_and_docstring(&mut parsed, symbol, pending_docstring.take());
            index = next_index;
            continue;
        } else if let Some(symbol) = parse_assignment_function(line, line_number) {
            push_symbol_and_docstring(&mut parsed, symbol, pending_docstring.take());
        } else {
            pending_docstring = None;
        }
        index += 1;
    }

    parsed
}

fn parse_inline_docstring(line: &str) -> Option<PendingDocstring> {
    let content = line.strip_prefix("\"\"\"")?.strip_suffix("\"\"\"")?;
    (line.len() >= 6).then(|| PendingDocstring {
        content: content.trim().to_string(),
    })
}

fn collect_docstring(lines: &[&str], mut index: usize) -> (PendingDocstring, usize) {
    let mut content = Vec::new();
    while index < lines.len() {
        let line = lines[index];
        index += 1;
        if line.trim() == "\"\"\"" {
            break;
        }
        content.push(line);
    }
    (
        PendingDocstring {
            content: content.join("\n").trim().to_string(),
        },
        index,
    )
}

fn parse_module_name(line: &str) -> Option<String> {
    line.strip_prefix("module ")
        .or_else(|| line.strip_prefix("baremodule "))
        .and_then(|rest| rest.split_whitespace().next())
        .filter(|name| !name.is_empty())
        .map(str::to_string)
}

fn parse_exports(line: &str) -> Option<Vec<String>> {
    line.strip_prefix("export ").map(|rest| {
        rest.split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string)
            .collect()
    })
}

fn parse_import(line: &str) -> Option<ParsedImport> {
    let (reexported, rest) = line
        .strip_prefix("@reexport using ")
        .map(|rest| (true, rest))
        .or_else(|| line.strip_prefix("using ").map(|rest| (false, rest)))?;
    let module = rest
        .split([',', ':'])
        .next()
        .map(str::trim)
        .filter(|module| !module.is_empty())?;
    Some(ParsedImport {
        module: module.to_string(),
        dependency_kind: "using".to_string(),
        reexported,
    })
}

fn parse_include(line: &str) -> Option<String> {
    let rest = line.strip_prefix("include(")?;
    let quoted = rest.trim_end_matches(')').trim();
    quoted
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            quoted
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .filter(|path| !path.is_empty())
        .map(str::to_string)
}

fn parse_struct(lines: &[&str], index: usize, line_number: i64) -> Option<(ParsedSymbol, usize)> {
    let line = lines[index].trim();
    let rest = line.strip_prefix("struct ")?;
    let name = rest
        .split(['{', '<', ' ', '\t'])
        .next()
        .filter(|name| !name.is_empty())?;
    let end_index = find_block_end(lines, index).unwrap_or(index);
    let line_end = i64::try_from(end_index + 1).unwrap_or(line_number);
    Some((
        ParsedSymbol {
            name: name.to_string(),
            kind: SymbolKind::Type,
            signature: format!("struct {name}"),
            line_start: line_number,
            line_end,
        },
        end_index + 1,
    ))
}

fn parse_block_function(
    lines: &[&str],
    index: usize,
    line_number: i64,
) -> Option<(ParsedSymbol, usize)> {
    let line = lines[index].trim();
    let signature = line.strip_prefix("function ")?.trim();
    let name = parse_function_name(signature)?;
    let end_index = find_block_end(lines, index).unwrap_or(index);
    let line_end = i64::try_from(end_index + 1).unwrap_or(line_number);
    Some((
        ParsedSymbol {
            name,
            kind: SymbolKind::Function,
            signature: signature.to_string(),
            line_start: line_number,
            line_end,
        },
        end_index + 1,
    ))
}

fn parse_assignment_function(line: &str, line_number: i64) -> Option<ParsedSymbol> {
    if !line.contains('=') || line.starts_with('@') || line.starts_with("const ") {
        return None;
    }
    let name = parse_function_name(line)?;
    Some(ParsedSymbol {
        name,
        kind: SymbolKind::Function,
        signature: line.to_string(),
        line_start: line_number,
        line_end: line_number,
    })
}

fn parse_function_name(signature: &str) -> Option<String> {
    signature
        .split(['(', ' ', '\t'])
        .next()
        .map(str::trim)
        .filter(|name| {
            !name.is_empty()
                && name
                    .chars()
                    .next()
                    .is_some_and(|first| first == '_' || first.is_alphabetic())
        })
        .map(str::to_string)
}

fn find_block_end(lines: &[&str], start_index: usize) -> Option<usize> {
    lines
        .iter()
        .enumerate()
        .skip(start_index + 1)
        .find(|(_, line)| line.trim() == "end")
        .map(|(index, _)| index)
}

fn push_symbol_and_docstring(
    parsed: &mut ParsedJuliaFile,
    symbol: ParsedSymbol,
    pending_docstring: Option<PendingDocstring>,
) {
    if let Some(pending_docstring) = pending_docstring {
        let target_path = parsed.module_name.as_ref().map_or_else(
            || symbol.name.clone(),
            |module_name| format!("{module_name}.{}", symbol.name),
        );
        parsed.docstrings.push(ParsedDocstring {
            target_name: symbol.name.clone(),
            content: pending_docstring.content,
            target_path,
            line_start: symbol.line_start,
            line_end: symbol.line_end,
        });
    }
    parsed.symbols.push(symbol);
}

#[cfg(test)]
mod tests {
    use super::{ParserSummaryRequest, ROOT_SUMMARY_KIND, build_response_rows};

    #[test]
    fn inline_docstring_is_attached_to_next_symbol() -> Result<(), String> {
        let request = ParserSummaryRequest {
            request_id: "julia-root-summary:inline-docstring".to_string(),
            source_id: "src/ProjectionPkg.jl".to_string(),
            source_text: "module ProjectionPkg\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n"
                .to_string(),
        };

        let rows = build_response_rows(&request);
        let docstring = rows
            .iter()
            .find(|row| row.item_group.as_deref() == Some("docstring"))
            .ok_or_else(|| "expected inline docstring row".to_string())?;

        assert_eq!(docstring.summary_kind, ROOT_SUMMARY_KIND);
        assert_eq!(docstring.item_target_name.as_deref(), Some("solve"));
        assert_eq!(docstring.item_content.as_deref(), Some("solve docs"));
        Ok(())
    }
}
