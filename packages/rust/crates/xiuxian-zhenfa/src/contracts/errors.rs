//! JSON-RPC protocol constants and error objects.

/// JSON-RPC protocol version enforced by Zhenfa.
pub const JSONRPC_VERSION: &str = "2.0";

/// Invalid JSON was received by the server.
pub const PARSE_ERROR_CODE: i32 = -32_700;
/// The JSON sent is not a valid Request object.
pub const INVALID_REQUEST_CODE: i32 = -32_600;
/// The method does not exist / is not available.
pub const METHOD_NOT_FOUND_CODE: i32 = -32_601;
/// Invalid method parameter(s).
pub const INVALID_PARAMS_CODE: i32 = -32_602;
/// Internal JSON-RPC error.
pub const INTERNAL_ERROR_CODE: i32 = -32_603;
