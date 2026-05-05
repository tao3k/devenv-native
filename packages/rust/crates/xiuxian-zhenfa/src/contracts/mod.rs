//! JSON-RPC contract types, errors, and file validation helpers.

mod errors;
mod types;
#[cfg(feature = "contract-validation")]
mod validation;

pub use errors::{
    INTERNAL_ERROR_CODE, INVALID_PARAMS_CODE, INVALID_REQUEST_CODE, JSONRPC_VERSION,
    METHOD_NOT_FOUND_CODE, PARSE_ERROR_CODE,
};
pub use types::{JsonRpcErrorObject, JsonRpcId, JsonRpcMeta, JsonRpcRequest, JsonRpcResponse};
#[cfg(feature = "contract-validation")]
pub use validation::{
    ZhenfaContractError, resolve_contract_path, validate_contract, validate_contract_reference,
};
