//! Integration test for rule dependency analysis in BingoEngine
//!
//! This test verifies that the rule dependency analyzer is properly integrated
//! into the BingoEngine and can analyze dependencies between rules.

use bingo_core::{BingoEngine, types::*};

fn create_test_rule(id: u64, name: &str, input_field: &str, output_field: &str) -> Rule {
    Rule {
        id,
        name: name.to_string(),
        conditions: vec![Condition::Simple {
            field: input_field.to_string(),
            operator: Operator::GreaterThan,
            value: FactValue::Integer(0),
        }],
        actions: vec![Action {
            action_type: ActionType::SetField {
                field: output_field.to_string(),
                value: FactValue::String("processed".to_string()),
            },
        }],
    }
}

#[test]
fn test_rule_dependency_analysis_integration() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Create test rules with data flow dependencies
    let rule1 = create_test_rule(1, "Rule A", "input", "intermediate");
    let rule2 = create_test_rule(2, "Rule B", "intermediate", "output");
    let rule3 = create_test_rule(3, "Rule C", "other_input", "other_output");

    // Add rules to engine
    engine.add_rule(rule1).expect("Failed to add rule 1");
    engine.add_rule(rule2).expect("Failed to add rule 2");
    engine.add_rule(rule3).expect("Failed to add rule 3");

    // Analyze rule dependencies
    let analysis_stats =
        engine.analyze_rule_dependencies().expect("Failed to analyze rule dependencies");

    // Verify analysis completed
    assert_eq!(analysis_stats.rules_analyzed, 3);
    assert!(analysis_stats.analysis_time_ms < 1000); // Should be fast

    // Get dependency analysis statistics
    let stats = engine.get_dependency_analysis_stats();
    assert_eq!(stats.rules_analyzed, 3);

    // Get all detected dependencies
    let dependencies = engine.get_rule_dependencies();
    println!("Dependencies found: {}", dependencies.len());
    for dep in &dependencies {
        println!(
            "  Rule {} -> Rule {} (type: {:?})",
            dep.source_rule, dep.target_rule, dep.dependency_type
        );
    }

    // Get circular dependencies (should be none for this simple case)
    let circular_deps = engine.get_circular_dependencies();
    assert_eq!(circular_deps.len(), 0, "No circular dependencies expected");

    // Get execution clusters
    let clusters = engine.get_execution_clusters().expect("Failed to get execution clusters");
    assert!(
        !clusters.is_empty(),
        "Should have at least one execution cluster"
    );

    println!("Execution clusters: {}", clusters.len());
    for (i, cluster) in clusters.iter().enumerate() {
        println!(
            "  Cluster {}: {} rules, parallel: {}",
            i,
            cluster.rules.len(),
            cluster.parallel_executable
        );
    }

    // Test configuration access
    let config = engine.get_dependency_analysis_config();
    assert!(config.enable_data_flow_analysis);

    // Test statistics reset
    engine.reset_dependency_analysis_stats();
    let reset_stats = engine.get_dependency_analysis_stats();
    assert_eq!(reset_stats.rules_analyzed, 0);
}

#[test]
fn test_rule_dependency_configuration() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Test updating configuration
    let new_config = bingo_core::rule_dependency::DependencyAnalysisConfig {
        enable_circular_detection: false,
        enable_data_flow_analysis: true,
        enable_condition_similarity: false,
        enable_field_conflict_detection: true,
        max_graph_size: 500,
        similarity_threshold: 0.8,
    };

    engine.update_dependency_analysis_config(new_config);

    let config = engine.get_dependency_analysis_config();
    assert!(!config.enable_circular_detection);
    assert!(config.enable_data_flow_analysis);
    assert!(!config.enable_condition_similarity);
    assert!(config.enable_field_conflict_detection);
    assert_eq!(config.max_graph_size, 500);
}

#[test]
fn test_rule_dependency_analysis_empty_rules() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Analyze with no rules
    let analysis_stats = engine
        .analyze_rule_dependencies()
        .expect("Failed to analyze dependencies with no rules");

    assert_eq!(analysis_stats.rules_analyzed, 0);
    assert_eq!(analysis_stats.dependencies_found, 0);

    let dependencies = engine.get_rule_dependencies();
    assert!(dependencies.is_empty());

    let circular_deps = engine.get_circular_dependencies();
    assert!(circular_deps.is_empty());

    let clusters = engine.get_execution_clusters().expect("Failed to get execution clusters");
    assert!(clusters.is_empty());
}
