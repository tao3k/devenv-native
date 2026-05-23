use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

use anyhow::{Context, Result, bail};
use xiuxian_wendao_parsers::{
    EpistemeFileRow, EpistemeSourceManifest, parse_episteme_files_tsv,
    parse_episteme_source_manifest_toml,
};

use super::super::manifest::resolve_ontology_artifact_path;
use super::{
    ids::{repo_relative_path, stable_document_id, stable_id},
    types::{
        EpistemeOntologyStructuralIdfAnchorRow, EpistemeOntologyStructuralIdfDocumentRow,
        EpistemeOntologyStructuralIdfRelationRow, EpistemeOntologyStructuralIdfRequest,
        EpistemeOntologyStructuralIdfSourceContractSummary,
        EpistemeOntologyStructuralIdfValidationMode,
    },
    validation::{
        parent_components, path_depth, source_files_path, validate_file_row, validate_source_file,
        validate_source_manifest,
    },
};

pub(super) struct StructuralIdfBuilder<'a> {
    request: &'a EpistemeOntologyStructuralIdfRequest,
    pub(super) source_contracts: Vec<EpistemeOntologyStructuralIdfSourceContractSummary>,
    pub(super) documents: Vec<EpistemeOntologyStructuralIdfDocumentRow>,
    pub(super) anchors: Vec<EpistemeOntologyStructuralIdfAnchorRow>,
    pub(super) relations: Vec<EpistemeOntologyStructuralIdfRelationRow>,
    seen_file_ids: BTreeSet<String>,
    seen_relative_paths: BTreeSet<String>,
    seen_anchor_ids: BTreeSet<String>,
    seen_relation_ids: BTreeSet<String>,
    folder_anchors: BTreeMap<(String, String), String>,
    order_key: usize,
}

impl<'a> StructuralIdfBuilder<'a> {
    pub(super) fn new(request: &'a EpistemeOntologyStructuralIdfRequest) -> Self {
        Self {
            request,
            source_contracts: Vec::new(),
            documents: Vec::new(),
            anchors: Vec::new(),
            relations: Vec::new(),
            seen_file_ids: BTreeSet::new(),
            seen_relative_paths: BTreeSet::new(),
            seen_anchor_ids: BTreeSet::new(),
            seen_relation_ids: BTreeSet::new(),
            folder_anchors: BTreeMap::new(),
            order_key: 0,
        }
    }

    pub(super) fn compile_source_manifest(
        &mut self,
        domain_id: &str,
        source_manifest_path: &str,
    ) -> Result<()> {
        let manifest_path = resolve_ontology_artifact_path(
            self.request.episteme_root.as_path(),
            source_manifest_path,
            "source_manifests",
        )
        .map_err(|source| anyhow::anyhow!(source))?;
        let raw = fs::read_to_string(manifest_path.as_path())
            .with_context(|| format!("failed to read `{}`", manifest_path.display()))?;
        let manifest = parse_episteme_source_manifest_toml(&raw)
            .with_context(|| format!("failed to parse `{}`", manifest_path.display()))?;
        validate_source_manifest(domain_id, &manifest, source_manifest_path)?;
        let files_path = source_files_path(
            &manifest_path,
            manifest.files.as_str(),
            source_manifest_path,
        )?;
        let files_raw = fs::read_to_string(files_path.as_path())
            .with_context(|| format!("failed to read `{}`", files_path.display()))?;
        let files = parse_episteme_files_tsv(&files_raw)
            .with_context(|| format!("failed to parse `{}`", files_path.display()))?;
        let source_manifest_repo_path = repo_relative_path(
            self.request.episteme_root.as_path(),
            manifest_path.as_path(),
        );
        let files_repo_path =
            repo_relative_path(self.request.episteme_root.as_path(), files_path.as_path());
        let root_anchor_id = self.ensure_source_contract_root_anchor(
            domain_id,
            &manifest,
            source_manifest_repo_path.as_str(),
        );

        for file in &files {
            self.compile_file(
                domain_id,
                &manifest,
                source_manifest_repo_path.as_str(),
                file,
                root_anchor_id.as_str(),
            )?;
        }
        self.source_contracts
            .push(EpistemeOntologyStructuralIdfSourceContractSummary {
                domain_id: domain_id.to_string(),
                source_contract_id: manifest.source_contract_id.clone(),
                source_manifest_path: source_manifest_repo_path,
                files_tsv_path: files_repo_path,
                primary_language: manifest.primary_language.clone(),
                file_count: files.len(),
            });
        Ok(())
    }

    fn ensure_source_contract_root_anchor(
        &mut self,
        domain_id: &str,
        manifest: &EpistemeSourceManifest,
        source_manifest_path: &str,
    ) -> String {
        let anchor_id = stable_id(
            "idf.anchor",
            &format!(
                "{}:{}:root",
                manifest.source_contract_id, source_manifest_path
            ),
        );
        if !self.seen_anchor_ids.insert(anchor_id.clone()) {
            return anchor_id;
        }
        self.order_key += 1;
        self.anchors.push(EpistemeOntologyStructuralIdfAnchorRow {
            anchor_id: anchor_id.clone(),
            anchor_kind: "source_contract_root".to_string(),
            document_id: String::new(),
            file_id: String::new(),
            parent_anchor_id: String::new(),
            domain_id: domain_id.to_string(),
            source_contract_id: manifest.source_contract_id.clone(),
            relative_path: String::new(),
            path_depth: 0,
            order_key: self.order_key,
            language: manifest.primary_language.clone(),
            extraction_route: String::new(),
            source_content_hash: String::new(),
            ontology_truth: false,
            status: "indexed".to_string(),
        });
        anchor_id
    }

    fn compile_file(
        &mut self,
        domain_id: &str,
        manifest: &EpistemeSourceManifest,
        source_manifest_path: &str,
        file: &EpistemeFileRow,
        root_anchor_id: &str,
    ) -> Result<()> {
        validate_file_row(file)?;
        self.validate_unique_file(file)?;
        let source_path = self.request.corpus_root.join(file.relative_path.as_str());
        validate_source_file(source_path.as_path(), file, self.request.validation_mode)?;
        let document_id = stable_document_id(file.file_id.as_str());
        let parent_anchor_id =
            self.ensure_path_anchors(domain_id, manifest, file, root_anchor_id)?;
        let document_anchor_id = stable_id(
            "idf.anchor",
            &format!(
                "{}:{}:document-root",
                manifest.source_contract_id, file.file_id
            ),
        );
        if !self.seen_anchor_ids.insert(document_anchor_id.clone()) {
            bail!(
                "duplicate structural document anchor id for file_id: {}",
                file.file_id
            );
        }
        self.push_document_anchor(
            domain_id,
            manifest,
            file,
            &document_id,
            &parent_anchor_id,
            &document_anchor_id,
        );
        self.add_relation(StructuralRelationInput {
            relation_kind: "contains",
            source_anchor_id: parent_anchor_id.as_str(),
            target_anchor_id: document_anchor_id.as_str(),
            document_id: &document_id,
            file,
            domain_id,
            source_contract_id: manifest.source_contract_id.as_str(),
        });
        self.push_document_row(domain_id, manifest, source_manifest_path, file, document_id);
        Ok(())
    }

    fn validate_unique_file(&mut self, file: &EpistemeFileRow) -> Result<()> {
        if !self.seen_file_ids.insert(file.file_id.clone()) {
            bail!(
                "duplicate file_id in structural IDF source manifests: {}",
                file.file_id
            );
        }
        if !self.seen_relative_paths.insert(file.relative_path.clone()) {
            bail!(
                "duplicate relative_path in structural IDF source manifests: {}",
                file.relative_path
            );
        }
        Ok(())
    }

    fn push_document_anchor(
        &mut self,
        domain_id: &str,
        manifest: &EpistemeSourceManifest,
        file: &EpistemeFileRow,
        document_id: &str,
        parent_anchor_id: &str,
        document_anchor_id: &str,
    ) {
        self.order_key += 1;
        self.anchors.push(EpistemeOntologyStructuralIdfAnchorRow {
            anchor_id: document_anchor_id.to_string(),
            anchor_kind: "document_root".to_string(),
            document_id: document_id.to_string(),
            file_id: file.file_id.clone(),
            parent_anchor_id: parent_anchor_id.to_string(),
            domain_id: domain_id.to_string(),
            source_contract_id: manifest.source_contract_id.clone(),
            relative_path: file.relative_path.clone(),
            path_depth: path_depth(file.relative_path.as_str()),
            order_key: self.order_key,
            language: file.language.clone(),
            extraction_route: file.extraction_route.clone(),
            source_content_hash: file.sha256.clone(),
            ontology_truth: false,
            status: "indexed".to_string(),
        });
    }

    fn push_document_row(
        &mut self,
        domain_id: &str,
        manifest: &EpistemeSourceManifest,
        source_manifest_path: &str,
        file: &EpistemeFileRow,
        document_id: String,
    ) {
        self.documents
            .push(EpistemeOntologyStructuralIdfDocumentRow {
                document_id,
                file_id: file.file_id.clone(),
                domain_id: domain_id.to_string(),
                source_contract_id: manifest.source_contract_id.clone(),
                source_manifest_path: source_manifest_path.to_string(),
                relative_path: file.relative_path.clone(),
                extension: file.extension.clone(),
                byte_size: file.byte_size,
                sha256: file.sha256.clone(),
                category: file.category.clone(),
                language: file.language.clone(),
                extraction_route: file.extraction_route.clone(),
                source_exists: true,
                byte_size_matches: true,
                sha256_matches: match self.request.validation_mode {
                    EpistemeOntologyStructuralIdfValidationMode::MetadataOnly => None,
                    EpistemeOntologyStructuralIdfValidationMode::FullHash => Some(true),
                },
                ontology_truth: false,
                status: "indexed".to_string(),
            });
    }

    fn ensure_path_anchors(
        &mut self,
        domain_id: &str,
        manifest: &EpistemeSourceManifest,
        file: &EpistemeFileRow,
        root_anchor_id: &str,
    ) -> Result<String> {
        let mut parent = root_anchor_id.to_string();
        let mut accumulated = Vec::new();
        for component in parent_components(file.relative_path.as_str())? {
            accumulated.push(component);
            let relative_path = accumulated.join("/");
            let key = (manifest.source_contract_id.clone(), relative_path.clone());
            if let Some(existing) = self.folder_anchors.get(&key) {
                parent.clone_from(existing);
                continue;
            }
            let anchor_id = stable_id(
                "idf.anchor",
                &format!("{}:path:{}", manifest.source_contract_id, relative_path),
            );
            if !self.seen_anchor_ids.insert(anchor_id.clone()) {
                bail!("duplicate structural path anchor id: {anchor_id}");
            }
            self.push_path_anchor(
                domain_id,
                manifest,
                file,
                &parent,
                &relative_path,
                &anchor_id,
            );
            self.add_relation(StructuralRelationInput {
                relation_kind: "contains",
                source_anchor_id: parent.as_str(),
                target_anchor_id: anchor_id.as_str(),
                document_id: "",
                file,
                domain_id,
                source_contract_id: manifest.source_contract_id.as_str(),
            });
            self.folder_anchors.insert(key, anchor_id.clone());
            parent = anchor_id;
        }
        Ok(parent)
    }

    fn push_path_anchor(
        &mut self,
        domain_id: &str,
        manifest: &EpistemeSourceManifest,
        file: &EpistemeFileRow,
        parent_anchor_id: &str,
        relative_path: &str,
        anchor_id: &str,
    ) {
        self.order_key += 1;
        self.anchors.push(EpistemeOntologyStructuralIdfAnchorRow {
            anchor_id: anchor_id.to_string(),
            anchor_kind: "path_segment".to_string(),
            document_id: String::new(),
            file_id: String::new(),
            parent_anchor_id: parent_anchor_id.to_string(),
            domain_id: domain_id.to_string(),
            source_contract_id: manifest.source_contract_id.clone(),
            relative_path: relative_path.to_string(),
            path_depth: relative_path.split('/').count(),
            order_key: self.order_key,
            language: file.language.clone(),
            extraction_route: String::new(),
            source_content_hash: String::new(),
            ontology_truth: false,
            status: "indexed".to_string(),
        });
    }

    fn add_relation(&mut self, input: StructuralRelationInput<'_>) {
        let relation_id = stable_id(
            "idf.relation",
            &format!(
                "{}:{}:{}",
                input.relation_kind, input.source_anchor_id, input.target_anchor_id
            ),
        );
        if !self.seen_relation_ids.insert(relation_id.clone()) {
            return;
        }
        self.order_key += 1;
        self.relations
            .push(EpistemeOntologyStructuralIdfRelationRow {
                relation_id,
                relation_kind: input.relation_kind.to_string(),
                source_anchor_id: input.source_anchor_id.to_string(),
                target_anchor_id: input.target_anchor_id.to_string(),
                document_id: input.document_id.to_string(),
                file_id: input.file.file_id.clone(),
                domain_id: input.domain_id.to_string(),
                source_contract_id: input.source_contract_id.to_string(),
                evidence_path: input.file.relative_path.clone(),
                order_key: self.order_key,
                ontology_truth: false,
                status: "indexed".to_string(),
            });
    }
}

#[derive(Clone, Copy)]
struct StructuralRelationInput<'a> {
    relation_kind: &'a str,
    source_anchor_id: &'a str,
    target_anchor_id: &'a str,
    document_id: &'a str,
    file: &'a EpistemeFileRow,
    domain_id: &'a str,
    source_contract_id: &'a str,
}
