use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result};
use quick_xml::{
    Reader,
    escape::unescape,
    events::{BytesStart, Event},
};

use super::types::{INSTANCE_RELATION_KIND, OBJECT_INSTANCE_KIND, SourcePatchRdfRow};

const OBJECT_INSTANCE_TYPE: &str =
    "https://wendao.ai/ontology/source-patch#ObjectInstanceSourcePatch";
const INSTANCE_RELATION_TYPE: &str =
    "https://wendao.ai/ontology/source-patch#InstanceRelationSourcePatch";

pub(super) fn read_source_patch_rows_from_rdf(
    rdf_path: &Path,
    expected_target_rdf_file: &str,
) -> Result<Vec<SourcePatchRdfRow>> {
    let content = fs::read_to_string(rdf_path)
        .with_context(|| format!("failed to read `{}`", rdf_path.display()))?;
    parse_source_patch_rows(content.as_str(), expected_target_rdf_file)
        .with_context(|| format!("failed to parse source-patch RDF `{}`", rdf_path.display()))
}

fn parse_source_patch_rows(
    content: &str,
    expected_target_rdf_file: &str,
) -> Result<Vec<SourcePatchRdfRow>> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);
    let mut rows = Vec::new();
    let mut active = None::<RawSourcePatchRecord>;
    let mut active_child = None::<String>;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                let tag = local_name(event.name().as_ref()).to_string();
                if tag == "Description" {
                    active = Some(RawSourcePatchRecord::default());
                } else if active.is_some() {
                    active_child = Some(tag);
                }
            }
            Ok(Event::Empty(event)) => {
                if let Some(record) = active.as_mut()
                    && local_name(event.name().as_ref()) == "type"
                    && let Some(resource) = attribute_value(&reader, &event, "resource")?
                {
                    record.rdf_type = Some(resource);
                }
            }
            Ok(Event::Text(event)) => {
                if let (Some(record), Some(child)) = (active.as_mut(), active_child.as_deref()) {
                    let decoded = event.decode()?;
                    let text = unescape(decoded.as_ref())?;
                    record.insert(child, text.as_ref());
                }
            }
            Ok(Event::CData(event)) => {
                if let (Some(record), Some(child)) = (active.as_mut(), active_child.as_deref()) {
                    let text = event.decode()?;
                    record.insert(child, text.as_ref());
                }
            }
            Ok(Event::End(event)) => {
                let tag = local_name(event.name().as_ref()).to_string();
                if tag == "Description" {
                    if let Some(record) = active.take()
                        && let Some(row) = record.into_row(expected_target_rdf_file)?
                    {
                        rows.push(row);
                    }
                    active_child = None;
                } else if active_child.as_deref() == Some(tag.as_str()) {
                    active_child = None;
                }
            }
            Ok(Event::Eof) => break,
            Err(error) => {
                anyhow::bail!(
                    "invalid source-patch RDF XML at byte {}: {error}",
                    reader.error_position()
                );
            }
            Ok(_) => {}
        }
    }

    Ok(rows)
}

#[derive(Default)]
struct RawSourcePatchRecord {
    rdf_type: Option<String>,
    fields: BTreeMap<String, String>,
}

impl RawSourcePatchRecord {
    fn insert(&mut self, field: &str, value: &str) {
        let value = value.trim();
        if value.is_empty() {
            return;
        }
        self.fields
            .entry(field.to_string())
            .and_modify(|existing| existing.push_str(value))
            .or_insert_with(|| value.to_string());
    }

    fn into_row(self, expected_target_rdf_file: &str) -> Result<Option<SourcePatchRdfRow>> {
        let Some(rdf_type) = self.rdf_type.as_deref() else {
            return Ok(None);
        };
        let expected_record_kind = match rdf_type {
            OBJECT_INSTANCE_TYPE => OBJECT_INSTANCE_KIND,
            INSTANCE_RELATION_TYPE => INSTANCE_RELATION_KIND,
            _ => return Ok(None),
        };
        let record_id = required(&self.fields, "recordId")?;
        let record_kind = required(&self.fields, "recordKind")?;
        if record_kind != expected_record_kind {
            anyhow::bail!(
                "source-patch RDF row `{record_id}` has `recordKind` `{record_kind}` but rdf:type expects `{expected_record_kind}`"
            );
        }
        let target_rdf_file = required(&self.fields, "targetRdfFile")?;
        if target_rdf_file != expected_target_rdf_file {
            anyhow::bail!(
                "source-patch RDF row `{record_id}` targets `{target_rdf_file}` but was read from `{expected_target_rdf_file}`"
            );
        }
        Ok(Some(SourcePatchRdfRow {
            record_id,
            record_kind,
            domain_id: required(&self.fields, "domainId")?,
            target_rdf_file,
            label: optional(&self.fields, "label"),
            object_type: optional(&self.fields, "objectType"),
            source_object_id: optional(&self.fields, "sourceObjectId"),
            predicate: optional(&self.fields, "predicate"),
            target_object_id: optional(&self.fields, "targetObjectId"),
            evidence_id: required(&self.fields, "evidenceId")?,
            review_decision: required(&self.fields, "reviewDecision")?,
            promotion_decision: required(&self.fields, "promotionDecision")?,
            reviewer_id: required(&self.fields, "reviewerId")?,
            apply_action: required(&self.fields, "applyAction")?,
            source_mutation_allowed: parse_bool(&self.fields, "sourceMutationAllowed")?,
            ontology_truth: parse_bool(&self.fields, "ontologyTruth")?,
        }))
    }
}

fn required(fields: &BTreeMap<String, String>, name: &str) -> Result<String> {
    fields
        .get(name)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .with_context(|| format!("source-patch RDF row missing `{name}`"))
}

fn optional(fields: &BTreeMap<String, String>, name: &str) -> String {
    fields.get(name).cloned().unwrap_or_default()
}

fn parse_bool(fields: &BTreeMap<String, String>, name: &str) -> Result<bool> {
    match required(fields, name)?.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        value => anyhow::bail!("source-patch RDF row has invalid `{name}` value `{value}`"),
    }
}

fn attribute_value(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    name: &str,
) -> Result<Option<String>> {
    for attribute in event.attributes() {
        let attribute = attribute?;
        if local_name(attribute.key.as_ref()) == name {
            let value = attribute.decode_and_unescape_value(reader.decoder())?;
            return Ok(Some(value.into_owned()));
        }
    }
    Ok(None)
}

fn local_name(name: &[u8]) -> &str {
    let local = name.rsplit(|byte| *byte == b':').next().unwrap_or(name);
    std::str::from_utf8(local).unwrap_or("")
}
