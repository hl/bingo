//! Comprehensive tests for the Conflict Resolution system
//!
//! This test suite validates the Phase 5 conflict resolution features,
//! including rule prioritization, execution ordering, and various resolution strategies.

use bingo_core::{
    conflict_resolution::{ConflictResolutionConfig, ConflictResolutionStrategy, RuleExecution},
    engine::BingoEngine,
    types::*,
};
use std::collections::HashMap;

#[test]
fn test_conflict_resolution_manager_creation() {
    let engine = BingoEngine::new().expect("Failed to create engine");

    // Test initial configuration
    let config = engine.get_conflict_resolution_config();
    assert_eq!(
        config.primary_strategy,
        ConflictResolutionStrategy::Priority
    );
    assert_eq!(
        config.tie_breaker,
        Some(ConflictResolutionStrategy::Recency)
    );
    assert!(!config.enable_logging);
    assert_eq!(config.max_conflict_set_size, 1000);

    println!("✅ Conflict resolution manager creation test passed");
}

#[test]
fn test_rule_priority_registration() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Register rule priorities
    let result1 = engine.register_rule_priority(1, 50, 100);
    assert!(result1.is_ok());

    let result2 = engine.register_rule_priority(2, 10, 50);
    assert!(result2.is_ok());

    // Test priority updates
    let result3 = engine.set_rule_priority(1, 75);
    assert!(result3.is_ok());

    let result4 = engine.set_rule_salience(2, 200);
    assert!(result4.is_ok());

    println!("✅ Rule priority registration test passed");
}

#[test]
fn test_conflict_resolution_configuration() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Test configuration update
    let new_config = ConflictResolutionConfig {
        primary_strategy: ConflictResolutionStrategy::Salience,
        tie_breaker: Some(ConflictResolutionStrategy::Specificity),
        enable_logging: true,
        max_conflict_set_size: 500,
    };

    engine.configure_conflict_resolution(new_config.clone());

    // Verify configuration was updated
    let updated_config = engine.get_conflict_resolution_config();
    assert_eq!(
        updated_config.primary_strategy,
        ConflictResolutionStrategy::Salience
    );
    assert_eq!(
        updated_config.tie_breaker,
        Some(ConflictResolutionStrategy::Specificity)
    );
    assert!(updated_config.enable_logging);
    assert_eq!(updated_config.max_conflict_set_size, 500);

    println!("✅ Conflict resolution configuration test passed");
}

#[test]
fn test_priority_based_conflict_resolution() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Create test rules with different priorities
    let rules = vec![
        create_test_rule(1, "Low Priority Rule", 1),
        create_test_rule(2, "High Priority Rule", 2),
        create_test_rule(3, "Medium Priority Rule", 3),
    ];

    // Register rule priorities
    engine.register_rule_priority(1, 10, 0).expect("Failed to register rule 1");
    engine.register_rule_priority(2, 100, 0).expect("Failed to register rule 2");
    engine.register_rule_priority(3, 50, 0).expect("Failed to register rule 3");

    // Create rule executions
    let rule_executions = vec![
        create_test_rule_execution(rules[0].clone(), 10, 0),
        create_test_rule_execution(rules[1].clone(), 100, 0),
        create_test_rule_execution(rules[2].clone(), 50, 0),
    ];

    // Resolve conflicts
    let result = engine.resolve_rule_conflicts(rule_executions);
    assert!(result.is_ok());

    let ordered_rules = result.unwrap();

    // Should be ordered by priority: High (100), Medium (50), Low (10)
    assert_eq!(ordered_rules[0].rule_id(), 2);
    assert_eq!(ordered_rules[1].rule_id(), 3);
    assert_eq!(ordered_rules[2].rule_id(), 1);

    println!("✅ Priority-based conflict resolution test passed");
}

#[test]
fn test_salience_based_conflict_resolution() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Configure for salience-based resolution
    let config = ConflictResolutionConfig {
        primary_strategy: ConflictResolutionStrategy::Salience,
        tie_breaker: None,
        enable_logging: false,
        max_conflict_set_size: 1000,
    };
    engine.configure_conflict_resolution(config);

    // Create test rules with different salience values
    let rules = vec![
        create_test_rule(1, "Low Salience Rule", 1),
        create_test_rule(2, "High Salience Rule", 2),
        create_test_rule(3, "Medium Salience Rule", 3),
    ];

    // Create rule executions with different salience
    let rule_executions = vec![
        create_test_rule_execution(rules[0].clone(), 0, 10), // Low salience
        create_test_rule_execution(rules[1].clone(), 0, 100), // High salience
        create_test_rule_execution(rules[2].clone(), 0, 50), // Medium salience
    ];

    // Resolve conflicts
    let ordered_rules = engine.resolve_rule_conflicts(rule_executions).unwrap();

    // Should be ordered by salience: High (100), Medium (50), Low (10)
    assert_eq!(ordered_rules[0].rule_id(), 2);
    assert_eq!(ordered_rules[1].rule_id(), 3);
    assert_eq!(ordered_rules[2].rule_id(), 1);

    println!("✅ Salience-based conflict resolution test passed");
}

#[test]
fn test_specificity_based_conflict_resolution() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Configure for specificity-based resolution
    let config = ConflictResolutionConfig {
        primary_strategy: ConflictResolutionStrategy::Specificity,
        tie_breaker: None,
        enable_logging: false,
        max_conflict_set_size: 1000,
    };
    engine.configure_conflict_resolution(config);

    // Create test rules with different numbers of conditions
    let simple_rule = create_test_rule_with_conditions(1, "Simple Rule", 1);
    let complex_rule = create_test_rule_with_conditions(2, "Complex Rule", 3);
    let medium_rule = create_test_rule_with_conditions(3, "Medium Rule", 2);

    let rule_executions = vec![
        create_test_rule_execution(simple_rule, 0, 0),
        create_test_rule_execution(complex_rule, 0, 0),
        create_test_rule_execution(medium_rule, 0, 0),
    ];

    // Resolve conflicts
    let ordered_rules = engine.resolve_rule_conflicts(rule_executions).unwrap();

    // Should be ordered by specificity: Complex (3), Medium (2), Simple (1)
    assert_eq!(ordered_rules[0].rule_id(), 2);
    assert_eq!(ordered_rules[1].rule_id(), 3);
    assert_eq!(ordered_rules[2].rule_id(), 1);

    println!("✅ Specificity-based conflict resolution test passed");
}

#[test]
fn test_lexicographic_conflict_resolution() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Configure for lexicographic resolution
    let config = ConflictResolutionConfig {
        primary_strategy: ConflictResolutionStrategy::Lexicographic,
        tie_breaker: None,
        enable_logging: false,
        max_conflict_set_size: 1000,
    };
    engine.configure_conflict_resolution(config);

    // Create test rules with different names
    let rules = vec![
        create_test_rule(1, "Zebra Rule", 1),
        create_test_rule(2, "Alpha Rule", 1),
        create_test_rule(3, "Beta Rule", 1),
    ];

    let rule_executions = vec![
        create_test_rule_execution(rules[0].clone(), 0, 0),
        create_test_rule_execution(rules[1].clone(), 0, 0),
        create_test_rule_execution(rules[2].clone(), 0, 0),
    ];

    // Resolve conflicts
    let ordered_rules = engine.resolve_rule_conflicts(rule_executions).unwrap();

    // Should be ordered alphabetically: Alpha, Beta, Zebra
    assert_eq!(ordered_rules[0].rule_name(), "Alpha Rule");
    assert_eq!(ordered_rules[1].rule_name(), "Beta Rule");
    assert_eq!(ordered_rules[2].rule_name(), "Zebra Rule");

    println!("✅ Lexicographic conflict resolution test passed");
}

#[test]
fn test_tie_breaker_resolution() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Configure with primary strategy and tie breaker
    let config = ConflictResolutionConfig {
        primary_strategy: ConflictResolutionStrategy::Priority,
        tie_breaker: Some(ConflictResolutionStrategy::Lexicographic),
        enable_logging: false,
        max_conflict_set_size: 1000,
    };
    engine.configure_conflict_resolution(config);

    // Create rules with same priority but different names
    let rules = vec![
        create_test_rule(1, "Zebra Rule", 1),
        create_test_rule(2, "Alpha Rule", 1),
        create_test_rule(3, "Low Priority", 1),
    ];

    let rule_executions = vec![
        create_test_rule_execution(rules[0].clone(), 50, 0), // Same priority
        create_test_rule_execution(rules[1].clone(), 50, 0), // Same priority
        create_test_rule_execution(rules[2].clone(), 10, 0), // Different priority
    ];

    // Resolve conflicts
    let ordered_rules = engine.resolve_rule_conflicts(rule_executions).unwrap();

    // Should be ordered by priority first, then alphabetically for ties
    // High priority rules first: Alpha (alphabetically before Zebra), then Low Priority
    assert_eq!(ordered_rules[0].rule_name(), "Alpha Rule");
    assert_eq!(ordered_rules[1].rule_name(), "Zebra Rule");
    assert_eq!(ordered_rules[2].rule_name(), "Low Priority");

    println!("✅ Tie-breaker conflict resolution test passed");
}

#[test]
fn test_conflict_resolution_statistics() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Initial statistics should be zero
    let initial_stats = engine.get_conflict_resolution_stats();
    assert_eq!(initial_stats.conflict_sets_resolved, 0);
    assert_eq!(initial_stats.rules_ordered, 0);

    // Perform conflict resolution
    let rules = [create_test_rule(1, "Rule 1", 1), create_test_rule(2, "Rule 2", 1)];

    let rule_executions = vec![
        create_test_rule_execution(rules[0].clone(), 10, 0),
        create_test_rule_execution(rules[1].clone(), 20, 0),
    ];

    engine.resolve_rule_conflicts(rule_executions).unwrap();

    // Check updated statistics
    let updated_stats = engine.get_conflict_resolution_stats();
    assert_eq!(updated_stats.conflict_sets_resolved, 1);
    assert_eq!(updated_stats.rules_ordered, 2);
    assert_eq!(updated_stats.max_conflict_set_size, 2);
    assert_eq!(updated_stats.average_conflict_set_size, 2.0);

    // Test statistics reset
    engine.reset_conflict_resolution_stats();
    let reset_stats = engine.get_conflict_resolution_stats();
    assert_eq!(reset_stats.conflict_sets_resolved, 0);
    assert_eq!(reset_stats.rules_ordered, 0);

    println!("✅ Conflict resolution statistics test passed");
}

#[test]
fn test_empty_conflict_set() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Test with empty conflict set
    let result = engine.resolve_rule_conflicts(vec![]);
    assert!(result.is_ok());

    let ordered_rules = result.unwrap();
    assert!(ordered_rules.is_empty());

    println!("✅ Empty conflict set test passed");
}

#[test]
fn test_conflict_set_size_limit() {
    let mut engine = BingoEngine::new().expect("Failed to create engine");

    // Configure with small limit
    let config = ConflictResolutionConfig {
        primary_strategy: ConflictResolutionStrategy::Priority,
        tie_breaker: None,
        enable_logging: false,
        max_conflict_set_size: 2,
    };
    engine.configure_conflict_resolution(config);

    // Create more rules than the limit
    let rules = vec![
        create_test_rule(1, "Rule 1", 1),
        create_test_rule(2, "Rule 2", 1),
        create_test_rule(3, "Rule 3", 1),
    ];

    let rule_executions = vec![
        create_test_rule_execution(rules[0].clone(), 10, 0),
        create_test_rule_execution(rules[1].clone(), 20, 0),
        create_test_rule_execution(rules[2].clone(), 30, 0),
    ];

    let ordered_rules = engine.resolve_rule_conflicts(rule_executions).unwrap();

    // Should be truncated to max size
    assert_eq!(ordered_rules.len(), 2);

    println!("✅ Conflict set size limit test passed");
}

// Helper functions to create test data

fn create_test_rule(id: u64, name: &str, conditions_count: usize) -> Rule {
    let conditions = (0..conditions_count)
        .map(|i| Condition::Simple {
            field: format!("field_{i}"),
            operator: Operator::Equal,
            value: FactValue::Integer(i as i64),
        })
        .collect();

    Rule {
        id,
        name: name.to_string(),
        conditions,
        actions: vec![Action {
            action_type: ActionType::Log { message: format!("Rule {name} fired") },
        }],
    }
}

fn create_test_rule_with_conditions(id: u64, name: &str, conditions_count: usize) -> Rule {
    create_test_rule(id, name, conditions_count)
}

fn create_test_fact(id: u64) -> Fact {
    let mut fields = HashMap::new();
    fields.insert("test_field".to_string(), FactValue::Integer(42));

    Fact {
        id,
        external_id: Some(format!("fact_{id}")),
        timestamp: chrono::Utc::now(),
        data: FactData { fields },
    }
}

fn create_test_rule_execution(rule: Rule, priority: i32, salience: i32) -> RuleExecution {
    let fact = create_test_fact(1);
    RuleExecution::new(rule, vec![fact], 1, priority, salience)
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Integration test demonstrating the complete Phase 5 conflict resolution workflow
    #[test]
    fn test_complete_phase5_conflict_resolution_workflow() {
        println!("\n🚀 Starting Phase 5 Complete Conflict Resolution Workflow Test");

        // Step 1: Create engine with conflict resolution capabilities
        let mut engine = BingoEngine::new().expect("Failed to create engine");
        println!("✅ Created BingoEngine with conflict resolution support");

        // Step 2: Configure conflict resolution strategy
        let config = ConflictResolutionConfig {
            primary_strategy: ConflictResolutionStrategy::Priority,
            tie_breaker: Some(ConflictResolutionStrategy::Salience),
            enable_logging: true,
            max_conflict_set_size: 100,
        };

        engine.configure_conflict_resolution(config.clone());
        println!("✅ Configured conflict resolution with Priority + Salience strategy");

        // Step 3: Create comprehensive rule set with various priorities
        let rules = vec![
            // Emergency rules (high priority)
            create_test_rule(1001, "Emergency Shutdown Rule", 1),
            create_test_rule(1002, "Safety Check Rule", 1),
            // Business logic rules (medium priority)
            create_test_rule(2001, "Customer Tier Rule", 2),
            create_test_rule(2002, "Discount Calculation Rule", 2),
            create_test_rule(2003, "Inventory Check Rule", 2),
            // Logging rules (low priority)
            create_test_rule(3001, "Audit Log Rule", 1),
            create_test_rule(3002, "Notification Rule", 1),
        ];

        // Step 4: Register rule priorities and salience values
        // Emergency rules
        engine
            .register_rule_priority(1001, 100, 1000)
            .expect("Failed to register emergency rule 1");
        engine
            .register_rule_priority(1002, 100, 999)
            .expect("Failed to register emergency rule 2");

        // Business logic rules
        engine
            .register_rule_priority(2001, 50, 500)
            .expect("Failed to register business rule 1");
        engine
            .register_rule_priority(2002, 50, 400)
            .expect("Failed to register business rule 2");
        engine
            .register_rule_priority(2003, 50, 300)
            .expect("Failed to register business rule 3");

        // Logging rules
        engine
            .register_rule_priority(3001, 10, 100)
            .expect("Failed to register logging rule 1");
        engine
            .register_rule_priority(3002, 10, 50)
            .expect("Failed to register logging rule 2");

        println!(
            "✅ Registered {} rules with priorities and salience values",
            rules.len()
        );

        // Step 5: Create conflict set simulating simultaneous rule triggers
        let conflict_set = vec![
            // Mix up the order to test conflict resolution
            create_test_rule_execution(rules[6].clone(), 10, 50), // Notification (low priority)
            create_test_rule_execution(rules[0].clone(), 100, 1000), // Emergency Shutdown (high)
            create_test_rule_execution(rules[3].clone(), 50, 400), // Discount Calc (medium)
            create_test_rule_execution(rules[5].clone(), 10, 100), // Audit Log (low priority)
            create_test_rule_execution(rules[1].clone(), 100, 999), // Safety Check (high)
            create_test_rule_execution(rules[2].clone(), 50, 500), // Customer Tier (medium)
            create_test_rule_execution(rules[4].clone(), 50, 300), // Inventory Check (medium)
        ];

        println!(
            "✅ Created conflict set with {} triggered rules",
            conflict_set.len()
        );

        // Step 6: Resolve conflicts using configured strategy
        let resolution_start = std::time::Instant::now();
        let ordered_executions = engine
            .resolve_rule_conflicts(conflict_set)
            .expect("Failed to resolve rule conflicts");
        let resolution_duration = resolution_start.elapsed();

        println!("✅ Resolved conflicts in {resolution_duration:?}");

        // Step 7: Verify correct execution order
        println!("📊 Final Rule Execution Order:");

        let expected_order = vec![
            // Emergency rules first (priority 100), ordered by salience
            (1001, "Emergency Shutdown Rule", 100, 1000),
            (1002, "Safety Check Rule", 100, 999),
            // Business logic rules (priority 50), ordered by salience
            (2001, "Customer Tier Rule", 50, 500),
            (2002, "Discount Calculation Rule", 50, 400),
            (2003, "Inventory Check Rule", 50, 300),
            // Logging rules (priority 10), ordered by salience
            (3001, "Audit Log Rule", 10, 100),
            (3002, "Notification Rule", 10, 50),
        ];

        for (index, (expected_id, expected_name, expected_priority, expected_salience)) in
            expected_order.iter().enumerate()
        {
            let actual_execution = &ordered_executions[index];

            println!(
                "   {}. Rule {} - {} (Priority: {}, Salience: {})",
                index + 1,
                actual_execution.rule_id(),
                actual_execution.rule_name(),
                actual_execution.priority,
                actual_execution.salience
            );

            assert_eq!(actual_execution.rule_id(), *expected_id);
            assert_eq!(actual_execution.rule_name(), *expected_name);
            assert_eq!(actual_execution.priority, *expected_priority);
            assert_eq!(actual_execution.salience, *expected_salience);
        }

        // Step 8: Verify conflict resolution statistics
        let stats = engine.get_conflict_resolution_stats();

        println!("📈 Conflict Resolution Statistics:");
        println!(
            "   Conflict sets resolved: {}",
            stats.conflict_sets_resolved
        );
        println!("   Rules ordered: {}", stats.rules_ordered);
        println!(
            "   Average conflict set size: {:.1}",
            stats.average_conflict_set_size
        );
        println!("   Max conflict set size: {}", stats.max_conflict_set_size);
        println!(
            "   Total resolution time: {}ms",
            stats.total_resolution_time_ms
        );
        println!(
            "   Tie-breaking decisions: {}",
            stats.tie_breaking_decisions
        );

        // Step 9: Test dynamic priority updates
        println!("🔄 Testing Dynamic Priority Updates:");

        // Temporarily elevate a logging rule to emergency priority
        engine.set_rule_priority(3001, 150).expect("Failed to update rule priority");
        engine.set_rule_salience(3001, 1500).expect("Failed to update rule salience");

        // Create new conflict set
        let updated_conflict_set = vec![
            create_test_rule_execution(rules[0].clone(), 100, 1000), // Emergency Shutdown
            create_test_rule_execution(rules[5].clone(), 150, 1500), // Audit Log (now highest priority)
            create_test_rule_execution(rules[2].clone(), 50, 500),   // Customer Tier
        ];

        let updated_ordered = engine
            .resolve_rule_conflicts(updated_conflict_set)
            .expect("Failed to resolve updated conflicts");

        // Audit Log should now be first due to highest priority and salience
        assert_eq!(updated_ordered[0].rule_id(), 3001);
        assert_eq!(updated_ordered[0].priority, 150);
        assert_eq!(updated_ordered[0].salience, 1500);

        println!("   ✅ Dynamic priority update successful - Audit Log now executes first");

        // Step 10: Test configuration change
        println!("⚙️  Testing Configuration Changes:");

        let specificity_config = ConflictResolutionConfig {
            primary_strategy: ConflictResolutionStrategy::Specificity,
            tie_breaker: Some(ConflictResolutionStrategy::Lexicographic),
            enable_logging: true,
            max_conflict_set_size: 50,
        };

        engine.configure_conflict_resolution(specificity_config);

        // Create rules with different numbers of conditions
        let specificity_rules = vec![
            create_test_rule_with_conditions(4001, "Simple Rule", 1),
            create_test_rule_with_conditions(4002, "Complex Rule", 4),
            create_test_rule_with_conditions(4003, "Medium Rule", 2),
        ];

        let specificity_conflict_set = vec![
            create_test_rule_execution(specificity_rules[0].clone(), 0, 0),
            create_test_rule_execution(specificity_rules[1].clone(), 0, 0),
            create_test_rule_execution(specificity_rules[2].clone(), 0, 0),
        ];

        let specificity_ordered = engine
            .resolve_rule_conflicts(specificity_conflict_set)
            .expect("Failed to resolve specificity conflicts");

        // Should be ordered by specificity: Complex (4), Medium (2), Simple (1)
        assert_eq!(specificity_ordered[0].rule_id(), 4002); // Complex Rule
        assert_eq!(specificity_ordered[1].rule_id(), 4003); // Medium Rule
        assert_eq!(specificity_ordered[2].rule_id(), 4001); // Simple Rule

        println!("   ✅ Specificity-based ordering successful");

        // Step 11: Verify final statistics
        let final_stats = engine.get_conflict_resolution_stats();

        println!("📊 Final Statistics Summary:");
        println!(
            "   Total conflict sets resolved: {}",
            final_stats.conflict_sets_resolved
        );
        println!("   Total rules ordered: {}", final_stats.rules_ordered);
        println!(
            "   Maximum conflict set encountered: {}",
            final_stats.max_conflict_set_size
        );

        println!("\n🎉 Phase 5 Complete Conflict Resolution Workflow Test PASSED!");
        println!("   ✅ Priority-based conflict resolution implemented");
        println!("   ✅ Multiple resolution strategies supported");
        println!("   ✅ Tie-breaking mechanism operational");
        println!("   ✅ Dynamic priority updates functional");
        println!("   ✅ Configuration changes applied successfully");
        println!("   ✅ Statistics tracking comprehensive");

        // Assert success criteria
        assert!(final_stats.conflict_sets_resolved >= 3);
        assert!(final_stats.rules_ordered >= 10);
        assert!(final_stats.max_conflict_set_size >= 3);
    }
}
