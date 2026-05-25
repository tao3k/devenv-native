use tonic::Status;
use tonic::metadata::MetadataMap;

use crate::transport::OntologyCandidateInspectionFlightRequest;
use crate::transport::query_contract::{
    DatasetOntologyFlightManifest, WENDAO_DATASET_ONTOLOGY_CONTRACT_ID_HEADER,
    WENDAO_DATASET_ONTOLOGY_MANIFEST_HEADER, WENDAO_DATASET_ONTOLOGY_MAPPING_ID_HEADER,
    WENDAO_ONTOLOGY_CANDIDATE_INSPECTION_REQUEST_HEADER, decode_dataset_ontology_manifest_header,
    decode_ontology_candidate_inspection_request_header,
};

pub(crate) fn validate_dataset_ontology_materialize_request_metadata(
    metadata: &MetadataMap,
) -> Result<DatasetOntologyFlightManifest, Status> {
    let contract_id = required_header(metadata, WENDAO_DATASET_ONTOLOGY_CONTRACT_ID_HEADER)?;
    let mapping_id = required_header(metadata, WENDAO_DATASET_ONTOLOGY_MAPPING_ID_HEADER)?;
    let manifest_value = required_header(metadata, WENDAO_DATASET_ONTOLOGY_MANIFEST_HEADER)?;
    let manifest =
        decode_dataset_ontology_manifest_header(manifest_value.as_str()).map_err(|error| {
            Status::invalid_argument(format!(
                "invalid dataset ontology manifest header `{WENDAO_DATASET_ONTOLOGY_MANIFEST_HEADER}`: {error}"
            ))
        })?;
    if manifest.contract_id != contract_id {
        return Err(Status::invalid_argument(format!(
            "dataset ontology contract header `{WENDAO_DATASET_ONTOLOGY_CONTRACT_ID_HEADER}` does not match manifest contract id"
        )));
    }
    if manifest.mapping_id != mapping_id {
        return Err(Status::invalid_argument(format!(
            "dataset ontology mapping header `{WENDAO_DATASET_ONTOLOGY_MAPPING_ID_HEADER}` does not match manifest mapping id"
        )));
    }
    Ok(manifest)
}

pub(crate) fn validate_ontology_candidate_inspection_request_metadata(
    metadata: &MetadataMap,
) -> Result<OntologyCandidateInspectionFlightRequest, Status> {
    let request_value = required_header(
        metadata,
        WENDAO_ONTOLOGY_CANDIDATE_INSPECTION_REQUEST_HEADER,
    )?;
    decode_ontology_candidate_inspection_request_header(request_value.as_str()).map_err(|error| {
        Status::invalid_argument(format!(
            "invalid ontology candidate inspection request header `{WENDAO_ONTOLOGY_CANDIDATE_INSPECTION_REQUEST_HEADER}`: {error}"
        ))
    })
}

fn required_header(metadata: &MetadataMap, header: &'static str) -> Result<String, Status> {
    let value = metadata
        .get(header)
        .ok_or_else(|| Status::invalid_argument(format!("missing required header `{header}`")))?
        .to_str()
        .map_err(|error| Status::invalid_argument(format!("invalid header `{header}`: {error}")))?
        .trim()
        .to_string();
    if value.is_empty() {
        return Err(Status::invalid_argument(format!(
            "required header `{header}` must not be blank"
        )));
    }
    Ok(value)
}
