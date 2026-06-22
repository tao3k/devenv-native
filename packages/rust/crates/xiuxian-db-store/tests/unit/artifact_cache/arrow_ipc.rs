use std::sync::Arc;

use arrow::array::{ArrayRef, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use super::{ArtifactBlobCache, ContentAddressedFilesystemBlobCache, sample_key};
use xiuxian_db_store::{read_record_batches_ipc_artifact, write_record_batches_ipc_artifact};

#[test]
fn arrow_ipc_artifact_roundtrips_record_batches() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    let cache = ContentAddressedFilesystemBlobCache::new(temp.path());
    let key = sample_key()?;
    let schema = Arc::new(Schema::new(vec![Field::new(
        "section",
        DataType::Utf8,
        false,
    )]));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![Arc::new(StringArray::from(vec!["authority"])) as ArrayRef],
    )?;

    let write = write_record_batches_ipc_artifact(&cache, &key, std::slice::from_ref(&batch))?;
    assert!(write.byte_len() > 0);
    assert!(cache.contains(&key)?);

    let restored = read_record_batches_ipc_artifact(&cache, &key)?.ok_or("missing IPC artifact")?;
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].num_rows(), 1);
    assert_eq!(restored[0].schema().field(0).name(), "section");
    Ok(())
}
