pub(crate) fn print_qianji_usage() {
    eprintln!("Usage:");
    eprintln!(
        "  Execution: qianji [-v|--log-verbose] <repo_path> <manifest_path> <context_json> [session_id]"
    );
    eprintln!("  Graph:     qianji [-v|--log-verbose] graph <manifest_path> <output_path>");
    eprintln!(
        "  BPMN:      qianji [-v|--log-verbose] bpmn run --bpmn <path> --process <id> --instance-id <id> [--context-json JSON] [--dmn <path>]... [--host-fixture <path>] [--event-fixture <path>] [--checkpoint-runtime]"
    );
    eprintln!(
        "             optional local backend: add `--checkpoint-sqlite <path>` when the `sqlite` feature is enabled"
    );
    eprintln!("  Show:      qianji [-v|--log-verbose] show --dir <path>");
    eprintln!("             qianji [-v|--log-verbose] show --graph <path>");
    eprintln!("             qianji [-v|--log-verbose] show --contract <id>");
    eprintln!(
        "  Materialize: qianji [-v|--log-verbose] materialize --anchor <path> --scenario <ref> --dir <path> [--current-node <node>]"
    );
    eprintln!("  Advance:   qianji [-v|--log-verbose] advance --dir <path> --to <node>");
    eprintln!("  Check:     qianji [-v|--log-verbose] check --dir <path>");
    eprintln!("  Lint:      qianji [-v|--log-verbose] lint --bpmn <path>");
    eprintln!("             qianji [-v|--log-verbose] lint --dmn <path>");
    eprintln!("             compatibility alias: same flags also parse under 'linter'");
    eprintln!(
        "  Contract:  qianji [-v|--log-verbose] contract-feedback rest-docs <openapi_path> [--workspace-root PATH] [--storage-path PATH] [--table-name NAME] [--role ROLE]... [--no-persist] [--live-advisory] [--model MODEL] [--temperature FLOAT] [--cognitive-threshold FLOAT]"
    );
}
