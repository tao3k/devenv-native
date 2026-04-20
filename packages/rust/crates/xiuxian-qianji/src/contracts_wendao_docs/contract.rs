use serde_json::Value;

#[path = "load.rs"]
mod load;
#[path = "validate.rs"]
mod validate;

use crate::error::QianjiError;

use self::load::WendaoDocsContractSnapshot;

#[derive(Debug, Clone)]
pub(crate) struct WendaoDocsContract {
    snapshot: WendaoDocsContractSnapshot,
    schema_json: Value,
}

pub(super) fn load_wendao_docs_contract(
    contract_id: &str,
) -> Result<WendaoDocsContract, QianjiError> {
    load::load_wendao_docs_contract(contract_id)
}
