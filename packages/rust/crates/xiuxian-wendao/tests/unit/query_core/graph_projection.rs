use std::sync::Arc;

use crate::link_graph::LinkGraphIndex;
use crate::query_core::WendaoRelation;
use crate::query_core::graph::graph_projection_from_relation;
use crate::query_core::operators::GraphDirection;
use crate::query_core::service::query_graph_neighbors_projection;
use crate::test_support::assert_wendao_json_snapshot;

use super::support::{tempdir_or_panic, write_fixture};

#[tokio::test]
async fn query_graph_neighbors_projection_returns_nodes_and_links() {
    let root = tempdir_or_panic("tempdir");
    write_fixture(
        &root.path().join("alpha.md"),
        "# Alpha\n\nSee [[beta]].\n",
        "write alpha",
    );
    write_fixture(
        &root.path().join("beta.md"),
        "# Beta\n\nBody.\n",
        "write beta",
    );

    let index = Arc::new(
        LinkGraphIndex::build(root.path())
            .unwrap_or_else(|error| panic!("build link graph: {error}")),
    );
    let projection = query_graph_neighbors_projection(
        Arc::clone(&index),
        "alpha",
        GraphDirection::Both,
        1,
        10,
        None,
    )
    .await
    .unwrap_or_else(|error| panic!("query graph neighbors projection: {error}"));

    assert_eq!(projection.center.path, "alpha.md");
    assert!(projection.nodes.iter().any(|node| node.path == "beta.md"));
    assert!(
        projection
            .links
            .iter()
            .any(|link| { link.source_path == "alpha.md" && link.target_path == "beta.md" })
    );
    assert_wendao_json_snapshot("query_core_graph_neighbors_projection", &projection);
}

#[test]
fn graph_projection_from_relation_extracts_unique_paths_by_distance() {
    let root = tempdir_or_panic("tempdir");
    write_fixture(
        &root.path().join("alpha.md"),
        "# Alpha\n\nSee [[beta]].\n",
        "write alpha",
    );
    write_fixture(
        &root.path().join("beta.md"),
        "# Beta\n\nBody.\n",
        "write beta",
    );
    let index = LinkGraphIndex::build(root.path())
        .unwrap_or_else(|error| panic!("build link graph: {error}"));

    let batch = arrow::record_batch::RecordBatch::try_new(
        Arc::new(arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("node_id", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("path", arrow::datatypes::DataType::Utf8, false),
            arrow::datatypes::Field::new("title", arrow::datatypes::DataType::Utf8, true),
            arrow::datatypes::Field::new("distance", arrow::datatypes::DataType::UInt64, false),
            arrow::datatypes::Field::new("direction", arrow::datatypes::DataType::Utf8, false),
        ])),
        vec![
            Arc::new(arrow::array::StringArray::from(vec![
                "alpha", "beta", "beta",
            ])) as arrow::array::ArrayRef,
            Arc::new(arrow::array::StringArray::from(vec![
                "alpha.md", "beta.md", "beta.md",
            ])) as arrow::array::ArrayRef,
            Arc::new(arrow::array::StringArray::from(vec![
                Some("Alpha"),
                Some("Beta"),
                Some("Beta"),
            ])) as arrow::array::ArrayRef,
            Arc::new(arrow::array::UInt64Array::from(vec![0, 1, 1])) as arrow::array::ArrayRef,
            Arc::new(arrow::array::StringArray::from(vec![
                "center", "both", "both",
            ])) as arrow::array::ArrayRef,
        ],
    )
    .unwrap_or_else(|error| panic!("graph batch: {error}"));
    let relation = WendaoRelation::new(batch.schema(), vec![batch]);
    let projection = graph_projection_from_relation(&index, &relation)
        .unwrap_or_else(|error| panic!("graph projection: {error}"));

    assert_eq!(projection.nodes.len(), 2);
    assert_eq!(
        projection.paths_at_distance(Some(1)),
        vec!["beta.md".to_string()]
    );
}
