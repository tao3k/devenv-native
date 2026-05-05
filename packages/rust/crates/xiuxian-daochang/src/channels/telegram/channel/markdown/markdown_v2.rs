//! `MarkdownV2` escaping and formatting helpers for Telegram messages.

use std::fmt::Write as _;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

use super::escape::{
    escape_markdown_v2_code, escape_markdown_v2_text, escape_markdown_v2_url,
    normalize_code_fence_language, trim_trailing_blank_lines,
};
use super::options::telegram_markdown_options;

struct MarkdownV2Renderer {
    rendered: String,
    ordered_list_stack: Vec<usize>,
    list_is_ordered_stack: Vec<bool>,
    link_stack: Vec<String>,
    in_code_block: bool,
    table_cell_index: usize,
}

impl MarkdownV2Renderer {
    fn new() -> Self {
        Self {
            rendered: String::new(),
            ordered_list_stack: Vec::new(),
            list_is_ordered_stack: Vec::new(),
            link_stack: Vec::new(),
            in_code_block: false,
            table_cell_index: 0,
        }
    }

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
            Event::Text(text) => self.push_text(text.as_ref()),
            Event::Code(text) => self.push_inline_code(text.as_ref()),
            Event::SoftBreak | Event::HardBreak => self.rendered.push('\n'),
            Event::Rule => self.rendered.push_str("\n\\-\\-\\-\\-\n"),
            Event::Html(text) | Event::InlineHtml(text) => {
                self.rendered
                    .push_str(&escape_markdown_v2_text(text.as_ref()));
            }
            Event::TaskListMarker(checked) => self.push_task_list_marker(checked),
            Event::FootnoteReference(name) => self.push_footnote_reference(name.as_ref()),
            _ => {}
        }
    }

    fn handle_start_tag(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Strong | Tag::Heading { .. } => self.rendered.push('*'),
            Tag::Emphasis => self.rendered.push('_'),
            Tag::Strikethrough => self.rendered.push('~'),
            Tag::CodeBlock(kind) => self.start_code_block(kind),
            Tag::Link { dest_url, .. } => self.start_link(dest_url.into_string()),
            Tag::List(start) => self.start_list(start),
            Tag::Item => self.start_list_item(),
            Tag::BlockQuote(_) => self.rendered.push_str("> "),
            Tag::Table(_) => self.start_table(),
            Tag::TableHead | Tag::TableRow => self.start_table_row(),
            Tag::TableCell => self.start_table_cell(),
            _ => {}
        }
    }

    fn handle_end_tag(&mut self, tag_end: TagEnd) {
        match tag_end {
            TagEnd::Strong => self.rendered.push('*'),
            TagEnd::Emphasis => self.rendered.push('_'),
            TagEnd::Strikethrough => self.rendered.push('~'),
            TagEnd::CodeBlock => self.end_code_block(),
            TagEnd::Link => self.end_link(),
            TagEnd::Heading(_) => self.rendered.push_str("*\n\n"),
            TagEnd::Paragraph => self.rendered.push_str("\n\n"),
            TagEnd::List(_) => self.end_list(),
            TagEnd::TableHead | TagEnd::TableRow => self.end_table_row(),
            TagEnd::Table => self.end_table(),
            _ => {}
        }
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
        self.rendered.push_str("\\| ");
    }

    fn start_table_cell(&mut self) {
        if self.table_cell_index > 0 {
            self.rendered.push_str(" \\| ");
        }
        self.table_cell_index += 1;
    }

    fn end_table_row(&mut self) {
        self.rendered.push_str(" \\|\n");
    }

    fn end_table(&mut self) {
        if !self.rendered.ends_with('\n') {
            self.rendered.push('\n');
        }
    }

    fn start_code_block(&mut self, kind: pulldown_cmark::CodeBlockKind<'_>) {
        self.in_code_block = true;
        self.rendered.push_str("```");
        if let Some(language) = normalize_code_fence_language(kind) {
            self.rendered.push_str(&language);
        }
        self.rendered.push('\n');
    }

    fn end_code_block(&mut self) {
        self.in_code_block = false;
        if !self.rendered.ends_with('\n') {
            self.rendered.push('\n');
        }
        self.rendered.push_str("```\n\n");
    }

    fn start_link(&mut self, target: String) {
        self.rendered.push('[');
        self.link_stack.push(target);
    }

    fn end_link(&mut self) {
        let link_target = self.link_stack.pop().unwrap_or_default();
        self.rendered.push(']');
        self.rendered.push('(');
        self.rendered
            .push_str(&escape_markdown_v2_url(&link_target));
        self.rendered.push(')');
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

    fn end_list(&mut self) {
        self.ordered_list_stack.pop();
        self.list_is_ordered_stack.pop();
        self.rendered.push('\n');
    }

    fn start_list_item(&mut self) {
        if !self.rendered.is_empty() && !self.rendered.ends_with('\n') {
            self.rendered.push('\n');
        }
        match self.list_is_ordered_stack.last().copied() {
            Some(true) => {
                if let Some(current) = self.ordered_list_stack.last_mut() {
                    let _ = write!(self.rendered, "{current}\\. ");
                    *current += 1;
                } else {
                    self.rendered.push_str("• ");
                }
            }
            _ => self.rendered.push_str("• "),
        }
    }

    fn push_text(&mut self, text: &str) {
        if self.in_code_block {
            self.rendered.push_str(&escape_markdown_v2_code(text));
        } else {
            self.rendered.push_str(&escape_markdown_v2_text(text));
        }
    }

    fn push_inline_code(&mut self, code: &str) {
        self.rendered.push('`');
        self.rendered.push_str(&escape_markdown_v2_code(code));
        self.rendered.push('`');
    }

    fn push_task_list_marker(&mut self, checked: bool) {
        if checked {
            self.rendered.push_str("\\[x\\] ");
        } else {
            self.rendered.push_str("\\[ \\] ");
        }
    }

    fn push_footnote_reference(&mut self, name: &str) {
        self.rendered.push_str("\\[");
        self.rendered.push_str(&escape_markdown_v2_text(name));
        self.rendered.push_str("\\]");
    }

    fn finish(mut self, markdown: &str) -> String {
        trim_trailing_blank_lines(&mut self.rendered);
        if self.rendered.is_empty() {
            escape_markdown_v2_text(markdown)
        } else {
            self.rendered
        }
    }
}

#[must_use]
/// Renders generic Markdown into Telegram `MarkdownV2` syntax.
pub fn markdown_to_telegram_markdown_v2(markdown: &str) -> String {
    MarkdownV2Renderer::new().render(markdown)
}
