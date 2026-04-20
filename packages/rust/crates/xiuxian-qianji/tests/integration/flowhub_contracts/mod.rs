//! Contract tests for Flowhub root/module discovery, show, and check.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use xiuxian_config_core::resolve_project_root;
use xiuxian_qianji::{
    FlowhubGraphTopology, FlowhubModuleKind, FlowhubScenarioCaseSummary, FlowhubShow,
    check_flowhub, classify_flowhub_dir, render_flowhub_check_markdown, render_flowhub_graph_show,
    render_flowhub_show, show_flowhub, show_flowhub_graph,
};

fn repo_root() -> PathBuf {
    resolve_project_root()
        .unwrap_or_else(|| panic!("workspace root should resolve from PRJ_ROOT or git ancestry"))
}

fn flowhub_root() -> PathBuf {
    repo_root().join("qianji-flowhub")
}

fn real_flowhub_fixture_available() -> bool {
    flowhub_root().join("qianji.toml").is_file()
}

fn assert_common_diagnostic_shape(rendered: &str) {
    assert!(rendered.contains("# Validation Failed"));
    assert!(rendered.contains("Location:"));
    assert!(rendered.contains("Problem:"));
    assert!(rendered.contains("Why it blocks:"));
    assert!(rendered.contains("Fix:"));
}

fn assert_common_show_shape(rendered: &str) {
    assert!(rendered.starts_with("# "));
    assert!(rendered.contains("Location:"));
    assert!(rendered.contains("\n## "));
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("should create {}: {error}", parent.display()));
    }
    fs::write(path, content)
        .unwrap_or_else(|error| panic!("should write {}: {error}", path.display()));
}

fn create_invalid_flowhub(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path().join("flowhub");
    let module_dir = root.join("broken-module");
    fs::create_dir_all(&module_dir).unwrap_or_else(|error| {
        panic!("should create module dir {}: {error}", module_dir.display())
    });
    write_file(
        &root.join("qianji.toml"),
        r#"
version = 1

[flowhub]
name = "test-flowhub"

[contract]
register = ["broken-module"]
required = ["*/qianji.toml"]
"#,
    );
    write_file(
        &module_dir.join("qianji.toml"),
        r#"
version = 1

[module]
name = "broken-module"
tags = ["planning", "broken"]

[exports]
entry = "task.start"
ready = "task.ready"

[contract]
register = ["missing-child"]
required = ["*/qianji.toml"]
"#,
    );
    root
}

fn create_missing_root_contract_flowhub(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path().join("flowhub");
    let module_dir = root.join("coding");
    fs::create_dir_all(&module_dir).unwrap_or_else(|error| {
        panic!("should create module dir {}: {error}", module_dir.display())
    });
    write_file(
        &root.join("qianji.toml"),
        r#"
version = 1

[flowhub]
name = "broken-flowhub"
"#,
    );
    write_file(
        &module_dir.join("qianji.toml"),
        r#"
version = 1

[module]
name = "coding"
tags = ["planning", "coding"]

[exports]
entry = "task.coding-start"
ready = "task.coding-ready"
"#,
    );
    root
}

fn create_leaf_with_unregistered_child_dir_flowhub(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path().join("flowhub");
    let rust_dir = root.join("rust");
    let stray_dir = rust_dir.join("style");
    fs::create_dir_all(&stray_dir)
        .unwrap_or_else(|error| panic!("should create stray dir {}: {error}", stray_dir.display()));
    write_file(
        &root.join("qianji.toml"),
        r#"
version = 1

[flowhub]
name = "test-flowhub"

[contract]
register = ["rust"]
required = ["*/qianji.toml"]
"#,
    );
    write_file(
        &rust_dir.join("qianji.toml"),
        r#"
version = 1

[module]
name = "rust"
tags = ["planning", "coding", "rust"]

[exports]
entry = "task.rust-start"
ready = "task.rust-ready"
"#,
    );
    root
}

fn create_flowhub_with_unregistered_top_level_dir(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path().join("flowhub");
    let rust_dir = root.join("rust");
    let stray_dir = root.join("scratch");
    fs::create_dir_all(&rust_dir)
        .unwrap_or_else(|error| panic!("should create rust dir {}: {error}", rust_dir.display()));
    fs::create_dir_all(&stray_dir).unwrap_or_else(|error| {
        panic!(
            "should create stray top-level dir {}: {error}",
            stray_dir.display()
        )
    });
    write_file(
        &root.join("qianji.toml"),
        r#"
version = 1

[flowhub]
name = "test-flowhub"

[contract]
register = ["rust"]
required = ["*/qianji.toml"]
"#,
    );
    write_file(
        &rust_dir.join("qianji.toml"),
        r#"
version = 1

[module]
name = "rust"
tags = ["planning", "coding", "rust"]

[exports]
entry = "task.rust-start"
ready = "task.rust-ready"
"#,
    );
    root
}

fn create_flowhub_with_invalid_mermaid_case(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path().join("flowhub");
    let plan_dir = root.join("plan");
    fs::create_dir_all(&plan_dir)
        .unwrap_or_else(|error| panic!("should create plan dir {}: {error}", plan_dir.display()));
    write_file(
        &root.join("qianji.toml"),
        r#"
version = 1

[flowhub]
name = "test-flowhub"

[contract]
register = ["plan"]
required = ["*/qianji.toml"]
"#,
    );
    write_file(
        &plan_dir.join("qianji.toml"),
        r#"
version = 1

[module]
name = "plan"
tags = ["planning", "plan"]

[exports]
entry = "task.plan-start"
ready = "task.plan-ready"

[contract]
required = ["codex-plan.mmd"]

[[graph]]
path = "codex-plan.mmd"
topology = "dag"

[[graph.node]]
label = "diagnostics"
kind = "process"
role = "capture blocking diagnostics for bounded-surface repair"
agent_action = "use diagnostics to repair the bounded work surface before retrying"
"#,
    );
    write_file(
        &plan_dir.join("codex-plan.mmd"),
        r#"
flowchart LR
  A["diagnostics"]
"#,
    );
    root
}

fn create_flowhub_with_disconnected_mermaid_case(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path().join("flowhub");
    let plan_dir = root.join("plan");
    fs::create_dir_all(&plan_dir)
        .unwrap_or_else(|error| panic!("should create plan dir {}: {error}", plan_dir.display()));
    write_file(
        &root.join("qianji.toml"),
        r#"
version = 1

[flowhub]
name = "test-flowhub"

[contract]
register = ["coding", "rust", "blueprint", "plan"]
required = ["*/qianji.toml"]
"#,
    );
    for module_name in ["coding", "rust", "blueprint"] {
        let module_dir = root.join(module_name);
        fs::create_dir_all(&module_dir).unwrap_or_else(|error| {
            panic!("should create module dir {}: {error}", module_dir.display())
        });
        write_file(
            &module_dir.join("qianji.toml"),
            &format!(
                r#"
version = 1

[module]
name = "{module_name}"
tags = ["planning", "{module_name}"]

[exports]
entry = "task.{module_name}-start"
ready = "task.{module_name}-ready"
"#
            ),
        );
    }
    write_file(
        &plan_dir.join("qianji.toml"),
        r#"
version = 1

[module]
name = "plan"
tags = ["planning", "plan"]

[exports]
entry = "task.plan-start"
ready = "task.plan-ready"

[contract]
required = ["codex-plan.mmd"]

[[graph]]
path = "codex-plan.mmd"
topology = "dag"

[[graph.node]]
label = "coding"
kind = "context"
role = "define the top-level coding lane"
agent_action = "treat as upstream scope, not a writable artifact"
"#,
    );
    write_file(
        &plan_dir.join("codex-plan.mmd"),
        r#"
flowchart LR
  A["coding"] --> B["rust"]
  C["blueprint"] --> D["plan"]
"#,
    );
    root
}

fn create_flowhub_with_leaf_local_mermaid_case(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path().join("flowhub");
    let wendao_dir = root.join("wendao");
    fs::create_dir_all(&wendao_dir).unwrap_or_else(|error| {
        panic!("should create wendao dir {}: {error}", wendao_dir.display())
    });
    write_file(
        &root.join("qianji.toml"),
        r#"
version = 1

[flowhub]
name = "test-flowhub"

[contract]
register = ["coding", "rust", "blueprint", "plan", "wendao"]
required = ["*/qianji.toml"]
"#,
    );
    for module_name in ["coding", "rust", "blueprint", "plan"] {
        let module_dir = root.join(module_name);
        fs::create_dir_all(&module_dir).unwrap_or_else(|error| {
            panic!("should create module dir {}: {error}", module_dir.display())
        });
        write_file(
            &module_dir.join("qianji.toml"),
            &format!(
                r#"
version = 1

[module]
name = "{module_name}"
tags = ["planning", "{module_name}"]

[exports]
entry = "task.{module_name}-start"
ready = "task.{module_name}-ready"
"#
            ),
        );
    }
    write_file(
        &wendao_dir.join("qianji.toml"),
        r#"
version = 1

[module]
name = "wendao"
tags = ["planning", "wendao"]

[exports]
entry = "task.wendao-start"
ready = "task.wendao-ready"

[contract]
required = ["docs-search.mmd"]

[[graph]]
path = "docs-search.mmd"
topology = "dag"

[[graph.node]]
label = "wendao.docs.search"
kind = "capability_contract"
role = "open the stable docs search capability contract"
agent_action = "treat this label as the owner-owned contract id and materialize its invocation surface from Wendao contract assets"

[[graph.node]]
label = "wendao.docs.document"
kind = "capability_contract"
role = "open the stable docs document capability contract"
agent_action = "treat this label as the owner-owned contract id and reopen one docs page through the declared contract surface"

[[graph.node]]
label = "done gate"
kind = "gate"
role = "allow completion only when required guards and validators pass"
agent_action = "do not treat the slice as complete before qianji check passes"

[[graph.node]]
label = "diagnostics"
kind = "process"
role = "capture blocking diagnostics for bounded-surface repair"
agent_action = "use diagnostics to repair the bounded retrieval loop before retrying"
"#,
    );
    write_file(
        &wendao_dir.join("docs-search.mmd"),
        r#"
flowchart LR
  A["wendao"] --> B["wendao.docs.search"]
  B --> C["wendao.docs.document"]
  C --> D["done gate"]
"#,
    );
    root
}

fn create_flowhub_with_topology_mismatch_case(temp_dir: &TempDir) -> PathBuf {
    let root = temp_dir.path().join("flowhub");
    let wendao_dir = root.join("wendao");
    fs::create_dir_all(&wendao_dir).unwrap_or_else(|error| {
        panic!("should create wendao dir {}: {error}", wendao_dir.display())
    });
    write_file(
        &root.join("qianji.toml"),
        r#"
version = 1

[flowhub]
name = "test-flowhub"

[contract]
register = ["wendao"]
required = ["*/qianji.toml"]
"#,
    );
    write_file(
        &wendao_dir.join("qianji.toml"),
        r#"
version = 1

[module]
name = "wendao"
tags = ["planning", "wendao"]

[exports]
entry = "task.wendao-start"
ready = "task.wendao-ready"

[contract]
required = ["docs-search.mmd"]

[[graph]]
path = "docs-search.mmd"
topology = "dag"

[[graph.node]]
label = "wendao.docs.search"
kind = "capability_contract"
role = "open the stable docs search capability contract"
agent_action = "treat this label as the owner-owned contract id and materialize its invocation surface from Wendao contract assets"

[[graph.node]]
label = "wendao.docs.document"
kind = "capability_contract"
role = "open the stable docs document capability contract"
agent_action = "treat this label as the owner-owned contract id and reopen one docs page through the declared contract surface"

[[graph.node]]
label = "done gate"
kind = "gate"
role = "allow completion only when required guards and validators pass"
agent_action = "do not treat the slice as complete before qianji check passes"

[[graph.node]]
label = "diagnostics"
kind = "process"
role = "capture blocking diagnostics for bounded-surface repair"
agent_action = "use diagnostics to repair the bounded retrieval loop before retrying"
"#,
    );
    write_file(
        &wendao_dir.join("docs-search.mmd"),
        r#"
flowchart LR
  A["wendao"] --> B["wendao.docs.search"]
  B --> C["wendao.docs.document"]
  C --> D["done gate"]
  B -- fail --> R["diagnostics"]
  R --> B
"#,
    );
    root
}

fn planning_flowhub_root_manifest() -> &'static str {
    r#"
version = 1

[flowhub]
name = "test-flowhub"

[contract]
register = ["coding", "rust", "blueprint", "plan"]
required = ["*/qianji.toml"]
"#
}

fn planning_module_manifest(module_name: &str) -> String {
    format!(
        r#"
version = 1

[module]
name = "{module_name}"
tags = ["planning", "{module_name}"]

[exports]
entry = "task.{module_name}-start"
ready = "task.{module_name}-ready"
"#
    )
}

fn planning_plan_manifest(topology: &str) -> String {
    format!(
        r#"
version = 1

[module]
name = "plan"
tags = ["planning", "plan"]

[exports]
entry = "task.plan-start"
ready = "task.plan-ready"

[contract]
required = ["codex-plan.mmd"]

[[graph]]
path = "codex-plan.mmd"
topology = "{topology}"

[[graph.node]]
label = "Codex write bounded surface"
kind = "process"
role = "write the bounded work surface from the graph contract"
agent_action = "write qianji.toml, flowchart.mmd, blueprint/, and plan/ for the bounded slice"

[[graph.node]]
label = "surface check"
kind = "guard"
role = "require the bounded work surface to exist"
agent_action = "ensure qianji.toml, flowchart.mmd, blueprint/, and plan/ exist"

[[graph.node]]
label = "flowchart alignment"
kind = "guard"
role = "ensure the flowchart matches the current bounded artifact surface"
agent_action = "keep flowchart.mmd aligned with blueprint and plan"

[[graph.node]]
label = "boundary and drift check"
kind = "guard"
role = "ensure the bounded artifact state remains inside contract boundaries without drift"
agent_action = "keep blueprint/ and plan/ inside the bounded surface and consistent with the shown graph"

[[graph.node]]
label = "domain validators"
kind = "validator"
role = "require domain validators to pass before completion"
agent_action = "prepare the artifact state so required domain validators can succeed"

[[graph.node]]
label = "done gate"
kind = "gate"
role = "allow completion only when required guards and validators pass"
agent_action = "do not treat the slice as complete before qianji check passes"

[[graph.node]]
label = "diagnostics"
kind = "process"
role = "capture blocking diagnostics for bounded-surface repair"
agent_action = "use diagnostics to repair the bounded work surface before retrying"
"#
    )
}

fn write_planning_modules(root: &Path) {
    for module_name in ["coding", "rust", "blueprint"] {
        let module_dir = root.join(module_name);
        fs::create_dir_all(&module_dir).unwrap_or_else(|error| {
            panic!("should create module dir {}: {error}", module_dir.display())
        });
        write_file(
            &module_dir.join("qianji.toml"),
            &planning_module_manifest(module_name),
        );
    }
}

fn create_planning_flowhub_case(temp_dir: &TempDir, topology: &str, plan_graph: &str) -> PathBuf {
    let root = temp_dir.path().join("flowhub");
    let plan_dir = root.join("plan");
    fs::create_dir_all(&plan_dir)
        .unwrap_or_else(|error| panic!("should create plan dir {}: {error}", plan_dir.display()));
    write_file(&root.join("qianji.toml"), planning_flowhub_root_manifest());
    write_planning_modules(&root);
    write_file(
        &plan_dir.join("qianji.toml"),
        &planning_plan_manifest(topology),
    );
    write_file(&plan_dir.join("codex-plan.mmd"), plan_graph);
    root
}

fn create_flowhub_with_undeclared_mermaid_nodes_case(temp_dir: &TempDir) -> PathBuf {
    create_planning_flowhub_case(
        temp_dir,
        "dag",
        r#"
flowchart LR
  A["coding"] --> B["rust"]
  B --> C["style"]
  C --> D["blueprint"]
  D --> E["plan"]
"#,
    )
}

fn create_flowhub_with_mermaid_presentation_directives_case(temp_dir: &TempDir) -> PathBuf {
    create_planning_flowhub_case(
        temp_dir,
        "bounded_loop",
        r#"
flowchart LR
  A["coding"] --> B["rust"]
  B --> C["blueprint"]
  C --> D["plan"]

  D --> E["Codex write bounded surface"]
  E --> F["surface check"]
  F --> G["flowchart alignment"]
  G --> H["boundary and drift check"]
  H --> I["domain validators"]
  I --> J["done gate"]

  F -- fail --> R["diagnostics"]
  G -- fail --> R
  H -- fail --> R
  I -- fail --> R
  R --> E

  classDef highlight fill:#f9f,stroke:#333,stroke-width:2px;
  class A,B highlight;
  style C fill:#e0f7fa,stroke:#006064;
  click G "https://example.com/flowchart-alignment" "flowchart alignment docs"
"#,
    )
}

mod check_surface;
mod show_surface;

xiuxian_testing::crate_test_policy_harness!();
