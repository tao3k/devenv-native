use std::fs;

use xiuxian_wendao_episteme::{
    EpistemeOntologyPromotionApplyPlanRequest, write_episteme_ontology_promotion_apply_plan,
};

#[test]
fn ontology_promotion_apply_plan_writes_pending_only_noop_plan()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_promotion_review(temp.path(), "pending_review", "")?;

    let request = EpistemeOntologyPromotionApplyPlanRequest::new(temp.path());
    let report = write_episteme_ontology_promotion_apply_plan(&request)?;

    assert_eq!(report.promotion_review_row_count, 2);
    assert_eq!(report.pending_review_count, 2);
    assert_eq!(report.approved_count, 0);
    assert_eq!(report.apply_plan_row_count, 0);
    assert!(!report.source_mutation_allowed);
    assert!(!report.ontology_truth);

    let plan_tsv = fs::read_to_string(&report.promotion_apply_plan_tsv)?;
    assert!(plan_tsv.contains("apply_action"));
    assert_eq!(plan_tsv.lines().count(), 1);

    let plan_org = fs::read_to_string(&report.promotion_apply_plan_org)?;
    assert!(plan_org.contains(":WENDAO_KIND: ontology_promotion_apply_plan"));
    assert!(plan_org.contains("| approved_count | 0 |"));

    let plan_json = fs::read_to_string(&report.promotion_apply_plan_json)?;
    assert!(plan_json.contains("\"sourceMutationAllowed\": false"));
    assert!(plan_json.contains("\"ontologyTruth\": false"));
    assert!(!plan_json.contains("raw private text"));
    Ok(())
}

#[test]
fn ontology_promotion_apply_plan_blocks_approval_without_reviewer()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_promotion_review(temp.path(), "approved", "")?;

    let request = EpistemeOntologyPromotionApplyPlanRequest::new(temp.path());
    let Err(error) = write_episteme_ontology_promotion_apply_plan(&request) else {
        return Err("approved rows without reviewer provenance must be rejected".into());
    };

    assert!(error.to_string().contains("requires reviewer provenance"));
    assert!(!temp.path().join("promotion_apply_plan.tsv").exists());
    assert!(!temp.path().join("promotion_apply_plan.org").exists());
    assert!(!temp.path().join("promotion_apply_plan.json").exists());
    Ok(())
}

#[test]
fn ontology_promotion_apply_plan_writes_approved_plan_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_promotion_review(temp.path(), "approved", "reviewer.example")?;

    let request = EpistemeOntologyPromotionApplyPlanRequest::new(temp.path());
    let report = write_episteme_ontology_promotion_apply_plan(&request)?;

    assert_eq!(report.pending_review_count, 1);
    assert_eq!(report.approved_count, 1);
    assert_eq!(report.apply_plan_row_count, 1);

    let plan_tsv = fs::read_to_string(&report.promotion_apply_plan_tsv)?;
    assert!(plan_tsv.contains("propose_source_patch"));
    assert!(plan_tsv.contains("reviewer.example"));
    assert!(plan_tsv.contains("candidate.source"));
    assert!(plan_tsv.contains("candidate.term"));
    assert!(plan_tsv.contains("\tfalse\tfalse"));
    Ok(())
}

fn write_promotion_review(
    root: &std::path::Path,
    first_decision: &str,
    reviewer_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        root.join("promotion_review.tsv"),
        format!(
            "record_id\trecord_kind\tlabel\tsuggested_term_key\treview_decision\tquality_score\tevidence_strength\tissue_codes\tpromotion_precondition_met\tsource_file_id\tsource_queue_id\textraction_run_id\trelation_source_candidate_id\trelation_target_candidate_id\tpromotion_decision\tsource_mutation_allowed\tontology_truth\treviewer_id\treviewer_note\ncandidate.term\tontology_candidate.object_term\tPolicy Term\tpolicy.document\tready_for_review\t80\tmapping_ledger\t\ttrue\t\t\t\t\t\tpending_review\tfalse\tfalse\t\t\nrelation.source.term\tontology_candidate.source_artifact.suggested_object_type\t\t\tready_for_review\t75\thash_provenance\t\ttrue\tfile.source\tqueue.source\t\tcandidate.source\tcandidate.term\t{first_decision}\tfalse\tfalse\t{reviewer_id}\tapproved by test reviewer\n",
        ),
    )?;
    Ok(())
}
