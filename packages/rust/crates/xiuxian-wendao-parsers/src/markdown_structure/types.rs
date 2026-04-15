use crate::references::MarkdownReference;
use crate::targets::MarkdownTargetOccurrence;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct MarkdownDocumentMetadata {
    pub(crate) title: Option<String>,
    pub(crate) lead: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownHeading {
    pub(crate) label: String,
    pub(crate) level: usize,
    pub(crate) start_line: usize,
    pub(crate) end_line: usize,
    pub(crate) byte_start: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownTask {
    pub(crate) label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MarkdownStructuralItem {
    Heading(MarkdownHeading),
    Task(MarkdownTask),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct MarkdownStructure {
    pub(crate) items: Vec<MarkdownStructuralItem>,
    pub(crate) lead: Option<String>,
    pub(crate) references: Vec<MarkdownReference>,
    pub(crate) targets: Vec<MarkdownTargetOccurrence>,
}

impl MarkdownStructure {
    pub(crate) fn headings(&self) -> impl Iterator<Item = &MarkdownHeading> {
        self.items.iter().filter_map(|item| match item {
            MarkdownStructuralItem::Heading(heading) => Some(heading),
            MarkdownStructuralItem::Task(_) => None,
        })
    }

    pub(crate) fn first_heading_title(&self) -> Option<&str> {
        self.headings()
            .next()
            .map(|heading| heading.label.as_str())
            .filter(|title| !title.trim().is_empty())
    }

    pub(crate) fn lead_snippet(&self) -> Option<&str> {
        self.lead
            .as_deref()
            .map(str::trim)
            .filter(|lead| !lead.is_empty())
    }

    pub(crate) fn references(&self) -> &[MarkdownReference] {
        self.references.as_slice()
    }

    pub(crate) fn targets(&self) -> &[MarkdownTargetOccurrence] {
        self.targets.as_slice()
    }
}

impl MarkdownDocumentMetadata {
    pub(crate) fn title(&self) -> Option<&str> {
        self.title
            .as_deref()
            .map(str::trim)
            .filter(|title| !title.is_empty())
    }

    pub(crate) fn lead_snippet(&self) -> Option<&str> {
        self.lead
            .as_deref()
            .map(str::trim)
            .filter(|lead| !lead.is_empty())
    }
}
