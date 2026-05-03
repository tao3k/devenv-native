use comrak::{Arena, nodes::NodeValue, parse_document};

use crate::studio::analysis::markdown::compile::types::{CompiledDocument, MarkdownCompiler};
use crate::studio::analysis::markdown::compile::utils::markdown_options;

pub(crate) fn compile_markdown_ir(path: &str, content: &str) -> CompiledDocument {
    MarkdownCompiler::new(path, content).compile()
}

impl MarkdownCompiler<'_> {
    pub(crate) fn compile(mut self) -> CompiledDocument {
        let arena = Arena::new();
        let root = parse_document(&arena, self.content, &markdown_options());

        for node in root.descendants() {
            match &node.data().value {
                NodeValue::Heading(heading) => {
                    self.handle_heading(node, heading.level as usize);
                }
                NodeValue::TaskItem(_) => self.handle_task_item(node),
                NodeValue::CodeBlock(block) => self.handle_code_block(node, block.info.as_str()),
                NodeValue::Math(math) if math.display_math => self.handle_math_node(node),
                NodeValue::Table(..) => self.handle_table(node),
                NodeValue::BlockQuote => self.handle_observation(node),
                NodeValue::Link(link) => self.handle_reference_node(node, link.url.as_str()),
                _ => {}
            }
        }

        self.finalize_section_ranges();
        let retrieval_atoms = self.build_retrieval_atoms();

        CompiledDocument {
            document_hash: blake3::hash(self.content.as_bytes()).to_hex().to_string(),
            nodes: self.nodes,
            edges: self.edges,
            retrieval_atoms,
            diagnostics: self.diagnostics,
        }
    }
}
