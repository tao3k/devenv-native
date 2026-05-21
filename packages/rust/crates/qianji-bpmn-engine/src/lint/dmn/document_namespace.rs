use super::evidence::{augment_evidence, root_context};
use super::snapshot_classify::snapshot_import_count;
use crate::dmn_model_api::DmnDocumentSnapshot;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn missing_dmn_model_namespace_issue(
    source_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::from_parts(
        "dmn.missing_model_namespace",
        "DMN document root is missing a supported model namespace",
        format!(
            "Source '{source_id}' does not declare a DMN model namespace such as `https://www.omg.org/spec/DMN/20191111/MODEL/` on the root element."
        ),
        format!(
            "The bounded parser needs one DMN model namespace declaration on the root element so it can recognize the document as DMN and classify its bounded schema/version surface.{}",
            root_context(snapshot)
        ),
        vec![
            "Add one DMN model namespace declaration on the root `<definitions>` element.".to_string(),
            "Prefer one of the bounded model namespaces already proven in this crate, such as `http://www.omg.org/spec/DMN/20180521/MODEL/` or `https://www.omg.org/spec/DMN/20191111/MODEL/`.".to_string(),
            "Do not change business decision ids or decision-table content while adding the namespace declaration.".to_string(),
        ],
        format!(
            "Edit DMN source '{source_id}' and add one supported DMN model namespace declaration to the root `<definitions>` element. Preserve the existing business logic and identifiers while restoring the document-level DMN namespace."
        ),
        augment_evidence(
            json!({
                "source_id": source_id,
                "supported_model_namespaces": [
                    "http://www.omg.org/spec/DMN/20180521/MODEL/",
                    "https://www.omg.org/spec/DMN/20191111/MODEL/"
                ],
            }),
            snapshot,
            None,
        ),
    )
}

pub(super) fn unsupported_dmn_model_namespace_issue(
    source_id: &str,
    model_namespace_uri: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::from_parts(
        "dmn.unsupported_model_namespace",
        "DMN model namespace is outside the bounded parser slice",
        format!(
            "Source '{source_id}' declares DMN model namespace '{model_namespace_uri}', which is outside the bounded document-version slice."
        ),
        format!(
            "The current parser recognizes the bounded DMN model namespaces already proven in this crate and rejects other document-version namespaces before decision parsing continues.{}",
            root_context(snapshot)
        ),
        vec![
            "If the file should stay inside the bounded slice, rewrite only the DMN model namespace declaration to one supported version while preserving the business namespace and decision content.".to_string(),
            "If the file intentionally targets a broader DMN version, keep it as a non-executable artifact and report that the model namespace is not yet supported in this slice.".to_string(),
            "Do not fabricate decision-table rewrites just to work around a document-version mismatch.".to_string(),
        ],
        format!(
            "Inspect DMN source '{source_id}' and do not rewrite decision logic just to bypass the model namespace mismatch. Either move the root DMN model namespace to one supported bounded version while preserving business semantics, or keep the file non-executable and report unsupported DMN model namespace '{model_namespace_uri}'."
        ),
        augment_evidence(
            json!({
                "source_id": source_id,
                "model_namespace_uri": model_namespace_uri,
                "supported_model_namespaces": [
                    "http://www.omg.org/spec/DMN/20180521/MODEL/",
                    "https://www.omg.org/spec/DMN/20191111/MODEL/"
                ],
            }),
            snapshot,
            None,
        ),
    )
}

pub(super) fn missing_dmn_attribute_issue(
    source_id: &str,
    element: &str,
    attribute: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::from_parts(
        "dmn.missing_attribute",
        "Required DMN attribute is missing",
        format!(
            "Element '<{element}>' in source '{source_id}' is missing required attribute '{attribute}'."
        ),
        "The bounded DMN parser needs this attribute to identify a decision, table, clause, or rule consistently.",
        vec![
            format!("Add the missing '{attribute}' attribute on `<{element}>`."),
            "Use a stable identifier or value that remains consistent with the surrounding decision-table structure.".to_string(),
        ],
        format!(
            "Edit DMN source '{source_id}' and add the required '{attribute}' attribute to `<{element}>`. Keep related ids and references stable so the decision table remains coherent."
        ),
        augment_evidence(
            json!({
                "source_id": source_id,
                "element": element,
                "attribute": attribute,
            }),
            snapshot,
            None,
        ),
    )
}

pub(super) fn unsupported_dmn_import_issue(
    source_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    let import_count = snapshot_import_count(snapshot);
    let noun = if import_count == 1 {
        "element"
    } else {
        "elements"
    };

    LintIssue::from_parts(
        "dmn.unsupported_import",
        "DMN file uses top-level imports outside the bounded parser slice",
        format!("Source '{source_id}' declares {import_count} top-level `<import>` {noun}."),
        format!(
            "The current executable DMN slice does not resolve cross-document `<import>` dependencies, so parsing stops before decision-table execution begins.{}",
            root_context(snapshot)
        ),
        vec![
            "Use the reported `document_root.imports` metadata to identify the external DMN namespace, alias, locationURI, and importType before attempting a repair.".to_string(),
            "If the file must become executable in this slice, replace the external dependency only with an explicit and lossless local definition.".to_string(),
            "If the model intentionally depends on another DMN document, preserve the `<import>` and keep the file non-executable in this slice.".to_string(),
            "Do not simply delete the `<import>` element to force parsing when business logic still depends on external decisions or item definitions.".to_string(),
        ],
        format!(
            "Inspect DMN source '{source_id}' and do not delete top-level `<import>` declarations blindly. Either vendor the imported decisions or item definitions into the same file with explicit and lossless semantics, or keep the file non-executable and report that DMN import resolution is unsupported in this slice."
        ),
        augment_evidence(
            json!({
                "source_id": source_id,
                "import_count": import_count,
                "import_resolution": "unsupported",
            }),
            snapshot,
            None,
        ),
    )
}
