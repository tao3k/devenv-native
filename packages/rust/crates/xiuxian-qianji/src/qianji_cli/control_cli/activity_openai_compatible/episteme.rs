use std::borrow::Cow;
use std::io;

use serde_json::Value;

use super::io_support::read_local_artifact_text;
use super::types::LlmRequestAudit;
use crate::qianji_cli::invalid_input;

const EPISTEME_REASONING_CONTEXT_KIND: &str = "episteme.reasoning_fill_context";
const EPISTEME_REASONING_CONTEXT_SCHEMA: &str = "xiuxian.wendao.episteme.reasoning_fill_context.v1";
const EPISTEME_REASONING_TARGET_CONTRACT_SCHEMA: &str =
    "xiuxian.wendao.episteme.reasoning_target_contract.v1";
const EPISTEME_REASONING_REVIEW_SCHEMA: &str = "xiuxian.wendao.episteme.reasoning_fill_review.v1";
const EPISTEME_OBJECT_MODEL_COMPATIBILITY: &str = "foundry_style_object_model_v1";
const EPISTEME_OBJECT_MODEL_TARGET_LAYER: &str = "object_model";
const EPISTEME_RDF_SOURCE_AUTHORITY: &str = "rdf";
const OBJECT_FIELD_GROUP: &str = "object_proposal";
const RELATION_FIELD_GROUP: &str = "relation_proposal";
const SERVICE_CATALOG_FIELD_GROUP: &str = "service_catalog_review";
const OBJECT_INSTANCE_FIELD_GROUP: &str = "object_instance_review";
const OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND: &str = "object_model_object_type_candidate";
const OBJECT_MODEL_LINK_TYPE_PATCH_KIND: &str = "object_model_link_type_candidate";
const OBJECT_CANDIDATE_PATCH_KIND: &str = "object_candidate";

pub(super) struct EpistemeReasoningContextExpectation {
    pub(super) fill_item_id: String,
    pub(super) target_ledger_field_group: String,
    pub(super) allowed_patch_kinds: Vec<String>,
}

struct EpistemeTargetContractExpectation {
    target_ledger_field_group: String,
    allowed_patch_kinds: Vec<String>,
}

pub(super) fn read_validated_episteme_reasoning_context(
    audit: &LlmRequestAudit,
) -> io::Result<(String, EpistemeReasoningContextExpectation)> {
    let context_ref = audit.context_ref.as_ref().ok_or_else(|| {
        invalid_input("Episteme ontology reasoning tasks require context_ref evidence")
    })?;
    if context_ref.artifact_kind.as_str() != EPISTEME_REASONING_CONTEXT_KIND {
        return Err(invalid_input(format!(
            "Episteme ontology reasoning context_ref must use artifact kind `{EPISTEME_REASONING_CONTEXT_KIND}`"
        )));
    }
    let context_text = read_local_artifact_text(context_ref)?;
    if context_text.trim().is_empty() {
        return Err(invalid_input(
            "Episteme ontology reasoning context artifact must not be empty",
        ));
    }
    let context_json: Value = serde_json::from_str(&context_text).map_err(|error| {
        invalid_input(format!(
            "Episteme ontology reasoning context artifact must be JSON: {error}"
        ))
    })?;
    let expectation = validate_episteme_reasoning_context_json(&context_json)?;
    Ok((context_text, expectation))
}

fn validate_episteme_reasoning_context_json(
    context: &Value,
) -> io::Result<EpistemeReasoningContextExpectation> {
    if context.get("schema").and_then(Value::as_str) != Some(EPISTEME_REASONING_CONTEXT_SCHEMA) {
        return Err(invalid_input(format!(
            "Episteme ontology reasoning context schema must be `{EPISTEME_REASONING_CONTEXT_SCHEMA}`"
        )));
    }
    let fill_item_id = context
        .pointer("/fillItem/fillItemId")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            invalid_input("Episteme ontology reasoning context requires fillItem.fillItemId")
        })?
        .to_owned();
    let evidence = context
        .get("contextEvidence")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            invalid_input("Episteme ontology reasoning context requires contextEvidence array")
        })?;
    if evidence.is_empty() {
        return Err(invalid_input(
            "Episteme ontology reasoning contextEvidence must not be empty",
        ));
    }
    let has_text = evidence.iter().any(|row| {
        row.get("extractedText")
            .and_then(Value::as_str)
            .is_some_and(|text| !text.trim().is_empty())
    });
    if !has_text {
        return Err(invalid_input(
            "Episteme ontology reasoning contextEvidence must include extractedText",
        ));
    }
    let target_contract = validate_episteme_reasoning_target_contract(context)?;
    Ok(EpistemeReasoningContextExpectation {
        fill_item_id,
        target_ledger_field_group: target_contract.target_ledger_field_group,
        allowed_patch_kinds: target_contract.allowed_patch_kinds,
    })
}

fn validate_episteme_reasoning_target_contract(
    context: &Value,
) -> io::Result<EpistemeTargetContractExpectation> {
    let target_contract = context.get("targetContract").ok_or_else(|| {
        invalid_input("Episteme ontology reasoning context requires targetContract")
    })?;
    if target_contract.get("schema").and_then(Value::as_str)
        != Some(EPISTEME_REASONING_TARGET_CONTRACT_SCHEMA)
    {
        return Err(invalid_input(format!(
            "Episteme ontology reasoning targetContract schema must be `{EPISTEME_REASONING_TARGET_CONTRACT_SCHEMA}`"
        )));
    }
    for field in ["targetLedgerFieldGroup", "patchKind", "reviewMode"] {
        if target_contract
            .get(field)
            .and_then(Value::as_str)
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(invalid_input(format!(
                "Episteme ontology reasoning targetContract requires non-empty {field}"
            )));
        }
    }
    if target_contract
        .get("objectModelCompatibility")
        .and_then(Value::as_str)
        != Some(EPISTEME_OBJECT_MODEL_COMPATIBILITY)
    {
        return Err(invalid_input(format!(
            "Episteme ontology reasoning targetContract objectModelCompatibility must be `{EPISTEME_OBJECT_MODEL_COMPATIBILITY}`"
        )));
    }
    if target_contract
        .get("operationalTargetLayer")
        .and_then(Value::as_str)
        != Some(EPISTEME_OBJECT_MODEL_TARGET_LAYER)
    {
        return Err(invalid_input(format!(
            "Episteme ontology reasoning targetContract operationalTargetLayer must be `{EPISTEME_OBJECT_MODEL_TARGET_LAYER}`"
        )));
    }
    if target_contract
        .get("semanticSourceAuthority")
        .and_then(Value::as_str)
        != Some(EPISTEME_RDF_SOURCE_AUTHORITY)
    {
        return Err(invalid_input(format!(
            "Episteme ontology reasoning targetContract semanticSourceAuthority must be `{EPISTEME_RDF_SOURCE_AUTHORITY}`"
        )));
    }
    if target_contract.get("reviewMode").and_then(Value::as_str) != Some("proposal_patch_only") {
        return Err(invalid_input(
            "Episteme ontology reasoning targetContract reviewMode must be `proposal_patch_only`",
        ));
    }
    for field in ["runtimeMutationAllowed", "rdfMutationAllowed"] {
        if target_contract.get(field).and_then(Value::as_bool) != Some(false) {
            return Err(invalid_input(format!(
                "Episteme ontology reasoning targetContract must keep {field}=false"
            )));
        }
    }
    if !target_contract
        .get("candidatePatchShape")
        .is_some_and(Value::is_object)
    {
        return Err(invalid_input(
            "Episteme ontology reasoning targetContract requires candidatePatchShape object",
        ));
    }
    let target_ledger_field_group = target_contract
        .get("targetLedgerFieldGroup")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let allowed_patch_kinds = allowed_patch_kinds(target_contract)?;
    validate_target_patch_shape(
        target_ledger_field_group.as_str(),
        &allowed_patch_kinds,
        &target_contract["candidatePatchShape"],
    )?;
    Ok(EpistemeTargetContractExpectation {
        target_ledger_field_group,
        allowed_patch_kinds,
    })
}

fn allowed_patch_kinds(target_contract: &Value) -> io::Result<Vec<String>> {
    let allowed = target_contract
        .get("allowedPatchKinds")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            invalid_input("Episteme ontology reasoning targetContract requires allowedPatchKinds")
        })?;
    if allowed.is_empty() {
        return Err(invalid_input(
            "Episteme ontology reasoning targetContract allowedPatchKinds must not be empty",
        ));
    }
    let mut kinds = Vec::with_capacity(allowed.len());
    for kind in allowed {
        let kind = kind
            .as_str()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                invalid_input(
                    "Episteme ontology reasoning targetContract allowedPatchKinds must be strings",
                )
            })?;
        if !is_supported_patch_kind(kind) {
            return Err(invalid_input(format!(
                "Episteme ontology reasoning targetContract unsupported patch kind `{kind}`"
            )));
        }
        kinds.push(kind.to_owned());
    }
    Ok(kinds)
}

fn validate_target_patch_shape(
    field_group: &str,
    allowed_patch_kinds: &[String],
    candidate_patch_shape: &Value,
) -> io::Result<()> {
    let patch_kind = candidate_patch_shape
        .get("patchKind")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            invalid_input(
                "Episteme ontology reasoning targetContract candidatePatchShape requires patchKind",
            )
        })?;
    if !allowed_patch_kinds
        .iter()
        .any(|allowed| allowed == patch_kind)
    {
        return Err(invalid_input(
            "Episteme ontology reasoning targetContract candidatePatchShape patchKind must be allowed",
        ));
    }
    match field_group {
        OBJECT_FIELD_GROUP
            if patch_kind == OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND
                && !candidate_patch_shape
                    .get("objectType")
                    .is_some_and(Value::is_object) =>
        {
            return Err(invalid_input(
                "Episteme ontology reasoning object targetContract requires objectType shape",
            ));
        }
        OBJECT_FIELD_GROUP if patch_kind == OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND => {}
        RELATION_FIELD_GROUP
            if patch_kind == OBJECT_MODEL_LINK_TYPE_PATCH_KIND
                && !candidate_patch_shape
                    .get("linkType")
                    .is_some_and(Value::is_object) =>
        {
            return Err(invalid_input(
                "Episteme ontology reasoning relation targetContract requires linkType shape",
            ));
        }
        RELATION_FIELD_GROUP if patch_kind == OBJECT_MODEL_LINK_TYPE_PATCH_KIND => {}
        OBJECT_FIELD_GROUP | RELATION_FIELD_GROUP => {
            return Err(invalid_input(
                "Episteme ontology reasoning targetContract patchKind does not match targetLedgerFieldGroup",
            ));
        }
        SERVICE_CATALOG_FIELD_GROUP | OBJECT_INSTANCE_FIELD_GROUP
            if patch_kind == OBJECT_CANDIDATE_PATCH_KIND =>
        {
            validate_concrete_object_candidate_shape(candidate_patch_shape)
                .map_err(invalid_input)?;
        }
        SERVICE_CATALOG_FIELD_GROUP | OBJECT_INSTANCE_FIELD_GROUP => {
            return Err(invalid_input(
                "Episteme ontology reasoning concrete-object targetContract requires object_candidate patchKind",
            ));
        }
        _ => {}
    }
    Ok(())
}

fn is_supported_patch_kind(kind: &str) -> bool {
    matches!(
        kind,
        OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND
            | OBJECT_MODEL_LINK_TYPE_PATCH_KIND
            | OBJECT_CANDIDATE_PATCH_KIND
    )
}

pub(super) fn episteme_reasoning_review_json(
    response_text: &str,
    expectation: Option<&EpistemeReasoningContextExpectation>,
) -> Result<Option<Value>, String> {
    let Some(expectation) = expectation else {
        return Ok(None);
    };
    let review_text = strip_markdown_json_fence(response_text);
    let review: Value = serde_json::from_str(review_text.as_ref()).map_err(|error| {
        format!("Episteme ontology reasoning provider content must be JSON: {error}")
    })?;
    validate_episteme_reasoning_review_json(&review, expectation)?;
    Ok(Some(review))
}

fn validate_episteme_reasoning_review_json(
    review: &Value,
    expectation: &EpistemeReasoningContextExpectation,
) -> Result<(), String> {
    if review.get("schema").and_then(Value::as_str) != Some(EPISTEME_REASONING_REVIEW_SCHEMA) {
        return Err(format!(
            "Episteme ontology reasoning review schema must be `{EPISTEME_REASONING_REVIEW_SCHEMA}`"
        ));
    }
    if review.get("status").and_then(Value::as_str) != Some("review_only") {
        return Err("Episteme ontology reasoning review status must be `review_only`".to_owned());
    }
    if review.get("fillItemId").and_then(Value::as_str) != Some(expectation.fill_item_id.as_str()) {
        return Err("Episteme ontology reasoning review fillItemId mismatch".to_owned());
    }
    if review.get("targetLedgerFieldGroup").and_then(Value::as_str)
        != Some(expectation.target_ledger_field_group.as_str())
    {
        return Err(
            "Episteme ontology reasoning review targetLedgerFieldGroup mismatch".to_owned(),
        );
    }
    if review.get("rdfMutation").and_then(Value::as_bool) != Some(false) {
        return Err("Episteme ontology reasoning review must keep rdfMutation=false".to_owned());
    }
    let candidate_patches = review
        .get("candidatePatches")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            "Episteme ontology reasoning review requires candidatePatches array".to_owned()
        })?;
    let candidate_patch_count = review
        .get("candidatePatchCount")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            "Episteme ontology reasoning review requires integer candidatePatchCount".to_owned()
        })?;
    let candidate_patch_count = usize::try_from(candidate_patch_count).map_err(|_| {
        "Episteme ontology reasoning review candidatePatchCount exceeds platform bounds".to_owned()
    })?;
    if candidate_patch_count != candidate_patches.len() {
        return Err(
            "Episteme ontology reasoning review candidatePatchCount must match candidatePatches length"
                .to_owned(),
        );
    }
    for patch in candidate_patches {
        validate_episteme_candidate_patch_json(patch, expectation)?;
    }
    Ok(())
}

fn validate_episteme_candidate_patch_json(
    patch: &Value,
    expectation: &EpistemeReasoningContextExpectation,
) -> Result<(), String> {
    let patch_kind = patch
        .get("patchKind")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            "Episteme ontology reasoning candidate patch requires patchKind".to_owned()
        })?;
    if !expectation
        .allowed_patch_kinds
        .iter()
        .any(|allowed| allowed == patch_kind)
    {
        return Err(format!(
            "Episteme ontology reasoning candidate patch kind `{patch_kind}` is not allowed by targetContract"
        ));
    }
    if patch
        .get("fillItemId")
        .and_then(Value::as_str)
        .is_some_and(|value| value != expectation.fill_item_id)
    {
        return Err("Episteme ontology reasoning candidate patch fillItemId mismatch".to_owned());
    }
    if patch
        .get("targetLedgerFieldGroup")
        .and_then(Value::as_str)
        .is_some_and(|value| value != expectation.target_ledger_field_group)
    {
        return Err(
            "Episteme ontology reasoning candidate patch targetLedgerFieldGroup mismatch"
                .to_owned(),
        );
    }
    validate_candidate_source_evidence(patch)?;
    match patch_kind {
        OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND => validate_object_model_object_patch(patch),
        OBJECT_MODEL_LINK_TYPE_PATCH_KIND => validate_object_model_link_patch(patch),
        OBJECT_CANDIDATE_PATCH_KIND => validate_concrete_object_candidate_patch(patch),
        _ => Err(format!(
            "Episteme ontology reasoning unsupported patch kind `{patch_kind}`"
        )),
    }
}

fn validate_candidate_source_evidence(patch: &Value) -> Result<(), String> {
    let evidence = patch
        .get("sourceEvidence")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            "Episteme ontology reasoning candidate patch requires sourceEvidence array".to_owned()
        })?;
    if evidence.is_empty() {
        return Err(
            "Episteme ontology reasoning candidate patch sourceEvidence must not be empty"
                .to_owned(),
        );
    }
    let has_quote = evidence.iter().any(|row| {
        row.get("fileId")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
            && row
                .get("quote")
                .and_then(Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
    });
    if !has_quote {
        return Err(
            "Episteme ontology reasoning candidate patch sourceEvidence must cite fileId and quote"
                .to_owned(),
        );
    }
    Ok(())
}

fn validate_object_model_object_patch(patch: &Value) -> Result<(), String> {
    let object_type = patch
        .get("objectType")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            "Episteme ontology reasoning object model patch requires objectType object".to_owned()
        })?;
    for field in ["apiName", "displayName", "rdfClass"] {
        if object_type
            .get(field)
            .and_then(Value::as_str)
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(format!(
                "Episteme ontology reasoning objectType requires non-empty {field}"
            ));
        }
    }
    Ok(())
}

fn validate_object_model_link_patch(patch: &Value) -> Result<(), String> {
    let link_type = patch
        .get("linkType")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            "Episteme ontology reasoning object model patch requires linkType object".to_owned()
        })?;
    for field in [
        "apiName",
        "displayName",
        "rdfProperty",
        "fromObjectType",
        "toObjectType",
    ] {
        if link_type
            .get(field)
            .and_then(Value::as_str)
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(format!(
                "Episteme ontology reasoning linkType requires non-empty {field}"
            ));
        }
    }
    Ok(())
}

fn validate_concrete_object_candidate_shape(shape: &Value) -> Result<(), String> {
    for field in ["provisionalObjectKey", "label", "ontologyClassKey"] {
        if shape
            .get(field)
            .and_then(Value::as_str)
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(format!(
                "Episteme ontology reasoning object_candidate shape requires non-empty {field}"
            ));
        }
    }
    Ok(())
}

fn validate_concrete_object_candidate_patch(patch: &Value) -> Result<(), String> {
    validate_concrete_object_candidate_shape(patch)
}

fn strip_markdown_json_fence(response_text: &str) -> Cow<'_, str> {
    let trimmed = response_text.trim();
    let Some(rest) = trimmed.strip_prefix("```") else {
        return Cow::Borrowed(trimmed);
    };
    let Some(first_newline) = rest.find('\n') else {
        return Cow::Borrowed(trimmed);
    };
    let body = &rest[first_newline + 1..];
    let Some(fence_start) = body.rfind("```") else {
        return Cow::Borrowed(trimmed);
    };
    Cow::Owned(body[..fence_start].trim().to_owned())
}
