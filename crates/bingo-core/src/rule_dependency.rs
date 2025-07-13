//! Rule Dependency Analysis and Optimization
//!
//! This module implements sophisticated dependency analysis between rules to optimize
//! execution order, detect circular dependencies, and enable advanced optimizations
//! based on rule relationships and data flow patterns.
//!
//! ## Dependency Analysis Goals
//!
//! - **Data Flow Analysis**: Track how facts flow between rules through actions
//! - **Execution Optimization**: Order rules to minimize redundant evaluations  
//! - **Circular Dependency Detection**: Identify and resolve potential infinite loops
//! - **Parallel Execution Planning**: Find rules that can execute in parallel
//! - **Cache Optimization**: Group rules that share similar fact patterns
//!
//! ## Dependency Types
//!
//! 1. **Data Dependencies**: Rule A creates/modifies facts that Rule B depends on
//! 2. **Condition Dependencies**: Rules sharing similar condition patterns
//! 3. **Action Dependencies**: Rules that modify the same fact fields
//! 4. **Temporal Dependencies**: Rules that must execute in specific time order
//! 5. **Mutual Exclusion**: Rules that cannot execute simultaneously
//!
//! ## Architecture Overview
//!
//! ```text
//! Rule Dependency Analysis Flow:
//!
//! Rules → Dependency → Optimization → Execution
//!   ↓        Graph        Engine        Plan
//! Parse   Build Graph   Apply Opts   Generate
//! Rules   & Analyze     & Resolve     Schedule
//!           Cycles      Dependencies
//! ```

use crate::error::{BingoError, BingoResult};
use crate::types::{ActionType, Condition, Rule, RuleId};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{debug, info, instrument, warn};

/// Type of dependency between rules
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DependencyType {
    /// Rule A creates facts that Rule B consumes
    DataFlow,
    /// Rule A modifies facts that Rule B reads
    DataModification,
    /// Rules share similar condition patterns (optimization opportunity)
    ConditionSimilarity,
    /// Rules modify the same fact fields (potential conflict)
    FieldConflict,
    /// Rules must execute in specific order due to business logic
    TemporalOrdering,
    /// Rules cannot execute simultaneously
    MutualExclusion,
}

/// Represents a dependency between two rules
#[derive(Debug, Clone)]
pub struct RuleDependency {
    /// Source rule ID (depends on target)
    pub source_rule: RuleId,
    /// Target rule ID (dependency target)
    pub target_rule: RuleId,
    /// Type of dependency
    pub dependency_type: DependencyType,
    /// Strength of dependency (0.0 to 1.0)
    pub strength: f64,
    /// Fields involved in the dependency
    pub involved_fields: Vec<String>,
    /// Additional metadata about the dependency
    pub metadata: HashMap<String, String>,
}

impl RuleDependency {
    /// Create a new rule dependency
    pub fn new(
        source_rule: RuleId,
        target_rule: RuleId,
        dependency_type: DependencyType,
        strength: f64,
        involved_fields: Vec<String>,
    ) -> Self {
        Self {
            source_rule,
            target_rule,
            dependency_type,
            strength,
            involved_fields,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the dependency
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Represents a circular dependency in the rule graph
#[derive(Debug, Clone)]
pub struct CircularDependency {
    /// Rules involved in the cycle
    pub cycle_rules: Vec<RuleId>,
    /// Dependencies forming the cycle
    pub cycle_dependencies: Vec<RuleDependency>,
    /// Severity of the circular dependency
    pub severity: CircularDependencySeverity,
}

/// Severity levels for circular dependencies
#[derive(Debug, Clone, PartialEq)]
pub enum CircularDependencySeverity {
    /// Low severity - optimization opportunity only
    Low,
    /// Medium severity - potential performance impact
    Medium,
    /// High severity - logical inconsistency
    High,
    /// Critical severity - infinite loop risk
    Critical,
}

/// Configuration for dependency analysis
#[derive(Debug, Clone)]
pub struct DependencyAnalysisConfig {
    /// Enable data flow analysis
    pub enable_data_flow_analysis: bool,
    /// Enable condition similarity analysis
    pub enable_condition_similarity: bool,
    /// Enable field conflict detection
    pub enable_field_conflict_detection: bool,
    /// Minimum similarity threshold for condition grouping (0.0 to 1.0)
    pub similarity_threshold: f64,
    /// Maximum dependency graph size before optimization
    pub max_graph_size: usize,
    /// Enable circular dependency detection
    pub enable_circular_detection: bool,
}

impl Default for DependencyAnalysisConfig {
    fn default() -> Self {
        Self {
            enable_data_flow_analysis: true,
            enable_condition_similarity: true,
            enable_field_conflict_detection: true,
            similarity_threshold: 0.7,
            max_graph_size: 10000,
            enable_circular_detection: true,
        }
    }
}

/// Statistics for dependency analysis
#[derive(Debug, Default, Clone)]
pub struct DependencyAnalysisStats {
    /// Total rules analyzed
    pub rules_analyzed: usize,
    /// Total dependencies found
    pub dependencies_found: usize,
    /// Data flow dependencies
    pub data_flow_dependencies: usize,
    /// Condition similarity groups
    pub similarity_groups: usize,
    /// Field conflicts detected
    pub field_conflicts: usize,
    /// Circular dependencies found
    pub circular_dependencies: usize,
    /// Analysis time in milliseconds
    pub analysis_time_ms: u64,
    /// Optimization opportunities identified
    pub optimization_opportunities: usize,
}

/// Represents an execution cluster of related rules
#[derive(Debug, Clone)]
pub struct ExecutionCluster {
    /// Rules in this cluster
    pub rules: Vec<RuleId>,
    /// Cluster priority for execution ordering
    pub priority: i32,
    /// Whether rules in cluster can execute in parallel
    pub parallel_executable: bool,
    /// Shared fact patterns in the cluster
    pub shared_patterns: Vec<String>,
}

/// Main rule dependency analyzer
pub struct RuleDependencyAnalyzer {
    config: DependencyAnalysisConfig,
    dependencies: Vec<RuleDependency>,
    circular_dependencies: Vec<CircularDependency>,
    stats: DependencyAnalysisStats,
    /// Adjacency list for dependency graph
    dependency_graph: HashMap<RuleId, Vec<RuleId>>,
    /// Reverse dependency graph
    reverse_graph: HashMap<RuleId, Vec<RuleId>>,
}

impl RuleDependencyAnalyzer {
    /// Create a new rule dependency analyzer
    pub fn new(config: DependencyAnalysisConfig) -> Self {
        Self {
            config,
            dependencies: Vec::new(),
            circular_dependencies: Vec::new(),
            stats: DependencyAnalysisStats::default(),
            dependency_graph: HashMap::new(),
            reverse_graph: HashMap::new(),
        }
    }

    /// Analyze dependencies between a set of rules
    #[instrument(skip(self, rules))]
    pub fn analyze_dependencies(&mut self, rules: &[Rule]) -> BingoResult<DependencyAnalysisStats> {
        let start_time = std::time::Instant::now();

        info!(
            rule_count = rules.len(),
            "Starting rule dependency analysis"
        );

        // Clear previous analysis
        self.clear_analysis();

        // Check size limits
        if rules.len() > self.config.max_graph_size {
            warn!(
                rule_count = rules.len(),
                max_size = self.config.max_graph_size,
                "Rule set exceeds maximum graph size, truncating analysis"
            );
        }

        let rules_to_analyze = if rules.len() > self.config.max_graph_size {
            &rules[..self.config.max_graph_size]
        } else {
            rules
        };

        // Perform different types of dependency analysis
        if self.config.enable_data_flow_analysis {
            self.analyze_data_flow_dependencies(rules_to_analyze)?;
        }

        if self.config.enable_condition_similarity {
            self.analyze_condition_similarities(rules_to_analyze)?;
        }

        if self.config.enable_field_conflict_detection {
            self.analyze_field_conflicts(rules_to_analyze)?;
        }

        // Build dependency graph
        self.build_dependency_graph();

        // Detect circular dependencies
        if self.config.enable_circular_detection {
            self.detect_circular_dependencies()?;
        }

        // Update statistics
        let analysis_time = start_time.elapsed();
        self.update_analysis_stats(rules_to_analyze.len(), analysis_time);

        info!(
            dependencies_found = self.dependencies.len(),
            circular_deps = self.circular_dependencies.len(),
            analysis_time_ms = analysis_time.as_millis(),
            "Completed rule dependency analysis"
        );

        Ok(self.stats.clone())
    }

    /// Analyze data flow dependencies between rules
    fn analyze_data_flow_dependencies(&mut self, rules: &[Rule]) -> BingoResult<()> {
        debug!("Analyzing data flow dependencies");

        // Build maps of fields created and consumed by each rule
        let mut field_creators: HashMap<String, Vec<RuleId>> = HashMap::new();
        let mut field_consumers: HashMap<String, Vec<RuleId>> = HashMap::new();

        // Analyze what fields each rule creates and consumes
        for rule in rules {
            // Analyze actions to see what fields are created/modified
            for action in &rule.actions {
                match &action.action_type {
                    ActionType::SetField { field, .. } => {
                        field_creators.entry(field.clone()).or_default().push(rule.id);
                    }
                    ActionType::CreateFact { data } => {
                        for field in data.fields.keys() {
                            field_creators.entry(field.clone()).or_default().push(rule.id);
                        }
                    }
                    _ => {} // Other actions don't create/modify fields
                }
            }

            // Analyze conditions to see what fields are consumed
            for condition in &rule.conditions {
                match condition {
                    Condition::Simple { field, .. } => {
                        field_consumers.entry(field.clone()).or_default().push(rule.id);
                    }
                    Condition::Complex { conditions, .. } => {
                        // Recursively analyze complex conditions
                        Self::extract_fields_from_conditions(
                            conditions,
                            &mut field_consumers,
                            rule.id,
                        );
                    }
                    _ => {} // Handle other condition types
                }
            }
        }

        // Create dependencies where one rule creates what another consumes
        for (field, consumers) in &field_consumers {
            if let Some(creators) = field_creators.get(field) {
                for &creator_id in creators {
                    for &consumer_id in consumers {
                        if creator_id != consumer_id {
                            let dependency = RuleDependency::new(
                                consumer_id,
                                creator_id,
                                DependencyType::DataFlow,
                                0.8, // High strength for data flow
                                vec![field.clone()],
                            );
                            self.dependencies.push(dependency);
                            self.stats.data_flow_dependencies += 1;
                        }
                    }
                }
            }
        }

        debug!(
            data_flow_deps = self.stats.data_flow_dependencies,
            "Completed data flow dependency analysis"
        );

        Ok(())
    }

    /// Extract fields from complex conditions recursively
    fn extract_fields_from_conditions(
        conditions: &[Condition],
        field_consumers: &mut HashMap<String, Vec<RuleId>>,
        rule_id: RuleId,
    ) {
        for condition in conditions {
            match condition {
                Condition::Simple { field, .. } => {
                    field_consumers.entry(field.clone()).or_default().push(rule_id);
                }
                Condition::Complex { conditions, .. } => {
                    Self::extract_fields_from_conditions(conditions, field_consumers, rule_id);
                }
                _ => {} // Handle other condition types
            }
        }
    }

    /// Analyze condition similarities between rules
    fn analyze_condition_similarities(&mut self, rules: &[Rule]) -> BingoResult<()> {
        debug!("Analyzing condition similarities");

        let mut similarity_groups = 0;

        // Compare each pair of rules for condition similarity
        for i in 0..rules.len() {
            for j in (i + 1)..rules.len() {
                let rule_a = &rules[i];
                let rule_b = &rules[j];

                let similarity = self.calculate_condition_similarity(rule_a, rule_b);

                if similarity >= self.config.similarity_threshold {
                    let shared_fields = self.find_shared_condition_fields(rule_a, rule_b);

                    let dependency = RuleDependency::new(
                        rule_a.id,
                        rule_b.id,
                        DependencyType::ConditionSimilarity,
                        similarity,
                        shared_fields,
                    )
                    .with_metadata("similarity_score".to_string(), format!("{similarity:.2}"));

                    self.dependencies.push(dependency);
                    similarity_groups += 1;
                }
            }
        }

        self.stats.similarity_groups = similarity_groups;
        debug!(
            similarity_groups = similarity_groups,
            "Completed condition similarity analysis"
        );

        Ok(())
    }

    /// Calculate similarity score between two rules' conditions
    fn calculate_condition_similarity(&self, rule_a: &Rule, rule_b: &Rule) -> f64 {
        let fields_a: HashSet<String> = Self::extract_condition_fields(&rule_a.conditions);
        let fields_b: HashSet<String> = Self::extract_condition_fields(&rule_b.conditions);

        if fields_a.is_empty() && fields_b.is_empty() {
            return 0.0;
        }

        let intersection = fields_a.intersection(&fields_b).count();
        let union = fields_a.union(&fields_b).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Extract all field names from a set of conditions
    fn extract_condition_fields(conditions: &[Condition]) -> HashSet<String> {
        let mut fields = HashSet::new();

        for condition in conditions {
            match condition {
                Condition::Simple { field, .. } => {
                    fields.insert(field.clone());
                }
                Condition::Complex { conditions, .. } => {
                    fields.extend(Self::extract_condition_fields(conditions));
                }
                _ => {} // Handle other condition types
            }
        }

        fields
    }

    /// Find shared condition fields between two rules
    fn find_shared_condition_fields(&self, rule_a: &Rule, rule_b: &Rule) -> Vec<String> {
        let fields_a = Self::extract_condition_fields(&rule_a.conditions);
        let fields_b = Self::extract_condition_fields(&rule_b.conditions);

        fields_a.intersection(&fields_b).cloned().collect()
    }

    /// Analyze field conflicts between rules
    fn analyze_field_conflicts(&mut self, rules: &[Rule]) -> BingoResult<()> {
        debug!("Analyzing field conflicts");

        let mut field_modifiers: HashMap<String, Vec<RuleId>> = HashMap::new();

        // Find rules that modify the same fields
        for rule in rules {
            for action in &rule.actions {
                match &action.action_type {
                    ActionType::SetField { field, .. } => {
                        field_modifiers.entry(field.clone()).or_default().push(rule.id);
                    }
                    ActionType::CreateFact { data } => {
                        for field in data.fields.keys() {
                            field_modifiers.entry(field.clone()).or_default().push(rule.id);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Create conflict dependencies
        let mut conflicts = 0;
        for (field, modifiers) in &field_modifiers {
            if modifiers.len() > 1 {
                // Multiple rules modify the same field - potential conflict
                for i in 0..modifiers.len() {
                    for j in (i + 1)..modifiers.len() {
                        let dependency = RuleDependency::new(
                            modifiers[i],
                            modifiers[j],
                            DependencyType::FieldConflict,
                            0.6, // Medium strength for conflicts
                            vec![field.clone()],
                        )
                        .with_metadata(
                            "conflict_type".to_string(),
                            "field_modification".to_string(),
                        );

                        self.dependencies.push(dependency);
                        conflicts += 1;
                    }
                }
            }
        }

        self.stats.field_conflicts = conflicts;
        debug!(
            field_conflicts = conflicts,
            "Completed field conflict analysis"
        );

        Ok(())
    }

    /// Build dependency graph from analyzed dependencies
    fn build_dependency_graph(&mut self) {
        debug!("Building dependency graph");

        self.dependency_graph.clear();
        self.reverse_graph.clear();

        for dependency in &self.dependencies {
            // Forward graph: target -> source (for execution order: target must execute before source)
            self.dependency_graph
                .entry(dependency.target_rule)
                .or_default()
                .push(dependency.source_rule);

            // Reverse graph: source -> target
            self.reverse_graph
                .entry(dependency.source_rule)
                .or_default()
                .push(dependency.target_rule);
        }

        debug!(
            nodes = self.dependency_graph.len(),
            edges = self.dependencies.len(),
            "Built dependency graph"
        );
    }

    /// Detect circular dependencies in the rule graph
    fn detect_circular_dependencies(&mut self) -> BingoResult<()> {
        debug!("Detecting circular dependencies");

        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut cycles_found = 0;

        // Check each node for cycles
        for &rule_id in self.dependency_graph.keys() {
            if !visited.contains(&rule_id) {
                if let Some(cycle) =
                    self.detect_cycle_dfs(rule_id, &mut visited, &mut rec_stack, &mut Vec::new())
                {
                    let severity = self.assess_cycle_severity(&cycle);
                    let cycle_deps = self.get_cycle_dependencies(&cycle);

                    let circular_dep = CircularDependency {
                        cycle_rules: cycle,
                        cycle_dependencies: cycle_deps,
                        severity,
                    };

                    self.circular_dependencies.push(circular_dep);
                    cycles_found += 1;
                }
            }
        }

        self.stats.circular_dependencies = cycles_found;

        if cycles_found > 0 {
            warn!(
                circular_dependencies = cycles_found,
                "Found circular dependencies in rule graph"
            );
        } else {
            debug!("No circular dependencies detected");
        }

        Ok(())
    }

    /// Depth-first search to detect cycles
    fn detect_cycle_dfs(
        &self,
        node: RuleId,
        visited: &mut HashSet<RuleId>,
        rec_stack: &mut HashSet<RuleId>,
        path: &mut Vec<RuleId>,
    ) -> Option<Vec<RuleId>> {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        if let Some(neighbors) = self.dependency_graph.get(&node) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    if let Some(cycle) = self.detect_cycle_dfs(neighbor, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(&neighbor) {
                    // Found a cycle - extract it from the path
                    if let Some(start_pos) = path.iter().position(|&x| x == neighbor) {
                        return Some(path[start_pos..].to_vec());
                    }
                }
            }
        }

        path.pop();
        rec_stack.remove(&node);
        None
    }

    /// Assess the severity of a circular dependency
    fn assess_cycle_severity(&self, cycle: &[RuleId]) -> CircularDependencySeverity {
        // Count data flow dependencies in the cycle
        let mut data_flow_count = 0;
        let mut field_conflict_count = 0;

        for dependency in &self.dependencies {
            if cycle.contains(&dependency.source_rule) && cycle.contains(&dependency.target_rule) {
                match dependency.dependency_type {
                    DependencyType::DataFlow => data_flow_count += 1,
                    DependencyType::FieldConflict => field_conflict_count += 1,
                    _ => {}
                }
            }
        }

        // Assess severity based on dependency types and cycle length
        if data_flow_count > 0 && cycle.len() <= 3 {
            CircularDependencySeverity::Critical // Small data flow cycles are dangerous
        } else if field_conflict_count > 2 {
            CircularDependencySeverity::High // Many field conflicts
        } else if data_flow_count > 0 {
            CircularDependencySeverity::Medium // Data flow cycles
        } else {
            CircularDependencySeverity::Low // Just similarity cycles
        }
    }

    /// Get dependencies that form a cycle
    fn get_cycle_dependencies(&self, cycle: &[RuleId]) -> Vec<RuleDependency> {
        let cycle_set: HashSet<_> = cycle.iter().collect();

        self.dependencies
            .iter()
            .filter(|dep| {
                cycle_set.contains(&dep.source_rule) && cycle_set.contains(&dep.target_rule)
            })
            .cloned()
            .collect()
    }

    /// Generate execution clusters based on dependency analysis
    pub fn generate_execution_clusters(&self) -> BingoResult<Vec<ExecutionCluster>> {
        debug!("Generating execution clusters");

        let mut clusters = Vec::new();
        let mut assigned_rules = HashSet::new();

        // Find strongly connected components (clusters of mutually dependent rules)
        let sccs = self.find_strongly_connected_components();

        for (cluster_id, scc) in sccs.into_iter().enumerate() {
            if scc.len() == 1 && !self.dependency_graph.contains_key(&scc[0]) {
                // Single rule with no dependencies - can be in its own cluster
                let cluster = ExecutionCluster {
                    rules: scc.clone(),
                    priority: 0,
                    parallel_executable: true,
                    shared_patterns: vec![],
                };
                clusters.push(cluster);
            } else {
                // Multiple rules or rules with dependencies
                let parallel_executable = self.can_execute_in_parallel(&scc);
                let shared_patterns = self.find_shared_patterns(&scc);

                let cluster = ExecutionCluster {
                    rules: scc.clone(),
                    priority: cluster_id as i32,
                    parallel_executable,
                    shared_patterns,
                };
                clusters.push(cluster);
            }

            for rule_id in &scc {
                assigned_rules.insert(*rule_id);
            }
        }

        info!(
            clusters_generated = clusters.len(),
            "Generated execution clusters"
        );

        Ok(clusters)
    }

    /// Find strongly connected components in the dependency graph
    fn find_strongly_connected_components(&self) -> Vec<Vec<RuleId>> {
        // Simplified SCC algorithm (Kosaraju's algorithm)
        let mut visited = HashSet::new();
        let mut finish_order = Vec::new();

        // First DFS to get finish order
        for &node in self.dependency_graph.keys() {
            if !visited.contains(&node) {
                self.dfs_finish_order(node, &mut visited, &mut finish_order);
            }
        }

        // Second DFS on reverse graph
        let mut sccs = Vec::new();
        visited.clear();

        for &node in finish_order.iter().rev() {
            if !visited.contains(&node) {
                let mut scc = Vec::new();
                self.dfs_scc(node, &mut visited, &mut scc);
                if !scc.is_empty() {
                    sccs.push(scc);
                }
            }
        }

        sccs
    }

    /// DFS to determine finish order
    fn dfs_finish_order(
        &self,
        node: RuleId,
        visited: &mut HashSet<RuleId>,
        finish_order: &mut Vec<RuleId>,
    ) {
        visited.insert(node);

        if let Some(neighbors) = self.dependency_graph.get(&node) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    self.dfs_finish_order(neighbor, visited, finish_order);
                }
            }
        }

        finish_order.push(node);
    }

    /// DFS for SCC detection
    fn dfs_scc(&self, node: RuleId, visited: &mut HashSet<RuleId>, scc: &mut Vec<RuleId>) {
        visited.insert(node);
        scc.push(node);

        if let Some(neighbors) = self.reverse_graph.get(&node) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    self.dfs_scc(neighbor, visited, scc);
                }
            }
        }
    }

    /// Check if rules in a cluster can execute in parallel
    fn can_execute_in_parallel(&self, rules: &[RuleId]) -> bool {
        // Rules can execute in parallel if they don't have field conflicts
        for dependency in &self.dependencies {
            if rules.contains(&dependency.source_rule) && rules.contains(&dependency.target_rule) {
                match dependency.dependency_type {
                    DependencyType::FieldConflict | DependencyType::DataFlow => {
                        return false; // Cannot execute in parallel
                    }
                    _ => {}
                }
            }
        }
        true
    }

    /// Find shared patterns among rules in a cluster
    fn find_shared_patterns(&self, rules: &[RuleId]) -> Vec<String> {
        // Look for condition similarity dependencies among cluster rules
        let mut patterns = HashSet::new();

        for dependency in &self.dependencies {
            if rules.contains(&dependency.source_rule)
                && rules.contains(&dependency.target_rule)
                && dependency.dependency_type == DependencyType::ConditionSimilarity
            {
                patterns.extend(dependency.involved_fields.clone());
            }
        }

        patterns.into_iter().collect()
    }

    /// Get topological ordering of rules based on dependencies
    pub fn get_topological_order(&self) -> BingoResult<Vec<RuleId>> {
        let mut in_degree: HashMap<RuleId, usize> = HashMap::new();
        let mut all_rules = HashSet::new();

        // Initialize in-degree for all rules
        for dependency in &self.dependencies {
            all_rules.insert(dependency.source_rule);
            all_rules.insert(dependency.target_rule);
        }

        for &rule_id in &all_rules {
            in_degree.insert(rule_id, 0);
        }

        // Calculate in-degrees
        for dependency in &self.dependencies {
            *in_degree.get_mut(&dependency.source_rule).unwrap() += 1;
        }

        // Topological sort using Kahn's algorithm
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Add all nodes with in-degree 0
        for (&rule_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(rule_id);
            }
        }

        while let Some(rule_id) = queue.pop_front() {
            result.push(rule_id);

            // Reduce in-degree of neighbors
            if let Some(neighbors) = self.dependency_graph.get(&rule_id) {
                for &neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(&neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        // Check for cycles
        if result.len() != all_rules.len() {
            return Err(BingoError::rete_network(
                "dependency_graph",
                "Topological sort failed due to circular dependencies",
            ));
        }

        Ok(result)
    }

    /// Clear previous analysis results
    fn clear_analysis(&mut self) {
        self.dependencies.clear();
        self.circular_dependencies.clear();
        self.dependency_graph.clear();
        self.reverse_graph.clear();
        self.stats = DependencyAnalysisStats::default();
    }

    /// Update analysis statistics
    fn update_analysis_stats(&mut self, rules_analyzed: usize, analysis_time: std::time::Duration) {
        self.stats.rules_analyzed = rules_analyzed;
        self.stats.dependencies_found = self.dependencies.len();
        self.stats.analysis_time_ms = analysis_time.as_millis() as u64;

        // Count optimization opportunities
        self.stats.optimization_opportunities = self
            .dependencies
            .iter()
            .filter(|dep| matches!(dep.dependency_type, DependencyType::ConditionSimilarity))
            .count();
    }

    /// Get current dependencies
    pub fn get_dependencies(&self) -> &[RuleDependency] {
        &self.dependencies
    }

    /// Get circular dependencies
    pub fn get_circular_dependencies(&self) -> &[CircularDependency] {
        &self.circular_dependencies
    }

    /// Get analysis statistics
    pub fn get_stats(&self) -> &DependencyAnalysisStats {
        &self.stats
    }

    /// Get current configuration
    pub fn get_config(&self) -> &DependencyAnalysisConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, new_config: DependencyAnalysisConfig) {
        info!("Updating dependency analysis configuration");
        self.config = new_config;
    }
}

impl Default for RuleDependencyAnalyzer {
    fn default() -> Self {
        Self::new(DependencyAnalysisConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Action, ActionType, Condition, FactValue, Operator};

    fn create_rule_with_fields(
        id: RuleId,
        name: &str,
        input_fields: &[&str],
        output_fields: &[&str],
    ) -> Rule {
        let conditions = input_fields
            .iter()
            .map(|&field| Condition::Simple {
                field: field.to_string(),
                operator: Operator::Equal,
                value: FactValue::Integer(1),
            })
            .collect();

        let actions = output_fields
            .iter()
            .map(|&field| Action {
                action_type: ActionType::SetField {
                    field: field.to_string(),
                    value: FactValue::String("output".to_string()),
                },
            })
            .collect();

        Rule { id, name: name.to_string(), conditions, actions }
    }

    #[test]
    fn test_dependency_analyzer_creation() {
        let config = DependencyAnalysisConfig::default();
        let analyzer = RuleDependencyAnalyzer::new(config);

        assert_eq!(analyzer.get_dependencies().len(), 0);
        assert_eq!(analyzer.get_circular_dependencies().len(), 0);
        assert_eq!(analyzer.get_stats().rules_analyzed, 0);
    }

    #[test]
    fn test_data_flow_dependency_analysis() {
        let mut analyzer = RuleDependencyAnalyzer::default();

        let rules = vec![
            create_rule_with_fields(1, "Producer", &["input"], &["intermediate"]),
            create_rule_with_fields(2, "Consumer", &["intermediate"], &["output"]),
            create_rule_with_fields(3, "Independent", &["other"], &["result"]),
        ];

        analyzer.analyze_dependencies(&rules).unwrap();

        let deps = analyzer.get_dependencies();
        let data_flow_deps: Vec<_> =
            deps.iter().filter(|d| d.dependency_type == DependencyType::DataFlow).collect();

        assert_eq!(data_flow_deps.len(), 1);
        assert_eq!(data_flow_deps[0].source_rule, 2); // Consumer depends on
        assert_eq!(data_flow_deps[0].target_rule, 1); // Producer
    }

    #[test]
    fn test_condition_similarity_analysis() {
        let mut analyzer = RuleDependencyAnalyzer::default();

        let rules = vec![
            create_rule_with_fields(1, "Similar A", &["field1", "field2"], &["out1"]),
            create_rule_with_fields(2, "Similar B", &["field1", "field2"], &["out2"]),
            create_rule_with_fields(3, "Different", &["field3"], &["out3"]),
        ];

        analyzer.analyze_dependencies(&rules).unwrap();

        let deps = analyzer.get_dependencies();
        let similarity_deps: Vec<_> = deps
            .iter()
            .filter(|d| d.dependency_type == DependencyType::ConditionSimilarity)
            .collect();

        assert!(!similarity_deps.is_empty());
        assert!(similarity_deps[0].strength >= analyzer.config.similarity_threshold);
    }

    #[test]
    fn test_field_conflict_analysis() {
        let mut analyzer = RuleDependencyAnalyzer::default();

        let rules = vec![
            create_rule_with_fields(1, "Modifier A", &["input"], &["shared_field"]),
            create_rule_with_fields(2, "Modifier B", &["input"], &["shared_field"]),
            create_rule_with_fields(3, "No Conflict", &["input"], &["unique_field"]),
        ];

        analyzer.analyze_dependencies(&rules).unwrap();

        let deps = analyzer.get_dependencies();
        let conflict_deps: Vec<_> = deps
            .iter()
            .filter(|d| d.dependency_type == DependencyType::FieldConflict)
            .collect();

        assert_eq!(conflict_deps.len(), 1);
        assert!(conflict_deps[0].involved_fields.contains(&"shared_field".to_string()));
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut analyzer = RuleDependencyAnalyzer::default();

        // Create rules that form a cycle
        let rules = vec![
            create_rule_with_fields(1, "Rule A", &["field_c"], &["field_a"]),
            create_rule_with_fields(2, "Rule B", &["field_a"], &["field_b"]),
            create_rule_with_fields(3, "Rule C", &["field_b"], &["field_c"]),
        ];

        analyzer.analyze_dependencies(&rules).unwrap();

        let circular_deps = analyzer.get_circular_dependencies();
        assert!(!circular_deps.is_empty());
        assert_eq!(circular_deps[0].cycle_rules.len(), 3);
    }

    #[test]
    fn test_execution_cluster_generation() {
        let mut analyzer = RuleDependencyAnalyzer::default();

        let rules = vec![
            create_rule_with_fields(1, "Independent A", &["input1"], &["output1"]),
            create_rule_with_fields(2, "Independent B", &["input2"], &["output2"]),
            create_rule_with_fields(3, "Similar C", &["input1"], &["output3"]),
        ];

        analyzer.analyze_dependencies(&rules).unwrap();
        let clusters = analyzer.generate_execution_clusters().unwrap();

        assert!(!clusters.is_empty());

        // Check that independent rules can execute in parallel
        let parallel_clusters: Vec<_> = clusters.iter().filter(|c| c.parallel_executable).collect();
        assert!(!parallel_clusters.is_empty());
    }

    #[test]
    fn test_topological_ordering() {
        let mut analyzer = RuleDependencyAnalyzer::default();

        let rules = vec![
            create_rule_with_fields(1, "First", &["input"], &["intermediate1"]),
            create_rule_with_fields(2, "Second", &["intermediate1"], &["intermediate2"]),
            create_rule_with_fields(3, "Third", &["intermediate2"], &["output"]),
        ];

        analyzer.analyze_dependencies(&rules).unwrap();
        let topo_order = analyzer.get_topological_order().unwrap();

        // Should be ordered: 1 -> 2 -> 3 (reverse dependency order)
        assert_eq!(topo_order, vec![1, 2, 3]);
    }

    #[test]
    fn test_configuration_updates() {
        let mut analyzer = RuleDependencyAnalyzer::default();

        let new_config = DependencyAnalysisConfig {
            similarity_threshold: 0.9,
            enable_circular_detection: false,
            ..Default::default()
        };

        analyzer.update_config(new_config.clone());
        assert_eq!(analyzer.get_config().similarity_threshold, 0.9);
        assert!(!analyzer.get_config().enable_circular_detection);
    }

    #[test]
    fn test_statistics_tracking() {
        let mut analyzer = RuleDependencyAnalyzer::default();

        let rules = vec![
            create_rule_with_fields(1, "Rule A", &["field1"], &["field2"]),
            create_rule_with_fields(2, "Rule B", &["field2"], &["field3"]),
        ];

        analyzer.analyze_dependencies(&rules).unwrap();

        let stats = analyzer.get_stats();
        assert_eq!(stats.rules_analyzed, 2);
        assert!(stats.dependencies_found > 0);
        assert!(stats.analysis_time_ms < 10000); // Should complete quickly
    }
}
