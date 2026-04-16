use std::collections::BTreeSet;
#[cfg(feature = "performance")]
use std::time::Instant;

use crate::repo_index::state::collect::{
    collect_code_documents, collect_incremental_code_documents,
};

#[test]
fn collect_code_documents_returns_none_when_cancelled() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    std::fs::write(tempdir.path().join("module.jl"), "module Demo\nend\n")
        .unwrap_or_else(|error| panic!("write file: {error}"));

    let documents = collect_code_documents(tempdir.path(), || true);

    assert!(documents.is_none());
}

#[test]
fn collect_incremental_code_documents_only_reads_changed_supported_files() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let src = tempdir.path().join("src");
    std::fs::create_dir_all(src.as_path())
        .unwrap_or_else(|error| panic!("create src dir: {error}"));
    std::fs::write(src.join("kept.jl"), "module Kept\nend\n")
        .unwrap_or_else(|error| panic!("write kept file: {error}"));
    std::fs::write(src.join("changed.jl"), "module Changed\nend\n")
        .unwrap_or_else(|error| panic!("write changed file: {error}"));
    std::fs::write(src.join("ignored.txt"), "not supported\n")
        .unwrap_or_else(|error| panic!("write ignored file: {error}"));

    let collection = collect_incremental_code_documents(
        tempdir.path(),
        &BTreeSet::from(["src/changed.jl".to_string(), "src/ignored.txt".to_string()]),
        &BTreeSet::from(["src/deleted.jl".to_string()]),
        || false,
    )
    .unwrap_or_else(|| panic!("incremental collection should not cancel"));

    assert_eq!(collection.changed_documents.len(), 1);
    assert_eq!(collection.changed_documents[0].path, "src/changed.jl");
    assert_eq!(
        collection.deleted_paths,
        BTreeSet::from(["src/deleted.jl".to_string(), "src/ignored.txt".to_string(),])
    );
}

#[test]
fn collect_incremental_code_documents_marks_missing_changed_paths_as_deleted() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));

    let collection = collect_incremental_code_documents(
        tempdir.path(),
        &BTreeSet::from(["src/missing.jl".to_string()]),
        &BTreeSet::new(),
        || false,
    )
    .unwrap_or_else(|| panic!("incremental collection should not cancel"));

    assert!(collection.changed_documents.is_empty());
    assert_eq!(
        collection.deleted_paths,
        BTreeSet::from(["src/missing.jl".to_string()])
    );
}

#[cfg(feature = "performance")]
#[test]
fn collect_incremental_code_documents_reports_probe_latency_profile() {
    const FILE_COUNT: usize = 2_048;
    const LINE_COUNT: usize = 20;

    fn write_repo_file(path: &std::path::Path, module_name: &str, line_seed: usize) {
        let mut body = format!("module {module_name}\n");
        for line in 0..LINE_COUNT {
            body.push_str(
                format!(
                    "export symbol_{line_seed}_{line}\nfunction symbol_{line_seed}_{line}(x)\n    x + {}\nend\n\n",
                    line_seed + line
                )
                .as_str(),
            );
        }
        body.push_str("end\n");
        std::fs::write(path, body).unwrap_or_else(|error| panic!("write file: {error}"));
    }

    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let src = tempdir.path().join("src");
    std::fs::create_dir_all(src.as_path())
        .unwrap_or_else(|error| panic!("create src dir: {error}"));

    for index in 0..FILE_COUNT {
        write_repo_file(
            src.join(format!("module_{index:04}.jl")).as_path(),
            format!("Module{index:04}").as_str(),
            index,
        );
    }

    write_repo_file(src.join("module_0007.jl").as_path(), "Module0007", 7_000);
    write_repo_file(src.join("module_0512.jl").as_path(), "Module0512", 51_200);
    write_repo_file(src.join("module_4096.jl").as_path(), "Module4096", 409_600);
    std::fs::remove_file(src.join("module_1024.jl"))
        .unwrap_or_else(|error| panic!("remove file: {error}"));

    let changed_paths = BTreeSet::from([
        "src/module_0007.jl".to_string(),
        "src/module_0512.jl".to_string(),
        "src/module_4096.jl".to_string(),
    ]);
    let deleted_paths = BTreeSet::from(["src/module_1024.jl".to_string()]);

    let full_started = Instant::now();
    let full_documents = collect_code_documents(tempdir.path(), || false)
        .unwrap_or_else(|| panic!("full collection should not cancel"));
    let full_elapsed = full_started.elapsed();

    let incremental_started = Instant::now();
    let incremental =
        collect_incremental_code_documents(tempdir.path(), &changed_paths, &deleted_paths, || {
            false
        })
        .unwrap_or_else(|| panic!("incremental collection should not cancel"));
    let incremental_elapsed = incremental_started.elapsed();

    eprintln!(
        "repo_index_collect_probe full_scan={full_elapsed:?} incremental={incremental_elapsed:?} ratio={:.2}x full_docs={} incremental_docs={} deleted_paths={}",
        full_elapsed.as_secs_f64() / incremental_elapsed.as_secs_f64(),
        full_documents.len(),
        incremental.changed_documents.len(),
        incremental.deleted_paths.len(),
    );

    assert_eq!(full_documents.len(), FILE_COUNT);
    assert_eq!(incremental.changed_documents.len(), 3);
    assert_eq!(incremental.deleted_paths.len(), 1);
}
