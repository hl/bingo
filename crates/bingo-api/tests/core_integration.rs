//! Integration test to ensure API stability between `bingo-api` and `bingo-core`.

// This test lives in `bingo-api` but tests the public interface of `bingo-core`.
// Its purpose is to fail compilation if a breaking change is made in `bingo-core`'s
// public API, thus preventing cross-crate API drift.

use bingo_core::{
    BingoEngine, RuleExecutionResult,
    types::{Action, ActionType, Condition, Fact, FactData, FactValue, Operator, Rule},
};
use std::collections::HashMap;

#[test]
fn test_core_api_signature_stability() {
    // This test ensures that the public API of bingo-core remains compatible
    // with its usage in bingo-api. A compilation failure here indicates a
    // breaking change in bingo-core's public interface, as identified in the
    // codebase analysis.

    // 1. Define a simple rule using bingo_core types.
    // This relies on the public structure of `Rule`, `Condition`, etc.
    let rule = Rule {
        id: 1,
        name: "Core Integration Test Rule".to_string(),
        conditions: vec![Condition::Simple {
            field: "status".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("active".to_string()),
        }],
        actions: vec![Action {
            action_type: ActionType::Log { message: "Integration test rule fired".to_string() },
        }],
    };

    // 2. Create an engine. The correct API is to create an empty engine and then add rules.
    // This test verifies the API that `bingo-api` actually uses.
    let mut engine = BingoEngine::new().expect("Failed to create engine");
    engine.add_rule(rule).expect("Failed to add rule to engine");

    // 3. Define a fact that should match the rule.
    let mut fields = HashMap::new();
    fields.insert(
        "status".to_string(),
        FactValue::String("active".to_string()),
    );
    let fact = Fact::new(100, FactData { fields });

    // 4. Process the fact. This tests the `process_facts` signature.
    let results: Vec<RuleExecutionResult> =
        engine.process_facts(vec![fact]).expect("Failed to process facts");

    // 5. Assert the outcome to ensure the engine is behaving as expected.
    // This validates the structure of `RuleExecutionResult`.
    assert_eq!(results.len(), 1, "The rule should have fired once.");
    let result = &results[0];
    assert_eq!(result.rule_id.to_string(), "1");
    assert_eq!(result.fact_id.to_string(), "100");
    assert!(!result.actions_executed.is_empty());
}
