use std::{
    fs,
    io::Cursor,
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use arrow::array::{BooleanArray, Float64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;

use super::{
    SearchStrategyFlowCandidateInput, SearchStrategyFlowCandidateInputBatch,
    run_wendaograph_search_strategy_flow_json_with_candidate_batch,
    run_wendaograph_search_strategy_flow_json_with_candidate_batch_and_branch_judgements,
    search_strategy_flow_candidate_input_batch,
};

#[test]
fn search_strategy_flow_rust_bridge_reserves_required_evidence_frontier() {
    let candidates = vec![
        candidate(
            "packages/rust/crates/xiuxian-julia-core/README.md",
            "76-searchstrategyflow-flight-materialization-now-keeps-route-namespaces",
            "SearchStrategyFlow Flight materialization",
            (76, 90),
            (1.0, 1.0, 0.97, 0.98, 0.02),
            &["search-strategy", "authority"],
        ),
        candidate(
            "docs/rfcs/2026-03-26-wendao-query-engine-rfc.md",
            "735-the-goal-is-not-to-freeze-an-api-immediately-the-goal-is-to-establish-the-ownership-boundary-between",
            "Ownership boundary",
            (735, 760),
            (0.96, 0.95, 0.93, 0.92, 0.05),
            &["authority", "ownership"],
        ),
        candidate(
            "docs/testing/README.md",
            "89-default-validation-path-both-local-just-validate-and-just-ci",
            "Default validation path",
            (89, 105),
            (0.95, 0.94, 0.90, 0.91, 0.05),
            &["validation", "package-test"],
        ),
        candidate(
            "packages/rust/crates/xiuxian-julia-core/tests/unit/integration_support/wendaograph/search_strategy.rs",
            "77-search-strategy-flow-link-graph-python-julia-toml",
            "Search strategy flow LinkGraph path",
            (77, 120),
            (0.94, 0.93, 0.88, 0.90, 0.06),
            &["link-graph", "relation"],
        ),
    ];
    let batch = candidate_batch(&candidates);

    let trace = run_wendaograph_search_strategy_flow_json_with_candidate_batch(
        "find the SearchStrategyFlow ownership boundary, validation path, and relation path",
        ".",
        batch,
    )
    .unwrap_or_else(|error| panic!("run required evidence frontier bridge trace: {error}"));
    let trace: serde_json::Value = serde_json::from_str(&trace)
        .unwrap_or_else(|error| panic!("parse required evidence frontier trace: {error}"));
    let validation = trace
        .get("validation")
        .unwrap_or_else(|| panic!("validation object must exist"));

    assert_eq!(
        validation
            .get("requiredEvidenceCovered")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        validation.get("selectedRequiredEvidence"),
        Some(&serde_json::json!([
            "ownership_boundary",
            "validation_path",
            "relation_path"
        ]))
    );
    assert_eq!(
        validation.get("missingRequiredEvidence"),
        Some(&serde_json::json!([]))
    );
    assert!(
        trace
            .get("frontier")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .any(|row| {
                row.get("selected").and_then(serde_json::Value::as_bool) == Some(true)
                    && row
                        .get("candidateId")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|candidate_id| {
                            candidate_id
                                .starts_with("docs/testing/README.md#89-default-validation-path")
                        })
            }),
        "validation-path candidate must be selected by the required-evidence frontier"
    );
}

#[test]
fn search_strategy_flow_rust_bridge_applies_agent_branch_judgements() {
    let candidates = vec![
        candidate(
            "docs/a.md",
            "owner",
            "Owner branch",
            (1, 12),
            (0.88, 0.80, 0.60, 0.80, 0.10),
            &["general"],
        ),
        candidate(
            "docs/b.md",
            "validation",
            "Validation branch",
            (13, 24),
            (0.86, 0.78, 0.60, 0.78, 0.10),
            &["general"],
        ),
        candidate(
            "docs/c.md",
            "relation",
            "Relation branch",
            (25, 36),
            (0.84, 0.76, 0.60, 0.76, 0.10),
            &["general"],
        ),
        candidate(
            "docs/d.md",
            "blocked",
            "Blocked branch",
            (37, 48),
            (0.90, 0.90, 0.90, 0.90, 0.02),
            &["general"],
        ),
    ];
    let batch = candidate_batch(&candidates);
    let branch_judgements = BranchJudgementsArrowFile::new(&[
        BranchJudgementRow::new(
            "docs/a.md#owner",
            "authority",
            0.95,
            "keep",
            false,
            "Agent judged ownership boundary evidence.",
        ),
        BranchJudgementRow::new(
            "docs/b.md#validation",
            "validation",
            0.94,
            "keep",
            false,
            "Agent judged validation path evidence.",
        ),
        BranchJudgementRow::new(
            "docs/c.md#relation",
            "link_graph",
            0.93,
            "keep",
            false,
            "Agent judged relation path evidence.",
        ),
        BranchJudgementRow::new(
            "docs/d.md#blocked",
            "search_strategy",
            0.10,
            "reject",
            true,
            "Agent rejected this branch.",
        ),
    ]);

    let trace =
        run_wendaograph_search_strategy_flow_json_with_candidate_batch_and_branch_judgements(
            "find the SearchStrategyFlow ownership boundary and validation path",
            ".",
            batch,
            branch_judgements.path_str(),
        )
        .unwrap_or_else(|error| panic!("run branch judgement frontier bridge trace: {error}"));
    let trace: serde_json::Value = serde_json::from_str(&trace)
        .unwrap_or_else(|error| panic!("parse branch judgement frontier trace: {error}"));
    let selected_ids = trace
        .get("frontier")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter(|row| row.get("selected").and_then(serde_json::Value::as_bool) == Some(true))
        .filter_map(|row| row.get("candidateId").and_then(serde_json::Value::as_str))
        .collect::<Vec<_>>();

    assert!(selected_ids.contains(&"docs/a.md#owner"));
    assert!(selected_ids.contains(&"docs/b.md#validation"));
    assert!(selected_ids.contains(&"docs/c.md#relation"));
    assert!(!selected_ids.contains(&"docs/d.md#blocked"));
    assert_eq!(
        trace
            .get("validation")
            .and_then(|validation| validation.get("requiredEvidenceCovered"))
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
}

struct BranchJudgementRow<'a> {
    candidate_id: &'a str,
    branch_role: &'a str,
    judgement_score: f64,
    decision: &'a str,
    blocked: bool,
    reason: &'a str,
}

impl<'a> BranchJudgementRow<'a> {
    fn new(
        candidate_id: &'a str,
        branch_role: &'a str,
        judgement_score: f64,
        decision: &'a str,
        blocked: bool,
        reason: &'a str,
    ) -> Self {
        Self {
            candidate_id,
            branch_role,
            judgement_score,
            decision,
            blocked,
            reason,
        }
    }
}

struct BranchJudgementsArrowFile {
    dir: PathBuf,
    path_string: String,
}

impl BranchJudgementsArrowFile {
    fn new(rows: &[BranchJudgementRow<'_>]) -> Self {
        let dir = unique_test_dir("branch-judgements");
        fs::create_dir(&dir)
            .unwrap_or_else(|error| panic!("create branch judgement Arrow temp dir: {error}"));
        let path = dir.join("payload.arrow");
        fs::write(&path, branch_judgements_arrow_ipc(rows))
            .unwrap_or_else(|error| panic!("write branch judgement Arrow IPC: {error}"));
        let path_string = path.to_string_lossy().into_owned();
        Self { dir, path_string }
    }

    fn path_str(&self) -> &str {
        self.path_string.as_str()
    }
}

impl Drop for BranchJudgementsArrowFile {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn candidate(
    relative_path: &str,
    heading_anchor: &str,
    title: &str,
    line_range: (usize, usize),
    scores: (f64, f64, f64, f64, f64),
    edge_kinds: &[&str],
) -> SearchStrategyFlowCandidateInput {
    let (line_start, line_end) = line_range;
    let (evidence_coverage, graph_score, authority_score, structural_score, uncertainty) = scores;
    SearchStrategyFlowCandidateInput {
        relative_path: relative_path.to_owned(),
        heading_anchor: heading_anchor.to_owned(),
        title: title.to_owned(),
        line_start,
        line_end,
        context_cost: 8,
        evidence_coverage,
        graph_score,
        authority_score,
        structural_score,
        uncertainty,
        blocked: false,
        edge_kinds: edge_kinds.iter().map(|kind| (*kind).to_owned()).collect(),
    }
}

fn candidate_batch(
    candidates: &[SearchStrategyFlowCandidateInput],
) -> SearchStrategyFlowCandidateInputBatch {
    search_strategy_flow_candidate_input_batch("rust-code-intelligence-inventory", candidates)
        .unwrap_or_else(|error| panic!("build required-evidence Arrow candidate batch: {error}"))
}

fn branch_judgements_arrow_ipc(rows: &[BranchJudgementRow<'_>]) -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("flow_id", DataType::Utf8, false),
        Field::new("candidate_id", DataType::Utf8, false),
        Field::new("branch_role", DataType::Utf8, false),
        Field::new("judgement_score", DataType::Float64, false),
        Field::new("confidence", DataType::Float64, false),
        Field::new("decision", DataType::Utf8, false),
        Field::new("blocked", DataType::Boolean, false),
        Field::new("reason", DataType::Utf8, false),
    ]));
    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(vec![
                "pi-wendao-search-strategy-flow";
                rows.len()
            ])),
            Arc::new(StringArray::from(
                rows.iter().map(|row| row.candidate_id).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter().map(|row| row.branch_role).collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter()
                    .map(|row| row.judgement_score)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(vec![0.9; rows.len()])),
            Arc::new(StringArray::from(
                rows.iter().map(|row| row.decision).collect::<Vec<_>>(),
            )),
            Arc::new(BooleanArray::from(
                rows.iter().map(|row| row.blocked).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter().map(|row| row.reason).collect::<Vec<_>>(),
            )),
        ],
    )
    .unwrap_or_else(|error| panic!("build branch judgement Arrow batch: {error}"));
    let mut writer = StreamWriter::try_new(Cursor::new(Vec::new()), batch.schema().as_ref())
        .unwrap_or_else(|error| panic!("build branch judgement Arrow writer: {error}"));
    writer
        .write(&batch)
        .unwrap_or_else(|error| panic!("write branch judgement Arrow batch: {error}"));
    writer
        .finish()
        .unwrap_or_else(|error| panic!("finish branch judgement Arrow stream: {error}"));
    match writer.into_inner() {
        Ok(cursor) => cursor.into_inner(),
        Err(error) => panic!("finalize branch judgement Arrow stream: {error}"),
    }
}

fn unique_test_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|error| panic!("resolve test time: {error}"))
        .as_nanos();
    std::env::temp_dir().join(format!(
        "xiuxian-search-strategy-flow-test-{label}-{}-{nanos}",
        std::process::id()
    ))
}
