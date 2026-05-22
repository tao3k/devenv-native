#![cfg(feature = "julia")]

use std::{
    env, fs,
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use super::support::EpistemeGatewayFixture;
use super::{
    EpistemeOntologyRegistryQualityProofModeRequest,
    EpistemeOntologyRegistryReadModelGatewayRequest,
    admit_episteme_ontology_registry_read_model_from_payload_with_quality_proof,
};

const RUN_GATEWAY_WENDAOGRAPH_QUALITY_LIVE_ENV: &str =
    "RUN_EPISTEME_REGISTRY_GATEWAY_WENDAOGRAPH_QUALITY_LIVE_TEST";
const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_BASE_URL_ENV: &str =
    "WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_BASE_URL";

#[tokio::test]
async fn episteme_ontology_registry_gateway_live_quality_proof_uses_configured_wendaograph()
-> Result<(), Box<dyn std::error::Error>> {
    if env::var_os(RUN_GATEWAY_WENDAOGRAPH_QUALITY_LIVE_ENV).is_none() {
        eprintln!(
            "skipping episteme registry Gateway WendaoGraph live quality proof; set {RUN_GATEWAY_WENDAOGRAPH_QUALITY_LIVE_ENV}=1"
        );
        return Ok(());
    }

    let service = LiveWendaoGraphQualityService::start().await?;
    let fixture = EpistemeGatewayFixture::new()?;
    fixture.write_contract()?;
    fixture.write_ontology_registry_snapshot()?;
    write_gateway_wendaograph_quality_config(&fixture, service.base_url.as_str())?;

    let request = EpistemeOntologyRegistryReadModelGatewayRequest {
        episteme_root: None,
        episteme_registry_id: Some("synthetic".to_string()),
        quality_proof_mode: Some(EpistemeOntologyRegistryQualityProofModeRequest::IfConfigured),
    };
    let report = admit_episteme_ontology_registry_read_model_from_payload_with_quality_proof(
        fixture.project_root.as_path(),
        fixture.config_root.as_path(),
        &request,
    )
    .await
    .map_err(|error| format!("{error:?}"))?;

    let Some(proof) = report.quality_proof.as_ref() else {
        return Err("live quality proof should be present".into());
    };
    assert_eq!(proof.status, "passed");
    assert_eq!(proof.selected_transport.as_deref(), Some("ArrowFlight"));
    assert!(proof.response_batch_count.unwrap_or_default() > 0);
    assert!(proof.response_row_count.unwrap_or_default() > 0);
    let pass_rows = proof
        .response_status_counts
        .as_ref()
        .and_then(|counts| counts.get("pass"))
        .copied()
        .unwrap_or_default();
    assert!(pass_rows > 0);

    write_gateway_quality_evidence(repo_root()?.as_path(), &service, &report)?;
    Ok(())
}

struct LiveWendaoGraphQualityService {
    _process_guard: Option<ChildGuard>,
    mode: &'static str,
    base_url: String,
    ready_ms: f64,
}

impl LiveWendaoGraphQualityService {
    async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(base_url) = external_base_url()? {
            return Ok(Self {
                _process_guard: None,
                mode: "external",
                base_url,
                ready_ms: 0.0,
            });
        }

        let repo_root = repo_root()?;
        let graph_root = wendaograph_project_root(repo_root.as_path())?;
        let runner = graph_root
            .join("scripts")
            .join("run_ontology_read_model_quality_service.jl");
        if !runner.is_file() {
            return Err(format!(
                "missing WendaoGraph ontology read-model quality runner `{}`",
                runner.display()
            )
            .into());
        }
        let port = reserve_loopback_port()?;
        let base_url = format!("http://127.0.0.1:{port}");
        let started_at = Instant::now();
        let guard = ChildGuard::spawn(
            Command::new("julia")
                .arg(format!("--project={}", graph_root.display()))
                .arg(runner)
                .arg("--host=127.0.0.1")
                .arg(format!("--port={port}"))
                .arg("--max-active-requests=4")
                .arg("--request-capacity=4")
                .arg("--response-capacity=4")
                .stdout(Stdio::null())
                .stderr(Stdio::inherit()),
        )?;
        wait_for_tcp_ready(port).await?;

        Ok(Self {
            _process_guard: Some(guard),
            mode: "spawned",
            base_url,
            ready_ms: started_at.elapsed().as_secs_f64() * 1000.0,
        })
    }
}

struct ChildGuard {
    child: Child,
}

impl ChildGuard {
    fn spawn(command: &mut Command) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            child: command.spawn()?,
        })
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn write_gateway_wendaograph_quality_config(
    fixture: &EpistemeGatewayFixture,
    base_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        fixture.config_root.join("wendao.toml"),
        format!(
            r#"[episteme.registries.synthetic]
path = "source-contract"

[wendaograph.ontology_read_model_quality]
base_url = "{base_url}"
timeout_seconds = 30
max_in_flight_requests = 1
"#
        ),
    )?;
    Ok(())
}

fn external_base_url() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let Some(raw) = env::var_os(WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_BASE_URL_ENV) else {
        return Ok(None);
    };
    let base_url = raw
        .to_string_lossy()
        .trim()
        .trim_end_matches('/')
        .to_string();
    if base_url.is_empty() {
        return Ok(None);
    }
    if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
        return Err(format!(
            "{WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_BASE_URL_ENV} must start with http:// or https://"
        )
        .into());
    }
    Ok(Some(base_url))
}

fn repo_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for ancestor in manifest_dir.ancestors() {
        if ancestor.join(".data/WendaoGraph.jl").is_dir() {
            return Ok(ancestor.to_path_buf());
        }
    }
    Err(format!("could not find repo root from `{}`", manifest_dir.display()).into())
}

fn wendaograph_project_root(repo_root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let raw = env::var_os("WENDAOGRAPH_PACKAGE_DIR")
        .map_or_else(|| repo_root.join(".data/WendaoGraph.jl"), PathBuf::from);
    let project = if raw.is_absolute() {
        raw
    } else {
        repo_root.join(raw)
    };
    if project.join("Project.toml").is_file() {
        Ok(project)
    } else {
        Err(format!(
            "missing WendaoGraph Project.toml under `{}`",
            project.display()
        )
        .into())
    }
}

fn reserve_loopback_port() -> Result<u16, Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

async fn wait_for_tcp_ready(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let address = format!("127.0.0.1:{port}");
    let deadline = Instant::now() + Duration::from_secs(90);
    let mut last_error = String::new();
    while Instant::now() < deadline {
        match tokio::net::TcpStream::connect(address.as_str()).await {
            Ok(_) => return Ok(()),
            Err(error) => {
                last_error = error.to_string();
            }
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    Err(format!("WendaoGraph service did not become ready: {last_error}").into())
}

fn write_gateway_quality_evidence(
    repo_root: &Path,
    service: &LiveWendaoGraphQualityService,
    report: &super::EpistemeOntologyRegistryReadModelGatewayReport,
) -> Result<(), Box<dyn std::error::Error>> {
    let evidence_dir = repo_root
        .join(".cache")
        .join("agent/evidence/episteme_registry_gateway_wendaograph_quality");
    fs::create_dir_all(&evidence_dir)?;
    let body = serde_json::json!({
        "schemaVersion": "xiuxian_wendao.episteme_registry_gateway_wendaograph_quality_live_report.v1",
        "serviceMode": service.mode,
        "serviceBaseUrl": service.base_url,
        "serviceReadyMs": service.ready_ms,
        "gatewayReport": report,
        "rawCorpusReadByJulia": false,
        "rdfPromotion": false
    });
    let body = format!("{}\n", serde_json::to_string_pretty(&body)?);
    fs::write(evidence_dir.join("latest.json"), body.as_bytes())?;
    fs::write(
        evidence_dir.join(format!("report-{}.json", unix_timestamp_secs()?)),
        body,
    )?;
    Ok(())
}

fn unix_timestamp_secs() -> Result<u64, std::time::SystemTimeError> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
