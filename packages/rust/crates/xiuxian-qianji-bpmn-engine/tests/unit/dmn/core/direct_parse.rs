use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::parse_dmn_decision;

#[test]
fn dmn_parser_direct_literal_expression_materializes_contract() {
    let decision = parse_dmn_decision(&fixture_source(
        "versioned-literal-expression-decision-20191111.dmn",
    ))
    .must("bounded direct literal-expression DMN source should parse");

    assert_eq!(
        decision.decision.decision_id.as_ref(),
        "Decision_literal_expression"
    );
    assert!(decision.table.inputs.is_empty());
    assert!(decision.table.outputs.is_empty());
    assert!(decision.table.rules.is_empty());
    let literal = decision
        .literal_expression
        .as_ref()
        .must("direct literal expression should be retained");
    assert_eq!(
        literal.expression_id.as_deref(),
        Some("literal_expression_1")
    );
    assert_eq!(literal.text.as_ref(), "applicant.age + 1");
}

#[test]
fn dmn_parser_direct_list_expression_materializes_contract() {
    let decision = parse_dmn_decision(&fixture_source("versioned-list-decision-20191111.dmn"))
        .must("bounded direct list DMN source should parse");

    assert_eq!(decision.decision.decision_id.as_ref(), "Decision_list");
    assert!(decision.table.inputs.is_empty());
    assert!(decision.table.outputs.is_empty());
    assert!(decision.table.rules.is_empty());
    let list = decision
        .list_expression
        .as_ref()
        .must("direct list expression should be retained");
    assert_eq!(list.list_id.as_deref(), Some("list_1"));
    assert_eq!(list.items.len(), 2);
    assert_eq!(
        list.items[0].expression_id.as_deref(),
        Some("literal_expression_1")
    );
    assert_eq!(list.items[0].text.as_ref(), "\"approve\"");
    assert_eq!(list.items[1].text.as_ref(), "\"review\"");
}

#[test]
fn dmn_parser_direct_context_expression_materializes_contract() {
    let decision = parse_dmn_decision(&fixture_source("versioned-context-decision-20191111.dmn"))
        .must("bounded direct context DMN source should parse");

    assert_eq!(decision.decision.decision_id.as_ref(), "Decision_context");
    assert!(decision.table.inputs.is_empty());
    assert!(decision.table.outputs.is_empty());
    assert!(decision.table.rules.is_empty());
    let context = decision
        .context_expression
        .as_ref()
        .must("direct context expression should be retained");
    assert_eq!(context.context_id.as_deref(), Some("context_1"));
    assert_eq!(context.entries.len(), 2);
    assert_eq!(
        context.entries[0].entry_id.as_deref(),
        Some("context_entry_1")
    );
    assert_eq!(
        context.entries[0].variable_id.as_deref(),
        Some("variable_1")
    );
    assert_eq!(context.entries[0].variable_name.as_deref(), Some("nextAge"));
    assert_eq!(
        context.entries[0].expression.text.as_ref(),
        "applicant.age + 1"
    );
    assert_eq!(context.entries[1].variable_name, None);
    assert_eq!(context.entries[1].expression.text.as_ref(), "nextAge");
}

#[test]
fn dmn_parser_direct_relation_expression_materializes_contract() {
    let decision = parse_dmn_decision(&fixture_source("versioned-relation-decision-20191111.dmn"))
        .must("bounded direct relation DMN source should parse");

    assert_eq!(decision.decision.decision_id.as_ref(), "Decision_relation");
    assert!(decision.table.inputs.is_empty());
    assert!(decision.table.outputs.is_empty());
    assert!(decision.table.rules.is_empty());
    let relation = decision
        .relation_expression
        .as_ref()
        .must("direct relation expression should be retained");
    assert_eq!(relation.relation_id.as_deref(), Some("relation_1"));
    assert_eq!(relation.columns.len(), 2);
    assert_eq!(relation.columns[0].column_id.as_ref(), "column_1");
    assert_eq!(relation.columns[0].name.as_deref(), Some("lender"));
    assert_eq!(relation.columns[0].type_ref.as_deref(), Some("string"));
    assert_eq!(relation.columns[1].output_key(), "rate");
    assert_eq!(relation.rows.len(), 2);
    assert_eq!(relation.rows[0].row_id.as_deref(), Some("row_1"));
    assert_eq!(relation.rows[0].cells[0].text.as_ref(), "\"Lender A\"");
    assert_eq!(relation.rows[0].cells[1].text.as_ref(), "3.95");
    assert_eq!(relation.rows[1].cells[0].text.as_ref(), "\"Lender B\"");
    assert_eq!(relation.rows[1].cells[1].text.as_ref(), "4.10");
}

#[test]
fn dmn_parser_direct_invocation_materializes_contract() {
    let decision = parse_dmn_decision(&fixture_source(
        "versioned-invocation-decision-20191111.dmn",
    ))
    .must("bounded direct invocation DMN source should parse");

    assert_eq!(
        decision.decision.decision_id.as_ref(),
        "Decision_invocation"
    );
    assert!(decision.table.inputs.is_empty());
    assert!(decision.table.outputs.is_empty());
    assert!(decision.table.rules.is_empty());
    let invocation = decision
        .invocation
        .as_ref()
        .must("direct invocation should be retained");
    assert_eq!(invocation.invocation_id.as_deref(), Some("invocation_1"));
    assert_eq!(
        invocation
            .invoked_expression
            .as_ref()
            .and_then(|expression| expression.expression_id.as_deref()),
        Some("literal_expression_function")
    );
    assert_eq!(
        invocation
            .invoked_expression
            .as_ref()
            .must("invoked expression should be present")
            .text
            .as_ref(),
        "scoreCard"
    );
    assert_eq!(invocation.bindings.len(), 1);
    assert_eq!(
        invocation.bindings[0].binding_id.as_deref(),
        Some("binding_1")
    );
    assert_eq!(
        invocation.bindings[0]
            .parameter
            .as_ref()
            .and_then(|parameter| parameter.name.as_deref()),
        Some("age")
    );
    assert_eq!(
        invocation.bindings[0]
            .argument
            .as_ref()
            .must("binding argument should be present")
            .text
            .as_ref(),
        "applicant.age"
    );
}
