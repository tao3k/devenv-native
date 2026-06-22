use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::snapshot_dmn_source;

#[test]
fn dmn_snapshot_classifies_literal_expression_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-literal-expression-decision-20191111.dmn",
    ))
    .must("literal-expression DMN source should still produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(
        snapshot.decisions[0].decision_id,
        "Decision_literal_expression"
    );
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
    assert_eq!(snapshot.decisions[0].literal_expression_count, 1);
    assert_eq!(snapshot.decisions[0].context_count, 0);
    assert_eq!(snapshot.decisions[0].invocation_count, 0);
    assert_eq!(snapshot.decisions[0].relation_count, 0);
    assert_eq!(snapshot.decisions[0].function_definition_count, 0);
    assert_eq!(snapshot.decisions[0].list_count, 0);
}

#[test]
fn dmn_snapshot_classifies_context_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source("versioned-context-decision-20191111.dmn"))
        .must("context DMN source should still produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].decision_id, "Decision_context");
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
    assert_eq!(snapshot.decisions[0].literal_expression_count, 0);
    assert_eq!(snapshot.decisions[0].context_count, 1);
    assert_eq!(snapshot.decisions[0].invocation_count, 0);
    assert_eq!(snapshot.decisions[0].relation_count, 0);
    assert_eq!(snapshot.decisions[0].function_definition_count, 0);
    assert_eq!(snapshot.decisions[0].list_count, 0);
}

#[test]
fn dmn_snapshot_classifies_invocation_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-invocation-decision-20191111.dmn",
    ))
    .must("invocation DMN source should still produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].decision_id, "Decision_invocation");
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
    assert_eq!(snapshot.decisions[0].literal_expression_count, 0);
    assert_eq!(snapshot.decisions[0].context_count, 0);
    assert_eq!(snapshot.decisions[0].invocation_count, 1);
    assert_eq!(snapshot.decisions[0].relation_count, 0);
    assert_eq!(snapshot.decisions[0].function_definition_count, 0);
    assert_eq!(snapshot.decisions[0].list_count, 0);
    assert_eq!(snapshot.decisions[0].invocations.len(), 1);

    let invocation = &snapshot.decisions[0].invocations[0];
    assert_eq!(invocation.invocation_id.as_deref(), Some("invocation_1"));
    let invoked_expression = invocation
        .invoked_expression
        .as_ref()
        .must("invocation should preserve invoked expression");
    assert_eq!(
        invoked_expression.expression_id.as_deref(),
        Some("literal_expression_function")
    );
    assert_eq!(invoked_expression.text.as_deref(), Some("scoreCard"));
    assert_eq!(invocation.bindings.len(), 1);

    let binding = &invocation.bindings[0];
    assert_eq!(binding.binding_id.as_deref(), Some("binding_1"));
    let parameter = binding
        .parameter
        .as_ref()
        .must("invocation binding should preserve parameter");
    assert_eq!(parameter.parameter_id.as_deref(), Some("parameter_1"));
    assert_eq!(parameter.name.as_deref(), Some("age"));
    let argument = binding
        .argument
        .as_ref()
        .must("invocation binding should preserve argument");
    assert_eq!(
        argument.expression_id.as_deref(),
        Some("literal_expression_argument")
    );
    assert_eq!(argument.text.as_deref(), Some("applicant.age"));
}

#[test]
fn dmn_snapshot_classifies_relation_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source("versioned-relation-decision-20191111.dmn"))
        .must("relation DMN source should still produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].decision_id, "Decision_relation");
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
    assert_eq!(snapshot.decisions[0].literal_expression_count, 0);
    assert_eq!(snapshot.decisions[0].context_count, 0);
    assert_eq!(snapshot.decisions[0].invocation_count, 0);
    assert_eq!(snapshot.decisions[0].relation_count, 1);
    assert_eq!(snapshot.decisions[0].function_definition_count, 0);
    assert_eq!(snapshot.decisions[0].list_count, 0);
}

#[test]
fn dmn_snapshot_classifies_function_definition_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-function-definition-decision-20191111.dmn",
    ))
    .must("function-definition DMN source should still produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(
        snapshot.decisions[0].decision_id,
        "Decision_function_definition"
    );
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
    assert_eq!(snapshot.decisions[0].literal_expression_count, 0);
    assert_eq!(snapshot.decisions[0].context_count, 0);
    assert_eq!(snapshot.decisions[0].invocation_count, 0);
    assert_eq!(snapshot.decisions[0].relation_count, 0);
    assert_eq!(snapshot.decisions[0].function_definition_count, 1);
    assert_eq!(snapshot.decisions[0].list_count, 0);
    assert_eq!(snapshot.decisions[0].function_definitions.len(), 1);
    let function_definition = &snapshot.decisions[0].function_definitions[0];
    assert_eq!(
        function_definition.function_definition_id.as_deref(),
        Some("function_definition_1")
    );
    assert_eq!(function_definition.kind.as_deref(), Some("FEEL"));
    assert_eq!(function_definition.parameters.len(), 1);
    assert_eq!(
        function_definition.parameters[0].parameter_id.as_deref(),
        Some("parameter_1")
    );
    assert_eq!(
        function_definition.parameters[0].name.as_deref(),
        Some("riskScore")
    );
    assert_eq!(
        function_definition.parameters[0].type_ref.as_deref(),
        Some("number")
    );
    let body = function_definition
        .body
        .as_ref()
        .must("function definition should preserve body literal expression");
    assert_eq!(body.expression_id.as_deref(), Some("literal_expression_1"));
    assert_eq!(body.text.as_deref(), Some("riskScore"));
}

#[test]
fn dmn_snapshot_classifies_list_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source("versioned-list-decision-20191111.dmn"))
        .must("list DMN source should still produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].decision_id, "Decision_list");
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
    assert_eq!(snapshot.decisions[0].literal_expression_count, 0);
    assert_eq!(snapshot.decisions[0].context_count, 0);
    assert_eq!(snapshot.decisions[0].invocation_count, 0);
    assert_eq!(snapshot.decisions[0].relation_count, 0);
    assert_eq!(snapshot.decisions[0].function_definition_count, 0);
    assert_eq!(snapshot.decisions[0].list_count, 1);
}
