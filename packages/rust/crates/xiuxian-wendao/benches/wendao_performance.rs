//! Criterion microbenchmarks for xiuxian-wendao performance trend analysis.

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use criterion::{Criterion, Throughput, black_box};
use tempfile::{TempDir, tempdir};
use xiuxian_wendao::repo_index::perf_support::{
    RepoBootstrapBenchmarkFixture, benchmark_collect_full_repo_code_documents,
    benchmark_collect_incremental_repo_code_documents,
};
use xiuxian_wendao::{
    LinkGraphHit, LinkGraphIndex, LinkGraphPprSubgraphMode, LinkGraphRelatedPprOptions,
    narrate_subgraph,
};

const NODE_COUNT: usize = 2_048;
const HUB_COUNT: usize = 32;
const RELATED_MAX_DISTANCE: usize = 4;
const RELATED_LIMIT: usize = 24;
const REPO_BENCH_FILE_COUNT: usize = 2_048;
const REPO_BENCH_FILE_LINES: usize = 20;
const REPO_BOOTSTRAP_BENCH_REPO_COUNT: usize = 10_000;

fn note_id(i: usize) -> String {
    format!("note-{i:05}")
}

fn hub_id(i: usize) -> String {
    format!("hub-{i:03}")
}

fn write_note(path: &Path, body: &str) {
    if let Err(error) = fs::write(path, body) {
        panic!("write benchmark fixture note {}: {error}", path.display());
    }
}

fn write_repo_file(path: &Path, module_name: &str, line_seed: usize) {
    let mut body = format!("module {module_name}\n");
    for line in 0..REPO_BENCH_FILE_LINES {
        body.push_str(
            format!(
                "export symbol_{line_seed}_{line}\nfunction symbol_{line_seed}_{line}(x)\n    x + {}\nend\n\n",
                line_seed + line
            )
            .as_str(),
        );
    }
    body.push_str("end\n");
    write_note(path, body.as_str());
}

fn build_fixture(root: &Path) {
    for i in 0..NODE_COUNT {
        let current = note_id(i);
        let next = note_id((i + 1) % NODE_COUNT);
        let jump = note_id((i + 97) % NODE_COUNT);
        let hub = hub_id(i % HUB_COUNT);
        let body = format!(
            "# {current}\n\nSynthetic benchmark note {i}.\n\nLinks: [[{next}]] [[{jump}]] [[{hub}]]\n"
        );
        write_note(&root.join(format!("{current}.md")), &body);
    }

    for i in 0..HUB_COUNT {
        let hub = hub_id(i);
        let mut links = String::new();
        let stride = HUB_COUNT * 2;
        let mut cursor = i;
        let mut emitted = 0_usize;
        while cursor < NODE_COUNT && emitted < 160 {
            if !links.is_empty() {
                links.push(' ');
            }
            links.push_str("[[");
            links.push_str(&note_id(cursor));
            links.push_str("]]");
            emitted += 1;
            cursor += stride;
        }
        let body = format!("# {hub}\n\nSynthetic benchmark hub {i}.\n\nOutbound links: {links}\n");
        write_note(&root.join(format!("{hub}.md")), &body);
    }
}

fn repo_file_path(i: usize) -> String {
    format!("src/module_{i:04}.jl")
}

fn build_repo_collection_fixture(root: &Path) {
    let src = root.join("src");
    if let Err(error) = fs::create_dir_all(src.as_path()) {
        panic!("create repo benchmark src {}: {error}", src.display());
    }
    for i in 0..REPO_BENCH_FILE_COUNT {
        let module_name = format!("Module{i:04}");
        write_repo_file(
            src.join(format!("module_{i:04}.jl")).as_path(),
            module_name.as_str(),
            i,
        );
    }
}

fn build_index_fixture() -> (TempDir, LinkGraphIndex, Vec<String>) {
    let tmp = match tempdir() {
        Ok(tmp) => tmp,
        Err(error) => panic!("create benchmark tempdir: {error}"),
    };
    build_fixture(tmp.path());
    let index = match LinkGraphIndex::build(tmp.path()) {
        Ok(index) => index,
        Err(error) => panic!("build benchmark index: {error}"),
    };
    let seeds = (0..192)
        .map(|turn| note_id((turn * 211) % NODE_COUNT))
        .collect();
    (tmp, index, seeds)
}

fn ppr_options() -> LinkGraphRelatedPprOptions {
    LinkGraphRelatedPprOptions {
        alpha: Some(0.9),
        max_iter: Some(30),
        tol: Some(1e-6),
        subgraph_mode: Some(LinkGraphPprSubgraphMode::Auto),
    }
}

fn bench_related_ppr(c: &mut Criterion) {
    let (_tmp, index, seeds) = build_index_fixture();
    let ppr = ppr_options();
    let cursor = AtomicUsize::new(0);

    let mut group = c.benchmark_group("search_related_ppr");
    group.throughput(Throughput::Elements(1));
    group.bench_function("related_with_diagnostics", |bench| {
        bench.iter(|| {
            let position = cursor.fetch_add(1, Ordering::Relaxed) % seeds.len();
            let seed = &seeds[position];
            let (rows, diagnostics) = index.related_with_diagnostics(
                black_box(seed),
                RELATED_MAX_DISTANCE,
                RELATED_LIMIT,
                Some(&ppr),
            );
            assert!(
                !(rows.is_empty() || diagnostics.is_none()),
                "benchmark fixture produced empty or diagnostics-free result"
            );
            black_box(rows.len())
        });
    });
    group.finish();
}

fn build_repo_collection_benchmark_fixture() -> (
    TempDir,
    std::collections::BTreeSet<String>,
    std::collections::BTreeSet<String>,
    std::collections::BTreeMap<String, xiuxian_wendao::search::SearchFileFingerprint>,
) {
    let tmp = match tempdir() {
        Ok(tmp) => tmp,
        Err(error) => panic!("create repo benchmark tempdir: {error}"),
    };
    build_repo_collection_fixture(tmp.path());
    let baseline = benchmark_collect_full_repo_code_documents(tmp.path());

    write_repo_file(
        tmp.path().join(repo_file_path(7)).as_path(),
        "Module0007",
        7_000,
    );
    write_repo_file(
        tmp.path().join(repo_file_path(512)).as_path(),
        "Module0512",
        51_200,
    );
    write_repo_file(
        tmp.path().join("src/module_4096.jl").as_path(),
        "Module4096",
        409_600,
    );
    if let Err(error) = fs::remove_file(tmp.path().join(repo_file_path(1024))) {
        panic!(
            "remove repo benchmark file {}: {error}",
            repo_file_path(1024)
        );
    }

    (
        tmp,
        std::collections::BTreeSet::from([
            repo_file_path(7),
            repo_file_path(512),
            "src/module_4096.jl".to_string(),
        ]),
        std::collections::BTreeSet::from([repo_file_path(1024)]),
        baseline.file_fingerprints,
    )
}

fn bench_incremental_repo_code_documents(c: &mut Criterion) {
    let (tmp, changed_paths, deleted_paths, previous_fingerprints) =
        build_repo_collection_benchmark_fixture();
    let mut group = c.benchmark_group("incremental_repo_code_documents");
    group.throughput(Throughput::Elements(
        u64::try_from(REPO_BENCH_FILE_COUNT).unwrap_or(u64::MAX),
    ));
    group.bench_function("full_scan_2048_files", |bench| {
        bench.iter(|| {
            let snapshot = benchmark_collect_full_repo_code_documents(black_box(tmp.path()));
            assert_eq!(snapshot.document_count, REPO_BENCH_FILE_COUNT);
            black_box(snapshot.document_count)
        });
    });
    group.bench_function("incremental_3_changes", |bench| {
        bench.iter(|| {
            let snapshot = benchmark_collect_incremental_repo_code_documents(
                black_box(tmp.path()),
                black_box(&changed_paths),
                black_box(&deleted_paths),
                black_box(&previous_fingerprints),
            );
            assert_eq!(snapshot.changed_document_count, 3);
            assert_eq!(snapshot.deleted_path_count, 1);
            assert_eq!(snapshot.file_fingerprints.len(), REPO_BENCH_FILE_COUNT);
            black_box(snapshot.changed_document_count + snapshot.deleted_path_count)
        });
    });
    group.finish();
}

fn bench_repo_bootstrap_statuses(c: &mut Criterion) {
    let fixture = RepoBootstrapBenchmarkFixture::synthetic(REPO_BOOTSTRAP_BENCH_REPO_COUNT);
    assert!(!fixture.snapshot_file_exists());

    let mut group = c.benchmark_group("repo_bootstrap_statuses");
    group.throughput(Throughput::Elements(
        u64::try_from(fixture.repo_count()).unwrap_or(u64::MAX),
    ));
    group.bench_function("bootstrap_statuses_10k_repo_records", |bench| {
        bench.iter(|| {
            let status_count = fixture.bootstrap_status_count();
            assert_eq!(status_count, fixture.repo_count());
            black_box(status_count)
        });
    });
    group.finish();
}

fn bench_narration_fusion(c: &mut Criterion) {
    let hits: Vec<LinkGraphHit> = (0..240)
        .map(|i| {
            let i_u32 = u32::try_from(i).unwrap_or(u32::MAX);
            LinkGraphHit {
                stem: format!("node_{i:04}"),
                score: 1.0 - (f64::from(i_u32) * 0.002),
                title: format!("Narration benchmark node {i}"),
                path: format!("docs/node_{i:04}.md"),
                doc_type: None,
                tags: vec!["benchmark".to_string()],
                best_section: None,
                match_reason: None,
            }
        })
        .collect();

    let mut group = c.benchmark_group("fusion_narration");
    group.throughput(Throughput::Elements(
        u64::try_from(hits.len()).unwrap_or(u64::MAX),
    ));
    group.bench_function("narrate_subgraph_240_hits", |bench| {
        bench.iter(|| black_box(narrate_subgraph(black_box(&hits))));
    });
    group.finish();
}

fn main() {
    let mut criterion = Criterion::default().configure_from_args();
    bench_related_ppr(&mut criterion);
    bench_incremental_repo_code_documents(&mut criterion);
    bench_repo_bootstrap_statuses(&mut criterion);
    bench_narration_fusion(&mut criterion);
    criterion.final_summary();
}
