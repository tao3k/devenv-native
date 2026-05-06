use crate::{
    SchemaBenchmarkCase, SchemaBenchmarkEvidence, SchemaBenchmarkReport,
    SchemaBenchmarkReportError, SchemaStrategyCandidate, SchemaStrategyPreference,
};

#[test]
fn candidates_have_stable_identifiers() {
    assert_eq!(
        SchemaStrategyCandidate::ProfileSpecific.as_str(),
        "profile_specific"
    );
    assert_eq!(
        SchemaStrategyCandidate::GlobalSuperSchema.as_str(),
        "global_super_schema"
    );
}

#[test]
fn null_density_uses_basis_points() {
    let evidence = SchemaBenchmarkEvidence::global_super_schema().with_null_density(7_500, 10_000);

    assert_eq!(evidence.null_density_basis_points(), 7_500);
}

#[test]
fn supplied_observations_drive_advisory_preference() {
    let profile = SchemaBenchmarkEvidence::profile_specific()
        .with_validation_cost(10)
        .with_encoded_bytes(16 * 1024)
        .with_null_density(100, 10_000)
        .with_pressure_bytes(8 * 1024, 8 * 1024)
        .with_schema_evolution_cost(50);
    let super_schema = SchemaBenchmarkEvidence::global_super_schema()
        .with_validation_cost(10)
        .with_encoded_bytes(64 * 1024)
        .with_null_density(7_500, 10_000)
        .with_pressure_bytes(64 * 1024, 128 * 1024)
        .with_schema_evolution_cost(10);

    assert_eq!(
        profile.preference_against(super_schema),
        SchemaStrategyPreference::Left
    );
}

#[test]
fn global_super_schema_is_not_privileged_without_evidence() {
    let profile = SchemaBenchmarkEvidence::profile_specific();
    let super_schema = SchemaBenchmarkEvidence::global_super_schema();

    assert_eq!(
        profile.preference_against(super_schema),
        SchemaStrategyPreference::Tie
    );
}

#[test]
fn benchmark_evidence_serializes_candidate_and_costs() -> Result<(), serde_json::Error> {
    let evidence = SchemaBenchmarkEvidence::normalized_long_table()
        .with_row_count(32)
        .with_lossy_projection_count(1);

    let serialized = serde_json::to_string(&evidence)?;

    assert!(serialized.contains("normalized_long_table"));
    assert!(serialized.contains("lossy_projection_count"));
    Ok(())
}

#[test]
fn report_rejects_empty_evidence() {
    let case = SchemaBenchmarkCase::new("doc-table-small", "small document table");

    let Err(error) = SchemaBenchmarkReport::new(case, Vec::new()) else {
        panic!("empty evidence should be rejected");
    };

    assert_eq!(
        error,
        SchemaBenchmarkReportError::EmptyEvidence {
            case_id: "doc-table-small".to_string(),
        }
    );
}

#[test]
fn report_rejects_duplicate_candidates() {
    let case = SchemaBenchmarkCase::new("doc-table-small", "small document table");
    let evidence = vec![
        SchemaBenchmarkEvidence::profile_specific(),
        SchemaBenchmarkEvidence::profile_specific().with_row_count(10),
    ];

    let Err(error) = SchemaBenchmarkReport::new(case, evidence) else {
        panic!("duplicate candidates should be rejected");
    };

    assert_eq!(
        error,
        SchemaBenchmarkReportError::DuplicateCandidate {
            case_id: "doc-table-small".to_string(),
            candidate: SchemaStrategyCandidate::ProfileSpecific,
        }
    );
}

#[test]
fn report_returns_unique_preferred_candidate() -> Result<(), SchemaBenchmarkReportError> {
    let case = SchemaBenchmarkCase::new("memory-profile", "memory profile projection")
        .with_input_size(128, 32 * 1024);
    let report = SchemaBenchmarkReport::new(
        case,
        vec![
            SchemaBenchmarkEvidence::profile_specific()
                .with_validation_cost(10)
                .with_encoded_bytes(16 * 1024),
            SchemaBenchmarkEvidence::global_super_schema()
                .with_validation_cost(20)
                .with_encoded_bytes(128 * 1024)
                .with_null_density(9_000, 10_000),
        ],
    )?;

    assert_eq!(
        report.preferred_candidate(),
        Some(SchemaStrategyCandidate::ProfileSpecific)
    );
    Ok(())
}

#[test]
fn report_returns_no_preference_for_ties() -> Result<(), SchemaBenchmarkReportError> {
    let case = SchemaBenchmarkCase::new("tie-case", "tie case");
    let report = SchemaBenchmarkReport::new(
        case,
        vec![
            SchemaBenchmarkEvidence::profile_specific(),
            SchemaBenchmarkEvidence::global_super_schema(),
        ],
    )?;

    assert_eq!(report.preferred_candidate(), None);
    Ok(())
}
