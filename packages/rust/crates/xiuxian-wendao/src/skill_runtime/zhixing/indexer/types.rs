use crate::graph::KnowledgeGraph;
use crate::skill_runtime::zhixing::Result;
use crate::sync::IncrementalSyncPolicy;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Indexing summary for a full Zhixing ingestion run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ZhixingIndexSummary {
    /// Number of journal markdown files indexed as `DOCUMENT`.
    pub journal_documents: usize,
    /// Number of agenda markdown files indexed as `DOCUMENT`.
    pub agenda_documents: usize,
    /// Number of embedded skill-reference graph entities indexed into the graph.
    pub skill_reference_entities_added: usize,
    /// Number of embedded skill-reference graph relations linked into the graph.
    pub skill_reference_relations: usize,
    /// Number of task checklist entries indexed as `OTHER(Task)`.
    pub task_entities: usize,
    /// Number of entities that were newly inserted (not updated in place).
    pub entities_added: usize,
    /// Number of task relations linked from agenda documents.
    pub relations_linked: usize,
}

/// Specialized indexer for Zhixing domain concepts into Wendao graph schema.
pub struct ZhixingWendaoIndexer {
    /// Underlying Wendao graph where agenda/journal entities are written.
    pub graph: Arc<KnowledgeGraph>,
    /// Notebook root path containing `journal/` and `agenda/`.
    pub notebook_root: PathBuf,
}

impl ZhixingWendaoIndexer {
    /// Create a new indexer for mapping Zhixing domain objects to Wendao schema.
    #[must_use]
    pub fn new(graph: Arc<KnowledgeGraph>, notebook_root: PathBuf) -> Self {
        Self {
            graph,
            notebook_root,
        }
    }

    /// Trigger a full scan of domain objects and map them into graph entities.
    ///
    /// # Errors
    /// Returns an error when file discovery, markdown reading, or graph operations fail.
    pub fn index_all_domain_objects(&self) -> Result<ZhixingIndexSummary> {
        log::debug!("Starting full Zhixing domain index for Wendao integration...");
        let mut summary = ZhixingIndexSummary::default();

        summary.journal_documents = self.index_document_dir("journal", "Journal", &mut summary)?;
        summary.agenda_documents = self.index_document_dir("agenda", "Agenda", &mut summary)?;
        let (skill_reference_entities, skill_reference_relations) =
            self.index_embedded_skill_references()?;
        summary.skill_reference_entities_added = skill_reference_entities;
        summary.skill_reference_relations = skill_reference_relations;
        summary.task_entities = self.index_agenda_tasks(&mut summary)?;

        log::info!(
            "Zhixing domain indexed successfully into Wendao (journal_documents={}, agenda_documents={}, skill_reference_entities_added={}, skill_reference_relations={}, task_entities={}, entities_added={}, relations_linked={}).",
            summary.journal_documents,
            summary.agenda_documents,
            summary.skill_reference_entities_added,
            summary.skill_reference_relations,
            summary.task_entities,
            summary.entities_added,
            summary.relations_linked
        );
        Ok(summary)
    }

    /// Incrementally synchronize one changed notebook path into Wendao graph.
    ///
    /// Returns `Ok(true)` when the graph state changed, `Ok(false)` when the
    /// path is irrelevant or does not result in any update.
    ///
    /// # Errors
    /// Returns an error when file parsing or graph operations fail.
    pub fn sync_changed_path(
        &self,
        changed_path: &Path,
        policy: &IncrementalSyncPolicy,
    ) -> Result<bool> {
        if !policy.supports_path(changed_path) {
            return Ok(false);
        }
        let Some((segment, date)) = self.resolve_notebook_target(changed_path) else {
            return Ok(false);
        };
        let mut summary = ZhixingIndexSummary::default();
        let changed = match segment {
            NotebookSegment::Journal => self.sync_document_path(
                "journal",
                "Journal",
                changed_path,
                date.as_str(),
                &mut summary,
            )?,
            NotebookSegment::Agenda => {
                let doc_changed = self.sync_document_path(
                    "agenda",
                    "Agenda",
                    changed_path,
                    date.as_str(),
                    &mut summary,
                )?;
                summary.task_entities =
                    self.reindex_agenda_tasks_for_path(changed_path, date.as_str(), &mut summary)?;
                doc_changed || summary.task_entities > 0
            }
        };
        if changed {
            log::info!(
                "Zhixing incremental sync applied path={} (segment={}, date={}, task_entities={}, entities_added={}, relations_linked={})",
                changed_path.display(),
                segment.as_str(),
                date,
                summary.task_entities,
                summary.entities_added,
                summary.relations_linked
            );
        }
        Ok(changed)
    }

    fn resolve_notebook_target(&self, changed_path: &Path) -> Option<(NotebookSegment, String)> {
        let relative = changed_path.strip_prefix(&self.notebook_root).ok()?;
        let mut components = relative.components();
        let first = components.next()?.as_os_str().to_str()?;
        let segment = NotebookSegment::from_dir_name(first)?;
        let date = changed_path
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)?
            .trim()
            .to_string();
        if date.is_empty() {
            return None;
        }
        Some((segment, date))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NotebookSegment {
    Journal,
    Agenda,
}

impl NotebookSegment {
    fn from_dir_name(value: &str) -> Option<Self> {
        match value {
            "journal" => Some(Self::Journal),
            "agenda" => Some(Self::Agenda),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Journal => "journal",
            Self::Agenda => "agenda",
        }
    }
}
