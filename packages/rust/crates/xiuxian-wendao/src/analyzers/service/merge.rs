use std::collections::BTreeSet;
use std::path::Path;

use crate::analyzers::DocRecord;
use crate::analyzers::RegisteredRepository;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::RepositoryRecord;
use xiuxian_git_repo::LocalCheckoutMetadata;

pub(super) fn merge_repository_analysis(
    base: &mut RepositoryAnalysisOutput,
    mut overlay: RepositoryAnalysisOutput,
) {
    match (base.repository.take(), overlay.repository.take()) {
        (None, None) => {}
        (Some(base_record), None) => {
            base.repository = Some(base_record);
        }
        (None, Some(overlay_record)) => {
            base.repository = Some(overlay_record);
        }
        (Some(base_record), Some(overlay_record)) => {
            base.repository = Some(merge_repository_record(base_record, overlay_record));
        }
    }
    base.modules.append(&mut overlay.modules);
    base.symbols.append(&mut overlay.symbols);
    base.imports.append(&mut overlay.imports);
    base.examples.append(&mut overlay.examples);
    append_unique_docs_by_id(&mut base.docs, &mut overlay.docs);
    base.relations.append(&mut overlay.relations);
    base.diagnostics.append(&mut overlay.diagnostics);
}

fn append_unique_docs_by_id(base: &mut Vec<DocRecord>, overlay: &mut Vec<DocRecord>) {
    let mut docs = Vec::with_capacity(base.len() + overlay.len());
    docs.append(base);
    docs.append(overlay);

    let mut seen = BTreeSet::new();
    docs.retain(|doc| seen.insert(doc.doc_id.to_string()));
    *base = docs;
}

pub(super) fn merge_repository_record(
    base: RepositoryRecord,
    overlay: RepositoryRecord,
) -> RepositoryRecord {
    RepositoryRecord {
        repo_id: if base.repo_id.is_empty() {
            overlay.repo_id
        } else {
            base.repo_id
        },
        name: if base.name.is_empty() {
            overlay.name
        } else {
            base.name
        },
        path: if base.path.is_empty() {
            overlay.path
        } else {
            base.path
        },
        url: base.url.or(overlay.url),
        revision: base.revision.or(overlay.revision),
        version: base.version.or(overlay.version),
        uuid: base.uuid.or(overlay.uuid),
        dependencies: if base.dependencies.is_empty() {
            overlay.dependencies
        } else {
            base.dependencies
        },
    }
}

pub(super) fn hydrate_repository_record(
    record: &mut RepositoryRecord,
    repository: &RegisteredRepository,
    repository_root: &Path,
    checkout_metadata: Option<&LocalCheckoutMetadata>,
) {
    if record.repo_id.trim().is_empty() {
        record.repo_id = repository.id.clone().into();
    }
    if record.name.trim().is_empty() {
        record.name.clone_from(&repository.id);
    }
    if record.path.trim().is_empty() {
        record.path = repository_root.display().to_string().into();
    }
    if record.url.is_none() {
        record.url = repository
            .url
            .clone()
            .or_else(|| checkout_metadata.and_then(|metadata| metadata.remote_url.clone()));
    }
    if record.revision.is_none() {
        record.revision = checkout_metadata.and_then(|metadata| metadata.revision.clone());
    }
}
