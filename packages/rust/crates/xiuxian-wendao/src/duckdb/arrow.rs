use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use arrow::datatypes::SchemaRef;
use duckdb::core::{DataChunkHandle, LogicalTypeHandle, LogicalTypeId};
use duckdb::vtab::{
    BindInfo, InitInfo, TableFunctionInfo, VTab, record_batch_to_duckdb_data_chunk,
    to_duckdb_logical_type,
};
use xiuxian_db_store::EngineRecordBatch;

pub(crate) const WENDAO_ARROW_RELATION_FUNCTION_NAME: &str = "wendao_arrow_relation";

pub(crate) struct RegisteredArrowRelation {
    schema: SchemaRef,
    batches: Vec<EngineRecordBatch>,
}

impl RegisteredArrowRelation {
    pub(crate) fn new(schema: SchemaRef, batches: Vec<EngineRecordBatch>) -> Self {
        Self { schema, batches }
    }

    fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    fn batch(&self, index: usize) -> Option<&EngineRecordBatch> {
        self.batches.get(index)
    }
}

pub(crate) struct DuckDbArrowRelationStore {
    namespace: String,
    registered_tables: Mutex<HashSet<String>>,
}

impl DuckDbArrowRelationStore {
    pub(crate) fn new(namespace: String) -> Self {
        Self {
            namespace,
            registered_tables: Mutex::new(HashSet::new()),
        }
    }

    pub(crate) fn namespace(&self) -> &str {
        &self.namespace
    }

    pub(crate) fn insert(
        &self,
        table_name: &str,
        schema: SchemaRef,
        batches: Vec<EngineRecordBatch>,
    ) -> Result<(), String> {
        let mut relations = global_arrow_relations()
            .lock()
            .map_err(|_| "duckdb arrow relation store mutex is poisoned".to_string())?;
        relations.insert(
            self.relation_key(table_name),
            Arc::new(RegisteredArrowRelation::new(schema, batches)),
        );
        self.registered_tables
            .lock()
            .map_err(|_| "duckdb arrow relation table-set mutex is poisoned".to_string())?
            .insert(table_name.to_string());
        Ok(())
    }

    pub(crate) fn remove(&self, table_name: &str) -> Result<(), String> {
        let mut relations = global_arrow_relations()
            .lock()
            .map_err(|_| "duckdb arrow relation store mutex is poisoned".to_string())?;
        let _ = relations.remove(&self.relation_key(table_name));
        self.registered_tables
            .lock()
            .map_err(|_| "duckdb arrow relation table-set mutex is poisoned".to_string())?
            .remove(table_name);
        Ok(())
    }

    pub(crate) fn clear(&self) -> Result<(), String> {
        let registered_tables = self
            .registered_tables
            .lock()
            .map_err(|_| "duckdb arrow relation table-set mutex is poisoned".to_string())?
            .clone();
        let mut relations = global_arrow_relations()
            .lock()
            .map_err(|_| "duckdb arrow relation store mutex is poisoned".to_string())?;
        for table_name in registered_tables {
            let _ = relations.remove(&self.relation_key(&table_name));
        }
        Ok(())
    }

    fn lookup_relation(
        namespace: &str,
        table_name: &str,
    ) -> Result<Arc<RegisteredArrowRelation>, String> {
        let relations = global_arrow_relations()
            .lock()
            .map_err(|_| "duckdb arrow relation store mutex is poisoned".to_string())?;
        relations
            .get(&relation_key(namespace, table_name))
            .cloned()
            .ok_or_else(|| {
                format!("duckdb arrow relation `{namespace}:{table_name}` is not registered")
            })
    }

    fn relation_key(&self, table_name: &str) -> String {
        relation_key(&self.namespace, table_name)
    }
}

fn global_arrow_relations() -> &'static Mutex<HashMap<String, Arc<RegisteredArrowRelation>>> {
    static GLOBAL_ARROW_RELATIONS: OnceLock<Mutex<HashMap<String, Arc<RegisteredArrowRelation>>>> =
        OnceLock::new();
    GLOBAL_ARROW_RELATIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn relation_key(namespace: &str, table_name: &str) -> String {
    format!("{namespace}:{table_name}")
}

pub(crate) struct WendaoArrowRelationBindData {
    relation: Arc<RegisteredArrowRelation>,
}

pub(crate) struct WendaoArrowRelationInitData {
    next_batch_index: AtomicUsize,
}

pub(crate) struct WendaoArrowRelationVTab;

impl VTab for WendaoArrowRelationVTab {
    type InitData = WendaoArrowRelationInitData;
    type BindData = WendaoArrowRelationBindData;

    fn bind(bind: &BindInfo) -> Result<Self::BindData, Box<dyn std::error::Error>> {
        let namespace = bind.get_parameter(0).to_string();
        let table_name = bind.get_parameter(1).to_string();
        let relation = DuckDbArrowRelationStore::lookup_relation(&namespace, &table_name)?;
        for field in relation.schema().fields() {
            let logical_type = to_duckdb_logical_type(field.data_type())?;
            bind.add_result_column(field.name(), logical_type);
        }
        Ok(WendaoArrowRelationBindData { relation })
    }

    fn init(_: &InitInfo) -> Result<Self::InitData, Box<dyn std::error::Error>> {
        Ok(WendaoArrowRelationInitData {
            next_batch_index: AtomicUsize::new(0),
        })
    }

    fn func(
        func: &TableFunctionInfo<Self>,
        output: &mut DataChunkHandle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let init_data = func.get_init_data();
        let bind_data = func.get_bind_data();
        let batch_index = init_data.next_batch_index.fetch_add(1, Ordering::Relaxed);
        if let Some(batch) = bind_data.relation.batch(batch_index) {
            record_batch_to_duckdb_data_chunk(batch, output)?;
        } else {
            output.set_len(0);
        }
        Ok(())
    }

    fn parameters() -> Option<Vec<LogicalTypeHandle>> {
        Some(vec![
            LogicalTypeHandle::from(LogicalTypeId::Varchar),
            LogicalTypeHandle::from(LogicalTypeId::Varchar),
        ])
    }
}
