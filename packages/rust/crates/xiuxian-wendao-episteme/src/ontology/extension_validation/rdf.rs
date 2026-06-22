//! RDF validation for Episteme extension contracts.

use std::{collections::BTreeMap, path::Path};

use anyhow::{Context, Result, bail};
use quick_xml::{
    Reader,
    escape::unescape,
    events::{BytesStart, Event},
};

use crate::ontology::manifest::{EpistemeOntologyManifest, resolve_ontology_artifact_path};

use super::pathing::{has_cjk, read_to_string};

#[derive(Debug, Default)]
pub(super) struct ExtensionRdfTerms {
    classes: BTreeMap<String, RdfTerm>,
    object_properties: BTreeMap<String, RdfTerm>,
}

impl ExtensionRdfTerms {
    pub(super) fn class_count(&self) -> usize {
        self.classes.len()
    }

    pub(super) fn object_property_count(&self) -> usize {
        self.object_properties.len()
    }

    pub(super) fn has_class(&self, iri: &str) -> bool {
        self.classes.contains_key(iri)
    }

    pub(super) fn has_object_property(&self, iri: &str) -> bool {
        self.object_properties.contains_key(iri)
    }
}

#[derive(Debug, Default)]
struct RdfTerm {
    has_cjk_label: bool,
}

pub(super) fn collect_extension_rdf_terms(
    episteme_root: &Path,
    manifest: &EpistemeOntologyManifest,
) -> Result<ExtensionRdfTerms> {
    let mut terms = ExtensionRdfTerms::default();
    let require_cjk_label = manifest
        .primary_language
        .as_deref()
        .is_some_and(|language| language.starts_with("zh"));
    for domain in &manifest.domains {
        for rdf_file in &domain.rdf_files {
            let path = resolve_ontology_artifact_path(episteme_root, rdf_file, "rdf_files")
                .map_err(|source| anyhow::anyhow!(source))?;
            collect_rdf_file(path.as_path(), &mut terms)?;
        }
    }
    if require_cjk_label {
        require_labeled_terms("RDF class", &terms.classes)?;
        require_labeled_terms("RDF object property", &terms.object_properties)?;
    }
    Ok(terms)
}

fn require_labeled_terms(label: &str, terms: &BTreeMap<String, RdfTerm>) -> Result<()> {
    for (iri, term) in terms {
        if !term.has_cjk_label {
            bail!("{label} `{iri}` must have a Chinese label");
        }
    }
    Ok(())
}

fn collect_rdf_file(path: &Path, terms: &mut ExtensionRdfTerms) -> Result<()> {
    let content = read_to_string(path)?;
    let mut reader = Reader::from_str(content.as_str());
    reader.config_mut().trim_text(true);
    let mut active = None::<ActiveTerm>;
    let mut in_label = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                let name = event.name();
                let tag = local_name(name.as_ref());
                match tag {
                    "Class" => {
                        active = Some(ActiveTerm {
                            kind: RdfTermKind::Class,
                            iri: rdf_resource(&reader, &event, "about")?.unwrap_or_default(),
                        });
                    }
                    "ObjectProperty" => {
                        active = Some(ActiveTerm {
                            kind: RdfTermKind::ObjectProperty,
                            iri: rdf_resource(&reader, &event, "about")?.unwrap_or_default(),
                        });
                    }
                    "label" if active.is_some() => in_label = true,
                    _ => {}
                }
            }
            Ok(Event::Text(event)) => {
                if in_label && let Some(active) = active.as_ref() {
                    let decoded = event.decode()?;
                    let text = unescape(decoded.as_ref())?;
                    if has_cjk(text.as_ref()) {
                        mark_label(terms, active);
                    }
                }
            }
            Ok(Event::End(event)) => {
                let name = event.name();
                let tag = local_name(name.as_ref());
                match tag {
                    "Class" | "ObjectProperty" => {
                        if let Some(term) = active.take()
                            && !term.iri.is_empty()
                        {
                            ensure_term(terms, &term);
                        }
                        in_label = false;
                    }
                    "label" => in_label = false,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(error) => {
                bail!(
                    "invalid RDF XML `{}` at byte {}: {error}",
                    path.display(),
                    reader.error_position()
                );
            }
            Ok(_) => {}
        }
    }
    Ok(())
}

#[derive(Debug)]
struct ActiveTerm {
    kind: RdfTermKind,
    iri: String,
}

#[derive(Debug)]
enum RdfTermKind {
    Class,
    ObjectProperty,
}

fn ensure_term(terms: &mut ExtensionRdfTerms, active: &ActiveTerm) {
    match active.kind {
        RdfTermKind::Class => {
            terms.classes.entry(active.iri.clone()).or_default();
        }
        RdfTermKind::ObjectProperty => {
            terms
                .object_properties
                .entry(active.iri.clone())
                .or_default();
        }
    }
}

fn mark_label(terms: &mut ExtensionRdfTerms, active: &ActiveTerm) {
    match active.kind {
        RdfTermKind::Class => {
            terms
                .classes
                .entry(active.iri.clone())
                .or_default()
                .has_cjk_label = true;
        }
        RdfTermKind::ObjectProperty => {
            terms
                .object_properties
                .entry(active.iri.clone())
                .or_default()
                .has_cjk_label = true;
        }
    }
}

fn rdf_resource(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    name: &str,
) -> Result<Option<String>> {
    for attribute in event.attributes() {
        let attribute = attribute.with_context(|| "failed to parse RDF attribute")?;
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
