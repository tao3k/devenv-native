//! Markdown and JSON conversion helpers for XML-facing prompts.

use pulldown_cmark::{
    CodeBlockKind, Event, LinkType, MetadataBlockKind, Options, Parser, Tag, TagEnd,
};
use serde_json::Value;

/// Convert a `serde_json` Value into a normalized XML payload.
#[must_use]
pub fn json_to_xml(value: &Value) -> String {
    let mut writer = XmlWriter::new();
    writer.open("document", &[("type", "json".to_string())]);
    write_json_value(&mut writer, value);
    writer.close("document");
    writer.finish()
}

/// Parse JSON text and convert it into a normalized XML payload.
///
/// # Errors
///
/// Returns `serde_json::Error` when `input` is not valid JSON text.
pub fn json_str_to_xml(input: &str) -> Result<String, serde_json::Error> {
    let value: Value = serde_json::from_str(input)?;
    Ok(json_to_xml(&value))
}

/// Convert Markdown into a normalized XML payload.
#[must_use]
pub fn markdown_to_xml(markdown: &str) -> String {
    let mut writer = XmlWriter::new();
    writer.open("document", &[("type", "markdown".to_string())]);

    let options = Options::all();
    let parser = Parser::new_ext(markdown, options);
    for event in parser {
        render_markdown_event(&mut writer, event);
    }

    writer.close("document");
    writer.finish()
}

fn write_json_value(writer: &mut XmlWriter, value: &Value) {
    match value {
        Value::Null => writer.leaf("null", &[], None),
        Value::Bool(v) => writer.leaf("boolean", &[], Some(if *v { "true" } else { "false" })),
        Value::Number(n) => {
            let (kind, text) = if n.is_i64() || n.is_u64() {
                ("integer", n.to_string())
            } else {
                ("float", n.to_string())
            };
            writer.leaf("number", &[("type", kind.to_string())], Some(text.as_str()));
        }
        Value::String(s) => writer.leaf("string", &[], Some(s.as_str())),
        Value::Array(values) => {
            writer.open("array", &[]);
            for entry in values {
                writer.open("item", &[]);
                write_json_value(writer, entry);
                writer.close("item");
            }
            writer.close("array");
        }
        Value::Object(map) => {
            writer.open("object", &[]);
            for (key, entry) in map {
                writer.open("field", &[("name", key.clone())]);
                write_json_value(writer, entry);
                writer.close("field");
            }
            writer.close("object");
        }
    }
}

fn render_markdown_event(writer: &mut XmlWriter, event: Event) {
    match event {
        Event::Start(tag) => open_markdown_tag(writer, tag),
        Event::End(tag) => close_markdown_tag(writer, tag),
        Event::Text(text) => writer.leaf("text", &[], Some(text.as_ref())),
        Event::Code(code) => writer.leaf("inline_code", &[], Some(code.as_ref())),
        Event::InlineMath(math) => writer.leaf("inline_math", &[], Some(math.as_ref())),
        Event::DisplayMath(math) => writer.leaf("display_math", &[], Some(math.as_ref())),
        Event::Html(html) => writer.leaf("html", &[], Some(html.as_ref())),
        Event::InlineHtml(html) => writer.leaf("inline_html", &[], Some(html.as_ref())),
        Event::FootnoteReference(label) => {
            writer.leaf("footnote_ref", &[("id", label.to_string())], None);
        }
        Event::SoftBreak => writer.leaf("soft_break", &[], None),
        Event::HardBreak => writer.leaf("hard_break", &[], None),
        Event::Rule => writer.leaf("rule", &[], None),
        Event::TaskListMarker(checked) => {
            writer.leaf("task_marker", &[("checked", checked.to_string())], None);
        }
    }
}

fn open_markdown_tag(writer: &mut XmlWriter, tag: Tag) {
    match tag {
        Tag::Paragraph => writer.open("paragraph", &[]),
        Tag::Heading {
            level,
            id,
            classes,
            attrs,
        } => {
            open_heading_tag(writer, level, id, &classes, attrs);
        }
        Tag::BlockQuote(_) => writer.open("blockquote", &[]),
        Tag::CodeBlock(kind) => {
            let mut attributes = Vec::new();
            match kind {
                CodeBlockKind::Fenced(lang) => {
                    attributes.push(("kind", "fenced".to_string()));
                    if !lang.is_empty() {
                        attributes.push(("lang", lang.to_string()));
                    }
                }
                CodeBlockKind::Indented => {
                    attributes.push(("kind", "indented".to_string()));
                }
            }
            writer.open("code_block", &attributes);
        }
        Tag::HtmlBlock => writer.open("html_block", &[]),
        Tag::List(start) => {
            let mut attributes = vec![("ordered", start.is_some().to_string())];
            if let Some(value) = start {
                attributes.push(("start", value.to_string()));
            }
            writer.open("list", &attributes);
        }
        Tag::Item => writer.open("item", &[]),
        Tag::FootnoteDefinition(label) => {
            writer.open("footnote", &[("id", label.to_string())]);
        }
        Tag::Table(_) => writer.open("table", &[]),
        Tag::TableHead => writer.open("table_head", &[]),
        Tag::TableRow => writer.open("table_row", &[]),
        Tag::TableCell => writer.open("table_cell", &[]),
        Tag::DefinitionList => writer.open("definition_list", &[]),
        Tag::DefinitionListTitle => writer.open("definition_title", &[]),
        Tag::DefinitionListDefinition => writer.open("definition_definition", &[]),
        Tag::Emphasis => writer.open("emphasis", &[]),
        Tag::Strong => writer.open("strong", &[]),
        Tag::Strikethrough => writer.open("strikethrough", &[]),
        Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        } => {
            let attributes = media_attributes(link_type, &dest_url, &title, &id);
            writer.open("link", &attributes);
        }
        Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        } => {
            let attributes = media_attributes(link_type, &dest_url, &title, &id);
            writer.open("image", &attributes);
        }
        Tag::MetadataBlock(kind) => {
            let kind = match kind {
                MetadataBlockKind::YamlStyle => "yaml",
                MetadataBlockKind::PlusesStyle => "plus",
            };
            writer.open("metadata", &[("kind", kind.to_string())]);
        }
    }
}

fn open_heading_tag<'a>(
    writer: &mut XmlWriter,
    level: pulldown_cmark::HeadingLevel,
    id: Option<pulldown_cmark::CowStr<'a>>,
    classes: &[pulldown_cmark::CowStr<'a>],
    attrs: Vec<(
        pulldown_cmark::CowStr<'a>,
        Option<pulldown_cmark::CowStr<'a>>,
    )>,
) {
    let mut attributes = vec![("level".to_string(), (level as u8).to_string())];
    if let Some(id) = id {
        attributes.push(("id".to_string(), id.to_string()));
    }
    if !classes.is_empty() {
        let joined = classes
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect::<Vec<_>>()
            .join(" ");
        attributes.push(("class".to_string(), joined));
    }
    for (name, value) in attrs {
        let value = value.map_or_else(|| "true".to_string(), |value| value.to_string());
        attributes.push((name.to_string(), value));
    }

    let borrowed_attributes = attributes
        .iter()
        .map(|(name, value)| (name.as_str(), value.clone()))
        .collect::<Vec<_>>();
    writer.open("heading", &borrowed_attributes);
}

fn media_attributes<'a>(
    link_type: LinkType,
    dest_url: &pulldown_cmark::CowStr<'a>,
    title: &pulldown_cmark::CowStr<'a>,
    id: &pulldown_cmark::CowStr<'a>,
) -> Vec<(&'static str, String)> {
    let mut attributes = vec![("kind", link_type_to_string(link_type))];
    if !dest_url.is_empty() {
        attributes.push(("dest", dest_url.to_string()));
    }
    if !title.is_empty() {
        attributes.push(("title", title.to_string()));
    }
    if !id.is_empty() {
        attributes.push(("id", id.to_string()));
    }
    attributes
}

fn close_markdown_tag(writer: &mut XmlWriter, tag: TagEnd) {
    let name = match tag {
        TagEnd::Paragraph => "paragraph",
        TagEnd::Heading(_) => "heading",
        TagEnd::BlockQuote(_) => "blockquote",
        TagEnd::CodeBlock => "code_block",
        TagEnd::HtmlBlock => "html_block",
        TagEnd::List(_) => "list",
        TagEnd::Item => "item",
        TagEnd::FootnoteDefinition => "footnote",
        TagEnd::Table => "table",
        TagEnd::TableHead => "table_head",
        TagEnd::TableRow => "table_row",
        TagEnd::TableCell => "table_cell",
        TagEnd::DefinitionList => "definition_list",
        TagEnd::DefinitionListTitle => "definition_title",
        TagEnd::DefinitionListDefinition => "definition_definition",
        TagEnd::Emphasis => "emphasis",
        TagEnd::Strong => "strong",
        TagEnd::Strikethrough => "strikethrough",
        TagEnd::Link => "link",
        TagEnd::Image => "image",
        TagEnd::MetadataBlock(_) => "metadata",
    };
    writer.close(name);
}

fn link_type_to_string(link_type: LinkType) -> String {
    match link_type {
        LinkType::Inline => "inline",
        LinkType::Reference => "reference",
        LinkType::ReferenceUnknown => "reference_unknown",
        LinkType::Collapsed => "collapsed",
        LinkType::CollapsedUnknown => "collapsed_unknown",
        LinkType::Shortcut => "shortcut",
        LinkType::ShortcutUnknown => "shortcut_unknown",
        LinkType::Autolink => "autolink",
        LinkType::Email => "email",
    }
    .to_string()
}

fn escape_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

fn escape_attr(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

struct XmlWriter {
    out: String,
    indent: usize,
}

impl XmlWriter {
    fn new() -> Self {
        Self {
            out: String::new(),
            indent: 0,
        }
    }

    fn finish(self) -> String {
        self.out
    }

    fn write_line(&mut self, line: &str) {
        for _ in 0..self.indent {
            self.out.push_str("  ");
        }
        self.out.push_str(line);
        self.out.push('\n');
    }

    fn open(&mut self, name: &str, attrs: &[(&str, String)]) {
        let attrs = render_attrs(attrs);
        self.write_line(&format!("<{name}{attrs}>"));
        self.indent += 1;
    }

    fn close(&mut self, name: &str) {
        if self.indent > 0 {
            self.indent -= 1;
        }
        self.write_line(&format!("</{name}>"));
    }

    fn leaf(&mut self, name: &str, attrs: &[(&str, String)], text: Option<&str>) {
        let attrs = render_attrs(attrs);
        match text {
            Some(text) => {
                let escaped = escape_text(text);
                self.write_line(&format!("<{name}{attrs}>{escaped}</{name}>"));
            }
            None => self.write_line(&format!("<{name}{attrs}/>")),
        }
    }
}

fn render_attrs(attrs: &[(&str, String)]) -> String {
    if attrs.is_empty() {
        return String::new();
    }
    let mut rendered = String::new();
    for (key, value) in attrs {
        rendered.push(' ');
        rendered.push_str(key);
        rendered.push_str("=\"");
        rendered.push_str(&escape_attr(value));
        rendered.push('"');
    }
    rendered
}

#[cfg(test)]
#[path = "../tests/unit/xml_transform.rs"]
mod tests;
