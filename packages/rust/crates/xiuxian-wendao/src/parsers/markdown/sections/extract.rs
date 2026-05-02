use std::path::Path;

use super::types::ParsedSection;
use crate::parsers::markdown::code_observation::extract_observations;
use crate::parsers::markdown::links::extract_link_targets_from_occurrences_in_range;
#[cfg(test)]
use xiuxian_wendao_parsers::sections as parser_sections;
use xiuxian_wendao_parsers::sections::MarkdownSection;
#[cfg(test)]
use xiuxian_wendao_parsers::targets as parser_targets;
use xiuxian_wendao_parsers::targets::MarkdownTargetOccurrence;

#[cfg(test)]
pub(crate) fn extract_sections(body: &str, source_path: &Path, root: &Path) -> Vec<ParsedSection> {
    let occurrences = parser_targets::extract_targets(body);
    adapt_sections(
        parser_sections::extract_sections(body),
        &occurrences,
        source_path,
        root,
    )
}

pub(crate) fn adapt_sections(
    sections: Vec<MarkdownSection>,
    occurrences: &[MarkdownTargetOccurrence],
    source_path: &Path,
    root: &Path,
) -> Vec<ParsedSection> {
    sections
        .into_iter()
        .map(|section| {
            let observations = if section.heading_level() > 0 {
                extract_observations(section.attributes())
            } else {
                Vec::new()
            };
            let entities = extract_link_targets_from_occurrences_in_range(
                occurrences,
                source_path,
                root,
                Some((section.byte_start(), section.byte_end())),
            )
            .note_links;
            ParsedSection::from_parser_owned(section, entities, observations)
        })
        .collect()
}
