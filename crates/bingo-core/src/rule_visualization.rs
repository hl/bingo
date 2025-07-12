//! Rule dependency visualization and analysis tools
//!
//! This module provides comprehensive visualization capabilities for analyzing rule dependencies,
//! RETE network structure, and rule execution flow patterns.

use super::types::{ActionType, Condition, Rule, RuleId};
use crate::types::NodeId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Write;
use tracing::{debug, info, warn};

/// Comprehensive rule dependency analyzer
#[derive(Debug)]
pub struct RuleDependencyAnalyzer {
    /// Rules being analyzed
    rules: Vec<Rule>,
    /// Rule dependency graph
    dependency_graph: RuleDependencyGraph,
    /// Field usage analysis
    field_analysis: FieldUsageAnalysis,
    /// Rule complexity metrics
    complexity_metrics: HashMap<RuleId, RuleComplexityMetrics>,
    /// RETE network topology
    network_topology: NetworkTopology,
}

/// Rule dependency graph structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDependencyGraph {
    /// Direct dependencies between rules
    pub dependencies: HashMap<RuleId, Vec<RuleId>>,
    /// Reverse dependencies (which rules depend on this rule)
    pub reverse_dependencies: HashMap<RuleId, Vec<RuleId>>,
    /// Strongly connected components (circular dependencies)
    pub cycles: Vec<Vec<RuleId>>,
    /// Topological ordering of rules
    pub execution_order: Vec<RuleId>,
    /// Critical path through rule dependencies
    pub critical_path: Vec<RuleId>,
}

/// Field usage analysis across rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldUsageAnalysis {
    /// Fields read by each rule
    pub fields_read: HashMap<RuleId, HashSet<String>>,
    /// Fields written by each rule
    pub fields_written: HashMap<RuleId, HashSet<String>>,
    /// Field dependency chains
    pub field_chains: HashMap<String, Vec<RuleId>>,
    /// Most frequently used fields
    pub popular_fields: Vec<(String, usize)>,
    /// Orphaned fields (written but never read)
    pub orphaned_fields: HashSet<String>,
    /// Missing fields (read but never written)
    pub missing_fields: HashSet<String>,
}

/// Rule complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleComplexityMetrics {
    /// Rule identifier
    pub rule_id: RuleId,
    /// Number of conditions
    pub condition_count: usize,
    /// Number of actions
    pub action_count: usize,
    /// Cyclomatic complexity
    pub cyclomatic_complexity: usize,
    /// Fan-in (number of rules that this rule depends on)
    pub fan_in: usize,
    /// Fan-out (number of rules that depend on this rule)
    pub fan_out: usize,
    /// Field access count
    pub field_access_count: usize,
    /// Estimated execution cost
    pub estimated_cost: f64,
    /// Complexity rating
    pub complexity_rating: ComplexityRating,
}

/// Complexity rating categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityRating {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

/// RETE network topology representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    /// Alpha nodes and their connections
    pub alpha_nodes: HashMap<NodeId, AlphaNodeInfo>,
    /// Beta nodes and their connections
    pub beta_nodes: HashMap<NodeId, BetaNodeInfo>,
    /// Terminal nodes and their rules
    pub terminal_nodes: HashMap<NodeId, TerminalNodeInfo>,
    /// Node connection graph
    pub connections: Vec<NodeConnection>,
    /// Network depth levels
    pub depth_levels: HashMap<NodeId, usize>,
    /// Bottleneck nodes
    pub bottlenecks: Vec<NodeBottleneck>,
}

/// Alpha node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlphaNodeInfo {
    pub node_id: NodeId,
    pub condition: String,
    pub fact_selectivity: f64,
    pub successor_count: usize,
    pub estimated_load: f64,
}

/// Beta node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetaNodeInfo {
    pub node_id: NodeId,
    pub join_conditions: Vec<String>,
    pub left_input_nodes: Vec<NodeId>,
    pub right_input_nodes: Vec<NodeId>,
    pub successor_count: usize,
    pub estimated_load: f64,
}

/// Terminal node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalNodeInfo {
    pub node_id: NodeId,
    pub rule_id: RuleId,
    pub rule_name: String,
    pub action_count: usize,
    pub complexity_score: f64,
}

/// Node connection in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConnection {
    pub from_node: NodeId,
    pub to_node: NodeId,
    pub connection_type: ConnectionType,
    pub estimated_traffic: f64,
}

/// Types of connections in the RETE network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    AlphaToBeta,
    BetaToBeta,
    AlphaToTerminal,
    BetaToTerminal,
}

/// Node bottleneck analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeBottleneck {
    pub node_id: NodeId,
    pub bottleneck_type: BottleneckType,
    pub severity: f64,
    pub description: String,
    pub recommendation: String,
}

/// Types of bottlenecks in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckType {
    HighFanOut,
    ComplexJoin,
    SlowCondition,
    MemoryIntensive,
}

/// Visualization output formats
#[derive(Debug, Clone)]
pub enum VisualizationFormat {
    /// Graphviz DOT format
    Graphviz,
    /// Mermaid diagram format
    Mermaid,
    /// SVG format
    Svg,
    /// JSON format for web visualization
    Json,
}

/// Visualization options
#[derive(Debug, Clone)]
pub struct VisualizationOptions {
    /// Output format
    pub format: VisualizationFormat,
    /// Include performance metrics
    pub include_performance: bool,
    /// Show only critical paths
    pub critical_path_only: bool,
    /// Maximum depth to visualize
    pub max_depth: Option<usize>,
    /// Highlight problematic nodes
    pub highlight_issues: bool,
    /// Include field dependencies
    pub show_field_deps: bool,
}

impl Default for RuleDependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleDependencyAnalyzer {
    /// Create new dependency analyzer
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            dependency_graph: RuleDependencyGraph::new(),
            field_analysis: FieldUsageAnalysis::new(),
            complexity_metrics: HashMap::new(),
            network_topology: NetworkTopology::new(),
        }
    }

    /// Analyze rules and build dependency graph
    pub fn analyze_rules(&mut self, rules: &[Rule]) -> anyhow::Result<()> {
        debug!(
            rule_count = rules.len(),
            "Starting rule dependency analysis"
        );

        self.rules = rules.to_vec();

        // Build dependency graph
        self.build_dependency_graph()?;

        // Analyze field usage
        self.analyze_field_usage()?;

        // Calculate complexity metrics
        self.calculate_complexity_metrics()?;

        // Detect cycles
        self.detect_dependency_cycles()?;

        // Calculate execution order
        self.calculate_execution_order()?;

        // Find critical path
        self.find_critical_path()?;

        info!(
            rule_count = rules.len(),
            dependency_count = self.dependency_graph.dependencies.len(),
            cycle_count = self.dependency_graph.cycles.len(),
            "Rule dependency analysis completed"
        );

        Ok(())
    }

    /// Build dependency graph from rule analysis
    fn build_dependency_graph(&mut self) -> anyhow::Result<()> {
        debug!("Building rule dependency graph");

        // Analyze field dependencies between rules
        for rule in &self.rules {
            let fields_read = self.extract_fields_read(rule);
            let fields_written = self.extract_fields_written(rule);

            self.field_analysis.fields_read.insert(rule.id, fields_read.clone());
            self.field_analysis.fields_written.insert(rule.id, fields_written.clone());

            // Find dependencies based on field usage
            let mut dependencies = Vec::new();
            for other_rule in &self.rules {
                if other_rule.id != rule.id {
                    let other_fields_written = self.extract_fields_written(other_rule);

                    // If this rule reads fields that another rule writes, there's a dependency
                    if fields_read.iter().any(|field| other_fields_written.contains(field)) {
                        dependencies.push(other_rule.id);
                    }
                }
            }

            self.dependency_graph.dependencies.insert(rule.id, dependencies);
        }

        // Build reverse dependencies
        for (rule_id, deps) in &self.dependency_graph.dependencies {
            for &dep_rule_id in deps {
                self.dependency_graph
                    .reverse_dependencies
                    .entry(dep_rule_id)
                    .or_default()
                    .push(*rule_id);
            }
        }

        Ok(())
    }

    /// Extract fields that a rule reads
    fn extract_fields_read(&self, rule: &Rule) -> HashSet<String> {
        let mut fields = HashSet::new();

        for condition in &rule.conditions {
            self.extract_fields_from_condition(condition, &mut fields);
        }

        fields
    }

    /// Recursively extract fields from a condition
    #[allow(clippy::only_used_in_recursion)]
    fn extract_fields_from_condition(&self, condition: &Condition, fields: &mut HashSet<String>) {
        match condition {
            Condition::Simple { field, .. } => {
                fields.insert(field.clone());
            }
            Condition::Complex { conditions, .. } => {
                for sub_condition in conditions {
                    self.extract_fields_from_condition(sub_condition, fields);
                }
            }
            Condition::Aggregation(agg) => {
                fields.insert(agg.source_field.clone());
                for group_field in &agg.group_by {
                    fields.insert(group_field.clone());
                }
                if let Some(having) = &agg.having {
                    self.extract_fields_from_condition(having, fields);
                }
            }
            Condition::Stream(stream) => {
                if let Some(filter) = &stream.filter {
                    self.extract_fields_from_condition(filter, fields);
                }
                if let Some(having) = &stream.having {
                    self.extract_fields_from_condition(having, fields);
                }
            }
            Condition::And { conditions } => {
                for cond in conditions {
                    self.extract_fields_from_condition(cond, fields);
                }
            }
            Condition::Or { conditions } => {
                for cond in conditions {
                    self.extract_fields_from_condition(cond, fields);
                }
            }
        }
    }

    /// Extract fields that a rule writes
    fn extract_fields_written(&self, rule: &Rule) -> HashSet<String> {
        let mut fields = HashSet::new();

        for action in &rule.actions {
            match &action.action_type {
                ActionType::SetField { field, .. } => {
                    fields.insert(field.clone());
                }
                ActionType::CreateFact { data } => {
                    // New facts create new field opportunities
                    for field_name in data.fields.keys() {
                        fields.insert(field_name.clone());
                    }
                }
                _ => {} // Other actions don't write fields
            }
        }

        fields
    }

    /// Analyze field usage patterns across all rules
    fn analyze_field_usage(&mut self) -> anyhow::Result<()> {
        debug!("Analyzing field usage patterns");

        // Count field usage frequency
        let mut field_usage_count: HashMap<String, usize> = HashMap::new();

        for fields in self.field_analysis.fields_read.values() {
            for field in fields {
                *field_usage_count.entry(field.clone()).or_insert(0) += 1;
            }
        }

        for fields in self.field_analysis.fields_written.values() {
            for field in fields {
                *field_usage_count.entry(field.clone()).or_insert(0) += 1;
            }
        }

        // Sort by popularity
        let mut popular_fields: Vec<(String, usize)> = field_usage_count.into_iter().collect();
        popular_fields.sort_by(|a, b| b.1.cmp(&a.1));
        self.field_analysis.popular_fields = popular_fields;

        // Find orphaned fields (written but never read)
        let all_written: HashSet<String> = self
            .field_analysis
            .fields_written
            .values()
            .flat_map(|fields| fields.iter())
            .cloned()
            .collect();

        let all_read: HashSet<String> = self
            .field_analysis
            .fields_read
            .values()
            .flat_map(|fields| fields.iter())
            .cloned()
            .collect();

        self.field_analysis.orphaned_fields = all_written.difference(&all_read).cloned().collect();
        self.field_analysis.missing_fields = all_read.difference(&all_written).cloned().collect();

        // Build field dependency chains
        for field in all_written.union(&all_read) {
            let mut chain = Vec::new();

            // Find rules that write this field
            for (rule_id, fields_written) in &self.field_analysis.fields_written {
                if fields_written.contains(field) {
                    chain.push(*rule_id);
                }
            }

            // Find rules that read this field
            for (rule_id, fields_read) in &self.field_analysis.fields_read {
                if fields_read.contains(field) && !chain.contains(rule_id) {
                    chain.push(*rule_id);
                }
            }

            if !chain.is_empty() {
                self.field_analysis.field_chains.insert(field.clone(), chain);
            }
        }

        Ok(())
    }

    /// Calculate complexity metrics for each rule
    fn calculate_complexity_metrics(&mut self) -> anyhow::Result<()> {
        debug!("Calculating rule complexity metrics");

        for rule in &self.rules {
            let fan_in = self
                .dependency_graph
                .dependencies
                .get(&rule.id)
                .map(|deps| deps.len())
                .unwrap_or(0);

            let fan_out = self
                .dependency_graph
                .reverse_dependencies
                .get(&rule.id)
                .map(|deps| deps.len())
                .unwrap_or(0);

            let field_access_count = self
                .field_analysis
                .fields_read
                .get(&rule.id)
                .map(|fields| fields.len())
                .unwrap_or(0)
                + self
                    .field_analysis
                    .fields_written
                    .get(&rule.id)
                    .map(|fields| fields.len())
                    .unwrap_or(0);

            // Calculate cyclomatic complexity (simplified)
            let cyclomatic_complexity = rule.conditions.len() + 1;

            // Estimate execution cost
            let estimated_cost = (rule.conditions.len() as f64 * 1.0)
                + (rule.actions.len() as f64 * 2.0)
                + (fan_in as f64 * 0.5)
                + (fan_out as f64 * 0.3);

            // Determine complexity rating
            let complexity_rating = if estimated_cost < 5.0 {
                ComplexityRating::Simple
            } else if estimated_cost < 15.0 {
                ComplexityRating::Moderate
            } else if estimated_cost < 30.0 {
                ComplexityRating::Complex
            } else {
                ComplexityRating::VeryComplex
            };

            let metrics = RuleComplexityMetrics {
                rule_id: rule.id,
                condition_count: rule.conditions.len(),
                action_count: rule.actions.len(),
                cyclomatic_complexity,
                fan_in,
                fan_out,
                field_access_count,
                estimated_cost,
                complexity_rating,
            };

            self.complexity_metrics.insert(rule.id, metrics);
        }

        Ok(())
    }

    /// Detect circular dependencies in rules
    fn detect_dependency_cycles(&mut self) -> anyhow::Result<()> {
        debug!("Detecting dependency cycles");

        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();
        let mut cycles = Vec::new();

        for &rule_id in self.dependency_graph.dependencies.keys() {
            if !visited.contains(&rule_id) {
                if let Some(cycle) = self.dfs_cycle_detection(
                    rule_id,
                    &mut visited,
                    &mut recursion_stack,
                    &mut Vec::new(),
                ) {
                    cycles.push(cycle);
                }
            }
        }

        self.dependency_graph.cycles = cycles;

        if !self.dependency_graph.cycles.is_empty() {
            warn!(
                cycle_count = self.dependency_graph.cycles.len(),
                "Detected circular dependencies in rules"
            );
        }

        Ok(())
    }

    /// DFS-based cycle detection
    fn dfs_cycle_detection(
        &self,
        node: RuleId,
        visited: &mut HashSet<RuleId>,
        recursion_stack: &mut HashSet<RuleId>,
        path: &mut Vec<RuleId>,
    ) -> Option<Vec<RuleId>> {
        visited.insert(node);
        recursion_stack.insert(node);
        path.push(node);

        if let Some(dependencies) = self.dependency_graph.dependencies.get(&node) {
            for &neighbor in dependencies {
                if !visited.contains(&neighbor) {
                    if let Some(cycle) =
                        self.dfs_cycle_detection(neighbor, visited, recursion_stack, path)
                    {
                        return Some(cycle);
                    }
                } else if recursion_stack.contains(&neighbor) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|&id| id == neighbor).unwrap();
                    return Some(path[cycle_start..].to_vec());
                }
            }
        }

        path.pop();
        recursion_stack.remove(&node);
        None
    }

    /// Calculate topological execution order
    fn calculate_execution_order(&mut self) -> anyhow::Result<()> {
        debug!("Calculating topological execution order");

        let mut in_degree: HashMap<RuleId, usize> = HashMap::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Initialize in-degrees
        for &rule_id in self.dependency_graph.dependencies.keys() {
            in_degree.insert(rule_id, 0);
        }

        for dependencies in self.dependency_graph.dependencies.values() {
            for &dep in dependencies {
                *in_degree.entry(dep).or_insert(0) += 1;
            }
        }

        // Find nodes with no incoming edges
        for (&rule_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(rule_id);
            }
        }

        // Process nodes in topological order
        while let Some(rule_id) = queue.pop_front() {
            result.push(rule_id);

            if let Some(dependencies) = self.dependency_graph.reverse_dependencies.get(&rule_id) {
                for &dependent in dependencies {
                    if let Some(degree) = in_degree.get_mut(&dependent) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependent);
                        }
                    }
                }
            }
        }

        self.dependency_graph.execution_order = result;
        Ok(())
    }

    /// Find critical path through rule dependencies
    fn find_critical_path(&mut self) -> anyhow::Result<()> {
        debug!("Finding critical path through rule dependencies");

        // Use estimated costs to find the longest path
        let mut distances: HashMap<RuleId, f64> = HashMap::new();
        let mut predecessors: HashMap<RuleId, Option<RuleId>> = HashMap::new();

        // Initialize distances
        for &rule_id in self.dependency_graph.dependencies.keys() {
            distances.insert(rule_id, 0.0);
            predecessors.insert(rule_id, None);
        }

        // Process nodes in topological order
        for &rule_id in &self.dependency_graph.execution_order {
            if let Some(dependencies) = self.dependency_graph.dependencies.get(&rule_id) {
                for &dep_id in dependencies {
                    let rule_cost = self
                        .complexity_metrics
                        .get(&rule_id)
                        .map(|m| m.estimated_cost)
                        .unwrap_or(1.0);

                    let new_distance = distances.get(&dep_id).unwrap_or(&0.0) + rule_cost;

                    if new_distance > *distances.get(&rule_id).unwrap_or(&0.0) {
                        distances.insert(rule_id, new_distance);
                        predecessors.insert(rule_id, Some(dep_id));
                    }
                }
            }
        }

        // Find the rule with maximum distance (end of critical path)
        let max_rule = distances
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(&rule_id, _)| rule_id);

        // Reconstruct critical path
        if let Some(mut current) = max_rule {
            let mut path = Vec::new();

            while let Some(pred) = predecessors.get(&current).and_then(|p| *p) {
                path.push(current);
                current = pred;
            }
            path.push(current);

            path.reverse();
            self.dependency_graph.critical_path = path;
        }

        Ok(())
    }

    /// Generate visualization in specified format
    pub fn generate_visualization(&self, options: &VisualizationOptions) -> anyhow::Result<String> {
        match options.format {
            VisualizationFormat::Graphviz => self.generate_graphviz(options),
            VisualizationFormat::Mermaid => self.generate_mermaid(options),
            VisualizationFormat::Svg => self.generate_svg(options),
            VisualizationFormat::Json => self.generate_json(options),
        }
    }

    /// Generate Graphviz DOT format
    fn generate_graphviz(&self, options: &VisualizationOptions) -> anyhow::Result<String> {
        let mut dot = String::new();

        writeln!(dot, "digraph RuleDependencies {{")?;
        writeln!(dot, "  rankdir=TB;")?;
        writeln!(dot, "  node [shape=box, style=rounded];")?;

        // Add nodes
        for rule in &self.rules {
            let complexity = self.complexity_metrics.get(&rule.id);
            let color = match complexity.map(|c| &c.complexity_rating) {
                Some(ComplexityRating::Simple) => "lightgreen",
                Some(ComplexityRating::Moderate) => "yellow",
                Some(ComplexityRating::Complex) => "orange",
                Some(ComplexityRating::VeryComplex) => "red",
                None => "lightblue",
            };

            let is_critical = self.dependency_graph.critical_path.contains(&rule.id);
            let style = if is_critical { "bold,filled" } else { "filled" };

            writeln!(
                dot,
                "  {} [label=\"{}\nConditions: {}\nActions: {}\", fillcolor={}, style={}];",
                rule.id,
                rule.name,
                rule.conditions.len(),
                rule.actions.len(),
                color,
                style
            )?;
        }

        // Add edges
        for (rule_id, dependencies) in &self.dependency_graph.dependencies {
            for &dep_id in dependencies {
                let style = if options.critical_path_only
                    && !self.dependency_graph.critical_path.contains(rule_id)
                {
                    "style=dashed, color=gray"
                } else {
                    "style=solid"
                };

                writeln!(dot, "  {dep_id} -> {rule_id} [{style}];")?;
            }
        }

        writeln!(dot, "}}")?;
        Ok(dot)
    }

    /// Generate Mermaid diagram format
    fn generate_mermaid(&self, _options: &VisualizationOptions) -> anyhow::Result<String> {
        let mut mermaid = String::new();

        writeln!(mermaid, "graph TD")?;

        // Add nodes with styling
        for rule in &self.rules {
            let complexity = self.complexity_metrics.get(&rule.id);
            let class = match complexity.map(|c| &c.complexity_rating) {
                Some(ComplexityRating::Simple) => "simple",
                Some(ComplexityRating::Moderate) => "moderate",
                Some(ComplexityRating::Complex) => "complex",
                Some(ComplexityRating::VeryComplex) => "very-complex",
                None => "default",
            };

            writeln!(
                mermaid,
                "  {}[\"{}\\nConditions: {}\\nActions: {}\"]",
                rule.id,
                rule.name,
                rule.conditions.len(),
                rule.actions.len()
            )?;

            writeln!(mermaid, "  class {} {class}", rule.id)?;
        }

        // Add edges
        for (rule_id, dependencies) in &self.dependency_graph.dependencies {
            for &dep_id in dependencies {
                writeln!(mermaid, "  {dep_id} --> {rule_id}")?;
            }
        }

        // Add styling
        writeln!(mermaid, "  classDef simple fill:#90EE90")?;
        writeln!(mermaid, "  classDef moderate fill:#FFFF00")?;
        writeln!(mermaid, "  classDef complex fill:#FFA500")?;
        writeln!(mermaid, "  classDef very-complex fill:#FF0000")?;
        writeln!(mermaid, "  classDef default fill:#ADD8E6")?;

        Ok(mermaid)
    }

    /// Generate SVG format - basic implementation
    fn generate_svg(&self, _options: &VisualizationOptions) -> anyhow::Result<String> {
        let mut svg = String::new();

        // SVG header
        writeln!(
            svg,
            r#"<svg width="800" height="600" xmlns="http://www.w3.org/2000/svg">"#
        )?;
        writeln!(svg, r#"  <style>"#)?;
        writeln!(
            svg,
            r#"    .rule-node {{ fill: lightblue; stroke: black; stroke-width: 1; }}"#
        )?;
        writeln!(
            svg,
            r#"    .rule-text {{ font-family: Arial; font-size: 12px; }}"#
        )?;
        writeln!(
            svg,
            r#"    .dependency-line {{ stroke: gray; stroke-width: 1; marker-end: url(#arrowhead); }}"#
        )?;
        writeln!(svg, r#"  </style>"#)?;

        // Arrow marker definition
        writeln!(svg, r#"  <defs>"#)?;
        writeln!(
            svg,
            r#"    <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">"#
        )?;
        writeln!(
            svg,
            r#"      <polygon points="0 0, 10 3.5, 0 7" fill="gray" />"#
        )?;
        writeln!(svg, r#"    </marker>"#)?;
        writeln!(svg, r#"  </defs>"#)?;

        // Calculate layout - simple grid layout
        let rules: Vec<_> = self.dependency_graph.dependencies.keys().collect();
        let cols = (rules.len() as f64).sqrt().ceil() as usize;
        let node_width = 120;
        let node_height = 40;
        let spacing_x = 150;
        let spacing_y = 80;
        let margin = 50;

        let mut positions: HashMap<RuleId, (i32, i32)> = HashMap::new();

        // Position nodes in a grid
        for (i, &rule_id) in rules.iter().enumerate() {
            let row = i / cols;
            let col = i % cols;
            let x = margin + col as i32 * spacing_x;
            let y = margin + row as i32 * spacing_y;
            positions.insert(*rule_id, (x, y));
        }

        // Draw dependency lines first (so they appear behind nodes)
        for (&rule_id, dependencies) in &self.dependency_graph.dependencies {
            if let Some(&(x1, y1)) = positions.get(&rule_id) {
                for &dep_id in dependencies {
                    if let Some(&(x2, y2)) = positions.get(&dep_id) {
                        let start_x = x1 + node_width / 2;
                        let start_y = y1 + node_height / 2;
                        let end_x = x2 + node_width / 2;
                        let end_y = y2 + node_height / 2;
                        writeln!(
                            svg,
                            r#"  <line x1="{start_x}" y1="{start_y}" x2="{end_x}" y2="{end_y}" class="dependency-line" />"#
                        )?;
                    }
                }
            }
        }

        // Draw rule nodes
        for (&rule_id, &(x, y)) in &positions {
            // Draw rectangle
            writeln!(
                svg,
                r#"  <rect x="{x}" y="{y}" width="{node_width}" height="{node_height}" class="rule-node" />"#
            )?;

            // Draw rule ID text
            let text_x = x + node_width / 2;
            let text_y = y + node_height / 2 + 4; // Center vertically with slight offset
            writeln!(
                svg,
                r#"  <text x="{text_x}" y="{text_y}" text-anchor="middle" class="rule-text">Rule {rule_id}</text>"#
            )?;
        }

        // Add title
        writeln!(
            svg,
            r#"  <text x="400" y="30" text-anchor="middle" class="rule-text" style="font-size: 16px; font-weight: bold;">Rule Dependency Graph</text>"#
        )?;

        writeln!(svg, "</svg>")?;
        Ok(svg)
    }

    /// Generate JSON format for web visualization
    fn generate_json(&self, _options: &VisualizationOptions) -> anyhow::Result<String> {
        let visualization_data = serde_json::json!({
            "dependency_graph": self.dependency_graph,
            "field_analysis": self.field_analysis,
            "complexity_metrics": self.complexity_metrics,
            "network_topology": self.network_topology
        });

        Ok(serde_json::to_string_pretty(&visualization_data)?)
    }

    /// Get dependency analysis summary
    pub fn get_analysis_summary(&self) -> DependencyAnalysisSummary {
        let total_rules = self.rules.len();
        let total_dependencies =
            self.dependency_graph.dependencies.values().map(|deps| deps.len()).sum();

        let complexity_distribution =
            self.complexity_metrics.values().fold(HashMap::new(), |mut acc, metrics| {
                let rating_str = match metrics.complexity_rating {
                    ComplexityRating::Simple => "Simple",
                    ComplexityRating::Moderate => "Moderate",
                    ComplexityRating::Complex => "Complex",
                    ComplexityRating::VeryComplex => "VeryComplex",
                };
                *acc.entry(rating_str.to_string()).or_insert(0) += 1;
                acc
            });

        DependencyAnalysisSummary {
            total_rules,
            total_dependencies,
            cycle_count: self.dependency_graph.cycles.len(),
            orphaned_fields: self.field_analysis.orphaned_fields.len(),
            missing_fields: self.field_analysis.missing_fields.len(),
            critical_path_length: self.dependency_graph.critical_path.len(),
            complexity_distribution,
            most_popular_fields: self
                .field_analysis
                .popular_fields
                .iter()
                .take(5)
                .cloned()
                .collect(),
        }
    }
}

/// Summary of dependency analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysisSummary {
    pub total_rules: usize,
    pub total_dependencies: usize,
    pub cycle_count: usize,
    pub orphaned_fields: usize,
    pub missing_fields: usize,
    pub critical_path_length: usize,
    pub complexity_distribution: HashMap<String, usize>,
    pub most_popular_fields: Vec<(String, usize)>,
}

impl Default for RuleDependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleDependencyGraph {
    fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            reverse_dependencies: HashMap::new(),
            cycles: Vec::new(),
            execution_order: Vec::new(),
            critical_path: Vec::new(),
        }
    }
}

impl Default for FieldUsageAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldUsageAnalysis {
    fn new() -> Self {
        Self {
            fields_read: HashMap::new(),
            fields_written: HashMap::new(),
            field_chains: HashMap::new(),
            popular_fields: Vec::new(),
            orphaned_fields: HashSet::new(),
            missing_fields: HashSet::new(),
        }
    }
}

impl Default for NetworkTopology {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkTopology {
    fn new() -> Self {
        Self {
            alpha_nodes: HashMap::new(),
            beta_nodes: HashMap::new(),
            terminal_nodes: HashMap::new(),
            connections: Vec::new(),
            depth_levels: HashMap::new(),
            bottlenecks: Vec::new(),
        }
    }
}

impl Default for VisualizationOptions {
    fn default() -> Self {
        Self {
            format: VisualizationFormat::Mermaid,
            include_performance: false,
            critical_path_only: false,
            max_depth: None,
            highlight_issues: true,
            show_field_deps: true,
        }
    }
}
