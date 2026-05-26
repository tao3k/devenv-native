use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn qianji_adapters_do_not_write_control_events_directly() {
    let crates_dir = crates_dir();
    let source_roots = [
        crates_dir.join("xiuxian-qianji-runtime/src"),
        crates_dir.join("xiuxian-qianji/src"),
    ];
    let forbidden_patterns = [
        "ControlEvent::run(",
        "ControlEvent::step(",
        ".append_event(",
    ];

    let violations = collect_violations(&source_roots, &forbidden_patterns, |_| true);

    assert!(
        violations.is_empty(),
        "Qianji runtime/server crates must delegate durable event creation to \
         xiuxian-qianji-control journal helpers:\n{}",
        violations.join("\n")
    );
}

#[test]
fn control_ledger_appends_stay_inside_journal_modules() {
    let source_roots = [PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")];
    let forbidden_patterns = [
        "ControlEvent::run(",
        "ControlEvent::step(",
        ".append_event(",
    ];

    let violations = collect_violations(&source_roots, &forbidden_patterns, |path| {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        name.ends_with("_journal.rs") || name == "journal_batch.rs"
    });

    assert!(
        violations.is_empty(),
        "Durable event construction and ledger appends must stay in journal \
         modules or the control-owned batch helper:\n{}",
        violations.join("\n")
    );
}

fn collect_violations<F>(
    source_roots: &[PathBuf],
    forbidden_patterns: &[&str],
    allowed_file: F,
) -> Vec<String>
where
    F: Fn(&Path) -> bool,
{
    let mut files = Vec::new();
    for root in source_roots {
        collect_rust_files(root, &mut files);
    }

    let mut violations = Vec::new();
    for file in files {
        let text = fs::read_to_string(&file)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", file.display()));
        for (index, line) in text.lines().enumerate() {
            if forbidden_patterns
                .iter()
                .any(|pattern| line.contains(pattern))
                && !allowed_file(&file)
            {
                violations.push(format!(
                    "{}:{}: {}",
                    display_from_crates_dir(&file),
                    index + 1,
                    line.trim()
                ));
            }
        }
    }
    violations
}

fn collect_rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()));
    for entry in entries {
        let path = entry
            .unwrap_or_else(|error| panic!("failed to read entry in {}: {error}", dir.display()))
            .path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn display_from_crates_dir(path: &Path) -> String {
    path.strip_prefix(crates_dir())
        .unwrap_or(path)
        .display()
        .to_string()
}

fn crates_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(parent) = manifest_dir.parent() else {
        panic!(
            "qianji-control crate manifest dir should have a parent: {}",
            manifest_dir.display()
        );
    };
    parent.to_path_buf()
}
