pub(crate) fn print_qianji_usage() {
    eprintln!("Usage:");
    eprintln!(
        "  Execution: qianji [-v|--log-verbose] <repo_path> <manifest_path> <context_json> [session_id]"
    );
    eprintln!("  Graph:     qianji [-v|--log-verbose] graph <manifest_path> <output_path>");
    eprintln!(
        "  BPMN:      qianji [-v|--log-verbose] bpmn start --bpmn <path> --process <id> --instance-id <id> [--context-json JSON] [--dmn <path>]... [--host-fixture <path>] [--event-fixture <path>] [--trace-stream] [--external-host] [--checkpoint-runtime]"
    );
    eprintln!(
        "             local no-server backend defaults to DuckDB; use `--checkpoint-runtime` for Valkey"
    );
    eprintln!(
        "             compatibility alias: qianji [-v|--log-verbose] bpmn run --bpmn <path> --process <id> --instance-id <id> [--context-json JSON] [--dmn <path>]... [--host-fixture <path>] [--event-fixture <path>] [--trace-stream] [--external-host] [--checkpoint-runtime]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] bpmn resume --bpmn <path> --instance-id <id> [--dmn <path>]... [--host-fixture <path>] [--event-fixture <path>] [--trace-stream] [--external-host] [--checkpoint-runtime]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] bpmn events poll --bpmn <path> --instance-id <id> [--dmn <path>]... [--host-fixture <path>] [--event-fixture <path>] [--checkpoint-runtime]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] bpmn tasks complete --bpmn <path> --instance-id <id> [--dmn <path>]... [--host-fixture <path>] [--trace-stream] [--external-host] [--checkpoint-runtime]"
    );
    eprintln!(
        "             qianji [-v|--log-verbose] bpmn status --instance-id <id> [--bpmn <path>] [--dmn <path>] [--checkpoint-runtime]"
    );
    eprintln!("             qianji [-v|--log-verbose] bpmn instances [--checkpoint-runtime]");
    eprintln!(
        "             qianji [-v|--log-verbose] bpmn cancel --instance-id <id> [--checkpoint-runtime]"
    );
    eprintln!("  Show:      qianji [-v|--log-verbose] show --dir <path>");
    eprintln!("             qianji [-v|--log-verbose] show --graph <path>");
    eprintln!("             qianji [-v|--log-verbose] show --contract <id>");
    eprintln!(
        "  Materialize: qianji [-v|--log-verbose] materialize --anchor <path> --scenario <ref> --dir <path> [--current-node <node>]"
    );
    eprintln!("  Advance:   qianji [-v|--log-verbose] advance --dir <path> --to <node>");
    eprintln!("  Check:     qianji [-v|--log-verbose] check --dir <path>");
    eprintln!("  Emit:      qianji [-v|--log-verbose] emit <path> --bpmn");
    eprintln!("  Lint:      qianji [-v|--log-verbose] lint <path>");
    eprintln!("             qianji [-v|--log-verbose] lint --bpmn <path>");
    eprintln!("             qianji [-v|--log-verbose] lint --dmn <path>");
    eprintln!("             compatibility alias: same flags also parse under 'linter'");
    eprintln!("  Template:  qianji [-v|--log-verbose] template --bpmn");
    eprintln!("             qianji [-v|--log-verbose] template --dmn");
    eprintln!("  Construct: qianji [-v|--log-verbose] construct index [--json]");
    eprintln!("             qianji [-v|--log-verbose] construct show <id> [--json]");
    eprintln!(
        "  Contract:  qianji [-v|--log-verbose] contract-feedback rest-docs <openapi_path> [--workspace-root PATH] [--storage-path PATH] [--table-name NAME] [--role ROLE]... [--no-persist] [--live-advisory] [--model MODEL] [--temperature FLOAT] [--cognitive-threshold FLOAT]"
    );
}
