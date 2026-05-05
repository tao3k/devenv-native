use super::{
    DirCliCommand, ShowCliTarget, TempDir, anchored_workdir_fixture_anchor,
    anchored_workdir_fixture_graph, anchored_workdir_fixture_scenario, assert_common_show_shape,
    flowhub_root, fs, must_ok, run_dir_command, scenario_fixture_dir, write_file,
};
use std::path::PathBuf;

mod anchor;
mod contract;
mod graph;
mod surfaces;

fn create_anchored_runtime_state_fixture(temp_dir: &TempDir) -> PathBuf {
    let workdir = temp_dir.path().join("anchored-run");
    must_ok(
        fs::create_dir_all(workdir.join("state")),
        "should create anchored runtime state fixture",
    );
    write_file(
        &workdir.join("state/current_node.toml"),
        "current_node = \"claim_extract\"\n",
    );
    write_file(
        &workdir.join("state/allowed_next.json"),
        "[\"evidence_ground\", \"diagnostics\"]\n",
    );
    workdir
}
