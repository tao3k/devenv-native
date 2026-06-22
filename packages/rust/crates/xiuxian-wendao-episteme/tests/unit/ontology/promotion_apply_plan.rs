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

#[test]
fn ontology_promotion_apply_plan_ignores_poisoned_tsv_projection()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_promotion_review(temp.path(), "approved", "reviewer.example")?;
    fs::write(
        temp.path().join("promotion_review.tsv"),
        "not_the_authority\nthis row must be ignored\n",
    )?;

    let request = EpistemeOntologyPromotionApplyPlanRequest::new(temp.path());
    let report = write_episteme_ontology_promotion_apply_plan(&request)?;

    assert_eq!(report.approved_count, 1);
    assert_eq!(report.apply_plan_row_count, 1);
    Ok(())
}

fn write_promotion_review(
    root: &std::path::Path,
    first_decision: &str,
    reviewer_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        root.join("promotion_review.org"),
        format!(
            "#+TITLE: Private Ontology Promotion Review Packet\n\
             \n\
             * Promotion review packet\n\
             :PROPERTIES:\n\
             :WENDAO_KIND: ontology_promotion_review_packet\n\
             :ONTOLOGY_KIND: dataset_mapping\n\
             :LIFECYCLE_STATE: review\n\
             :PROMOTION_STATE: pending_review\n\
             :SOURCE_MUTATION_ALLOWED: false\n\
             :ONTOLOGY_TRUTH: false\n\
             :END:\n\
             \n\
             | record_id | record_kind | label | suggested_term_key | review_decision | quality_score | evidence_strength | issue_codes | promotion_precondition_met | source_file_id | source_queue_id | extraction_run_id | relation_source_candidate_id | relation_target_candidate_id | promotion_decision | source_mutation_allowed | ontology_truth | reviewer_id | reviewer_note |\n\
             | candidate.term | ontology_candidate.object_term | Policy Term | policy.document | ready_for_review | 80 | mapping_ledger |  | true |  |  |  |  |  | pending_review | false | false |  |  |\n\
             | relation.source.term | ontology_candidate.source_artifact.suggested_object_type |  |  | ready_for_review | 75 | hash_provenance |  | true | file.source | queue.source |  | candidate.source | candidate.term | {first_decision} | false | false | {reviewer_id} | approved by test reviewer |\n",
        ),
    )?;
    Ok(())
}
