use std::fs;
use std::io;

use tempfile::TempDir;

use super::scan_references;
use super::values::{skills_from_tool_list, yaml_value_to_opt_string_vec};

#[test]
fn test_skills_from_tool_list_unique_and_sorted() {
    let tools = vec![
        "researcher.run_research_graph".to_string(),
        "writer.polish".to_string(),
        "researcher.collect".to_string(),
    ];

    let skills = skills_from_tool_list(&tools);
    assert_eq!(skills, vec!["researcher".to_string(), "writer".to_string()]);
}

#[test]
fn test_yaml_value_to_opt_string_vec_supports_scalar_and_sequence()
-> Result<(), Box<dyn std::error::Error>> {
    let scalar: serde_yaml::Value = serde_yaml::from_str("\"researcher.run\"")?;
    assert_eq!(
        yaml_value_to_opt_string_vec(&scalar),
        Some(vec!["researcher.run".to_string()])
    );

    let sequence: serde_yaml::Value = serde_yaml::from_str(
        r"
- researcher.run
- writer.polish
",
    )?;
    assert_eq!(
        yaml_value_to_opt_string_vec(&sequence),
        Some(vec![
            "researcher.run".to_string(),
            "writer.polish".to_string()
        ])
    );

    Ok(())
}

#[test]
fn test_scan_references_builds_record_from_frontmatter() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let skill_path = temp_dir.path().join("researcher");
    let references_path = skill_path.join("references");
    fs::create_dir_all(&references_path)?;

    let reference_doc = references_path.join("run_research_graph.md");
    fs::write(
        &reference_doc,
        r#"---
type: skill
metadata:
  for_tools:
    - researcher.run_research_graph
  title: "Run Research Graph"
  routing_keywords: ["research", "graph"]
  intents: ["search_docs"]
---
# body
"#,
    )?;

    let records = scan_references(&skill_path, "researcher");
    assert_eq!(records.len(), 1);

    let record = records
        .first()
        .ok_or_else(|| io::Error::other("expected one reference record"))?;
    assert_eq!(record.ref_name, "run_research_graph");
    assert_eq!(record.title, "Run Research Graph");
    assert_eq!(record.skill_name, "researcher");
    assert_eq!(record.for_skills, vec!["researcher".to_string()]);
    assert_eq!(
        record.for_tools,
        Some(vec!["researcher.run_research_graph".to_string()])
    );
    assert_eq!(
        record.keywords,
        vec![
            "research".to_string(),
            "graph".to_string(),
            "search_docs".to_string()
        ]
    );
    assert_eq!(
        record.file_path,
        reference_doc.to_string_lossy().to_string()
    );

    Ok(())
}
