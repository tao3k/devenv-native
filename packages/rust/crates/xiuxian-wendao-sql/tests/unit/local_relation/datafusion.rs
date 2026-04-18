use crate::local_relation::datafusion::DataFusionLocalRelationEngine;

#[test]
fn new_with_information_schema_disables_repartitioned_window_and_sort_plans() {
    let engine = DataFusionLocalRelationEngine::new_with_information_schema();
    let options = engine.session().copied_config().options().optimizer.clone();

    assert!(!options.repartition_windows);
    assert!(!options.repartition_sorts);
}
