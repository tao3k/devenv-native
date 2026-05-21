use crate::studio::types::CodeAstRetrievalAtomScope;

pub(crate) trait RetrievalChunkLineExt {
    fn with_lines(self, line_start: usize, line_end: usize) -> Self;
    fn with_display(self, display_label: impl Into<String>, excerpt: impl Into<String>) -> Self;
    fn with_attributes(self, attributes: Vec<(String, String)>) -> Self;
}

impl RetrievalChunkLineExt for crate::studio::types::CodeAstRetrievalAtom {
    fn with_lines(mut self, line_start: usize, line_end: usize) -> Self {
        self.line_start = Some(line_start);
        self.line_end = Some(line_end);
        self
    }

    fn with_display(
        mut self,
        display_label: impl Into<String>,
        excerpt: impl Into<String>,
    ) -> Self {
        self.display_label = Some(display_label.into());
        self.excerpt = Some(excerpt.into());
        self
    }

    fn with_attributes(mut self, attributes: Vec<(String, String)>) -> Self {
        self.attributes = attributes;
        self
    }
}

fn slugify_segment(value: &str) -> String {
    let mut slug = String::with_capacity(value.len());
    let mut previous_dash = false;

    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            previous_dash = false;
        } else if !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
    }

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "item".to_string()
    } else {
        slug
    }
}

fn build_stable_fingerprint(value: &str) -> String {
    let mut hash = 5_381_u32;

    for byte in value.bytes() {
        hash = ((hash << 5).wrapping_add(hash)) ^ u32::from(byte);
    }

    format!("fp:{hash:08x}")
}

fn estimate_token_count(value: &str) -> usize {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        0
    } else {
        normalized.len().div_ceil(4)
    }
}

pub(crate) fn build_code_ast_retrieval_atom(
    owner_id: &str,
    path: &str,
    scope: CodeAstRetrievalAtomScope,
    semantic_type: &str,
    locator: &str,
    content: &str,
) -> crate::studio::types::CodeAstRetrievalAtom {
    let scope_slug = match scope {
        CodeAstRetrievalAtomScope::Declaration => "declaration",
        CodeAstRetrievalAtomScope::Block => "block",
        CodeAstRetrievalAtomScope::Symbol => "symbol",
        _ => unreachable!("code AST retrieval atoms only use declaration/symbol surfaces"),
    };
    let path_slug = slugify_segment(path);
    let semantic_slug = slugify_segment(semantic_type);
    let locator_slug = slugify_segment(locator);

    crate::studio::types::CodeAstRetrievalAtom {
        owner_id: owner_id.to_string().into(),
        chunk_id: format!("ast:{path_slug}:{scope_slug}:{semantic_slug}:{locator_slug}").into(),
        semantic_type: semantic_type.to_string().into(),
        fingerprint: build_stable_fingerprint(
            format!("{path}|{scope_slug}|{semantic_type}|{locator}|{content}").as_str(),
        ),
        token_estimate: estimate_token_count(content),
        display_label: None,
        excerpt: None,
        line_start: None,
        line_end: None,
        surface: Some(scope),
        attributes: Vec::new(),
    }
}
