use std::path::Path;

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
    base.docs.append(&mut overlay.docs);
    base.relations.append(&mut overlay.relations);
    base.diagnostics.append(&mut overlay.diagnostics);
}

pub(super) fn merge_repository_record(
    base: RepositoryRecord,
    overlay: RepositoryRecord,
) -> RepositoryRecord {
    RepositoryRecord {
        repo_id: if overlay.repo_id.is_empty() {
            base.repo_id
        } else {
            overlay.repo_id
        },
        name: if overlay.name.is_empty() {
            base.name
        } else {
            overlay.name
        },
        path: if overlay.path.is_empty() {
            base.path
        } else {
            overlay.path
        },
        url: overlay.url.or(base.url),
        revision: overlay.revision.or(base.revision),
        version: overlay.version.or(base.version),
        uuid: overlay.uuid.or(base.uuid),
        dependencies: if overlay.dependencies.is_empty() {
            base.dependencies
        } else {
            overlay.dependencies
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
