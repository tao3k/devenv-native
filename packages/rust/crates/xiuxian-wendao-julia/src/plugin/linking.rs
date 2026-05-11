use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use regex::Regex;
use xiuxian_wendao_core::repo_intelligence::{
    DocRecord, ExampleRecord, ModuleRecord, PluginLinkContext, RelationKind, RelationRecord,
    RepoIntelligenceError, SymbolRecord,
};

pub(crate) fn build_doc_relations(
    context: &PluginLinkContext,
) -> Result<Vec<RelationRecord>, RepoIntelligenceError> {
    let root_module = context
        .modules
        .first()
        .filter(|_| context.modules.len() == 1);
    let mut relations = Vec::new();

    for doc in &context.docs {
        if doc.format.as_deref() == Some("julia_docstring") {
            continue;
        }
        let evidence = DocLinkEvidence::load(&context.repository_root, doc)?;
        let mut target_ids = BTreeSet::new();

        if let Some(module) = root_module.filter(|_| evidence.is_root_readme()) {
            target_ids.insert(module.module_id.to_string());
        }

        for module in &context.modules {
            if evidence.matches_module(module) {
                target_ids.insert(module.module_id.to_string());
            }
        }

        for symbol in &context.symbols {
            if evidence.matches_symbol(symbol) {
                target_ids.insert(symbol.symbol_id.to_string());
            }
        }

        relations.extend(target_ids.into_iter().map(|target_id| RelationRecord {
            repo_id: doc.repo_id.clone(),
            source_id: doc.doc_id.clone().to_string(),
            target_id: target_id.to_string(),
            kind: RelationKind::Documents,
        }));
    }

    Ok(relations)
}

pub(crate) fn build_example_relations(
    context: &PluginLinkContext,
) -> Result<Vec<RelationRecord>, RepoIntelligenceError> {
    let mut relations = Vec::new();

    for example in &context.examples {
        let evidence = ExampleLinkEvidence::load(&context.repository_root, example)?;
        let target_ids = example_relation_target_ids(&evidence, &context.symbols, &context.modules);

        relations.extend(target_ids.into_iter().map(|target_id| RelationRecord {
            repo_id: example.repo_id.clone(),
            source_id: example.example_id.clone().to_string(),
            target_id,
            kind: RelationKind::ExampleOf,
        }));
    }

    Ok(relations)
}

fn example_relation_target_ids(
    evidence: &ExampleLinkEvidence,
    symbols: &[SymbolRecord],
    modules: &[ModuleRecord],
) -> BTreeSet<String> {
    let mut target_ids = example_symbol_target_ids(evidence, symbols);
    target_ids.extend(example_module_target_ids(evidence, modules));
    target_ids
}

fn example_symbol_target_ids(
    evidence: &ExampleLinkEvidence,
    symbols: &[SymbolRecord],
) -> BTreeSet<String> {
    let mut target_ids = BTreeSet::new();
    for symbol in symbols {
        if evidence.matches_symbol(symbol) {
            target_ids.insert(symbol.symbol_id.to_string());
            if let Some(module_id) = &symbol.module_id {
                target_ids.insert(module_id.to_string());
            }
        }
    }
    target_ids
}

fn example_module_target_ids(
    evidence: &ExampleLinkEvidence,
    modules: &[ModuleRecord],
) -> BTreeSet<String> {
    modules
        .iter()
        .filter(|module| evidence.matches_module(module))
        .map(|module| module.module_id.to_string())
        .collect()
}

struct DocLinkEvidence {
    doc_path: String,
    stem: String,
    title: String,
    headings: Vec<String>,
    inline_code_identifiers: BTreeSet<String>,
}

struct ExampleLinkEvidence {
    path: String,
    contents: String,
}

impl DocLinkEvidence {
    fn load(repository_root: &Path, doc: &DocRecord) -> Result<Self, RepoIntelligenceError> {
        let path = repository_root.join(doc.path.as_str());
        let contents =
            fs::read_to_string(&path).map_err(|error| RepoIntelligenceError::AnalysisFailed {
                message: format!(
                    "failed to read documentation file `{}`: {error}",
                    path.display()
                ),
            })?;
        let stem = Path::new(doc.path.as_str())
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        Ok(Self {
            doc_path: doc.path.to_ascii_lowercase(),
            stem,
            title: doc.title.to_ascii_lowercase(),
            headings: extract_markdown_headings(&contents),
            inline_code_identifiers: extract_inline_code_identifiers(&contents),
        })
    }

    fn is_root_readme(&self) -> bool {
        self.doc_path.starts_with("readme")
    }

    fn matches_module(&self, module: &ModuleRecord) -> bool {
        let qualified = module.qualified_name.to_ascii_lowercase();
        let short_name = module
            .qualified_name
            .rsplit('.')
            .next()
            .unwrap_or(module.qualified_name.as_str())
            .to_ascii_lowercase();

        self.matches_label(&qualified) || self.matches_label(&short_name)
    }

    fn matches_symbol(&self, symbol: &SymbolRecord) -> bool {
        let name = symbol.name.to_ascii_lowercase();
        let qualified = symbol.qualified_name.to_ascii_lowercase();

        self.matches_label(&name) || self.matches_label(&qualified)
    }

    fn matches_label(&self, label: &str) -> bool {
        self.stem == label
            || label_match(self.title.as_str(), label)
            || self
                .headings
                .iter()
                .any(|heading| label_match(heading.as_str(), label))
            || self.inline_code_identifiers.contains(label)
    }
}

impl ExampleLinkEvidence {
    fn load(
        repository_root: &Path,
        example: &ExampleRecord,
    ) -> Result<Self, RepoIntelligenceError> {
        let path = repository_root.join(example.path.as_str());
        let contents =
            fs::read_to_string(&path).map_err(|error| RepoIntelligenceError::AnalysisFailed {
                message: format!("failed to read example file `{}`: {error}", path.display()),
            })?;
        Ok(Self {
            path: example.path.to_ascii_lowercase(),
            contents: contents.to_ascii_lowercase(),
        })
    }

    fn matches_symbol(&self, symbol: &SymbolRecord) -> bool {
        self.matches_label(symbol.name.to_ascii_lowercase().as_str())
            || self.matches_label(symbol.qualified_name.to_ascii_lowercase().as_str())
    }

    fn matches_module(&self, module: &ModuleRecord) -> bool {
        let qualified = module.qualified_name.to_ascii_lowercase();
        let short_name = module
            .qualified_name
            .rsplit('.')
            .next()
            .unwrap_or(module.qualified_name.as_str())
            .to_ascii_lowercase();

        self.matches_label(qualified.as_str()) || self.matches_label(short_name.as_str())
    }

    fn matches_label(&self, label: &str) -> bool {
        self.path.contains(label) || regex_label_match(self.contents.as_str(), label)
    }
}

fn extract_markdown_headings(contents: &str) -> Vec<String> {
    contents
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let heading = trimmed.strip_prefix('#')?;
            Some(heading.trim_start_matches('#').trim().to_ascii_lowercase())
        })
        .filter(|heading| !heading.is_empty())
        .collect()
}

fn extract_inline_code_identifiers(contents: &str) -> BTreeSet<String> {
    let Ok(regex) = Regex::new(r"`([A-Za-z_][A-Za-z0-9_\.!]*)`") else {
        return BTreeSet::new();
    };

    regex
        .captures_iter(contents)
        .filter_map(|captures| captures.get(1))
        .map(|value| value.as_str().to_ascii_lowercase())
        .collect()
}

fn label_match(candidate: &str, label: &str) -> bool {
    candidate == label
        || candidate.starts_with(&format!("{label}("))
        || candidate.starts_with(&format!("{label} "))
        || candidate.starts_with(&format!("{label}:"))
}

fn regex_label_match(candidate: &str, label: &str) -> bool {
    let escaped = regex::escape(label);
    let Ok(regex) = Regex::new(&format!(r"(?m)(?<![A-Za-z0-9_]){escaped}(?![A-Za-z0-9_])")) else {
        return false;
    };
    regex.is_match(candidate)
}
