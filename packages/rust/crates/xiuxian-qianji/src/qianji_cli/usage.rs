pub(crate) fn print_qianji_usage() {
    eprintln!("Usage:");
    print_execution_usage();
    print_bpmn_usage();
    print_surface_usage();
    print_control_usage();
    print_contract_usage();
}

fn print_execution_usage() {
    eprintln!(
        "  Execution: qianji [-v|--log-verbose] <repo_path> <manifest_path> <context_json> [session_id]"
    );
    eprintln!("  Graph:     qianji [-v|--log-verbose] graph <manifest_path> <output_path>");
}

fn print_bpmn_usage() {
    eprintln!(
        "  BPMN:      qianji [-v|--log-verbose] bpmn start --bpmn <path> --process <id> --instance-id <id> [--context-json JSON] [--dmn <path>]... [--host-fixture <path>] [--event-fixture <path>] [--trace-stream] [--external-host] [--continue-until-human-boundary] [--checkpoint-runtime]"
    );
    eprintln!(
        "             local no-server backend defaults to DuckDB; use `--checkpoint-runtime` for Valkey"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] bpmn start-at --bpmn <path> --process <id> --node <node_id> --instance-id <id> [--context-json JSON] [--dmn <path>]... [--host-fixture <path>] [--event-fixture <path>] [--trace-stream] [--external-host] [--continue-until-human-boundary] [--checkpoint-runtime]"
    );
    eprintln!(
        "             compatibility alias: qianji [-v|--log-verbose] bpmn run --bpmn <path> --process <id> --instance-id <id> [--context-json JSON] [--dmn <path>]... [--host-fixture <path>] [--event-fixture <path>] [--trace-stream] [--external-host] [--continue-until-human-boundary] [--checkpoint-runtime]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] bpmn host-session --bpmn <path> --process <id> --instance-id <id> [--context-json JSON] [--node <node_id>] [--dmn <path>]... [--host-fixture <path>] [--event-fixture <path>] [--trace-stream] [--checkpoint-runtime]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] bpmn resume --bpmn <path> --instance-id <id> [--dmn <path>]... [--host-fixture <path>] [--event-fixture <path>] [--trace-stream] [--external-host] [--continue-until-human-boundary] [--checkpoint-runtime]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] bpmn events poll --bpmn <path> --instance-id <id> [--dmn <path>]... [--host-fixture <path>] [--event-fixture <path>] [--checkpoint-runtime]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] bpmn tasks complete --bpmn <path> --instance-id <id> --token-id <id> --process-id <id> --activity-id <id> --kind send|service|script|user|manual --data-json <json> [--claimant <id>] [--dmn <path>]... [--host-fixture <path>] [--event-fixture <path>] [--trace-stream] [--continue-until-human-boundary] [--checkpoint-runtime]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] bpmn tasks claim --instance-id <id> --token-id <id> --process-id <id> --activity-id <id> --claimant <id> [--checkpoint-runtime]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] bpmn tasks release --instance-id <id> --token-id <id> --process-id <id> --activity-id <id> --claimant <id> [--checkpoint-runtime]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] bpmn tasks worklist [--claimant <id>] [--assignment-resource <resource>] [--lane <lane>] [--checkpoint-runtime]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] bpmn status --instance-id <id> [--bpmn <path>] [--dmn <path>] [--checkpoint-runtime]"
    );
    eprintln!("             qianji [-v|--log-verbose] bpmn instances [--checkpoint-runtime]");
    eprintln!(
        "             qianji [-v|--log-verbose] bpmn cancel --instance-id <id> [--checkpoint-runtime]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] bpmn interrupt|stop --instance-id <id> [--checkpoint-runtime]"
    );
}

fn print_surface_usage() {
    eprintln!("  Show:      qianji [-v|--log-verbose] show --dir <path>");
    eprintln!("             qianji [-v|--log-verbose] show --graph <path>");
    eprintln!("             qianji [-v|--log-verbose] show --contract <id>");
    eprintln!(
        "  Materialize: qianji [-v|--log-verbose] materialize --anchor <path> --scenario <ref> --dir <path> [--current-node <node>]"
    );
    eprintln!("  Advance:   qianji [-v|--log-verbose] advance --dir <path> --to <node>");
    eprintln!("  Check:     qianji [-v|--log-verbose] check --dir <path>");
    eprintln!("  Emit:      qianji [-v|--log-verbose] emit <path> --bpmn");
    eprintln!("  Lint:      qianji [-v|--log-verbose] lint <path> [--llm|--json]");
    eprintln!("             qianji [-v|--log-verbose] lint --bpmn <path> [--llm|--json]");
    eprintln!("             qianji [-v|--log-verbose] lint --dmn <path> [--llm|--json]");
    eprintln!("             default output is compact LLM repair diagnostics, equivalent to --llm");
    eprintln!("             compatibility alias: same flags also parse under 'linter'");
    eprintln!("  Template:  qianji [-v|--log-verbose] template --bpmn");
    eprintln!("             qianji [-v|--log-verbose] template --dmn");
    eprintln!("             qianji [-v|--log-verbose] template --semantic-guard-route");
    eprintln!("  Construct: qianji [-v|--log-verbose] construct index [--json]");
    eprintln!("             qianji [-v|--log-verbose] construct show <id> [--json]");
}

fn print_control_usage() {
    eprintln!(
        "  Control:   qianji [-v|--log-verbose] control recovery-snapshot --ledger <path> --run-id <id> --now-ms <ms> [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control apply-recovery-plan --ledger <path> --valkey-url <url> --run-id <id> --now-ms <ms> --attempt <n> --reason <text> --max-attempts <n> [--namespace <ns>] [--backoff-ms <ms>] [--require-human-approval] [--priority <n>] [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control hot-state --valkey-url <url> --now-ms <ms> [--namespace <ns>] [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control query --ledger <path> --run-id <id> --state --now-ms <ms> [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control signal --ledger <path> --run-id <id> --signal-name <name> --payload <json> --received-at-ms <ms> [--step-id <id>] [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control signals --ledger <path> --run-id <id> [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control activity --ledger <path> --run-id <id> --activity-id <id> [--step-id <id>] [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control activity-complete --ledger <path> --run-id <id> --activity-id <id> --completed-at-ms <ms> [--step-id <id>] [--output-hash <hash>] [--metadata <json>] [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control activity-fail --ledger <path> --run-id <id> --activity-id <id> --failed-at-ms <ms> --error-code <code> --message <text> --retryable <true|false> --attempt <n> [--step-id <id>] [--metadata <json>] [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control activity-queue --ledger <path> --run-id <id> [--task-queue <queue>] [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control activity-start --ledger <path> --run-id <id> --activity-id <id> --worker-id <id> --started-at-ms <ms> --attempt <n> [--step-id <id>] [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control costs --ledger <path> --run-id <id> [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control lease --ledger <path> --run-id <id> --step-id <id> [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control leases --ledger <path> --run-id <id> [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control decision --ledger <path> --run-id <id> --decision-id <id> [--step-id <id>] [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control history --ledger <path> --run-id <id> [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control heartbeat --ledger <path> --run-id <id> --worker-id <id> --observed-at-ms <ms> --expires-at-ms <ms> [--metadata <json>] [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control view --ledger <path> --run-id <id> [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control step --ledger <path> --run-id <id> --step-id <id> [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control timer --ledger <path> --run-id <id> --timer-id <id> [--step-id <id>] [--json]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] control timers --ledger <path> --run-id <id> [--json]"
    );
}

fn print_contract_usage() {
    eprintln!(
        "  Contract:  qianji [-v|--log-verbose] contract-feedback rest-docs <openapi_path> [--workspace-root PATH] [--storage-path PATH] [--table-name NAME] [--role ROLE]... [--no-persist] [--live-advisory] [--model MODEL] [--temperature FLOAT] [--cognitive-threshold FLOAT]"
    );
}
