use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use sha2::{Digest, Sha256};
use xiuxian_wendao::{
    analyzers::PluginRegistry,
    search::{SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService},
};

use crate::studio::router::{GatewayState, StudioState};

pub(super) struct EpistemeGatewayFixture {
    _temp: tempfile::TempDir,
    pub(super) project_root: PathBuf,
    pub(super) config_root: PathBuf,
    pub(super) episteme_root: PathBuf,
    corpus_root: PathBuf,
    files: Vec<FileFixture>,
}

impl EpistemeGatewayFixture {
    pub(super) fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let project_root = temp.path().join("workspace");
        let config_root = project_root.join(".config");
        let episteme_root = project_root.join("source-contract");
        let corpus_root = project_root.join("corpus-root");
        fs::create_dir_all(episteme_root.join("ontology/SourceContract/corpus"))?;
        fs::create_dir_all(episteme_root.join("ontology/SourceContract/mappings"))?;
        fs::create_dir_all(&corpus_root)?;
        fs::create_dir_all(&config_root)?;
        Ok(Self {
            _temp: temp,
            project_root,
            config_root,
            episteme_root,
            corpus_root,
            files: Vec::new(),
        })
    }

    pub(super) fn add_source(
        &mut self,
        relative_path: &str,
        file_id: &str,
        queue_id: &str,
        category: &str,
        route: &str,
        priority: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_path = self.corpus_root.join(relative_path);
        if let Some(parent) = source_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&source_path, format!("fixture content for {relative_path}"))?;
        let metadata = fs::metadata(&source_path)?;
        self.files.push(FileFixture {
            file_id: file_id.to_string(),
            queue_id: queue_id.to_string(),
            relative_path: relative_path.to_string(),
            extension: Path::new(relative_path)
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string(),
            byte_size: metadata.len(),
            sha256: sha256_file(&source_path)?,
            category: category.to_string(),
            route: route.to_string(),
            priority,
        });
        Ok(())
    }

    pub(super) fn write_contract(&self) -> Result<(), Box<dyn std::error::Error>> {
        let corpus_dir = self.episteme_root.join("ontology/SourceContract/corpus");
        fs::write(
            self.episteme_root.join("ontology/manifest.toml"),
            r#"schema_version = 1
name = "synthetic-source-contract"
primary_language = "zh-CN"
artifact_mode = "source_contract"
mutation_allowed = false

[[domains]]
id = "episteme://synthetic/source-contract"
source_manifests = ["SourceContract/corpus/source_manifest.toml"]
mapping_ledgers = ["SourceContract/mappings/corpus_mapping.org"]
"#,
        )?;
        fs::write(
            self.episteme_root
                .join("ontology/SourceContract/mappings/corpus_mapping.org"),
            SYNTHETIC_MAPPING_LEDGER,
        )?;
        fs::write(
            corpus_dir.join("source_manifest.toml"),
            r#"schema_version = 1
source_contract_id = "episteme_source_contract.corpus.v1"
domain = "episteme://synthetic/source-contract"
primary_language = "zh-CN"
corpus_root_env = "WENDAO_SYNTHETIC_EPISTEME_CORPUS_ROOT"
files = "files.tsv"
extraction_queue = "extraction_queue.tsv"
copy_raw_files = false
raw_to_rdf_promotion_allowed = false

ignored_names = [".DS_Store"]

[routes]
document_text_evidence = ["docx", "txt"]
"#,
        )?;

        let mut files_tsv = fs::File::create(corpus_dir.join("files.tsv"))?;
        writeln!(
            files_tsv,
            "file_id\trelative_path\textension\tbyte_size\tsha256\tcategory\tlanguage\textraction_route"
        )?;
        for file in &self.files {
            writeln!(
                files_tsv,
                "{}\t{}\t{}\t{}\t{}\t{}\tzh-CN\t{}",
                file.file_id,
                file.relative_path,
                file.extension,
                file.byte_size,
                file.sha256,
                file.category,
                file.route
            )?;
        }

        let mut queue_tsv = fs::File::create(corpus_dir.join("extraction_queue.tsv"))?;
        writeln!(
            queue_tsv,
            "queue_id\tfile_id\trelative_path\tcategory\tlanguage\textraction_route\tpriority\toutput_contract\tstatus"
        )?;
        for file in &self.files {
            writeln!(
                queue_tsv,
                "{}\t{}\t{}\t{}\tzh-CN\t{}\t{}\tcache_only_no_rdf_promotion\tpending",
                file.queue_id,
                file.file_id,
                file.relative_path,
                file.category,
                file.route,
                file.priority
            )?;
        }
        Ok(())
    }

    pub(super) fn write_contract_extending(
        &self,
        target: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        fs::write(
            self.episteme_root.join("ontology/manifest.toml"),
            format!(
                r#"schema_version = 1
name = "synthetic-source-contract"
primary_language = "zh-CN"
artifact_mode = "source_contract"
mutation_allowed = false

[extends]
manifest = "{target}"

[[domains]]
id = "episteme://synthetic/source-contract"
source_manifests = ["SourceContract/corpus/source_manifest.toml"]
mapping_ledgers = ["SourceContract/mappings/corpus_mapping.org"]
"#
            ),
        )?;
        Ok(())
    }

    pub(super) fn write_common_domain(
        &self,
        root_name: &str,
        domain_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ontology_root = self.project_root.join(root_name).join("ontology");
        fs::create_dir_all(&ontology_root)?;
        fs::write(
            ontology_root.join("manifest.toml"),
            format!(
                r#"schema_version = 1
name = "common-episteme"

[[domains]]
id = "{domain_id}"
"#
            ),
        )?;
        Ok(())
    }

    pub(super) fn write_registry_config(
        &self,
        registry_id: &str,
        path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        fs::write(
            self.config_root.join("wendao.toml"),
            format!(
                r#"[episteme.registries.{registry_id}]
path = "{path}"
"#
            ),
        )?;
        Ok(())
    }

    pub(super) fn write_runtime_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::write(
            self.episteme_root.join("episteme.toml"),
            r#"schema_version = 1

[runtime]
corpus_root = "../corpus-root"
evidence_selection_run_root = "configured-runs/evidence-selection"
extraction_run_root = "configured-runs/extraction"
"#,
        )?;
        Ok(())
    }

    pub(super) fn write_selection_run(
        &self,
        run_id: &str,
        file_ids: &[&str],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let run_dir = self
            .episteme_root
            .join("configured-runs/evidence-selection")
            .join(run_id);
        fs::create_dir_all(&run_dir)?;
        let mut selection_tsv = fs::File::create(run_dir.join("selection.tsv"))?;
        writeln!(
            selection_tsv,
            "selection_index\tfile_id\trelative_path\textension\tbyte_size\tsha256\tcategory\tlanguage\textraction_route\tselection_reason\tnext_action"
        )?;
        for (index, file_id) in file_ids.iter().enumerate() {
            let file = self
                .files
                .iter()
                .find(|file| file.file_id == *file_id)
                .ok_or_else(|| format!("missing fixture file id: {file_id}"))?;
            writeln!(
                selection_tsv,
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\tzh-CN\t{}\tselected\t{}",
                index + 1,
                file.file_id,
                file.relative_path,
                file.extension,
                file.byte_size,
                file.sha256,
                file.category,
                file.route,
                file.route
            )?;
        }
        Ok(())
    }

    pub(super) fn gateway_state(&self) -> Arc<GatewayState> {
        let search_plane = SearchPlaneService::with_paths(
            self.project_root.clone(),
            self.project_root.join(".cache/search-plane"),
            SearchManifestKeyspace::new("xiuxian:test:source-contract-gateway"),
            SearchMaintenancePolicy::default(),
        );
        let studio = StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane(
            Arc::new(PluginRegistry::new()),
            self.project_root.clone(),
            self.config_root.clone(),
            search_plane,
        );
        Arc::new(GatewayState {
            index: None,
            signal_tx: None,
            webhook_url: None,
            studio: Arc::new(studio),
        })
    }
}

struct FileFixture {
    file_id: String,
    queue_id: String,
    relative_path: String,
    extension: String,
    byte_size: u64,
    sha256: String,
    category: String,
    route: String,
    priority: u32,
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

const SYNTHETIC_MAPPING_LEDGER: &str = r"#+TITLE: Synthetic Source Corpus Mapping Ledger

* Synthetic source corpus mapping
:PROPERTIES:
:ID: 16b4038b-2c91-4f70-b38a-e0152629752d
:WENDAO_KIND: ontology_mapping
:ONTOLOGY_KIND: corpus_mapping
:DOMAIN: episteme://synthetic/source-contract
:MAPPING_ID: episteme_source_contract.corpus.v1
:PROMOTION_STATE: candidate
:LIFECYCLE_STATE: candidate
:PRIMARY_LANGUAGE: zh-CN
:END:

This synthetic fixture verifies the source corpus mapping contract shape
without embedding customer source content in Rust tests.

** Corpus coverage

| source_group | evidence_role | extraction_route |
| synthetic_policy_group | synthetic policy evidence | document_text_evidence |
";
