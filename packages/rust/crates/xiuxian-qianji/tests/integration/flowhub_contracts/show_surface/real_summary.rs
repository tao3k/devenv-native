use super::{
    FlowhubModuleKind, FlowhubShow, assert_common_show_shape, classify_flowhub_dir, flowhub_root,
    real_flowhub_fixture_available, render_flowhub_show, show_flowhub,
};
use xiuxian_qianji::FlowhubDirKind;

#[test]
fn classify_flowhub_dir_detects_real_root_and_module() {
    if !real_flowhub_fixture_available() {
        return;
    }
    assert_eq!(
        classify_flowhub_dir(flowhub_root())
            .unwrap_or_else(|error| panic!("root should classify: {error}")),
        Some(FlowhubDirKind::Root)
    );
    assert_eq!(
        classify_flowhub_dir(flowhub_root().join("rust"))
            .unwrap_or_else(|error| panic!("module should classify: {error}")),
        Some(FlowhubDirKind::Module)
    );
}

#[test]
fn show_flowhub_summarizes_real_root() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let show = show_flowhub(flowhub_root())
        .unwrap_or_else(|error| panic!("real Flowhub root should show: {error}"));

    let FlowhubShow::Root(show) = show else {
        panic!("expected Flowhub root summary");
    };
    assert_eq!(show.modules.len(), 6);
    assert!(
        show.modules
            .iter()
            .any(|module| module.module_ref == "rust")
    );
    assert!(
        show.modules
            .iter()
            .any(|module| module.module_ref == "blueprint")
    );
    assert!(
        show.modules
            .iter()
            .any(|module| module.module_ref == "wendao")
    );
    assert!(
        show.modules
            .iter()
            .any(|module| module.module_ref == "research")
    );

    let rendered = render_flowhub_show(&FlowhubShow::Root(show));
    assert_common_show_shape(&rendered);
    assert!(rendered.contains("# Flowhub"));
    assert!(rendered.contains("## rust"));
}

#[test]
fn show_flowhub_summarizes_real_research_module() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let show = show_flowhub(flowhub_root().join("research"))
        .unwrap_or_else(|error| panic!("research node should show: {error}"));

    let FlowhubShow::Module(show) = show else {
        panic!("expected Flowhub module summary");
    };
    assert_eq!(show.summary.module_ref, "research");
    assert_eq!(show.summary.kind, FlowhubModuleKind::Composite);
    assert_eq!(show.registered_child_count, 1);
    assert!(
        show.summary
            .child_modules
            .iter()
            .any(|child| child == "research/paper")
    );

    let rendered = render_flowhub_show(&FlowhubShow::Module(show));
    assert_common_show_shape(&rendered);
    assert!(rendered.contains("Module: research"));
    assert!(rendered.contains("Kind: composite"));
    assert!(rendered.contains("Registered children: 1"));
}

#[test]
fn show_flowhub_summarizes_real_leaf_module() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let show = show_flowhub(flowhub_root().join("rust"))
        .unwrap_or_else(|error| panic!("real Flowhub module should show: {error}"));

    let FlowhubShow::Module(show) = show else {
        panic!("expected Flowhub module summary");
    };
    assert_eq!(show.summary.module_ref, "rust");
    assert_eq!(show.summary.kind, FlowhubModuleKind::Leaf);
    assert!(show.summary.child_modules.is_empty());

    let rendered = render_flowhub_show(&FlowhubShow::Module(show));
    assert_common_show_shape(&rendered);
    assert!(rendered.contains("# Flowhub Module"));
    assert!(rendered.contains("Module: rust"));
    assert!(rendered.contains("## Contract"));
    assert!(rendered.contains("Registered children: 0"));
}

#[test]
fn show_flowhub_keeps_required_only_plan_node_as_leaf() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let show = show_flowhub(flowhub_root().join("plan"))
        .unwrap_or_else(|error| panic!("plan node should show: {error}"));

    let FlowhubShow::Module(show) = show else {
        panic!("expected Flowhub module summary");
    };
    assert_eq!(show.summary.module_ref, "plan");
    assert_eq!(show.summary.kind, FlowhubModuleKind::Leaf);
    assert_eq!(show.registered_child_count, 0);
    assert_eq!(show.required_contract_count, 3);
    assert!(show.scenario_cases.is_empty());

    let rendered = render_flowhub_show(&FlowhubShow::Module(show));
    assert_common_show_shape(&rendered);
    assert!(rendered.contains("Required contract entries: 3"));
    assert!(!rendered.contains("## Scenario Cases"));
    assert!(!rendered.contains(".mmd"));
}

#[test]
fn show_flowhub_prefers_declared_graph_name_for_nested_module_summary() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let show = show_flowhub(flowhub_root().join("wendao"))
        .unwrap_or_else(|error| panic!("wendao node should show: {error}"));

    let FlowhubShow::Module(show) = show else {
        panic!("expected Flowhub module summary");
    };
    assert_eq!(show.summary.module_ref, "wendao");
    assert_eq!(show.summary.kind, FlowhubModuleKind::Composite);
    assert_eq!(show.registered_child_count, 1);
    assert_eq!(show.required_contract_count, 4);
    assert!(
        show.summary
            .child_modules
            .iter()
            .any(|child| child == "wendao/client")
    );

    let rendered = render_flowhub_show(&FlowhubShow::Module(show));
    assert_common_show_shape(&rendered);
    assert!(rendered.contains("Kind: composite"));
    assert!(rendered.contains("Registered children: 1"));
    assert!(rendered.contains("Required contract entries: 4"));
    assert!(!rendered.contains(".mmd"));
}
