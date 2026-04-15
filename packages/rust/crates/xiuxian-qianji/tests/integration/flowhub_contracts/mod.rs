//! Contract tests for Flowhub root/module discovery, show, and check.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use xiuxian_config_core::resolve_project_root;
use xiuxian_qianji::{
    FlowhubGraphNodeKind, FlowhubGraphTopology, FlowhubModuleKind, FlowhubScenarioCaseSummary,
    FlowhubShow, check_flowhub, classify_flowhub_dir, render_flowhub_check_markdown,
    render_flowhub_graph_show, render_flowhub_show, show_flowhub, show_flowhub_graph,
};

fn repo_root() -> PathBuf {
    resolve_project_root()
        .unwrap_or_else(|| panic!("workspace root should resolve from PRJ_ROOT or git ancestry"))
}

fn flowhub_root() -> PathBuf {
    repo_root().join("qianji-flowhub")
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
"#,
    );
    write_file(
        &wendao_dir.join("docs-search.mmd"),
        r#"
flowchart LR
  A["wendao"] --> B["GET /api/docs/search?repo=<repo>&query=<query>&kind=<kind>&limit=<n>"]
  B --> C["GET /api/docs/page?repo=<repo>&page_id=<page_id>"]
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
"#,
    );
    write_file(
        &wendao_dir.join("docs-search.mmd"),
        r#"
flowchart LR
  A["wendao"] --> B["GET /api/docs/search?repo=<repo>&query=<query>&kind=<kind>&limit=<n>"]
  B --> C["GET /api/docs/page?repo=<repo>&page_id=<page_id>"]
  C --> D["done gate"]
  B -- fail --> R["diagnostics"]
  R --> B
"#,
    );
    root
}

fn create_flowhub_with_undeclared_mermaid_nodes_case(temp_dir: &TempDir) -> PathBuf {
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
"#,
    );
    write_file(
        &plan_dir.join("codex-plan.mmd"),
        r#"
flowchart LR
  A["coding"] --> B["rust"]
  B --> C["style"]
  C --> D["blueprint"]
  D --> E["plan"]
"#,
    );
    root
}

fn create_flowhub_with_mermaid_presentation_directives_case(temp_dir: &TempDir) -> PathBuf {
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
"#,
    );
    write_file(
        &plan_dir.join("codex-plan.mmd"),
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
    );
    root
}

mod check_surface;
mod show_surface;

xiuxian_testing::crate_test_policy_harness!();
