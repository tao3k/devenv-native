use std::env;
use std::io;
use std::process::{Command, Stdio};

use arrow::array::{Array, StringArray};
use tempfile::tempdir;
use xiuxian_wendao_core::transport::PluginTransportKind;
use xiuxian_wendao_parsers::semantic_ssot::load_semantic_repository;
use xiuxian_wendao_sql::semantic_read_model::build_semantic_read_model_record_batches;

use crate::integration_support::service_runtime::{
    JuliaServiceGuard, reserve_service_port, wait_for_service_ready_with_attempts,
};
use crate::integration_support::wendaograph::wendaograph_julia_project;

use super::support::write_semantic_read_model_fixture;
use super::{
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ROUTE,
    WendaoGraphOntologyReadModelQualityFlightBindingOptions,
    WendaoGraphOntologyReadModelQualityRequestBatches,
    build_wendaograph_ontology_read_model_quality_flight_binding,
    roundtrip_wendaograph_ontology_read_model_quality_with_binding,
};

const RUN_LIVE_LOOPBACK_ENV: &str = "RUN_WENDAOGRAPH_ONTOLOGY_QUALITY_LIVE_LOOPBACK_TEST";

#[tokio::test]
async fn ontology_read_model_quality_live_loopback_uses_real_wendaograph_service() -> io::Result<()>
{
    if env::var_os(RUN_LIVE_LOOPBACK_ENV).is_none() {
        eprintln!("skipping live WendaoGraph ontology loopback; set {RUN_LIVE_LOOPBACK_ENV}=1");
        return Ok(());
    }

    let request_batches = sql_materialized_request_batches()?;
    let project = wendaograph_julia_project().map_err(io::Error::other)?;
    let runner = project
        .join("scripts")
        .join("run_ontology_read_model_quality_service.jl");
    if !runner.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "missing WendaoGraph ontology quality runner `{}`",
                runner.display()
            ),
        ));
    }

    let port = reserve_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let mut guard = JuliaServiceGuard::new(
        Command::new("julia")
            .arg(format!("--project={}", project.display()))
            .arg(&runner)
            .arg("--host=127.0.0.1")
            .arg(format!("--port={port}"))
            .arg("--max-active-requests=4")
            .arg("--request-capacity=4")
            .arg("--response-capacity=4")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()?,
    );

    wait_for_service_ready_with_attempts(&base_url, 300)
        .await
        .map_err(io::Error::other)?;
    let binding = build_wendaograph_ontology_read_model_quality_flight_binding(
        WendaoGraphOntologyReadModelQualityFlightBindingOptions {
            base_url,
            health_route: None,
            timeout_secs: Some(30),
            max_in_flight_requests: Some(1),
        },
    )
    .map_err(io::Error::other)?;
    let roundtrip =
        roundtrip_wendaograph_ontology_read_model_quality_with_binding(&binding, &request_batches)
            .await
            .map_err(|error| io::Error::other(format!("{error:?}")))?
            .unwrap_or_else(|| {
                panic!("live ontology quality Flight binding should negotiate a runtime transport")
            });

    assert_eq!(
        roundtrip.selection.selected_transport,
        PluginTransportKind::ArrowFlight
    );
    assert_eq!(roundtrip.selection.fallback_from, None);
    assert_eq!(
        binding.endpoint.route.as_deref(),
        Some(WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ROUTE)
    );
    assert_eq!(roundtrip.response_batches.len(), 1);
    assert!(roundtrip.response_batches[0].num_rows() > 0);
    assert_response_contains_quality_check(&roundtrip.response_batches[0]);

    guard.kill();
    Ok(())
}

fn sql_materialized_request_batches()
-> io::Result<WendaoGraphOntologyReadModelQualityRequestBatches> {
    let temp = tempdir()?;
    write_semantic_read_model_fixture(temp.path())?;
    let repository = load_semantic_repository(temp.path());
    let sql_batches =
        build_semantic_read_model_record_batches(&repository).map_err(io::Error::other)?;

    Ok(WendaoGraphOntologyReadModelQualityRequestBatches::new(
        sql_batches.objects,
        sql_batches.relations,
        sql_batches.projection_state,
    ))
}

fn assert_response_contains_quality_check(batch: &arrow::record_batch::RecordBatch) {
    let check_ids = batch
        .column_by_name("check_id")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .unwrap_or_else(|| panic!("response check_id column should decode as Utf8"));
    let statuses = batch
        .column_by_name("status")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .unwrap_or_else(|| panic!("response status column should decode as Utf8"));

    assert!(
        (0..check_ids.len()).any(|index| check_ids.value(index) == "object_graph_component_count")
    );
    assert!((0..statuses.len()).any(|index| statuses.value(index) == "pass"));
}
