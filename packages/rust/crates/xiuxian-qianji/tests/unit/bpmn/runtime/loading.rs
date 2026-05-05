use super::{TempDir, load_bpmn_package_from_files, ok_of, write_business_rule_bundle};

#[test]
fn load_bpmn_package_from_files_attaches_dmn_registry() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let bundle = write_business_rule_bundle(&temp_dir);

    let package = ok_of(
        load_bpmn_package_from_files(&bundle.bpmn_path, std::slice::from_ref(&bundle.dmn_path)),
        "bundle should load from disk",
    );

    assert_eq!(package.package_id.as_ref(), "pkg_review");
    assert_eq!(package.processes.len(), 1);
    assert_eq!(package.dmn_decisions().len(), 1);
    assert_eq!(
        package.dmn_decisions()[0].decision.decision_id.as_ref(),
        "loan-decision"
    );
}
