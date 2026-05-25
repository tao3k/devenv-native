use xiuxian_wendao_parsers::{
    OrgizeEvalPatchRequest, OrgizeEvalPlanRequest, render_eval_patch, render_eval_plan,
};

use super::support::tempdir_or_panic;

#[test]
fn orgize_eval_plan_renders_named_block_contract_without_running_code() {
    let temp = tempdir_or_panic();
    let path = temp.path().join("task.org");
    std::fs::write(&path, eval_fixture())
        .unwrap_or_else(|error| panic!("write eval fixture: {error}"));

    let rendered = render_eval_plan(&OrgizeEvalPlanRequest {
        name: "verify".to_string(),
        path,
        json: false,
    })
    .unwrap_or_else(|error| panic!("render eval plan: {error}"));

    assert!(rendered.contains("name: verify"), "rendered: {rendered}");
    assert!(rendered.contains("language: bash"), "rendered: {rendered}");
    assert!(
        rendered.contains("results: output replace"),
        "rendered: {rendered}"
    );
    assert!(!rendered.contains("source:"), "rendered: {rendered}");
}

#[test]
fn orgize_eval_patch_writes_host_supplied_results_without_running_code() {
    let temp = tempdir_or_panic();
    let path = temp.path().join("task.org");
    std::fs::write(&path, eval_fixture())
        .unwrap_or_else(|error| panic!("write eval fixture: {error}"));

    let rendered = render_eval_patch(&OrgizeEvalPatchRequest {
        name: "verify".to_string(),
        path: path.clone(),
        stdout: "ok".to_string(),
        stderr: String::new(),
        exit_code: Some(0),
        write: true,
        json: false,
    })
    .unwrap_or_else(|error| panic!("render eval patch: {error}"));

    assert!(rendered.contains("kind: insert"), "rendered: {rendered}");
    assert!(rendered.contains("written: true"), "rendered: {rendered}");
    assert_eq!(
        std::fs::read_to_string(path).unwrap_or_else(|error| panic!("read patched org: {error}")),
        concat!(
            "#+NAME: verify\n",
            "#+BEGIN_SRC bash :results output replace\n",
            "echo should-not-run\n",
            "#+END_SRC\n",
            "\n",
            "#+RESULTS: verify\n",
            ": ok\n",
        )
    );
}

fn eval_fixture() -> &'static str {
    concat!(
        "#+NAME: verify\n",
        "#+BEGIN_SRC bash :results output replace\n",
        "echo should-not-run\n",
        "#+END_SRC\n",
    )
}
