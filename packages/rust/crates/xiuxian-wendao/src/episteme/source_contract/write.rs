//! Episteme extraction run-plan artifact writer.
//!
//! This module persists cache-only `tasks.tsv` and receipt JSON artifacts from
//! validated source-contract run plans without executing extraction.

use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use serde::Serialize;

use super::{
    EpistemeError, EpistemeRunPlanReceipt, EpistemeRunPlanRequest, EpistemeRunTask,
    plan_episteme_extraction_run,
};

const WRITE_REPORT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_source_contract_run_plan_write_report.v1";
const TASKS_TSV: &str = "tasks.tsv";
const RECEIPT_JSON: &str = "receipt.json";
const OUTPUTS_DIR: &str = "outputs";

/// Report emitted after writing a episteme source-contract extraction run plan.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct EpistemeRunPlanWriteReport {
    /// Report schema version.
    pub schema_version: &'static str,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Concrete run directory.
    pub run_dir: PathBuf,
    /// Written tasks TSV path.
    pub tasks_path: PathBuf,
    /// Written receipt JSON path.
    pub receipt_path: PathBuf,
    /// Created outputs directory.
    pub outputs_dir: PathBuf,
    /// Total queue rows available before filtering.
    pub total_queue_rows: usize,
    /// Number of selected tasks.
    pub selected_count: usize,
    /// Selected row counts by route.
    pub route_counts: BTreeMap<String, usize>,
    /// Selected row counts by category.
    pub category_counts: BTreeMap<String, usize>,
    /// Whether extraction ran during planning.
    pub extraction_executed: bool,
    /// Whether direct RDF promotion is allowed.
    pub raw_to_rdf_promotion_allowed: bool,
    /// Validation mode used during planning.
    pub validation_mode: &'static str,
}

/// Write a deterministic episteme source-contract extraction run plan from Rust.
///
/// # Errors
///
/// Returns an error when source validation/planning fails, or when the target
/// run-plan files cannot be written.
pub fn write_episteme_extraction_run_plan(
    request: &EpistemeRunPlanRequest,
    run_root: impl AsRef<Path>,
) -> Result<EpistemeRunPlanWriteReport, EpistemeError> {
    let receipt = plan_episteme_extraction_run(request)?;
    let run_dir = run_root.as_ref().join(&receipt.run_id);
    let outputs_dir = run_dir.join(OUTPUTS_DIR);
    let tasks_path = run_dir.join(TASKS_TSV);
    let receipt_path = run_dir.join(RECEIPT_JSON);

    create_dir_all(&outputs_dir)?;
    write_tasks_tsv(&tasks_path, &receipt.tasks)?;
    write_receipt_json(&receipt_path, &receipt)?;

    Ok(EpistemeRunPlanWriteReport {
        schema_version: WRITE_REPORT_SCHEMA_VERSION,
        run_id: receipt.run_id,
        run_dir,
        tasks_path,
        receipt_path,
        outputs_dir,
        total_queue_rows: receipt.total_queue_rows,
        selected_count: receipt.selected_count,
        route_counts: receipt.route_counts,
        category_counts: receipt.category_counts,
        extraction_executed: receipt.extraction_executed,
        raw_to_rdf_promotion_allowed: receipt.raw_to_rdf_promotion_allowed,
        validation_mode: receipt.validation_mode,
    })
}

fn create_dir_all(path: &Path) -> Result<(), EpistemeError> {
    fs::create_dir_all(path).map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn write_tasks_tsv(path: &Path, tasks: &[EpistemeRunTask]) -> Result<(), EpistemeError> {
    let mut file = fs::File::create(path).map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    writeln!(
        file,
        "queue_id\tfile_id\trelative_path\tcategory\tlanguage\textraction_route\tpriority\tsource_sha256\tplanned_output_path\toutput_contract\tstatus"
    )
    .map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    for task in tasks {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            task.queue_id,
            task.file_id,
            task.relative_path,
            task.category,
            task.language,
            task.extraction_route,
            task.priority,
            task.source_sha256,
            task.planned_output_path,
            task.output_contract,
            task.status
        )
        .map_err(|source| EpistemeError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    }
    Ok(())
}

fn write_receipt_json(path: &Path, receipt: &EpistemeRunPlanReceipt) -> Result<(), EpistemeError> {
    let raw = serde_json::to_string_pretty(receipt).map_err(|source| EpistemeError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    fs::write(path, format!("{raw}\n")).map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })
}
