//! Repository loading for semantic `SSOT` artifact roots.

use super::artifact::{
    parse_semantic_change_intent, parse_semantic_object, parse_semantic_projection,
};
use super::validate::validate_repository;
use crate::semantic_ssot::types::{SemanticRepository, SemanticValidationReport};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Load and validate one semantic repository root.
#[must_use]
pub fn load_semantic_repository(root: impl AsRef<Path>) -> SemanticRepository {
    let root = root.as_ref().to_path_buf();
    let mut repository = SemanticRepository {
        root: root.clone(),
        objects: Vec::new(),
        projections: Vec::new(),
        change_intents: Vec::new(),
        report: SemanticValidationReport::default(),
    };

    if !root.exists() {
        repository.report.push(
            None,
            format!("semantic root `{}` does not exist", root.display()),
        );
        return repository;
    }

    load_objects(&root, &mut repository);
    load_projections(&root, &mut repository);
    load_change_intents(&root, &mut repository);
    validate_repository(&mut repository);
    repository
}

fn load_objects(root: &Path, repository: &mut SemanticRepository) {
    let objects_root = root.join("objects");
    if !objects_root.exists() {
        repository.report.push(
            Some(PathBuf::from("objects")),
            "semantic objects directory is missing",
        );
        return;
    }

    for entry in WalkDir::new(&objects_root) {
        let Ok(entry) = entry else {
            repository.report.push(
                Some(PathBuf::from("objects")),
                "failed to read semantic object entry",
            );
            continue;
        };
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|value| value.to_str()) != Some("md")
        {
            continue;
        }
        let relative_path = relative_path(root, entry.path());
        match fs::read_to_string(entry.path()) {
            Ok(content) => match parse_semantic_object(&relative_path, &content) {
                Ok(object) => repository.objects.push(object),
                Err(error) => repository.report.push(
                    Some(relative_path),
                    format!("failed to parse semantic object: {error}"),
                ),
            },
            Err(error) => repository.report.push(
                Some(relative_path),
                format!("failed to read semantic object: {error}"),
            ),
        }
    }
}

fn load_projections(root: &Path, repository: &mut SemanticRepository) {
    let projections_root = root.join("projections");
    if !projections_root.exists() {
        repository.report.push(
            Some(PathBuf::from("projections")),
            "semantic projections directory is missing",
        );
        return;
    }

    for entry in WalkDir::new(&projections_root) {
        let Ok(entry) = entry else {
            repository.report.push(
                Some(PathBuf::from("projections")),
                "failed to read semantic projection entry",
            );
            continue;
        };
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|value| value.to_str()) != Some("md")
        {
            continue;
        }
        let relative_path = relative_path(root, entry.path());
        match fs::read_to_string(entry.path()) {
            Ok(content) => match parse_semantic_projection(&relative_path, &content) {
                Ok(projection) => repository.projections.push(projection),
                Err(error) => repository.report.push(
                    Some(relative_path),
                    format!("failed to parse semantic projection: {error}"),
                ),
            },
            Err(error) => repository.report.push(
                Some(relative_path),
                format!("failed to read semantic projection: {error}"),
            ),
        }
    }
}

fn load_change_intents(root: &Path, repository: &mut SemanticRepository) {
    let intents_root = root.join("change-intents");
    if !intents_root.exists() {
        return;
    }

    for entry in WalkDir::new(&intents_root) {
        let Ok(entry) = entry else {
            repository.report.push(
                Some(PathBuf::from("change-intents")),
                "failed to read semantic change-intent entry",
            );
            continue;
        };
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|value| value.to_str()) != Some("md")
        {
            continue;
        }
        let relative_path = relative_path(root, entry.path());
        match fs::read_to_string(entry.path()) {
            Ok(content) => match parse_semantic_change_intent(&relative_path, &content) {
                Ok(intent) => repository.change_intents.push(intent),
                Err(error) => repository.report.push(
                    Some(relative_path),
                    format!("failed to parse semantic change intent: {error}"),
                ),
            },
            Err(error) => repository.report.push(
                Some(relative_path),
                format!("failed to read semantic change intent: {error}"),
            ),
        }
    }
}

fn relative_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root)
        .map_or_else(|_| path.to_path_buf(), std::path::Path::to_path_buf)
}
