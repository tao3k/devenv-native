use std::error::Error;

use xiuxian_db_store::{encode_record_batch_ipc, validate_arrow_ipc_stream};

use super::helpers::{TEST_TABLE, test_contract, valid_batch_with_table};

#[test]
fn validates_arrow_ipc_stream_against_contract() -> Result<(), Box<dyn Error>> {
    let batch = valid_batch_with_table(TEST_TABLE)?;
    let payload = encode_record_batch_ipc(&batch)?;

    validate_arrow_ipc_stream(payload.as_slice(), &test_contract(true))?;

    Ok(())
}

#[test]
fn rejects_empty_arrow_ipc_payload() -> Result<(), Box<dyn Error>> {
    let error = validate_arrow_ipc_stream(&[], &test_contract(true))
        .err()
        .ok_or("validation should reject an empty IPC payload")?;

    assert!(
        error
            .to_string()
            .contains("Arrow IPC payload must not be empty")
    );
    Ok(())
}
