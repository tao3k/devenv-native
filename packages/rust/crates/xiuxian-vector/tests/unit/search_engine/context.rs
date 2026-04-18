use datafusion::prelude::SessionConfig;
use xiuxian_vector::search_engine::SearchEngineContext;

#[test]
fn new_with_config_disables_repartitioned_window_and_sort_plans() {
    let mut config = SessionConfig::new();
    config.options_mut().optimizer.repartition_windows = true;
    config.options_mut().optimizer.repartition_sorts = true;

    let context = SearchEngineContext::new_with_config(config);

    let options = context
        .session()
        .copied_config()
        .options()
        .optimizer
        .clone();
    assert!(!options.repartition_windows);
    assert!(!options.repartition_sorts);
}
