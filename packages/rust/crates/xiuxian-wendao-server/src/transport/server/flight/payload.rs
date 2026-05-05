use std::sync::Arc;

use arrow_array::RecordBatch;
use arrow_flight::FlightData;
use arrow_flight::encode::FlightDataEncoderBuilder;
use futures::TryStreamExt;
use futures::stream;
use tokio::sync::OnceCell;
use tonic::Status;

type EngineRecordBatch = RecordBatch;
type LanceRecordBatch = RecordBatch;

#[derive(Debug)]
pub(super) struct FlightRoutePayload {
    pub(super) batches: Vec<EngineRecordBatch>,
    pub(super) app_metadata: Vec<u8>,
    encoded_do_get_frames: OnceCell<Arc<Vec<FlightData>>>,
}

impl FlightRoutePayload {
    pub(super) fn try_new(batch: LanceRecordBatch) -> Result<Self, Status> {
        let batches = [batch];
        Self::from_batches_with_app_metadata(&batches, Vec::new())
    }

    pub(super) fn try_with_app_metadata(
        batch: LanceRecordBatch,
        app_metadata: Vec<u8>,
    ) -> Result<Self, Status> {
        let batches = [batch];
        Self::from_batches_with_app_metadata(&batches, app_metadata)
    }

    pub(super) fn from_batches_with_app_metadata(
        batches: &[LanceRecordBatch],
        app_metadata: Vec<u8>,
    ) -> Result<Self, Status> {
        if batches.is_empty() {
            return Err(Status::internal(
                "Flight route payload must contain at least one record batch",
            ));
        }
        Ok(Self {
            batches: batches.to_vec(),
            app_metadata,
            encoded_do_get_frames: OnceCell::new(),
        })
    }

    pub(super) fn from_engine_batches_with_app_metadata(
        batches: &[EngineRecordBatch],
        app_metadata: Vec<u8>,
    ) -> Result<Self, Status> {
        if batches.is_empty() {
            return Err(Status::internal(
                "Flight route payload must contain at least one record batch",
            ));
        }
        Ok(Self {
            batches: batches.to_vec(),
            app_metadata,
            encoded_do_get_frames: OnceCell::new(),
        })
    }

    pub(super) fn schema(&self) -> Arc<arrow_schema::Schema> {
        self.batches[0].schema()
    }

    pub(super) fn total_rows(&self) -> Result<i64, Status> {
        i64::try_from(
            self.batches
                .iter()
                .map(EngineRecordBatch::num_rows)
                .sum::<usize>(),
        )
        .map_err(|error| Status::internal(format!("failed to represent total records: {error}")))
    }

    pub(super) async fn do_get_frames(&self) -> Result<Arc<Vec<FlightData>>, Status> {
        self.encoded_do_get_frames
            .get_or_try_init(|| async {
                encode_do_get_frames(self.batches.clone())
                    .await
                    .map(Arc::new)
            })
            .await
            .map(Arc::clone)
    }
}

async fn encode_do_get_frames(batches: Vec<EngineRecordBatch>) -> Result<Vec<FlightData>, Status> {
    FlightDataEncoderBuilder::new()
        .build(stream::iter(batches.into_iter().map(
            Ok::<EngineRecordBatch, arrow_flight::error::FlightError>,
        )))
        .map_err(|error| Status::internal(error.to_string()))
        .try_collect::<Vec<_>>()
        .await
}
