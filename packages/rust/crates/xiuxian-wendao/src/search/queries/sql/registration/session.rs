use datafusion::prelude::SessionConfig;
use xiuxian_db_store::SearchEngineContext;

pub(crate) fn new_datafusion_sql_query_engine() -> SearchEngineContext {
    let mut config = SessionConfig::new().with_information_schema(true);
    config.options_mut().execution.collect_statistics = true;
    SearchEngineContext::new_with_config(config)
}
