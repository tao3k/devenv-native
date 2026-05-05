//! HTML escaping and formatting helpers for Telegram messages.

use std::fmt::Write as _;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

use super::escape::{escape_html_attr, escape_html_text, trim_trailing_blank_lines};
use super::options::telegram_markdown_options;

#[derive(Default)]
struct HtmlRenderer {
    rendered: String,
    ordered_list_stack: Vec<usize>,
    list_is_ordered_stack: Vec<bool>,
    table_cell_index: usize,
}

impl HtmlRenderer {
    fn render(mut self, markdown: &str) -> String {
        for event in Parser::new_ext(markdown, telegram_markdown_options()) {
            self.handle_event(event);
        }
        self.finish(markdown)
    }

    fn handle_event(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.handle_start_tag(tag),
            Event::End(tag_end) => self.handle_end_tag(tag_end),
            Event::Text(text) | Event::Html(text) | Event::InlineHtml(text) => {
                self.rendered.push_str(&escape_html_text(text.as_ref()));
            }
            Event::Code(text) => {
                self.rendered.push_str("<code>");
                self.rendered.push_str(&escape_html_text(text.as_ref()));
                self.rendered.push_str("</code>");
            }
            Event::SoftBreak | Event::HardBreak => self.rendered.push('\n'),
            Event::Rule => self.rendered.push_str("\n----\n"),
            Event::TaskListMarker(checked) => {
                if checked {
                    self.rendered.push_str("[x] ");
                } else {
                    self.rendered.push_str("[ ] ");
                }
            }
            Event::FootnoteReference(name) => {
                self.rendered.push('[');
                self.rendered.push_str(&escape_html_text(name.as_ref()));
                self.rendered.push(']');
            }
            _ => {}
        }
    }

    fn handle_start_tag(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Strong | Tag::Heading { .. } => self.rendered.push_str("<b>"),
            Tag::Emphasis => self.rendered.push_str("<i>"),
            Tag::Strikethrough => self.rendered.push_str("<s>"),
            Tag::CodeBlock(_) => self.rendered.push_str("<pre><code>"),
            Tag::Link { dest_url, .. } => {
                self.rendered.push_str("<a href=\"");
                self.rendered.push_str(&escape_html_attr(dest_url.as_ref()));
                self.rendered.push_str("\">");
            }
            Tag::List(start) => self.start_list(start),
            Tag::Item => self.start_list_item(),
            Tag::BlockQuote(_) => self.rendered.push_str("&gt; "),
            Tag::Table(_) => self.start_table(),
            Tag::TableHead | Tag::TableRow => self.start_table_row(),
            Tag::TableCell => self.start_table_cell(),
            _ => {}
        }
    }

    fn handle_end_tag(&mut self, tag_end: TagEnd) {
        match tag_end {
            TagEnd::Strong => self.rendered.push_str("</b>"),
            TagEnd::Emphasis => self.rendered.push_str("</i>"),
            TagEnd::Strikethrough => self.rendered.push_str("</s>"),
            TagEnd::CodeBlock => self.rendered.push_str("</code></pre>\n\n"),
            TagEnd::Link => self.rendered.push_str("</a>"),
            TagEnd::Heading(_) => self.rendered.push_str("</b>\n\n"),
            TagEnd::Paragraph => self.rendered.push_str("\n\n"),
            TagEnd::List(_) => self.end_list(),
            TagEnd::TableHead | TagEnd::TableRow => self.rendered.push_str(" |\n"),
            TagEnd::Table => self.end_table(),
            _ => {}
        }
    }

    fn start_list(&mut self, start: Option<u64>) {
        if let Some(start_number) = start {
            self.ordered_list_stack
                .push(usize::try_from(start_number).unwrap_or(usize::MAX));
            self.list_is_ordered_stack.push(true);
        } else {
            self.ordered_list_stack.push(1);
            self.list_is_ordered_stack.push(false);
        }
    }

    fn start_list_item(&mut self) {
        if !self.rendered.is_empty() && !self.rendered.ends_with('\n') {
            self.rendered.push('\n');
        }
        match self.list_is_ordered_stack.last().copied() {
            Some(true) => {
                if let Some(current) = self.ordered_list_stack.last_mut() {
                    let _ = write!(self.rendered, "{current}. ");
                    *current += 1;
                } else {
                    self.rendered.push_str("• ");
                }
            }
            _ => self.rendered.push_str("• "),
        }
    }

    fn end_list(&mut self) {
        self.ordered_list_stack.pop();
        self.list_is_ordered_stack.pop();
        self.rendered.push('\n');
    }

    fn start_table(&mut self) {
        if !self.rendered.is_empty() && !self.rendered.ends_with('\n') {
            self.rendered.push('\n');
        }
    }

    fn start_table_row(&mut self) {
        if !self.rendered.is_empty() && !self.rendered.ends_with('\n') {
            self.rendered.push('\n');
        }
        self.table_cell_index = 0;
        self.rendered.push_str("| ");
    }

    fn start_table_cell(&mut self) {
        if self.table_cell_index > 0 {
            self.rendered.push_str(" | ");
        }
        self.table_cell_index += 1;
    }

    fn end_table(&mut self) {
        if !self.rendered.ends_with('\n') {
            self.rendered.push('\n');
        }
    }

    fn finish(mut self, markdown: &str) -> String {
        trim_trailing_blank_lines(&mut self.rendered);
        if self.rendered.is_empty() {
            escape_html_text(markdown)
        } else {
            self.rendered
        }
    }
}

#[must_use]
/// Renders generic Markdown into Telegram-compatible HTML markup.
pub fn markdown_to_telegram_html(markdown: &str) -> String {
    HtmlRenderer::default().render(markdown)
}
