use crate::search::SearchDocument;
use crate::unified_symbol::symbol::SymbolSource;
use crate::unified_symbol::{UnifiedSymbol, UnifiedSymbolIndex};

impl UnifiedSymbolIndex {
    /// Add a project symbol.
    pub fn add_project_symbol(&mut self, name: &str, kind: &str, location: &str, crate_name: &str) {
        let symbol = UnifiedSymbol::new_project(name, kind, location, crate_name);
        self.add_symbol(symbol);
    }

    /// Add an external dependency symbol.
    pub fn add_external_symbol(
        &mut self,
        name: &str,
        kind: &str,
        location: &str,
        crate_name: &str,
    ) {
        let symbol = UnifiedSymbol::new_external(name, kind, location, crate_name);
        self.add_symbol(symbol);
    }

    /// Add multiple symbols and index them in a single Tantivy batch.
    pub fn add_symbols_batch(&mut self, symbols: Vec<UnifiedSymbol>) {
        if symbols.is_empty() {
            return;
        }

        let base_idx = self.symbols.len();
        let mut documents = Vec::with_capacity(symbols.len());

        for (offset, symbol) in symbols.into_iter().enumerate() {
            let idx = base_idx + offset;
            self.by_name
                .entry(symbol.name.to_lowercase())
                .or_default()
                .push(idx);
            documents.push(search_document_for_symbol(idx, &symbol));
            self.symbols.push(symbol);
        }

        let _ = self.search_index.add_documents(documents);
    }

    /// Add a symbol from `repo_intelligence` analysis.
    pub fn add_symbol_record(&mut self, record: &crate::analyzers::SymbolRecord) {
        let kind_str = match record.kind {
            crate::analyzers::RepoSymbolKind::Function => "fn",
            crate::analyzers::RepoSymbolKind::Type => "type",
            crate::analyzers::RepoSymbolKind::Constant => "const",
            crate::analyzers::RepoSymbolKind::ModuleExport => "export",
            crate::analyzers::RepoSymbolKind::Other => "other",
        };

        let source = if record.repo_id == "stdlib" {
            SymbolSource::External("stdlib".to_string())
        } else {
            SymbolSource::Project
        };

        let symbol = UnifiedSymbol {
            name: record.name.clone(),
            kind: kind_str.to_string(),
            location: record.path.clone(),
            source,
            crate_name: record.repo_id.clone(),
        };
        self.add_symbol(symbol);
    }

    /// Record usage of an external symbol in a project file.
    pub fn record_external_usage(
        &mut self,
        crate_name: &str,
        symbol_name: &str,
        project_file: &str,
    ) {
        self.external_usage
            .entry(crate_name.to_string())
            .or_default()
            .push(project_file.to_string());

        self.project_files
            .entry(project_file.to_string())
            .or_default()
            .push(symbol_name.to_string());
    }

    pub(crate) fn add_symbol(&mut self, symbol: UnifiedSymbol) {
        self.add_symbols_batch(vec![symbol]);
    }

    /// Clear all symbols.
    pub fn clear(&mut self) {
        self.by_name.clear();
        self.symbols.clear();
        self.external_usage.clear();
        self.project_files.clear();
        self.search_index = crate::SearchDocumentIndex::new();
    }
}

fn search_document_for_symbol(id: usize, symbol: &UnifiedSymbol) -> SearchDocument {
    let scope = match &symbol.source {
        SymbolSource::Project => "project",
        SymbolSource::External(_) => "external",
    };

    SearchDocument {
        id: id.to_string(),
        title: symbol.name.clone(),
        kind: symbol.kind.clone(),
        path: symbol.location.clone(),
        scope: scope.to_string(),
        namespace: symbol.crate_name.clone(),
        terms: vec![
            symbol.crate_name.clone(),
            symbol.kind.clone(),
            symbol.location.clone(),
            scope.to_string(),
        ],
    }
}
