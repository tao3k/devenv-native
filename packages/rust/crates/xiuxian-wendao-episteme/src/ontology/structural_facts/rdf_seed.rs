//! RDF seed rendering for deterministic structural facts.

use std::{fmt::Write as FmtWrite, path::Path};

use anyhow::{Context, Result};

use super::{read_model::StructuralFactsReadModel, write::write_string};

pub(super) fn write_structural_facts_rdf_seed(
    path: &Path,
    read_model: &StructuralFactsReadModel,
) -> Result<()> {
    let mut ttl = String::from(
        "@prefix wdsf: <https://wendao.dev/ontology/structural-facts#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\n",
    );
    for object in &read_model.objects {
        writeln!(
            ttl,
            "<{}> a wdsf:StructuralObject ;\n  wdsf:objectId \"{}\" ;\n  wdsf:objectKind \"{}\" ;\n  wdsf:title \"{}\" ;\n  wdsf:domainId \"{}\" ;\n  wdsf:sourceContractId \"{}\" ;\n  wdsf:documentId \"{}\" ;\n  wdsf:fileId \"{}\" ;\n  wdsf:relativePath \"{}\" ;\n  wdsf:sourceContentHash \"{}\" ;\n  wdsf:ontologyTruth false .\n",
            structural_uri("object", object.id.as_str()),
            literal(object.id.as_str()),
            literal(object.kind.as_str()),
            literal(object.title.as_str()),
            literal(object.domain_id.as_str()),
            literal(object.source_contract_id.as_str()),
            literal(object.document_id.as_str()),
            literal(object.file_id.as_str()),
            literal(object.relative_path.as_str()),
            literal(object.source_content_hash.as_str()),
        )
        .context("failed to render structural facts RDF object")?;
    }
    for relation in &read_model.relations {
        writeln!(
            ttl,
            "<{}> a wdsf:StructuralRelation ;\n  wdsf:relationId \"{}\" ;\n  wdsf:relationKind \"{}\" ;\n  wdsf:source <{}> ;\n  wdsf:target <{}> ;\n  wdsf:domainId \"{}\" ;\n  wdsf:sourceContractId \"{}\" ;\n  wdsf:evidencePath \"{}\" ;\n  wdsf:ontologyTruth false .\n",
            structural_uri("relation", relation.id.as_str()),
            literal(relation.id.as_str()),
            literal(relation.kind.as_str()),
            structural_uri("object", relation.source.as_str()),
            structural_uri("object", relation.target.as_str()),
            literal(relation.domain_id.as_str()),
            literal(relation.source_contract_id.as_str()),
            literal(relation.evidence_path.as_str()),
        )
        .context("failed to render structural facts RDF relation")?;
    }
    write_string(path, &ttl)
}

fn structural_uri(kind: &str, id: &str) -> String {
    format!(
        "urn:wendao:episteme:structural-facts:{kind}:{}",
        uri_segment(id)
    )
}

fn uri_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'.' | b'_' | b'-' => {
                encoded.push(char::from(byte));
            }
            _ => push_percent_encoded(&mut encoded, byte),
        }
    }
    encoded
}

fn push_percent_encoded(encoded: &mut String, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    encoded.push('%');
    encoded.push(char::from(HEX[(byte >> 4) as usize]));
    encoded.push(char::from(HEX[(byte & 0x0F) as usize]));
}

fn literal(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
