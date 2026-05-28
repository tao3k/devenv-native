//! Built-in node execution mechanisms for the Qianji Box.
//!
//! Start in `api`; feature folders stay private behind this root seam.

#[path = "../executors_annotation.rs"]
mod annotation;
#[path = "../executors_api.rs"]
mod api;
#[path = "../executors_calibration.rs"]
mod calibration;
#[path = "../executors_cli_call.rs"]
mod cli_call;
#[path = "../executors_command.rs"]
mod command;
#[path = "../executors_formal_audit.rs"]
mod formal_audit;
#[path = "../executors_http_call.rs"]
mod http_call;
#[cfg(feature = "wendao-integration")]
#[path = "../executors_knowledge.rs"]
mod knowledge;
#[path = "../executors_mock.rs"]
mod mock;
#[path = "../executors_router.rs"]
mod router;
#[path = "../executors_security_scan.rs"]
mod security_scan;
#[path = "../executors_suspend.rs"]
mod suspend;
#[cfg(feature = "wendao-integration")]
#[path = "../executors_wendao_ingester.rs"]
mod wendao_ingester;
#[cfg(feature = "wendao-integration")]
#[path = "../executors_wendao_refresh.rs"]
mod wendao_refresh;
#[cfg(feature = "wendao-integration")]
#[path = "../executors_wendao_sql.rs"]
mod wendao_sql;
#[path = "../executors_write_file.rs"]
mod write_file;

#[cfg(feature = "llm")]
#[path = "../executors_llm.rs"]
mod llm;

pub use self::api::{
    CliCallMechanism, ContextAnnotator, FormalAuditMechanism, HttpCallMechanism, MockMechanism,
    ProbabilisticRouter, QianjiAdvisoryAuditExecutor, QianjiAdvisoryExecutionPlan,
    QianjiAdvisoryRolePlan, SecurityScanMechanism, ShellMechanism, SuspendMechanism,
    SynapseCalibrator, WriteFileMechanism,
};
#[cfg(feature = "wendao-integration")]
pub use self::api::{
    KnowledgeSeeker, WendaoIngesterMechanism, WendaoRefreshMechanism, WendaoSqlDiscoverMechanism,
    WendaoSqlExecuteMechanism, WendaoSqlValidateMechanism,
};
#[cfg(feature = "llm")]
pub use self::api::{
    LlmAnalyzer, LlmAugmentedAuditMechanism, OutputFlags, PipelineFlags,
    QianjiLlmAdvisoryAuditExecutor, StreamingLlmAnalyzer, StreamingLlmAnalyzerBuilder,
    StreamingPipelineSettings,
};

#[cfg(test)]
#[cfg(feature = "wendao-integration")]
pub(crate) use self::api::{parse_sql_author_spec_xml, parse_surface_bundle_xml};
