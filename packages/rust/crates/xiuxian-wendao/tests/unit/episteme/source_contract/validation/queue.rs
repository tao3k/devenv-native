use super::support::{EpistemeFixture, EpistemeRunPlanRequest, fs, plan_episteme_extraction_run};

#[test]
fn episteme_extraction_plan_shape_validation_rejects_queue_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = EpistemeFixture::new()?;
    fixture.add_source(
        "docs/a.txt",
        "episteme.file.a",
        "episteme.extract.a",
        "synthetic_policy_category",
        "document_text_evidence",
        10,
    )?;
    fixture.write_contract()?;
    fs::write(
        fixture
            .episteme_root
            .join("ontology/SourceContract/corpus/extraction_queue.tsv"),
        "queue_id\tfile_id\trelative_path\tcategory\tlanguage\textraction_route\tpriority\toutput_contract\tstatus\n",
    )?;

    let Err(error) = plan_episteme_extraction_run(
        &EpistemeRunPlanRequest::new(
            &fixture.episteme_root,
            &fixture.corpus_root,
            "queue_shape_plan_seed",
        )
        .with_limit(1),
    ) else {
        return Err("queue mismatch should fail shape-only planning".into());
    };
    assert!(
        error
            .to_string()
            .contains("extraction_queue.tsv missing file_id")
    );

    Ok(())
}
