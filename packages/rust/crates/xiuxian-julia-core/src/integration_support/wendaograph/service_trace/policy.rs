use serde_json::{Value, json};
use xiuxian_wendao_runtime::transport::WENDAO_ARROW_FLIGHT_DATA_PLANE;

pub(crate) fn search_strategy_flow_performance_policy_json() -> Value {
    json!({
        "serviceLifecycle": "managed-warm-julia-service",
        "currentDataPlane": WENDAO_ARROW_FLIGHT_DATA_PLANE,
        "payloadEncoding": "arrow-ipc-stream-bundle",
        "rustControlsMaterialization": true,
        "juliaOwnsAlgorithmCompute": true,
        "rustEmbeddingJulia": false,
        "jlrsAllowed": false,
        "cDataTransportEnabled": false,
        "cDataPolicy": "capability-observation-only",
        "primaryOptimizationLanes": [
            "gateway-managed-warmup",
            "rust-duckdb-candidate-narrowing",
            "arrow-flight-request-response-bundling",
            "payload-hash-cache",
            "structure-aware-scheduler-admission",
            "warm-submit-benchmark-gate",
        ],
    })
}
