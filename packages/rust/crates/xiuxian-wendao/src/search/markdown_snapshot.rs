use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use crate::gateway::studio::compile_markdown_nodes;
use crate::gateway::studio::search::{build_markdown_ast_hits_from_sections, markdown_scope_name};
use crate::gateway::studio::types::AstSearchHit;
use crate::parsers::markdown::{
    ParsedNote, adapt_markdown_note, adapt_org_note, is_org_note, is_supported_note,
};
use xiuxian_wendao_parsers::{
    fingerprint_markdown_note, parse_markdown_note_artifacts, parse_org_note,
    sections::MarkdownSection,
};

use super::ProjectScannedFile;

#[derive(Debug, Clone, Default)]
pub(crate) struct MarkdownProjectSnapshot {
    entries_by_path: BTreeMap<String, std::sync::Arc<MarkdownSnapshotEntry>>,
}

impl MarkdownProjectSnapshot {
    #[must_use]
    pub(crate) fn new(
        entries_by_path: BTreeMap<String, std::sync::Arc<MarkdownSnapshotEntry>>,
    ) -> Self {
        Self { entries_by_path }
    }

    #[must_use]
    pub(crate) fn entry(
        &self,
        normalized_path: &str,
    ) -> Option<&std::sync::Arc<MarkdownSnapshotEntry>> {
        self.entries_by_path.get(normalized_path)
    }
}

#[derive(Debug)]
pub(crate) struct MarkdownSnapshotEntry {
    pub(crate) file: ProjectScannedFile,
    pub(crate) parsed_note: Option<ParsedNote>,
    pub(crate) note_fingerprint: Option<String>,
    pub(crate) symbol_fingerprint: Option<String>,
    content: Option<Arc<str>>,
    parser_sections: Vec<MarkdownSection>,
    ast_hits: OnceLock<Vec<AstSearchHit>>,
}

impl MarkdownSnapshotEntry {
    #[must_use]
    pub(crate) fn clone_ast_hits(&self) -> Vec<AstSearchHit> {
        self.ast_hits.get_or_init(|| self.build_ast_hits()).clone()
    }

    fn build_ast_hits(&self) -> Vec<AstSearchHit> {
        let Some(content) = &self.content else {
            return Vec::new();
        };

        let nodes = compile_markdown_nodes(self.file.normalized_path.as_str(), content.as_ref());
        let crate_name = markdown_scope_name(Path::new(self.file.normalized_path.as_str()));
        let mut ast_hits = build_markdown_ast_hits_from_sections(
            self.file.normalized_path.as_str(),
            crate_name.as_str(),
            &nodes,
            self.parser_sections.as_slice(),
        );
        for hit in &mut ast_hits {
            if self.file.project_name.is_some() {
                hit.project_name.clone_from(&self.file.project_name);
                hit.navigation_target
                    .project_name
                    .clone_from(&self.file.project_name);
            }
            if self.file.root_label.is_some() {
                hit.root_label.clone_from(&self.file.root_label);
                hit.navigation_target
                    .root_label
                    .clone_from(&self.file.root_label);
            }
        }

        ast_hits
    }
}

#[must_use]
pub(crate) fn markdown_snapshot_entry_cache_key(
    project_root: &Path,
    file: &ProjectScannedFile,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(project_root.to_string_lossy().as_bytes());
    hasher.update(file.scan_root.to_string_lossy().as_bytes());
    hasher.update(file.partition_id.as_bytes());
    hasher.update(file.absolute_path.to_string_lossy().as_bytes());
    hasher.update(file.normalized_path.as_bytes());
    hasher.update(file.project_name.as_deref().unwrap_or_default().as_bytes());
    hasher.update(file.root_label.as_deref().unwrap_or_default().as_bytes());
    hasher.update(&file.size_bytes.to_le_bytes());
    hasher.update(&file.modified_secs.to_le_bytes());
    hasher.update(&u64::from(file.modified_nanos).to_le_bytes());
    hasher.finalize().to_hex().to_string()
}

#[must_use]
pub(crate) fn build_markdown_snapshot_entry(
    project_root: &Path,
    file: &ProjectScannedFile,
) -> MarkdownSnapshotEntry {
    if !is_supported_note(file.absolute_path.as_path()) {
        return MarkdownSnapshotEntry {
            file: file.clone(),
            parsed_note: None,
            note_fingerprint: None,
            symbol_fingerprint: None,
            content: None,
            parser_sections: Vec::new(),
            ast_hits: OnceLock::new(),
        };
    }

    let Ok(content) = std::fs::read_to_string(file.absolute_path.as_path()) else {
        return MarkdownSnapshotEntry {
            file: file.clone(),
            parsed_note: None,
            note_fingerprint: None,
            symbol_fingerprint: None,
            content: None,
            parser_sections: Vec::new(),
            ast_hits: OnceLock::new(),
        };
    };

    let fallback_title = file
        .absolute_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("page");
    let (parsed_note, note_fingerprint, symbol_fingerprint, parser_sections) =
        if is_org_note(file.absolute_path.as_path()) {
            let parser_note = parse_org_note(&content, fallback_title);
            let fingerprint = blake3::hash(content.as_bytes()).to_hex().to_string();
            let parser_sections = parser_note.core.sections.clone();
            let parsed_note =
                adapt_org_note(file.absolute_path.as_path(), project_root, parser_note);
            (
                parsed_note,
                fingerprint.clone(),
                fingerprint,
                parser_sections,
            )
        } else {
            let parser_artifacts = parse_markdown_note_artifacts(&content, fallback_title);
            let note_fingerprint = fingerprint_markdown_note(&parser_artifacts.note);
            let symbol_fingerprint = parser_artifacts.symbol_fingerprint.clone();
            let parser_note = parser_artifacts.note;
            let parser_sections = parser_note.core.sections.clone();
            let parsed_note =
                adapt_markdown_note(file.absolute_path.as_path(), project_root, parser_note);
            (
                parsed_note,
                note_fingerprint,
                symbol_fingerprint,
                parser_sections,
            )
        };

    MarkdownSnapshotEntry {
        file: file.clone(),
        parsed_note,
        note_fingerprint: Some(note_fingerprint),
        symbol_fingerprint: Some(symbol_fingerprint),
        content: Some(Arc::<str>::from(content)),
        parser_sections,
        ast_hits: OnceLock::new(),
    }
}
